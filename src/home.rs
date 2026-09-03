use crate::error::GripError;
use std::fs::{self, File, Metadata};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct GripHome(PathBuf);
impl GripHome {
    pub fn path(&self) -> &Path {
        &self.0
    }
}

pub trait HomeAccess {
    fn symlink_metadata(&self, path: &Path) -> std::io::Result<Metadata>;
    fn open_dir(&self, path: &Path) -> std::io::Result<File>;
}
pub struct OsHomeAccess;
impl HomeAccess for OsHomeAccess {
    fn symlink_metadata(&self, p: &Path) -> std::io::Result<Metadata> {
        fs::symlink_metadata(p)
    }
    fn open_dir(&self, p: &Path) -> std::io::Result<File> {
        File::open(p)
    }
}

pub fn select(
    override_value: Option<std::ffi::OsString>,
    default_home: Option<PathBuf>,
) -> Result<GripHome, GripError> {
    let path = match override_value {
        Some(value) => {
            if value.is_empty() {
                return Err(GripError::InvalidConfiguration(
                    "GRIP_HOME must not be empty".into(),
                ));
            }
            let path = PathBuf::from(value);
            if !path.is_absolute() {
                return Err(GripError::InvalidConfiguration(
                    "GRIP_HOME must be an absolute path".into(),
                ));
            }
            path
        }
        None => default_home
            .ok_or_else(|| GripError::InvalidConfiguration("home directory is unavailable".into()))?
            .join(".grip"),
    };
    Ok(GripHome(path))
}

pub fn validate_with(access: &dyn HomeAccess, home: &GripHome) -> Result<(), GripError> {
    let metadata = access
        .symlink_metadata(home.path())
        .map_err(|e| GripError::InvalidConfiguration(format!("Grip home is unavailable: {e}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(GripError::InvalidConfiguration(
            "Grip home must be a real directory".into(),
        ));
    }
    if metadata.uid() != rustix::process::geteuid().as_raw() {
        return Err(GripError::InvalidConfiguration(
            "Grip home must be owned by the current user".into(),
        ));
    }
    access
        .open_dir(home.path())
        .map_err(|e| GripError::InvalidConfiguration(format!("Grip home is inaccessible: {e}")))?;
    Ok(())
}
pub fn validate(home: &GripHome) -> Result<(), GripError> {
    validate_with(&OsHomeAccess, home)
}
