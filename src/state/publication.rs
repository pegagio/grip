use super::lock::PublicationLock;
use super::{StateEnvelopeV1, decode, encode};
use crate::error::GripError;
use crate::home::GripHome;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationFault {
    BeforeRecoveryRename,
    BeforeStateRename,
}

fn ensure_dir(path: &Path) -> Result<(), GripError> {
    match fs::symlink_metadata(path) {
        Ok(m)
            if !m.file_type().is_symlink()
                && m.is_dir()
                && m.uid() == rustix::process::geteuid().as_raw() =>
        {
            Ok(())
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

fn read_valid(path: &Path) -> Result<(StateEnvelopeV1, Vec<u8>), GripError> {
    let m = fs::symlink_metadata(path)
        .map_err(|e| GripError::CorruptState(format!("state is unavailable: {e}")))?;
    if m.file_type().is_symlink() || !m.is_file() || m.uid() != rustix::process::geteuid().as_raw()
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
