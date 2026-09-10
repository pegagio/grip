use super::lock::PublicationLock;
use super::{
    AcceptedState, AcceptedStateV3, AcceptedStateV4, accepted_v4_from_runtime, decode_v4,
    encode_v4, runtime_from_accepted_v4,
};
use crate::error::GripError;
use crate::project::ProjectPaths;
use sha2::Digest;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StateFileIdentity {
    device: u64,
    inode: u64,
    mode: u32,
}

/// Accepted state plus exact bytes and identity for expected-snapshot checks.
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub accepted: AcceptedState,
    pub complete: Option<AcceptedStateV3>,
    pub portable: Option<AcceptedStateV4>,
    pub rebinding: crate::state::rebinding::RebindingAssessment,
    pub bytes: Option<Vec<u8>>,
    identity: Option<StateFileIdentity>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationFault {
    CorruptV2Staging,
    BeforeV2StateRename,
    AfterV2StateRename,
}

fn ensure_dir(path: &Path) -> Result<(), GripError> {
    match fs::symlink_metadata(path) {
        Ok(m)
            if !m.file_type().is_symlink()
                && m.is_dir()
                && m.uid() == rustix::process::geteuid().as_raw() =>
        {
            if m.permissions().mode() & 0o7777 == 0o700 {
                Ok(())
            } else {
                Err(GripError::CorruptState(format!(
                    "state directory {} must have mode 0700",
                    path.display()
                )))
            }
        }
        Ok(_) => Err(GripError::CorruptState(format!(
            "unsafe state directory {}",
            path.display()
        ))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let mut b = fs::DirBuilder::new();
            b.mode(0o700);
            b.create(path)
                .map_err(|e| GripError::from_io("could not create state directory", e))?;
            fs::set_permissions(path, fs::Permissions::from_mode(0o700))
                .map_err(|e| GripError::from_io("could not secure state directory", e))
        }
        Err(e) => Err(GripError::from_io("could not inspect state directory", e)),
    }
}

/// Prepare the private state directory before acquiring its stable lock.
pub fn prepare_directory(home: &ProjectPaths) -> Result<std::path::PathBuf, GripError> {
    let state_dir = home.path().join("state");
    ensure_dir(&state_dir)?;
    Ok(state_dir)
}

fn state_identity(metadata: &fs::Metadata) -> StateFileIdentity {
    StateFileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        mode: metadata.permissions().mode() & 0o7777,
    }
}

