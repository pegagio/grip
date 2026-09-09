use crate::error::GripError;
use rustix::fs::{Mode, OFlags, open};
use std::fs::{self, File, OpenOptions};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

/// Create and validate the private project-local lock directory, then return one lock path.
pub fn project_lock_path(
    home: &crate::project::ProjectPaths,
    name: &str,
) -> Result<std::path::PathBuf, GripError> {
    let state = crate::state::publication::prepare_directory(home)?;
    let locks = state.join("locks");
    match fs::symlink_metadata(&locks) {
        Ok(metadata)
            if metadata.is_dir()
                && !metadata.file_type().is_symlink()
                && metadata.uid() == rustix::process::geteuid().as_raw()
                && metadata.permissions().mode() & 0o7777 == 0o700 => {}
        Ok(_) => {
            return Err(GripError::CorruptState(
                "unsafe project lock directory".into(),
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::DirBuilder::new()
                .recursive(false)
                .mode(0o700)
                .create(&locks)
                .map_err(|error| {
                    GripError::from_io("could not create project lock directory", error)
                })?;
            fs::set_permissions(&locks, fs::Permissions::from_mode(0o700)).map_err(|error| {
                GripError::from_io("could not secure project lock directory", error)
            })?;
        }
        Err(error) => {
            return Err(GripError::from_io(
                "could not inspect project lock directory",
                error,
            ));
        }
    }
    Ok(locks.join(name))
}

pub struct PublicationLock(File);
impl PublicationLock {
    pub fn acquire(path: &Path) -> Result<Self, GripError> {
        let file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
        {
            Ok(file) => {
                file.set_permissions(fs::Permissions::from_mode(0o600))
                    .map_err(|e| GripError::from_io("could not secure state lock", e))?;
                file
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let descriptor = open(
                    path,
                    OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOFOLLOW,
                    Mode::empty(),
                )
                .map_err(|e| GripError::from_io("could not open state lock safely", e.into()))?;
                let file = File::from(descriptor);
                let meta = file
                    .metadata()
                    .map_err(|e| GripError::from_io("could not inspect state lock", e))?;
                if !meta.is_file()
                    || meta.uid() != rustix::process::geteuid().as_raw()
                    || meta.permissions().mode() & 0o777 != 0o600
                {
                    return Err(GripError::CorruptState("unsafe state lock node".into()));
                }
                file
            }
            Err(error) => return Err(GripError::from_io("could not create state lock", error)),
        };
        match file.try_lock() {
            Ok(()) => Ok(Self(file)),
            Err(std::fs::TryLockError::WouldBlock) => Err(GripError::StateContention),
            Err(std::fs::TryLockError::Error(e)) => {
                Err(GripError::from_io("could not lock state", e))
            }
        }
    }
}
impl Drop for PublicationLock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}
