use crate::error::GripError;
use rustix::fs::{Mode, OFlags, open};
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
