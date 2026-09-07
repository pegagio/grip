use super::lock::PublicationLock;
use super::{AcceptedState, StateEnvelopeV1, decode, decode_accepted, encode, encode_v2};
use crate::error::GripError;
use crate::home::GripHome;
use sha2::Digest;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt, symlink};
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
    pub bytes: Option<Vec<u8>>,
    identity: Option<StateFileIdentity>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationFault {
    BeforeRecoveryRename,
    BeforeStateRename,
    CorruptV2Staging,
    BeforeV2StateRename,
    AfterV2StateRename,
    SubstituteV2StagingFile,
    SubstituteV2StagingSymlink,
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
pub fn prepare_directory(home: &GripHome) -> Result<std::path::PathBuf, GripError> {
    let state_dir = home.path().join("state");
    ensure_dir(&state_dir)?;
    Ok(state_dir)
}

fn read_valid(path: &Path) -> Result<(StateEnvelopeV1, Vec<u8>), GripError> {
    let m = fs::symlink_metadata(path)
        .map_err(|e| GripError::CorruptState(format!("state is unavailable: {e}")))?;
    if m.file_type().is_symlink()
        || !m.is_file()
        || m.uid() != rustix::process::geteuid().as_raw()
        || m.permissions().mode() & 0o7777 != 0o600
    {
        return Err(GripError::CorruptState("unsafe state file".into()));
    }
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|mut f| f.read_to_end(&mut bytes))
        .map_err(|e| GripError::from_io("could not read state", e))?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| GripError::CorruptState("state must be UTF-8 JSON".into()))?;
    Ok((decode(text)?, bytes))
}

fn state_identity(metadata: &fs::Metadata) -> StateFileIdentity {
    StateFileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        mode: metadata.permissions().mode() & 0o7777,
    }
}

/// Load absent, V1, or V2 accepted state without creating state artifacts.
pub fn load(home: &GripHome) -> Result<StateSnapshot, GripError> {
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
    let mut accepted = decode_accepted(Some(&bytes))?;
    accepted.accepted_bytes = Some(bytes.clone());
    Ok(StateSnapshot {
        accepted,
        bytes: Some(bytes),
        identity: Some(state_identity(&metadata)),
    })
}

