use crate::error::GripError;
use std::fs::{self, File, OpenOptions};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

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
                fs::set_permissions(path, fs::Permissions::from_mode(0o600))
                    .map_err(|e| GripError::from_io("could not secure state lock", e))?;
                file
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let meta = fs::symlink_metadata(path)
                    .map_err(|e| GripError::from_io("could not inspect state lock", e))?;
                if meta.file_type().is_symlink()
                    || !meta.is_file()
                    || meta.uid() != rustix::process::geteuid().as_raw()
                    || meta.permissions().mode() & 0o077 != 0
                {
                    return Err(GripError::CorruptState("unsafe state lock node".into()));
                }
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(path)
                    .map_err(|e| GripError::from_io("could not open state lock", e))?
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
