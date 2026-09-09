//! Atomic initialization of a Grip project metadata directory.

use crate::error::{GripError, ResultCategory};
use crate::project::{CANONICAL_GITIGNORE, DESCRIPTOR_FILE, IGNORE_FILE, METADATA_DIRECTORY};
use crate::registry::{ProjectDescriptorV2, decode_descriptor, encode_descriptor};
use rustix::fs::{CWD, RenameFlags, renameat_with};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static STAGING_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitializationOutcome {
    Initialized,
    AlreadyInitialized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitializationResult {
    pub root: PathBuf,
    pub outcome: InitializationOutcome,
}

pub fn initialize(target: Option<&Path>) -> Result<InitializationResult, GripError> {
    initialize_with_fault(target, None)
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitializationFault {
    BeforePublish,
}

#[doc(hidden)]
pub fn initialize_with_fault(
    target: Option<&Path>,
    fault: Option<InitializationFault>,
) -> Result<InitializationResult, GripError> {
    let submitted = match target {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => std::env::current_dir()
            .map_err(|error| GripError::from_io("could not read invocation directory", error))?
            .join(path),
        None => std::env::current_dir()
            .map_err(|error| GripError::from_io("could not read invocation directory", error))?,
    };
    let target_metadata = safe_directory(&submitted, "initialization target")?;
    let root = fs::canonicalize(&submitted).map_err(|error| {
        GripError::from_io("could not canonicalize initialization target", error)
    })?;
    reject_enclosing_project(&root)?;
    let metadata_path = root.join(METADATA_DIRECTORY);
    match fs::symlink_metadata(&metadata_path) {
        Ok(_) => {
            validate_existing(&metadata_path)?;
            return Ok(InitializationResult {
                root,
                outcome: InitializationOutcome::AlreadyInitialized,
            });
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(GripError::from_io(
                "could not inspect project metadata",
                error,
            ));
        }
    }

    let staging = root.join(format!(
        ".grip-init-{}-{}",
        std::process::id(),
        STAGING_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let result = stage_and_publish(&root, &staging, &metadata_path, &target_metadata, fault);
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    match result {
        Ok(()) => Ok(InitializationResult {
            root,
            outcome: InitializationOutcome::Initialized,
        }),
        Err(_error) if fs::symlink_metadata(&metadata_path).is_ok() => {
            validate_existing(&metadata_path)?;
            Ok(InitializationResult {
                root,
                outcome: InitializationOutcome::AlreadyInitialized,
            })
        }
        Err(error) => Err(error),
    }
}

fn stage_and_publish(
    root: &Path,
    staging: &Path,
    metadata_path: &Path,
    expected_root: &fs::Metadata,
    fault: Option<InitializationFault>,
) -> Result<(), GripError> {
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder.create(staging).map_err(|error| {
        initialization_error(
            "publication_failure",
            &format!("could not create private initialization staging: {error}"),
        )
    })?;
    let descriptor = encode_descriptor(&ProjectDescriptorV2::empty())?;
    write_staged(&staging.join(DESCRIPTOR_FILE), &descriptor)?;
    write_staged(&staging.join(IGNORE_FILE), CANONICAL_GITIGNORE)?;
    File::open(staging)
        .and_then(|file| file.sync_all())
        .map_err(|error| GripError::from_io("could not sync initialization staging", error))?;
    let current_root = safe_directory(root, "initialization target")?;
    if current_root.dev() != expected_root.dev() || current_root.ino() != expected_root.ino() {
        return Err(initialization_error(
            "project_changed",
            "initialization target changed before publication",
        ));
    }
    if fault == Some(InitializationFault::BeforePublish) {
        return Err(initialization_error(
            "publication_failure",
            "injected initialization failure before publication",
        ));
    }
    renameat_with(CWD, staging, CWD, metadata_path, RenameFlags::NOREPLACE).map_err(|error| {
        initialization_error(
            "publication_failure",
            &format!("could not publish project metadata: {error}"),
        )
    })?;
    File::open(root)
        .and_then(|file| file.sync_all())
        .map_err(|error| GripError::from_io("could not sync initialized project root", error))
}

fn write_staged(path: &Path, bytes: &[u8]) -> Result<(), GripError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o644)
        .open(path)
        .map_err(|error| GripError::from_io("could not create initialization metadata", error))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| GripError::from_io("could not sync initialization metadata", error))
}

fn reject_enclosing_project(root: &Path) -> Result<(), GripError> {
    let mut ancestor = root.parent();
    while let Some(path) = ancestor {
        match fs::symlink_metadata(path.join(METADATA_DIRECTORY)) {
            Ok(_) => {
                return Err(initialization_error(
                    "nested_project",
                    "initialization target is inside another Grip project",
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(GripError::from_io(
                    "could not inspect enclosing project boundary",
                    error,
                ));
            }
        }
        ancestor = path.parent();
    }
    Ok(())
}

fn validate_existing(metadata_path: &Path) -> Result<(), GripError> {
    safe_directory(metadata_path, "project metadata directory")?;
    let descriptor_path = metadata_path.join(DESCRIPTOR_FILE);
    let ignore_path = metadata_path.join(IGNORE_FILE);
    safe_file(&descriptor_path, "project descriptor")?;
    safe_file(&ignore_path, "project ignore file")?;
    let descriptor_bytes = fs::read(&descriptor_path)
        .map_err(|error| GripError::from_io("could not read project descriptor", error))?;
    let descriptor = std::str::from_utf8(&descriptor_bytes).map_err(|_| {
        initialization_error(
            "invalid_project_metadata",
            "project descriptor must be UTF-8",
        )
    })?;
    decode_descriptor(descriptor)?;
    if fs::read(&ignore_path)
        .map_err(|error| GripError::from_io("could not read project ignore file", error))?
        != CANONICAL_GITIGNORE
    {
        return Err(initialization_error(
            "invalid_project_metadata",
            ".grip/.gitignore is not canonical",
        ));
    }
    for entry in fs::read_dir(metadata_path)
        .map_err(|error| GripError::from_io("could not inspect project metadata", error))?
    {
        let name = entry
            .map_err(|error| GripError::from_io("could not inspect project metadata", error))?
            .file_name();
        if name != DESCRIPTOR_FILE && name != IGNORE_FILE && name != "state" {
            return Err(initialization_error(
                "invalid_project_metadata",
                "project metadata contains an unexpected entry",
            ));
        }
    }
    if let Ok(metadata) = fs::symlink_metadata(metadata_path.join("state"))
        && (metadata.file_type().is_symlink() || !metadata.is_dir())
    {
        return Err(initialization_error(
            "invalid_project_metadata",
            "project state is not a safe directory",
        ));
    }
    Ok(())
}

fn safe_directory(path: &Path, label: &str) -> Result<fs::Metadata, GripError> {
    safe_node(path, label, true)
}

fn safe_file(path: &Path, label: &str) -> Result<fs::Metadata, GripError> {
    safe_node(path, label, false)
}

fn safe_node(path: &Path, label: &str, directory: bool) -> Result<fs::Metadata, GripError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => {
            initialization_error("invalid_project_root", &format!("{label} does not exist"))
        }
        _ => GripError::from_io(&format!("could not inspect {label}"), error),
    })?;
    let kind_valid = if directory {
        metadata.is_dir()
    } else {
        metadata.is_file()
    };
    let mode = metadata.permissions().mode() & 0o7777;
    if metadata.file_type().is_symlink()
        || !kind_valid
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || mode & 0o022 != 0
        || mode & 0o400 == 0
    {
        return Err(initialization_error(
            "invalid_project_metadata",
            &format!("{label} is not a safe current-user-owned node"),
        ));
    }
    Ok(metadata)
}

fn initialization_error(reason: &str, message: &str) -> GripError {
    GripError::lifecycle(
        "init",
        reason,
        ResultCategory::InvalidConfiguration,
        message,
    )
}