/// Reject drift from a previously loaded accepted-state snapshot.
pub fn revalidate(home: &GripHome, expected: &StateSnapshot) -> Result<(), GripError> {
    let current = load(home)?;
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
pub fn publish_accepted_locked(
    home: &GripHome,
    expected: &StateSnapshot,
    next: &AcceptedState,
) -> Result<Option<u64>, GripError> {
    publish_accepted_locked_with_fault(home, expected, next, None)
}

/// Restore exact, integrity-valid State V2 bytes while the caller holds the mutation lock.
pub(crate) fn restore_exact(
    home: &GripHome,
    bytes: &[u8],
    expected_post_generation: u64,
    expected_post_digest: &str,
) -> Result<(), GripError> {
    let recovered = decode_accepted(Some(bytes))?;
    if recovered.generation.is_none() {
        return Err(GripError::CorruptState(
            "recovered state has no generation".into(),
        ));
    }
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
    let _lock = PublicationLock::acquire(&directory.join("state.lock"))?;
    write_v2_atomic(
        &directory,
        &directory.join("state.json"),
        bytes,
        Some(&recovered.baselines),
        None,
    )?;
    let reread = read_private_file(&directory.join("state.json"))?;
    if reread != bytes || decode_accepted(Some(&reread))? != recovered {
        return Err(GripError::CorruptState(
            "restored state verification failed".into(),
        ));
    }
    Ok(())
}

#[doc(hidden)]
pub fn publish_accepted_locked_with_fault(
    home: &GripHome,
    expected: &StateSnapshot,
    next: &AcceptedState,
    fault: Option<PublicationFault>,
) -> Result<Option<u64>, GripError> {
    revalidate(home, expected)?;
    if expected.accepted.baselines == next.baselines {
        return Ok(None);
    }
    let generation = match expected.accepted.generation {
        None => 0,
        Some(value) => value.checked_add(1).ok_or_else(|| {
            GripError::discovery_operational(
                "baseline_accept",
                "generation_exhausted",
                vec![home.path().join("state/state.json").display().to_string()],
                "accepted state generation cannot be advanced",
            )
        })?,
    };
    let state_dir = home.path().join("state");
    ensure_dir(&state_dir)?;
    let target = state_dir.join("state.json");
    let next_bytes = encode_v2(next, generation)?;
    if let Some(bytes) = expected.bytes.as_deref() {
        let recovery_root = state_dir.join("recovery");
        ensure_dir(&recovery_root)?;
        let prior_generation = expected.accepted.generation.ok_or_else(|| {
            GripError::CorruptState("accepted state bytes require a generation".into())
        })?;
        let generation_dir = recovery_root.join(format!("generation-{prior_generation}"));
        ensure_dir(&generation_dir)?;
        let recovery = generation_dir.join("state.json");
        let created = !recovery.exists();
        if recovery.exists() {
            if read_private_file(&recovery)? != bytes {
                return Err(GripError::CorruptState(
                    "recovery generation collision".into(),
                ));
            }
        } else {
            write_v2_atomic(&generation_dir, &recovery, bytes, None, None)?;
        }
        let manifest_path = generation_dir.join("manifest.json");
        if created && !manifest_path.exists() {
            let prior_digest = format!("{:x}", sha2::Sha256::digest(bytes));
            let next_digest = format!("{:x}", sha2::Sha256::digest(&next_bytes));
            let manifest = crate::recovery::model::RecoveryEnvelopeV1::new(
                crate::recovery::model::RecoveryManifestPayloadV1 {
                    reference: crate::recovery::model::RecoveryRef::AcceptedState {
                        generation: prior_generation,
                        digest: prior_digest.clone(),
                    },
                    kind: crate::recovery::model::RecoveryKind::AcceptedState,
                    created_at: recovery_timestamp(),
                    origin_operation: None,
                    origin_transition: "state_publication".into(),
                    managed_identity: None,
                    bound_side: None,
                    bound_target: None,
                    prior_evidence: serde_json::json!({"generation": prior_generation, "sha256": prior_digest}),
                    expected_post_evidence: serde_json::json!({"generation": generation, "sha256": next_digest}),
                    byte_count: bytes.len() as u64,
                    payload_ref: "state.json".into(),
                },
            )?;
            crate::operation::publication::publish_new_component(
                &generation_dir,
                "manifest.json",
                &crate::recovery::model::encode(&manifest)?,
            )?;
        }
    }
    write_v2_atomic(
        &state_dir,
        &target,
        &next_bytes,
        Some(&next.baselines),
        fault,
    )?;
    Ok(Some(generation))
}

fn recovery_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| format!("{}Z", duration.as_secs()))
        .unwrap_or_else(|_| "0Z".into())
}

