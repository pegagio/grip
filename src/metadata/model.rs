//! Platform-neutral models for equality-defining metadata and capability evidence.

use crate::discovery::model::NodeKind;
use crate::observation::model::ContentFingerprint;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// A value observation or the precise reason no value can be used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum Evidence<T> {
    Observed { value: T },
    Absent,
    Unavailable { reason: String },
    Unsupported { reason: String },
    Unreadable { reason: String },
    Unauthorized { reason: String },
}

impl<T> Evidence<T> {
    /// Returns whether this evidence can participate in equality.
    pub fn is_conclusive(&self) -> bool {
        matches!(self, Self::Observed { .. } | Self::Absent)
    }
}

/// Exact Darwin modification time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModificationTime {
    pub seconds: i64,
    pub nanoseconds: u32,
}

impl ModificationTime {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.nanoseconds >= 1_000_000_000 {
            return Err("modification-time nanoseconds must be below one billion");
        }
        Ok(())
    }
}

/// Equality-safe fingerprint for an allowlisted extended attribute.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XattrFingerprint {
    pub name: Vec<u8>,
    pub length: u64,
    pub algorithm: String,
    pub digest: String,
}

impl XattrFingerprint {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.name.is_empty() || self.name.contains(&0) {
            return Err("extended-attribute name must be non-empty and contain no NUL");
        }
        validate_sha256(&self.algorithm, &self.digest)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AclEntryKind {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AclPermission {
    ReadData,
    ListDirectory,
    WriteData,
    AddFile,
    Execute,
    Search,
    Delete,
    AppendData,
    AddSubdirectory,
    DeleteChild,
    ReadAttributes,
    WriteAttributes,
    ReadExtendedAttributes,
    WriteExtendedAttributes,
    ReadSecurity,
    WriteSecurity,
    ChangeOwner,
    Synchronize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AclEntryFlag {
    FileInherit,
    DirectoryInherit,
    LimitInherit,
    OnlyInherit,
    Inherited,
}

/// One ordered ACL entry. Permission and flag sets serialize canonically.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessControlEntry {
    pub principal_uuid: [u8; 16],
    pub kind: AclEntryKind,
    pub permissions: BTreeSet<AclPermission>,
    pub flags: BTreeSet<AclEntryFlag>,
}

/// Definitive extended ACL absence or its semantic ACE sequence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum AclState {
    Absent,
    Present { entries: Vec<AccessControlEntry> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BsdFlag {
    Nodump,
    Immutable,
    Append,
    Hidden,
    Opaque,
}

/// Complete equality-defining metadata for a supported entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataState {
    pub permission_mode: String,
    pub uid: u32,
    pub gid: u32,
    pub modified_time: ModificationTime,
    pub extended_attributes: Vec<XattrFingerprint>,
    pub acl: AclState,
    pub bsd_flags: BTreeSet<BsdFlag>,
}

impl MetadataState {
    pub fn validate(&self, node_kind: NodeKind) -> Result<(), &'static str> {
        if self.permission_mode.len() != 4
            || !self
                .permission_mode
                .bytes()
                .all(|byte| (b'0'..=b'7').contains(&byte))
        {
            return Err("permission mode must be four octal digits");
        }
        self.modified_time.validate()?;
        let mut previous: Option<&[u8]> = None;
        for attribute in &self.extended_attributes {
            attribute.validate()?;
            if previous.is_some_and(|name| name >= attribute.name.as_slice()) {
                return Err("extended attributes must be unique and sorted by exact name bytes");
            }
            previous = Some(&attribute.name);
        }
        if node_kind != NodeKind::Directory && self.bsd_flags.contains(&BsdFlag::Opaque) {
            return Err("opaque BSD flag is valid only for directories");
        }
        Ok(())
    }
}

/// Complete accepted equality state stored by State Envelope V3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupportedEntryStateV3 {
    pub node_kind: NodeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ContentFingerprint>,
    pub metadata: MetadataState,
}

impl SupportedEntryStateV3 {
    pub fn validate(&self) -> Result<(), &'static str> {
        match self.node_kind {
            NodeKind::File => {
                let content = self.content.as_ref().ok_or("file content is required")?;
                validate_sha256(&content.algorithm, &content.digest)?;
            }
            NodeKind::Directory => {
                if self.content.is_some() {
                    return Err("directory state forbids content");
                }
            }
            _ => return Err("unsupported node kind cannot be accepted"),
        }
        self.metadata.validate(self.node_kind)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointRole {
    Source,
    Destination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataDimension {
    Node,
    Content,
    PermissionMode,
    Owner,
    Group,
    ModificationTime,
    ExtendedAttribute,
    AccessControlList,
    BsdFlags,
    Mount,
    Case,
    Unicode,
}

/// Inspect/apply/verify evidence for one metadata dimension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityEvidence {
    pub dimension: MetadataDimension,
    pub inspect: Evidence<bool>,
    pub apply: Evidence<bool>,
    pub verify: Evidence<bool>,
}

/// Operation-scoped capabilities of one concrete endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointCapabilityProfile {
    pub endpoint: EndpointRole,
    pub root_display: String,
    pub root_raw_hex: Option<String>,
    pub filesystem_type: Evidence<String>,
    pub filesystem_identity: Evidence<u64>,
    pub mount_flags: Evidence<u64>,
    pub volume_capability_masks: Evidence<Vec<u64>>,
    pub case_sensitive: Evidence<bool>,
    pub case_preserving: Evidence<bool>,
    pub unicode_qualification: Evidence<String>,
    pub mtime_precision_nanoseconds: Evidence<u32>,
    pub capabilities: Vec<CapabilityEvidence>,
    pub qualification_reference: Option<String>,
}

