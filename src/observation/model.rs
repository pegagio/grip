//! Typed filesystem evidence and canonical managed-entry identity.

use crate::discovery::model::{NodeKind, SafePath};
use crate::error::GripError;
use crate::mapping::{Mapping, MappingKind};
use crate::registry::publication::RegistrySnapshot;
use crate::state::AcceptedState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

/// Durable mapping identity stored with every accepted baseline.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappingSnapshot {
    pub kind: MappingKind,
    pub source: PathBuf,
    pub destination: PathBuf,
}

impl From<&Mapping> for MappingSnapshot {
    fn from(value: &Mapping) -> Self {
        Self {
            kind: value.kind,
            source: value.source.clone(),
            destination: value.destination.clone(),
        }
    }
}

/// Lossless identity of one entry within a mapping.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntryIdentity {
    pub mapping: MappingSnapshot,
    pub relative_path: Vec<u8>,
}

impl EntryIdentity {
    pub fn new(mapping: MappingSnapshot, relative_path: Vec<u8>) -> Result<Self, &'static str> {
        validate_relative(&relative_path, mapping.kind)?;
        Ok(Self {
            mapping,
            relative_path,
        })
    }

    pub fn source_path(&self) -> PathBuf {
        append_raw(&self.mapping.source, &self.relative_path)
    }

    pub fn destination_path(&self) -> PathBuf {
        append_raw(&self.mapping.destination, &self.relative_path)
    }

    pub fn relative_safe_path(&self) -> Option<SafePath> {
        (!self.relative_path.is_empty()).then(|| SafePath::from_bytes(&self.relative_path))
    }
}

/// Content evidence for a regular file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentFingerprint {
    pub algorithm: String,
    pub digest: String,
    pub length: u64,
}

/// Complete supported equality state for one entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupportedState {
    pub node_kind: NodeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ContentFingerprint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
}

impl SupportedState {
    pub fn directory() -> Self {
        Self {
            node_kind: NodeKind::Directory,
            content: None,
            permission_mode: None,
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        match self.node_kind {
            NodeKind::File => {
                let content = self.content.as_ref().ok_or("file content is required")?;
                if content.algorithm != "sha256"
                    || content.digest.len() != 64
                    || !content
                        .digest
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                {
                    return Err("file content digest must be lowercase SHA-256");
                }
                let mode = self
                    .permission_mode
                    .as_deref()
                    .ok_or("file mode is required")?;
                if mode.len() != 4 || !mode.bytes().all(|byte| (b'0'..=b'7').contains(&byte)) {
                    return Err("file permission mode must be four octal digits");
                }
            }
            NodeKind::Directory => {
                if self.content.is_some() || self.permission_mode.is_some() {
                    return Err("directory state forbids file-only fields");
                }
            }
            _ => return Err("unsupported node kind cannot be accepted"),
        }
        Ok(())
    }
}

/// Diagnostic-only metadata excluded from equality and persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticEvidence {
    pub modified_seconds: i64,
    pub modified_nanoseconds: i64,
}

/// Complete state and operation-scoped metadata diagnostics for Feature 009 observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteObservedState {
    pub state: crate::metadata::model::SupportedEntryStateV3,
    pub excluded_xattrs: Vec<Vec<u8>>,
    pub unknown_xattrs: Vec<Vec<u8>>,
    pub unsupported_bsd_flags: Vec<String>,
}

/// Current membership interpretation for an observed identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Membership {
    Active,
    Ignored,
    DestinationOnly,
    Untracked,
}

/// Stable current evidence for one identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedEntry {
    pub identity: EntryIdentity,
    pub membership: Membership,
    pub source: Option<SupportedState>,
    pub destination: Option<SupportedState>,
    pub source_complete: Option<CompleteObservedState>,
    pub destination_complete: Option<CompleteObservedState>,
    pub metadata_findings: Vec<crate::metadata::model::CompatibilityFinding>,
    pub endpoint_capabilities: Vec<crate::metadata::model::EndpointCapabilityProfile>,
    pub source_diagnostic: Option<DiagnosticEvidence>,
    pub destination_diagnostic: Option<DiagnosticEvidence>,
    pub unsupported: Vec<String>,
    pub blocking: bool,
}

/// Selector path interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathSpace {
    Source,
    Destination,
}