/// Load absent, V1, or V2 accepted state without creating state artifacts.
pub fn load(home: &ProjectPaths) -> Result<StateSnapshot, GripError> {
    let state_dir = home.path().join("state");
    let state_dir_metadata = match fs::symlink_metadata(&state_dir) {
        Ok(metadata)
            if !metadata.file_type().is_symlink()
                && metadata.is_dir()
                && metadata.uid() == rustix::process::geteuid().as_raw()
                && metadata.permissions().mode() & 0o7777 == 0o700 =>
        {
            metadata
        }
        Ok(_) => {
            return Err(GripError::CorruptState(
                "state directory must be a current-user-owned 0700 non-symlink directory".into(),
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(StateSnapshot {
                accepted: AcceptedState::uninitialized(),
                complete: None,
                portable: None,
                rebinding: crate::state::rebinding::RebindingAssessment {
                    outcome: crate::result::RebindingOutcome::Uninitialized,
                    prior_root: None,
                    blockers: Vec::new(),
                },
                bytes: None,
                identity: None,
            });
        }
        Err(error) => {
            return Err(GripError::from_io(
                "could not inspect accepted state directory",
                error,
            ));
        }
    };
    let directory_descriptor = rustix::fs::open(
        &state_dir,
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::DIRECTORY
            | rustix::fs::OFlags::CLOEXEC
            | rustix::fs::OFlags::NOFOLLOW,
        rustix::fs::Mode::empty(),
    )
    .map_err(|error| {
        GripError::from_io(
            "could not open accepted state directory safely",
            error.into(),
        )
    })?;
    let directory = File::from(directory_descriptor);
    let opened_directory_metadata = directory.metadata().map_err(|error| {
        GripError::from_io("could not inspect opened accepted state directory", error)
    })?;
    if state_identity(&opened_directory_metadata) != state_identity(&state_dir_metadata) {
        return Err(GripError::CorruptState(
            "state directory changed during safe open".into(),
        ));
    }
    let descriptor = match rustix::fs::openat(
        &directory,
        "state.json",
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NOFOLLOW,
        rustix::fs::Mode::empty(),
    ) {
        Ok(descriptor) => descriptor,
        Err(rustix::io::Errno::NOENT) => {
            return Ok(StateSnapshot {
                accepted: AcceptedState::uninitialized(),
                complete: None,
                portable: None,
                rebinding: crate::state::rebinding::RebindingAssessment {
                    outcome: crate::result::RebindingOutcome::Uninitialized,
                    prior_root: None,
                    blockers: Vec::new(),
                },
                bytes: None,
                identity: None,
            });
        }
        Err(rustix::io::Errno::LOOP) => {
            return Err(GripError::CorruptState(
                "state.json must not be a symbolic link".into(),
            ));
        }
        Err(error) => {
            return Err(GripError::from_io(
                "could not open accepted state safely",
                error.into(),
            ));
        }
    };
    let mut file = File::from(descriptor);
    let metadata = file
        .metadata()
        .map_err(|error| GripError::from_io("could not inspect accepted state", error))?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.permissions().mode() & 0o7777 != 0o600
    {
        return Err(GripError::CorruptState(
            "state.json must be a current-user-owned 0600 regular file".into(),
        ));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| GripError::from_io("could not read accepted state", error))?;
    let portable = decode_v4(&bytes)?;
    let rebinding = crate::state::rebinding::assess(home, &portable)?;
    let complete = runtime_from_accepted_v4(home, &portable)?;
    let mut accepted = AcceptedState {
        generation: Some(complete.generation),
        complete_baselines: complete.baselines.clone(),
        accepted_bytes: Some(bytes.clone()),
    };
    accepted.accepted_bytes = Some(bytes.clone());
    Ok(StateSnapshot {
        accepted,
        complete: Some(complete),
        portable: Some(portable),
        rebinding,
        bytes: Some(bytes),
        identity: Some(state_identity(&metadata)),
    })
}

/// Reject drift from a previously loaded accepted-state snapshot.
pub fn revalidate(home: &ProjectPaths, expected: &StateSnapshot) -> Result<(), GripError> {
    let current = load(home).map_err(|error| {
        GripError::discovery_operational(
            "state_revalidate",
            "stale_state_evidence",
            vec![home.path().join("state/state.json").display().to_string()],
            &format!("accepted state changed during operation: {error}"),
        )
    })?;
    if current.bytes != expected.bytes || current.identity != expected.identity {
        return Err(GripError::discovery_operational(
            "state_revalidate",
            "stale_state_evidence",
            vec![home.path().join("state/state.json").display().to_string()],
            "accepted state changed during operation",
        ));
    }
    Ok(())
}

/// Publish a V2 baseline map while the caller holds the state publication lock.
pub fn publish_current_locked(
    home: &ProjectPaths,
    expected: &StateSnapshot,
    next: &AcceptedState,
) -> Result<Option<u64>, GripError> {
    publish_complete_locked(home, expected, &next.complete_baselines)
}

/// Restore exact, integrity-valid State V2 bytes while the caller holds the mutation lock.
#[allow(dead_code)]
pub(crate) fn restore_exact(
    home: &ProjectPaths,
    bytes: &[u8],
    expected_post_generation: u64,
    expected_post_digest: &str,
) -> Result<(), GripError> {
    let recovered = decode_v4(bytes)?;
    if let Ok(current) = load(home)
        && let Some(current_bytes) = current.bytes.as_deref()
    {
        let digest = format!("{:x}", sha2::Sha256::digest(current_bytes));
        if current.accepted.generation != Some(expected_post_generation)
            || digest != expected_post_digest
        {
            return Err(GripError::InvalidConfiguration(
                "current valid state differs from recorded post-transition authority".into(),
            ));
        }
    }
    let directory = prepare_directory(home)?;
    let lock_path = super::lock::project_lock_path(home, "state.lock")?;
    let _lock = PublicationLock::acquire(&lock_path)?;
    let runtime = runtime_from_accepted_v4(home, &recovered)?;
    write_v3_atomic(
        home,
        &directory,
        &directory.join("state.json"),
        bytes,
        &runtime.baselines,
        None,
    )?;
    let reread = read_private_file(&directory.join("state.json"))?;
    if reread != bytes || decode_v4(&reread)? != recovered {
        return Err(GripError::CorruptState(
            "restored state verification failed".into(),
        ));
    }
    Ok(())
}

#[doc(hidden)]
pub fn publish_complete_locked(
    home: &ProjectPaths,
    expected: &StateSnapshot,
    next_baselines: &std::collections::BTreeMap<
        crate::observation::model::EntryIdentity,
        crate::metadata::model::SupportedEntryStateV3,
    >,
) -> Result<Option<u64>, GripError> {
    publish_complete_locked_with_fault(home, expected, next_baselines, None)
}

#[doc(hidden)]
pub fn publish_complete_locked_with_fault(
    home: &ProjectPaths,
    expected: &StateSnapshot,
    next_baselines: &std::collections::BTreeMap<
        crate::observation::model::EntryIdentity,
        crate::metadata::model::SupportedEntryStateV3,
    >,
    fault: Option<PublicationFault>,
) -> Result<Option<u64>, GripError> {
    revalidate(home, expected)?;
    if expected
        .complete
        .as_ref()
        .is_some_and(|state| &state.baselines == next_baselines)
        && expected.rebinding.outcome != crate::result::RebindingOutcome::RebindEligible
    {
        return Ok(None);
    }
    let generation = expected
        .accepted
        .generation
        .map_or(Some(0), |value| value.checked_add(1))
        .ok_or_else(|| {
            GripError::discovery_operational(
                "baseline_accept",
                "generation_exhausted",
                vec![home.path().join("state/state.json").display().to_string()],
                "accepted state generation cannot be advanced",
            )
        })?;
    let state_dir = home.path().join("state");
    ensure_dir(&state_dir)?;
    let next = accepted_v4_from_runtime(home, generation, next_baselines)?;
    let bytes = encode_v4(&next)?;
    preserve_prior_for_complete_publication(&state_dir, expected, generation, &bytes)?;
    write_v3_atomic(
        home,
        &state_dir,
        &state_dir.join("state.json"),
        &bytes,
        next_baselines,
        fault,
    )?;
    Ok(Some(generation))
}

fn preserve_prior_for_complete_publication(
    _state_dir: &Path,
    _expected: &StateSnapshot,
    _next_generation: u64,
    _next_bytes: &[u8],
) -> Result<(), GripError> {
    // State is current deployment evidence, not a history or recovery mechanism.
    Ok(())
}

fn write_v3_atomic(
    home: &ProjectPaths,
    directory: &Path,
    target: &Path,
    bytes: &[u8],
    expected: &std::collections::BTreeMap<
        crate::observation::model::EntryIdentity,
        crate::metadata::model::SupportedEntryStateV3,
    >,
    fault: Option<PublicationFault>,
) -> Result<(), GripError> {
    let temp = directory.join(format!(
        ".state.tmp-{}-{}",
        std::process::id(),
        TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temp)
            .map_err(|error| GripError::from_io("could not stage State V3", error))?;
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|error| GripError::from_io("could not sync staged State V3", error))?;
        if fault == Some(PublicationFault::CorruptV2Staging) {
            file.write_all(b"corrupt").map_err(|error| {
                GripError::from_io("could not inject State V3 corruption", error)
            })?;
            file.sync_all().map_err(|error| {
                GripError::from_io("could not sync injected State V3 corruption", error)
            })?;
        }
        file.seek(SeekFrom::Start(0))
            .map_err(|error| GripError::from_io("could not rewind staged State V3", error))?;
        let mut reread = Vec::new();
        file.read_to_end(&mut reread)
            .map_err(|error| GripError::from_io("could not reread staged State V3", error))?;
        let portable = decode_v4(&reread)?;
        let decoded = runtime_from_accepted_v4(home, &portable)?;
        if reread != bytes || &decoded.baselines != expected {
            return Err(GripError::CorruptState(
                "staged State V3 verification failed".into(),
            ));
        }
        if fault == Some(PublicationFault::BeforeV2StateRename) {
            return Err(GripError::Internal(
                "injected pre-rename State V3 failure".into(),
            ));
        }
        fs::rename(&temp, target)
            .map_err(|error| GripError::from_io("could not publish State V3", error))?;
        if fault == Some(PublicationFault::AfterV2StateRename) {
            return Err(GripError::discovery_operational(
                "baseline_accept",
                "publication_failure",
                vec![target.display().to_string()],
                "State V3 became visible but durability confirmation failed",
            )
            .with_publication_visible(true));
        }
        File::open(directory)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| GripError::from_io("could not sync State V3 directory", error))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

#[allow(dead_code)]
fn recovery_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| format!("{}Z", duration.as_secs()))
        .unwrap_or_else(|_| "0Z".into())
}

