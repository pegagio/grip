//! Descriptor-bound fingerprints for supported filesystem entries.

use super::model::{
    ContentFingerprint, DestinationLeafLinkEvidence, DiagnosticEvidence, SupportedState,
};
use crate::discovery::model::NodeKind;
use crate::error::GripError;
use rustix::fd::OwnedFd;
use rustix::fs::{Mode, OFlags, fstat, open};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Construct a complete Feature 009 fingerprint without following the final path component.
pub fn inspect_complete(
    path: &Path,
    expected_kind: NodeKind,
) -> Result<super::model::CompleteObservedState, GripError> {
    if !matches!(expected_kind, NodeKind::File | NodeKind::Directory) {
        return Err(GripError::Internal(
            "unsupported nodes are not fingerprinted".into(),
        ));
    }
    let mut flags = OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK;
    if expected_kind == NodeKind::Directory {
        flags |= OFlags::DIRECTORY;
    }
    let descriptor = open(path, flags, Mode::empty()).map_err(|error| {
        GripError::from_io(
            "could not open entry for complete fingerprinting",
            error.into(),
        )
    })?;
    inspect_complete_descriptor(descriptor, expected_kind)
}

fn inspect_complete_descriptor(
    descriptor: OwnedFd,
    expected_kind: NodeKind,
) -> Result<super::model::CompleteObservedState, GripError> {
    let before = fstat(&descriptor)
        .map_err(|error| GripError::from_io("could not inspect entry descriptor", error.into()))?;
    let actual_kind = rustix::fs::FileType::from_raw_mode(before.st_mode);
    if expected_kind == NodeKind::File && actual_kind != rustix::fs::FileType::RegularFile
        || expected_kind == NodeKind::Directory && actual_kind != rustix::fs::FileType::Directory
    {
        return Err(GripError::Internal(
            "entry kind changed during observation".into(),
        ));
    }
    let mut file = File::from(descriptor);
    let content = if expected_kind == NodeKind::File {
        let mut digest = Sha256::new();
        let mut length = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer).map_err(|error| {
                GripError::from_io("could not read file for fingerprinting", error)
            })?;
            if read == 0 {
                break;
            }
            digest.update(&buffer[..read]);
            length += read as u64;
        }
        Some(ContentFingerprint {
            algorithm: "sha256".into(),
            digest: format!("{:x}", digest.finalize()),
            length,
        })
    } else {
        None
    };
    let observed_metadata = crate::metadata::macos::observe_complete_metadata(&file, expected_kind)
        .map_err(|error| GripError::from_io("could not inspect complete metadata", error))?;
    let after = fstat(&file).map_err(|error| {
        GripError::from_io("could not reinspect entry descriptor", error.into())
    })?;
    if before.st_dev != after.st_dev
        || before.st_ino != after.st_ino
        || before.st_mode != after.st_mode
        || before.st_nlink != after.st_nlink
        || before.st_size != after.st_size
        || before.st_mtime != after.st_mtime
        || before.st_mtime_nsec != after.st_mtime_nsec
    {
        return Err(GripError::discovery_operational(
            "classification",
            "stale_metadata_evidence",
            Vec::new(),
            "entry changed while complete metadata was observed",
        ));
    }
    let state = crate::metadata::model::SupportedEntryStateV3 {
        node_kind: expected_kind,
        content,
        metadata: observed_metadata.metadata,
    };
    state
        .validate()
        .map_err(|message| GripError::Internal(message.into()))?;
    Ok(super::model::CompleteObservedState {
        state,
        excluded_xattrs: observed_metadata.xattrs.excluded,
        unknown_xattrs: observed_metadata.xattrs.unknown,
        unsupported_bsd_flags: observed_metadata.unsupported_bsd_flags,
    })
}