impl EndpointCapabilityProfile {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.root_display.is_empty() {
            return Err("endpoint root display is required");
        }
        if matches!(
            self.mtime_precision_nanoseconds,
            Evidence::Observed { value: 0 }
        ) {
            return Err("mtime precision must be positive");
        }
        let mut dimensions = BTreeSet::new();
        if !self
            .capabilities
            .iter()
            .all(|capability| dimensions.insert(capability.dimension))
        {
            return Err("endpoint capabilities must be unique by dimension");
        }
        Ok(())
    }
}

/// Stable compatibility reason suitable for machine-readable output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityReason {
    Unavailable,
    Unsupported,
    Unreadable,
    Unauthorized,
    UnknownXattr,
    ExcludedXattr,
    ProtectedFlag,
    ApfsCollision,
    InconclusiveComparison,
    SparseFile,
    HardLink,
    MountBoundary,
}

/// One safe, operation-scoped incompatibility report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompatibilityFinding {
    pub endpoint: EndpointRole,
    pub path_display: String,
    pub path_raw_hex: Option<String>,
    pub field: MetadataDimension,
    pub required: String,
    pub evidence_state: String,
    pub reason: CompatibilityReason,
    pub message: String,
    pub corrective_choice: String,
    pub blocking: bool,
}

impl CompatibilityFinding {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.path_display.is_empty()
            || self.message.is_empty()
            || self.corrective_choice.is_empty()
            || self.evidence_state.is_empty()
        {
            return Err("compatibility finding requires path, state, and message");
        }
        Ok(())
    }
}

/// Every exact managed identity that collides under an endpoint comparator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilesystemIdentityCollision {
    pub endpoint: EndpointRole,
    pub comparison_evidence: Evidence<String>,
    pub identity_raw_paths: Vec<Vec<u8>>,
}

impl FilesystemIdentityCollision {
    pub fn validate(&self) -> Result<(), &'static str> {
        let distinct = self.identity_raw_paths.iter().collect::<BTreeSet<_>>();
        if distinct.len() < 2 {
            return Err("filesystem collision requires at least two distinct identities");
        }
        Ok(())
    }
}

/// Direction and complete before/after values for an indivisible metadata change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataTransition {
    pub identity_raw_path: Vec<u8>,
    pub direction: TransitionDirection,
    pub before: SupportedEntryStateV3,
    pub after: SupportedEntryStateV3,
    pub changed_dimensions: BTreeSet<MetadataDimension>,
    pub flags_to_clear: BTreeSet<BsdFlag>,
    pub capability_proofs: Vec<CapabilityEvidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionDirection {
    SourceToDestination,
    DestinationToSource,
}

impl MetadataTransition {
    pub fn validate(&self) -> Result<(), &'static str> {
        self.before.validate()?;
        self.after.validate()?;
        if self.identity_raw_path.is_empty() || self.changed_dimensions.is_empty() {
            return Err("metadata transition requires identity and changed dimensions");
        }
        Ok(())
    }
}

/// Reproducible environment evidence for product qualification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformQualificationRecord {
    pub macos_version: String,
    pub macos_build: String,
    pub darwin_kernel: String,
    pub apfs_bundle_version: String,
    pub filesystem_type: String,
    pub mount_flags: u64,
    pub volume_capability_masks: Vec<u64>,
    pub binary_build_profile: String,
    pub binary_revision: String,
    pub test_matrix_version: String,
    pub performance_host_description: String,
}

impl PlatformQualificationRecord {
    pub fn validate(&self) -> Result<(), &'static str> {
        if [
            &self.macos_version,
            &self.macos_build,
            &self.darwin_kernel,
            &self.apfs_bundle_version,
            &self.filesystem_type,
            &self.binary_build_profile,
            &self.binary_revision,
            &self.test_matrix_version,
            &self.performance_host_description,
        ]
        .iter()
        .any(|value| value.is_empty())
        {
            return Err("qualification record fields must be non-empty");
        }
        Ok(())
    }
}

fn validate_sha256(algorithm: &str, digest: &str) -> Result<(), &'static str> {
    if algorithm != "sha256"
        || digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("fingerprint must use lowercase SHA-256");
    }
    Ok(())
}