#[allow(dead_code)]
fn write_recovery_atomic(directory: &Path, target: &Path, bytes: &[u8]) -> Result<(), GripError> {
    let temp = directory.join(format!(
        ".state.tmp-{}-{}",
        std::process::id(),
        TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let mut attempt_identity = None;
    let result = (|| {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temp)
            .map_err(|error| GripError::from_io("could not stage accepted state", error))?;
        attempt_identity = Some(state_identity(&file.metadata().map_err(|error| {
            GripError::from_io("could not inspect new staged accepted state", error)
        })?));
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|error| GripError::from_io("could not sync staged accepted state", error))?;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|error| GripError::from_io("could not secure staged accepted state", error))?;
        verify_staged_path(&file, &temp)?;
        file.seek(SeekFrom::Start(0))
            .map_err(|error| GripError::from_io("could not rewind staged accepted state", error))?;
        let mut staged = Vec::new();
        file.read_to_end(&mut staged)
            .map_err(|error| GripError::from_io("could not reread staged accepted state", error))?;
        if staged != bytes {
            return Err(GripError::CorruptState(
                "staged state bytes changed before publication".into(),
            ));
        }
        verify_staged_path(&file, &temp)?;
        rustix::fs::renameat_with(
            rustix::fs::CWD,
            &temp,
            rustix::fs::CWD,
            target,
            rustix::fs::RenameFlags::NOREPLACE,
        )
        .map_err(|error| GripError::from_io("could not publish state recovery", error.into()))?;
        File::open(directory)
            .and_then(|file| file.sync_all())
            .map_err(|error| GripError::from_io("could not sync state directory", error))
    })();
    if result.is_err()
        && let (Some(expected), Ok(metadata)) = (attempt_identity, fs::symlink_metadata(&temp))
        && same_node(expected, state_identity(&metadata))
    {
        let _ = fs::remove_file(&temp);
    }
    result
}