/// Resolved classification selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    All,
    Mapping(MappingSnapshot),
    Entry(EntryIdentity),
    Subtree(EntryIdentity),
    Unmanaged(SafePath),
}

impl Selection {
    pub fn includes(&self, identity: &EntryIdentity) -> bool {
        match self {
            Self::All => true,
            Self::Mapping(mapping) => &identity.mapping == mapping,
            Self::Entry(selected) => identity == selected,
            Self::Subtree(selected) => {
                identity.mapping == selected.mapping
                    && (identity.relative_path == selected.relative_path
                        || raw_descendant(&identity.relative_path, &selected.relative_path))
            }
            Self::Unmanaged(_) => false,
        }
    }
}

pub type Observation = BTreeMap<EntryIdentity, ObservedEntry>;

/// Resolve one optional selector against current mappings and retained identities.
pub fn resolve_selection(
    registry: &RegistrySnapshot,
    accepted: &AcceptedState,
    selector: Option<&Path>,
    path_space: PathSpace,
    operation: &str,
) -> Result<Selection, GripError> {
    let Some(selector) = selector else {
        return Ok(Selection::All);
    };
    let canonical = crate::path_policy::resolve_selector(selector, operation)?;
    for mapping in registry.registry.mappings() {
        let snapshot = MappingSnapshot::from(mapping);
        let root = match path_space {
            PathSpace::Source => &mapping.source,
            PathSpace::Destination => &mapping.destination,
        };
        if canonical == *root {
            return if mapping.kind == MappingKind::File {
                Ok(Selection::Entry(
                    EntryIdentity::new(snapshot, Vec::new()).expect("file identity is valid"),
                ))
            } else {
                Ok(Selection::Mapping(snapshot))
            };
        }
        if mapping.kind == MappingKind::Tree
            && let Some(relative) = relative_bytes(root, &canonical)
        {
            return Ok(Selection::Subtree(
                EntryIdentity::new(snapshot, relative)
                    .map_err(|message| GripError::InvalidConfiguration(message.into()))?,
            ));
        }
    }
    for identity in accepted
        .baselines
        .keys()
        .chain(accepted.complete_baselines.keys())
    {
        let path = match path_space {
            PathSpace::Source => identity.source_path(),
            PathSpace::Destination => identity.destination_path(),
        };
        if canonical == path {
            return Ok(Selection::Entry(identity.clone()));
        }
        if identity.mapping.kind == MappingKind::Tree && path.starts_with(&canonical) {
            let root = match path_space {
                PathSpace::Source => &identity.mapping.source,
                PathSpace::Destination => &identity.mapping.destination,
            };
            if let Some(relative) = relative_bytes(root, &canonical) {
                return Ok(Selection::Subtree(
                    EntryIdentity::new(identity.mapping.clone(), relative)
                        .map_err(|message| GripError::InvalidConfiguration(message.into()))?,
                ));
            }
        }
    }
    Err(GripError::mapping(
        operation,
        "selector_outside_scope",
        vec![SafePath::from_path(&canonical).display],
        "selector is outside current mapping ownership and retained baseline identity",
    ))
}

pub fn append_raw(root: &Path, relative: &[u8]) -> PathBuf {
    let mut result = root.to_path_buf();
    if !relative.is_empty() {
        result.push(OsString::from_vec(relative.to_vec()));
    }
    result
}

pub fn relative_bytes(root: &Path, path: &Path) -> Option<Vec<u8>> {
    path.strip_prefix(root)
        .ok()
        .map(|value| value.as_os_str().as_bytes().to_vec())
}

fn raw_descendant(candidate: &[u8], parent: &[u8]) -> bool {
    parent.is_empty()
        || candidate
            .strip_prefix(parent)
            .is_some_and(|suffix| suffix.first() == Some(&b'/'))
}

fn validate_relative(relative: &[u8], kind: MappingKind) -> Result<(), &'static str> {
    if kind == MappingKind::File && !relative.is_empty() {
        return Err("file mapping identity must have an empty relative path");
    }
    if relative.first() == Some(&b'/')
        || relative
            .split(|byte| *byte == b'/')
            .any(|part| part.is_empty() && !relative.is_empty() || part == b"." || part == b"..")
    {
        return Err("relative path must contain safe non-empty components");
    }
    Ok(())
}