/// Result of probing one exact managed target below an opened tree root.
pub enum RelativeCompleteObservation {
    Missing,
    Supported(Box<super::model::CompleteObservedState>),
    DestinationLeafLink(DestinationLeafLinkEvidence),
    Blocking { reason: &'static str },
}

pub(crate) fn destination_leaf_link(
    metadata: crate::discovery::filesystem::NodeMetadata,
) -> DestinationLeafLinkEvidence {
    DestinationLeafLinkEvidence {
        device: metadata.stat.st_dev as u64,
        inode: metadata.stat.st_ino,
        mode: metadata.stat.st_mode as u32,
        size: metadata.stat.st_size as u64,
        modified_seconds: metadata.stat.st_mtime,
        modified_nanoseconds: metadata.stat.st_mtime_nsec,
    }
}

/// Reuses descriptor-bound ancestor directories during one observation pass.
pub struct RelativeInspector {
    root: crate::discovery::filesystem::Directory,
    directories: BTreeMap<Vec<u8>, crate::discovery::filesystem::Directory>,
}

impl RelativeInspector {
    pub fn open(root: &Path) -> Result<Self, GripError> {
        Ok(Self {
            root: crate::discovery::filesystem::Directory::open(root)
                .map_err(|error| GripError::from_io("could not open mapped tree root", error))?,
            directories: BTreeMap::new(),
        })
    }

    pub fn inspect(
        &mut self,
        relative: &[u8],
        expected_kind: NodeKind,
    ) -> Result<(SupportedState, DiagnosticEvidence), GripError> {
        let components = relative
            .split(|byte| *byte == b'/')
            .filter(|component| !component.is_empty())
            .collect::<Vec<_>>();
        if components.is_empty() {
            let metadata = self.root.root_metadata();
            return Ok((
                SupportedState::directory(),
                DiagnosticEvidence {
                    modified_seconds: metadata.st_mtime,
                    modified_nanoseconds: metadata.st_mtime_nsec,
                },
            ));
        }
        let mut parent_key = Vec::new();
        for component in &components[..components.len() - 1] {
            let mut child_key = parent_key.clone();
            if !child_key.is_empty() {
                child_key.push(b'/');
            }
            child_key.extend_from_slice(component);
            if !self.directories.contains_key(&child_key) {
                let parent = if parent_key.is_empty() {
                    &self.root
                } else {
                    self.directories
                        .get(&parent_key)
                        .expect("cached parent directory exists")
                };
                let child = parent.open_child_directory(component).map_err(|error| {
                    GripError::from_io("could not open mapped tree ancestor", error)
                })?;
                self.directories.insert(child_key.clone(), child);
            }
            parent_key = child_key;
        }
        let parent = if parent_key.is_empty() {
            &self.root
        } else {
            self.directories
                .get(&parent_key)
                .expect("cached parent directory exists")
        };
        let final_component = components[components.len() - 1];
        if expected_kind == NodeKind::Directory {
            let child = parent
                .open_child_directory(final_component)
                .map_err(|error| GripError::from_io("could not open observed directory", error))?;
            let metadata = child.root_metadata();
            return Ok((
                SupportedState::directory(),
                DiagnosticEvidence {
                    modified_seconds: metadata.st_mtime,
                    modified_nanoseconds: metadata.st_mtime_nsec,
                },
            ));
        }
        if expected_kind != NodeKind::File {
            return Err(GripError::Internal(
                "unsupported nodes are not fingerprinted".into(),
            ));
        }
        let descriptor = parent
            .open_child_file(final_component)
            .map_err(|error| GripError::from_io("could not open observed file", error))?;
        inspect_file_descriptor(descriptor, || {})
    }