#[allow(dead_code)]
fn same_node(first: StateFileIdentity, second: StateFileIdentity) -> bool {
    first.device == second.device && first.inode == second.inode
}

#[allow(dead_code)]
fn verify_staged_path(file: &File, path: &Path) -> Result<(), GripError> {
    let descriptor_metadata = file
        .metadata()
        .map_err(|error| GripError::from_io("could not inspect staged accepted state", error))?;
    let path_metadata = fs::symlink_metadata(path)
        .map_err(|error| GripError::from_io("could not inspect staged state path", error))?;
    if state_identity(&descriptor_metadata) != state_identity(&path_metadata)
        || descriptor_metadata.permissions().mode() & 0o7777 != 0o600
    {
        return Err(GripError::CorruptState(
            "staged state pathname no longer identifies the opened file".into(),
        ));
    }
    Ok(())
}

#[allow(dead_code)]
fn read_private_file(path: &Path) -> Result<Vec<u8>, GripError> {
    let descriptor = rustix::fs::open(
        path,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NOFOLLOW,
        rustix::fs::Mode::empty(),
    )
    .map_err(|error| GripError::from_io("could not open state recovery safely", error.into()))?;
    let mut file = File::from(descriptor);
    let metadata = file
        .metadata()
        .map_err(|error| GripError::from_io("could not inspect state recovery", error))?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.permissions().mode() & 0o7777 != 0o600
    {
        return Err(GripError::CorruptState(
            "state recovery must be a current-user-owned 0600 regular file".into(),
        ));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| GripError::from_io("could not read state recovery", error))?;
    Ok(bytes)
}
