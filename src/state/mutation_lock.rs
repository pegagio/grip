//! Persistent owner-aware advisory coordination for Grip writers.

use crate::error::GripError;
use crate::project::ProjectPaths;
use rustix::fs::{Mode, OFlags, open};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::time::{SystemTime, UNIX_EPOCH};

/// Diagnostic metadata stored in the stable mutation-lock file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationLockOwner {
    pub schema_version: u8,
    pub pid: u32,
    pub operation: String,
    pub acquired_at: String,
}

/// A held per-user writer lock.
#[derive(Debug)]
pub struct MutationLock {
    file: File,
}

impl MutationLock {
    /// Acquire the stable lock without waiting and publish current owner metadata.
    pub fn acquire(home: &ProjectPaths, operation: &str) -> Result<Self, GripError> {
        let path = crate::state::lock::project_lock_path(home, "mutation.lock")?;
        let descriptor = open(
            &path,
            OFlags::RDWR | OFlags::CREATE | OFlags::CLOEXEC | OFlags::NOFOLLOW,
            Mode::from_raw_mode(0o600),
        )
        .map_err(|error| {
            if error == rustix::io::Errno::LOOP {
                GripError::CorruptState("mutation lock must not be a symbolic link".into())
            } else {
                GripError::from_io("could not open mutation lock safely", error.into())
            }
        })?;
        let mut file = File::from(descriptor);
        let metadata = file
            .metadata()
            .map_err(|error| GripError::from_io("could not inspect mutation lock", error))?;
        if !metadata.is_file()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.permissions().mode() & 0o7777 != 0o600
        {
            return Err(GripError::CorruptState(
                "mutation lock must be a current-user-owned regular file with mode 0600".into(),
            ));
        }
        match file.try_lock() {
            Ok(()) => {}
            Err(std::fs::TryLockError::WouldBlock) => {
                return Err(GripError::MutationContention {
                    requested_operation: operation.into(),
                    owner: read_owner(&mut file),
                });
            }
            Err(std::fs::TryLockError::Error(error)) => {
                return Err(GripError::from_io("could not acquire mutation lock", error));
            }
        }
        let owner = MutationLockOwner {
            schema_version: 1,
            pid: std::process::id(),
            operation: operation.into(),
            acquired_at: unix_timestamp(),
        };
        let bytes = serde_json::to_vec(&owner).map_err(|error| {
            GripError::Internal(format!("could not encode lock owner: {error}"))
        })?;
        file.set_len(0)
            .and_then(|_| file.seek(SeekFrom::Start(0)).map(|_| ()))
            .and_then(|_| file.write_all(&bytes))
            .and_then(|_| file.sync_all())
            .map_err(|error| GripError::from_io("could not publish mutation lock owner", error))?;
        File::open(path.parent().expect("lock path has a parent"))
            .and_then(|directory| directory.sync_all())
            .map_err(|error| GripError::from_io("could not sync project lock directory", error))?;
        Ok(Self { file })
    }
}

impl Drop for MutationLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

fn read_owner(file: &mut File) -> Option<MutationLockOwner> {
    let mut bytes = Vec::new();
    file.seek(SeekFrom::Start(0)).ok()?;
    file.read_to_end(&mut bytes).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn unix_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| format!("{}Z", duration.as_secs()))
        .unwrap_or_else(|_| "0Z".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::ProjectPaths;
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use tempfile::tempdir;

    fn home() -> (tempfile::TempDir, ProjectPaths) {
        let root = tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let home = crate::project::ProjectPaths::project_metadata(
            root.path().to_path_buf(),
            root.path().parent().unwrap().to_path_buf(),
        );
        (root, home)
    }

    #[test]
    fn acquire_reports_owner_when_another_writer_holds_lock() {
        let (_root, home) = home();
        let _first = MutationLock::acquire(&home, "push").unwrap();
        let error = MutationLock::acquire(&home, "mapping_add").unwrap_err();
        match error {
            GripError::MutationContention {
                owner: Some(owner), ..
            } => {
                assert_eq!(owner.operation, "push");
                assert_eq!(owner.schema_version, 1);
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn acquire_overwrites_stale_metadata_after_lock_is_released() {
        let (_root, home) = home();
        drop(MutationLock::acquire(&home, "push").unwrap());
        let _next = MutationLock::acquire(&home, "baseline_accept").unwrap();
        let path = crate::state::lock::project_lock_path(&home, "mutation.lock").unwrap();
        let owner: MutationLockOwner = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(owner.operation, "baseline_accept");
    }

    #[test]
    fn acquire_rejects_unsafe_lock_nodes() {
        let (_root, home) = home();
        let target = home.path().join("target");
        fs::write(&target, b"").unwrap();
        let path = crate::state::lock::project_lock_path(&home, "mutation.lock").unwrap();
        symlink(&target, path).unwrap();
        assert!(matches!(
            MutationLock::acquire(&home, "push"),
            Err(GripError::CorruptState(_))
        ));
    }

    #[test]
    fn acquire_overwrites_malformed_stale_metadata() {
        let (_root, home) = home();
        let path = crate::state::lock::project_lock_path(&home, "mutation.lock").unwrap();
        fs::write(&path, b"not-json").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let _lock = MutationLock::acquire(&home, "push").unwrap();
        let owner: MutationLockOwner = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(owner.operation, "push");
    }
}