    /// Probe one exact managed target without enumerating siblings or following links.
    pub fn inspect_complete(
        &mut self,
        relative: &[u8],
        expected_kind: NodeKind,
    ) -> Result<RelativeCompleteObservation, GripError> {
        let components = relative
            .split(|byte| *byte == b'/')
            .filter(|component| !component.is_empty())
            .collect::<Vec<_>>();
        if components.is_empty() {
            return Ok(RelativeCompleteObservation::Blocking {
                reason: "tree_root_anchor",
            });
        }
        let root_device = self.root.root_metadata().st_dev as u64;
        let mut parent_key = Vec::new();
        for component in &components[..components.len() - 1] {
            let mut child_key = parent_key.clone();
            if !child_key.is_empty() {
                child_key.push(b'/');
            }
            child_key.extend_from_slice(component);
            if !self.directories.contains_key(&child_key) {
                let parent = if parent_key.is_empty() {
                    &self.root
                } else {
                    self.directories
                        .get(&parent_key)
                        .expect("cached parent directory exists")
                };
                let metadata = match parent.metadata(component) {
                    Ok(metadata) => metadata,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        return Ok(RelativeCompleteObservation::Missing);
                    }
                    Err(error) => {
                        return Err(GripError::from_io(
                            "could not inspect mapped tree ancestor",
                            error,
                        ));
                    }
                };
                let kind = metadata.classify(root_device);
                if kind != NodeKind::Directory {
                    return Ok(RelativeCompleteObservation::Blocking {
                        reason: crate::discovery::filesystem::unsupported_reason(kind)
                            .unwrap_or("wrong_node_kind"),
                    });
                }
                let child = parent.open_child_directory(component).map_err(|error| {
                    GripError::from_io("could not open mapped tree ancestor", error)
                })?;
                self.directories.insert(child_key.clone(), child);
            }
            parent_key = child_key;
        }
        let parent = if parent_key.is_empty() {
            &self.root
        } else {
            self.directories
                .get(&parent_key)
                .expect("cached parent directory exists")
        };
        let final_component = components[components.len() - 1];
        let metadata = match parent.metadata(final_component) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(RelativeCompleteObservation::Missing);
            }
            Err(error) => {
                return Err(GripError::from_io(
                    "could not inspect managed target",
                    error,
                ));
            }
        };
        let kind = metadata.classify(root_device);
        if kind == NodeKind::Symlink {
            return Ok(RelativeCompleteObservation::DestinationLeafLink(
                destination_leaf_link(metadata),
            ));
        }
        if kind != expected_kind || !matches!(kind, NodeKind::File | NodeKind::Directory) {
            return Ok(RelativeCompleteObservation::Blocking {
                reason: crate::discovery::filesystem::unsupported_reason(kind)
                    .unwrap_or("wrong_node_kind"),
            });
        }
        let descriptor = if kind == NodeKind::Directory {
            parent
                .open_child_directory(final_component)
                .map(|directory| directory.into_descriptor())
        } else {
            parent.open_child_file(final_component)
        }
        .map_err(|error| GripError::from_io("could not open managed target", error))?;
        inspect_complete_descriptor(descriptor, kind)
            .map(Box::new)
            .map(RelativeCompleteObservation::Supported)
    }
}

/// Fingerprint an exact ordinary path without following its final component.
pub fn inspect(
    path: &Path,
    expected_kind: NodeKind,
) -> Result<(SupportedState, DiagnosticEvidence), GripError> {
    if expected_kind == NodeKind::Directory {
        let metadata = std::fs::symlink_metadata(path)
            .map_err(|error| GripError::from_io("could not inspect directory", error))?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(GripError::Internal(
                "directory evidence changed during observation".into(),
            ));
        }
        use std::os::unix::fs::MetadataExt;
        return Ok((
            SupportedState::directory(),
            DiagnosticEvidence {
                modified_seconds: metadata.mtime(),
                modified_nanoseconds: metadata.mtime_nsec(),
            },
        ));
    }
    if expected_kind != NodeKind::File {
        return Err(GripError::Internal(
            "unsupported nodes are not fingerprinted".into(),
        ));
    }
    let descriptor = open(
        path,
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK,
        Mode::empty(),
    )
    .map_err(|error| GripError::from_io("could not open file for fingerprinting", error.into()))?;
    inspect_file_descriptor(descriptor, || {})
}

/// Fingerprint a tree entry through non-following directory descriptors.
pub fn inspect_relative(
    root: &Path,
    relative: &[u8],
    expected_kind: NodeKind,
) -> Result<(SupportedState, DiagnosticEvidence), GripError> {
    let components = relative
        .split(|byte| *byte == b'/')
        .filter(|component| !component.is_empty())
        .collect::<Vec<_>>();
    if components.is_empty() {
        return inspect(root, expected_kind);
    }
    let mut directory = crate::discovery::filesystem::Directory::open(root)
        .map_err(|error| GripError::from_io("could not open mapped tree root", error))?;
    for component in &components[..components.len() - 1] {
        directory = directory
            .open_child_directory(component)
            .map_err(|error| GripError::from_io("could not open mapped tree ancestor", error))?;
    }
    let final_component = components[components.len() - 1];
    if expected_kind == NodeKind::Directory {
        let child = directory
            .open_child_directory(final_component)
            .map_err(|error| GripError::from_io("could not open observed directory", error))?;
        let metadata = child.root_metadata();
        return Ok((
            SupportedState::directory(),
            DiagnosticEvidence {
                modified_seconds: metadata.st_mtime,
                modified_nanoseconds: metadata.st_mtime_nsec,
            },
        ));
    }
    if expected_kind != NodeKind::File {
        return Err(GripError::Internal(
            "unsupported nodes are not fingerprinted".into(),
        ));
    }
    let descriptor = directory
        .open_child_file(final_component)
        .map_err(|error| GripError::from_io("could not open observed file", error))?;
    inspect_file_descriptor(descriptor, || {})
}

