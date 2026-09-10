//! Grip project selection, authority, and initialization.

pub mod init;

use crate::error::{GripError, ResultCategory};
use crate::home::UserHome;
use crate::mapping::ResolvedMapping;
use crate::registry::{ProjectDescriptorV2, decode_descriptor};
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

pub const METADATA_DIRECTORY: &str = ".grip";
pub const DESCRIPTOR_FILE: &str = "config.toml";
pub const IGNORE_FILE: &str = ".gitignore";
pub const STATE_DIRECTORY: &str = "state";
pub const CANONICAL_GITIGNORE: &[u8] = b"/state/\n";

/// Paths bound to one already-selected Grip project.
#[derive(Debug, Clone)]
pub struct ProjectPaths {
    metadata_dir: PathBuf,
    destination_home: PathBuf,
}

impl ProjectPaths {
    #[doc(hidden)]
    pub fn project_metadata(metadata_dir: PathBuf, destination_home: PathBuf) -> Self {
        Self {
            metadata_dir,
            destination_home,
        }
    }

    pub fn path(&self) -> &Path {
        &self.metadata_dir
    }

    pub(crate) fn destination_home(&self) -> Option<&Path> {
        Some(&self.destination_home)
    }

    pub(crate) fn project_root(&self) -> Result<&Path, GripError> {
        self.metadata_dir.parent().ok_or_else(|| {
            GripError::InvalidConfiguration("project metadata has no project root".into())
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectSelection {
    Explicit(PathBuf),
    Discovered(PathBuf),
    Independent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeEvidence {
    pub path: PathBuf,
    pub device: u64,
    pub inode: u64,
    pub mode: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectEvidence {
    pub root: NodeEvidence,
    pub metadata: NodeEvidence,
    pub descriptor: NodeEvidence,
    pub ignore: NodeEvidence,
    pub user_home: NodeEvidence,
    pub descriptor_bytes: Vec<u8>,
    pub descriptor_digest: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub root: PathBuf,
    pub metadata_dir: PathBuf,
    pub descriptor_path: PathBuf,
    pub ignore_path: PathBuf,
    pub state_dir: PathBuf,
    pub user_home: UserHome,
    pub descriptor: ProjectDescriptorV2,
    pub resolved_mappings: Vec<ResolvedMapping>,
    pub evidence: ProjectEvidence,
}

impl ProjectContext {
    pub fn select(
        selection: ProjectSelection,
        user_home: UserHome,
    ) -> Result<Option<Self>, GripError> {
        match selection {
            ProjectSelection::Independent => Ok(None),
            ProjectSelection::Explicit(path) => load_exact(&path, user_home).map(Some),
            ProjectSelection::Discovered(cwd) => discover(&cwd, user_home).map(Some),
        }
    }

    pub fn revalidate(&self) -> Result<(), GripError> {
        let current = load_exact(&self.root, self.user_home.clone())?;
        if current.evidence != self.evidence || current.user_home != self.user_home {
            return Err(project_error(
                "project_changed",
                "selected Grip project changed after discovery",
            ));
        }
        Ok(())
    }

    /// Project the already validated declarations and endpoints for read-only output.
    pub fn mapping_details(&self) -> Vec<crate::result::MappingResultDetails> {
        self.resolved_mappings
            .iter()
            .map(crate::result::MappingResultDetails::from)
            .collect()
    }
}

fn discover(cwd: &Path, user_home: UserHome) -> Result<ProjectContext, GripError> {
    let canonical_cwd = fs::canonicalize(cwd)
        .map_err(|error| GripError::from_io("could not resolve invocation directory", error))?;
    let mut projects = Vec::new();
    for ancestor in canonical_cwd.ancestors() {
        let candidate = ancestor.join(METADATA_DIRECTORY);
        match fs::symlink_metadata(&candidate) {
            Ok(_) => projects.push(load_exact(ancestor, user_home.clone())?),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(GripError::from_io(
                    "could not inspect project ancestor",
                    error,
                ));
            }
        }
    }
    match projects.len() {
        0 => Err(project_error(
            "project_not_found",
            "no Grip project exists in the invocation directory ancestry",
        )),
        1 => Ok(projects.remove(0)),
        _ => Err(project_error(
            "ambiguous_project",
            "multiple Grip projects exist in the invocation directory ancestry",
        )),
    }
}

fn load_exact(root: &Path, user_home: UserHome) -> Result<ProjectContext, GripError> {
    let root_metadata = safe_metadata(root, true, "project root")?;
    let canonical_root = fs::canonicalize(root)
        .map_err(|error| GripError::from_io("could not canonicalize project root", error))?;
    let metadata_dir = canonical_root.join(METADATA_DIRECTORY);
    let descriptor_path = metadata_dir.join(DESCRIPTOR_FILE);
    let ignore_path = metadata_dir.join(IGNORE_FILE);
    let metadata = safe_metadata(&metadata_dir, true, "project metadata directory")?;
    let descriptor_metadata = safe_metadata(&descriptor_path, false, "project descriptor")?;
    let ignore_metadata = safe_metadata(&ignore_path, false, "project ignore file")?;
    let user_home_metadata = safe_metadata(user_home.path(), true, "destination home")?;
    let descriptor_bytes = fs::read(&descriptor_path)
        .map_err(|error| GripError::from_io("could not read project descriptor", error))?;
    let ignore_bytes = fs::read(&ignore_path)
        .map_err(|error| GripError::from_io("could not read project ignore file", error))?;
    if ignore_bytes != CANONICAL_GITIGNORE {
        return Err(project_error(
            "invalid_project_metadata",
            ".grip/.gitignore must contain exactly /state/ and a terminating newline",
        ));
    }
    let descriptor_text = std::str::from_utf8(&descriptor_bytes).map_err(|_| {
        project_error(
            "invalid_project_metadata",
            "project descriptor must be UTF-8",
        )
    })?;
    let descriptor = decode_descriptor(descriptor_text).map_err(project_metadata_error)?;
    let resolved_mappings = descriptor
        .resolve(&canonical_root, user_home.path(), "project_selection")
        .map_err(project_metadata_error)?;
    let evidence = ProjectEvidence {
        root: node_evidence(canonical_root.clone(), &root_metadata),
        metadata: node_evidence(metadata_dir.clone(), &metadata),
        descriptor: node_evidence(descriptor_path.clone(), &descriptor_metadata),
        ignore: node_evidence(ignore_path.clone(), &ignore_metadata),
        user_home: node_evidence(user_home.path().to_path_buf(), &user_home_metadata),
        descriptor_digest: Sha256::digest(&descriptor_bytes).into(),
        descriptor_bytes,
    };
    Ok(ProjectContext {
        root: canonical_root,
        metadata_dir: metadata_dir.clone(),
        descriptor_path,
        ignore_path,
        state_dir: metadata_dir.join(STATE_DIRECTORY),
        user_home,
        descriptor,
        resolved_mappings,
        evidence,
    })
}

fn safe_metadata(path: &Path, directory: bool, label: &str) -> Result<fs::Metadata, GripError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => {
            project_error("invalid_project_root", &format!("{label} is unavailable"))
        }
        _ => GripError::from_io(&format!("could not inspect {label}"), error),
    })?;
    let mode = metadata.permissions().mode() & 0o7777;
    let valid_kind = if directory {
        metadata.is_dir()
    } else {
        metadata.is_file()
    };
    if metadata.file_type().is_symlink()
        || !valid_kind
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || mode & 0o022 != 0
        || mode & 0o400 == 0
    {
        return Err(project_error(
            "invalid_project_metadata",
            &format!("{label} is not a safe current-user-owned node"),
        ));
    }
    Ok(metadata)
}

fn node_evidence(path: PathBuf, metadata: &fs::Metadata) -> NodeEvidence {
    NodeEvidence {
        path,
        device: metadata.dev(),
        inode: metadata.ino(),
        mode: metadata.permissions().mode() & 0o7777,
    }
}

fn project_error(reason: &str, message: &str) -> GripError {
    GripError::lifecycle(
        "project_selection",
        reason,
        ResultCategory::InvalidConfiguration,
        message,
    )
}

fn project_metadata_error(error: GripError) -> GripError {
    let category = error.category();
    GripError::lifecycle(
        "project_selection",
        "invalid_project_metadata",
        category,
        error.to_string(),
    )
}
