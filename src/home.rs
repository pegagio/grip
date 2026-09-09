use crate::error::GripError;
use std::fs::{self, File, Metadata};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Canonical invoking-user home used only to resolve portable destinations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserHome(PathBuf);

impl UserHome {
    pub fn path(&self) -> &Path {
        &self.0
    }
}

/// Select and validate the invoking user's destination home.
pub fn select_user_home(value: Option<PathBuf>) -> Result<UserHome, GripError> {
    let path = value.or_else(home::home_dir).ok_or_else(|| {
        GripError::InvalidConfiguration("invoking user home is unavailable".into())
    })?;
    if !path.is_absolute() {
        return Err(GripError::InvalidConfiguration(
            "invoking user home must be absolute".into(),
        ));
    }
    validate_user_home_with(&OsHomeAccess, &path)?;
    fs::canonicalize(&path)
        .map(UserHome)
        .map_err(|error| GripError::InvalidConfiguration(format!("home is unavailable: {error}")))
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

fn validate_user_home_with(access: &dyn HomeAccess, path: &Path) -> Result<(), GripError> {
    let metadata = access
        .symlink_metadata(path)
        .map_err(|e| GripError::InvalidConfiguration(format!("home is unavailable: {e}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(GripError::InvalidConfiguration(
            "home must be a real directory".into(),
        ));
    }
    if metadata.uid() != rustix::process::geteuid().as_raw() {
        return Err(GripError::InvalidConfiguration(
            "home must be owned by the current user".into(),
        ));
    }
    access
        .open_dir(path)
        .map_err(|e| GripError::InvalidConfiguration(format!("home is inaccessible: {e}")))?;
    Ok(())
}