/// Fingerprint one child relative to an already validated parent directory.
pub(crate) fn inspect_open_child(
    parent: &crate::discovery::filesystem::Directory,
    name: &[u8],
    expected_kind: NodeKind,
) -> Result<(SupportedState, DiagnosticEvidence), GripError> {
    if expected_kind == NodeKind::Directory {
        let child = parent
            .open_child_directory(name)
            .map_err(|error| GripError::from_io("could not open observed directory", error))?;
        let metadata = child.root_metadata();
        return Ok((
            SupportedState::directory(),
            DiagnosticEvidence {
                modified_seconds: metadata.st_mtime,
                modified_nanoseconds: metadata.st_mtime_nsec,
            },
        ));
    }
    if expected_kind != NodeKind::File {
        return Err(GripError::Internal(
            "unsupported nodes are not fingerprinted".into(),
        ));
    }
    let descriptor = parent
        .open_child_file(name)
        .map_err(|error| GripError::from_io("could not open observed file", error))?;
    inspect_file_descriptor(descriptor, || {})
}

fn inspect_file_descriptor<F>(
    descriptor: OwnedFd,
    after_read: F,
) -> Result<(SupportedState, DiagnosticEvidence), GripError>
where
    F: FnOnce(),
{
    let before = fstat(&descriptor)
        .map_err(|error| GripError::from_io("could not inspect file descriptor", error.into()))?;
    if rustix::fs::FileType::from_raw_mode(before.st_mode) != rustix::fs::FileType::RegularFile
        || before.st_nlink != 1
        || (before.st_size > 0
            && (before.st_blocks as u64).saturating_mul(512) < before.st_size as u64)
    {
        return Err(GripError::Internal(
            "file is no longer an ordinary supported node".into(),
        ));
    }
    let mut file = File::from(descriptor);
    let mut digest = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| GripError::from_io("could not read file for fingerprinting", error))?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
        length += read as u64;
    }
    after_read();
    let after = fstat(&file)
        .map_err(|error| GripError::from_io("could not reinspect file descriptor", error.into()))?;
    if before.st_dev != after.st_dev
        || before.st_ino != after.st_ino
        || before.st_mode != after.st_mode
        || before.st_nlink != after.st_nlink
        || before.st_size != after.st_size
        || before.st_mtime != after.st_mtime
        || before.st_mtime_nsec != after.st_mtime_nsec
        || length != after.st_size as u64
    {
        return Err(GripError::discovery_operational(
            "classification",
            "stale_content_evidence",
            Vec::new(),
            "file changed while being fingerprinted",
        ));
    }
    Ok((
        SupportedState {
            node_kind: NodeKind::File,
            content: Some(ContentFingerprint {
                algorithm: "sha256".into(),
                digest: format!("{:x}", digest.finalize()),
                length,
            }),
            permission_mode: Some(format!("{:04o}", (after.st_mode as u32) & 0o7777)),
        },
        DiagnosticEvidence {
            modified_seconds: after.st_mtime,
            modified_nanoseconds: after.st_mtime_nsec,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn fingerprint_captures_digest_length_mode_and_diagnostic_mtime() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("payload");
        std::fs::write(&path, b"abc").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o764)).unwrap();
        let (state, diagnostic) = inspect(&path, NodeKind::File).unwrap();
        assert_eq!(
            state.content.unwrap().digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(state.permission_mode.as_deref(), Some("0764"));
        assert!(diagnostic.modified_seconds > 0);
    }

    #[test]
    fn unsupported_node_is_rejected_before_open() {
        let error = inspect(Path::new("/definitely/not/opened"), NodeKind::Symlink).unwrap_err();
        assert!(error.to_string().contains("unsupported"));
    }

    #[test]
    fn fingerprint_rejects_drift_between_stream_and_final_evidence() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("payload");
        std::fs::write(&path, b"before").unwrap();
        let descriptor = open(
            &path,
            OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
            Mode::empty(),
        )
        .unwrap();
        let result = inspect_file_descriptor(descriptor, || {
            std::fs::write(&path, b"after and longer").unwrap();
        });
        assert!(matches!(
            result,
            Err(GripError::Mapping { ref reason, .. }) if reason == "stale_content_evidence"
        ));
    }
}