fn write_v2_atomic(
    directory: &Path,
    target: &Path,
    bytes: &[u8],
    expected_baselines: Option<
        &std::collections::BTreeMap<
            crate::observation::model::EntryIdentity,
            crate::observation::model::SupportedState,
        >,
    >,
    fault: Option<PublicationFault>,
) -> Result<(), GripError> {
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
        match fault {
            Some(PublicationFault::SubstituteV2StagingFile) => {
                fs::remove_file(&temp).map_err(|error| {
                    GripError::from_io("could not inject staged file substitution", error)
                })?;
                fs::write(&temp, b"substituted").map_err(|error| {
                    GripError::from_io("could not inject staged file substitution", error)
                })?;
                fs::set_permissions(&temp, fs::Permissions::from_mode(0o600)).map_err(|error| {
                    GripError::from_io("could not secure substituted staged file", error)
                })?;
            }
            Some(PublicationFault::SubstituteV2StagingSymlink) => {
                fs::remove_file(&temp).map_err(|error| {
                    GripError::from_io("could not inject staged symlink substitution", error)
                })?;
                symlink("state.json", &temp).map_err(|error| {
                    GripError::from_io("could not inject staged symlink substitution", error)
                })?;
            }
            _ => {}
        }
        if fault == Some(PublicationFault::CorruptV2Staging) {
            file.write_all(b"corrupt").map_err(|error| {
                GripError::from_io("could not inject staged state corruption", error)
            })?;
            file.sync_all().map_err(|error| {
                GripError::from_io("could not sync injected staged state corruption", error)
            })?;
        }
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
        if let Some(expected) = expected_baselines {
            let decoded = decode_accepted(Some(&staged))?;
            if &decoded.baselines != expected {
                return Err(GripError::CorruptState(
                    "staged state semantic verification failed".into(),
                ));
            }
        }
        verify_staged_path(&file, &temp)?;
        if expected_baselines.is_none() {
            rustix::fs::renameat_with(
                rustix::fs::CWD,
                &temp,
                rustix::fs::CWD,
                target,
                rustix::fs::RenameFlags::NOREPLACE,
            )
            .map_err(|error| {
                GripError::from_io("could not publish state recovery", error.into())
            })?;
        } else {
            if fault == Some(PublicationFault::BeforeV2StateRename) {
                return Err(GripError::Internal(
                    "injected pre-rename accepted-state failure".into(),
                ));
            }
            fs::rename(&temp, target)
                .map_err(|error| GripError::from_io("could not publish accepted state", error))?;
        }
        if fault == Some(PublicationFault::AfterV2StateRename) {
            return Err(GripError::discovery_operational(
                "baseline_accept",
                "publication_failure",
                Vec::new(),
                "accepted state is visible but directory durability could not be confirmed",
            )
            .with_publication_visible(true));
        }
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

fn same_node(first: StateFileIdentity, second: StateFileIdentity) -> bool {
    first.device == second.device && first.inode == second.inode
}

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

fn write_atomic(
    directory: &Path,
    target: &Path,
    bytes: &[u8],
    fail_before_rename: bool,
) -> Result<(), GripError> {
    let temp = directory.join(format!(
        ".state.tmp-{}-{}",
        std::process::id(),
        TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temp)
            .map_err(|e| GripError::from_io("could not stage state", e))?;
        f.write_all(bytes)
            .and_then(|_| f.sync_all())
            .map_err(|e| GripError::from_io("could not sync staged state", e))?;
        fs::set_permissions(&temp, fs::Permissions::from_mode(0o600))
            .map_err(|e| GripError::from_io("could not secure staged state", e))?;
        let staged =
            fs::read(&temp).map_err(|e| GripError::from_io("could not reread staged state", e))?;
        let text = std::str::from_utf8(&staged)
            .map_err(|_| GripError::CorruptState("staged state is not UTF-8".into()))?;
        decode(text)?;
        if fail_before_rename {
            return Err(GripError::Internal(
                "injected pre-rename publication failure".into(),
            ));
        }
        fs::rename(&temp, target).map_err(|e| GripError::from_io("could not publish state", e))?;
        File::open(directory)
            .and_then(|d| d.sync_all())
            .map_err(|e| GripError::from_io("could not sync state directory", e))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

pub fn publish(home: &GripHome, next: &StateEnvelopeV1) -> Result<(), GripError> {
    publish_with_fault(home, next, None)
}

#[doc(hidden)]
pub fn publish_with_fault(
    home: &GripHome,
    next: &StateEnvelopeV1,
    fault: Option<PublicationFault>,
) -> Result<(), GripError> {
    next.validate()?;
    let state_dir = home.path().join("state");
    ensure_dir(&state_dir)?;
    let _lock = PublicationLock::acquire(&state_dir.join("state.lock"))?;
    ensure_dir(&state_dir)?;
    let target = state_dir.join("state.json");
    if target.exists() {
        let (prior, bytes) = read_valid(&target)?;
        if next.payload.generation <= prior.payload.generation {
            return Err(GripError::CorruptState(
                "state generation must increase".into(),
            ));
        }
        let recovery_root = state_dir.join("recovery");
        ensure_dir(&recovery_root)?;
        let generation_dir = recovery_root.join(format!("generation-{}", prior.payload.generation));
        ensure_dir(&generation_dir)?;
        let recovery = generation_dir.join("state.json");
        if recovery.exists() {
            if fs::read(&recovery)
                .map_err(|e| GripError::from_io("could not read recovery state", e))?
                != bytes
            {
                return Err(GripError::CorruptState(
                    "recovery generation collision".into(),
                ));
            }
        } else {
            write_atomic(
                &generation_dir,
                &recovery,
                &bytes,
                fault == Some(PublicationFault::BeforeRecoveryRename),
            )?;
        }
    }
    write_atomic(
        &state_dir,
        &target,
        &encode(next)?,
        fault == Some(PublicationFault::BeforeStateRename),
    )
}
