//! Domain values for deterministic, non-persisted discovery inventories.

use crate::mapping::MappingKind;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// One exact path rendered safely for humans and machines.
#[derive(Debug, Clone, Eq, Serialize)]
pub struct SafePath {
    pub display: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_hex: Option<String>,
    #[serde(skip)]
    raw: Vec<u8>,
}

impl SafePath {
    /// Construct a safe representation from an observed Unix path.
    pub fn from_path(path: &Path) -> Self {
        Self::from_bytes(path.as_os_str().as_bytes())
    }

    /// Construct a safe representation from exact Unix path bytes.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let valid_utf8 = std::str::from_utf8(bytes).ok();
        let display = valid_utf8.map_or_else(|| escape_raw_bytes(bytes), escape_utf8);
        Self {
            display,
            raw_hex: valid_utf8.is_none().then(|| hex(bytes)),
            raw: bytes.to_vec(),
        }
    }

    /// Return the exact observed bytes used for deterministic identity and ordering.
    pub fn raw_bytes(&self) -> &[u8] {
        &self.raw
    }
}

fn escape_utf8(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        match character {
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\\' => output.push_str("\\\\"),
            character if character.is_control() => output.extend(character.escape_default()),
            character => output.push(character),
        }
    }
    output
}

impl PartialEq for SafePath {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl PartialOrd for SafePath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SafePath {
    fn cmp(&self, other: &Self) -> Ordering {
        self.raw.cmp(&other.raw)
    }
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(output, "{byte:02x}").expect("writing to a string cannot fail");
    }
    output
}

fn escape_raw_bytes(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut output = String::new();
    for byte in bytes {
        match byte {
            b'\n' => output.push_str("\\n"),
            b'\r' => output.push_str("\\r"),
            b'\t' => output.push_str("\\t"),
            b'\\' => output.push_str("\\\\"),
            0x20..=0x7e => output.push(char::from(*byte)),
            _ => write!(output, "\\x{byte:02x}").expect("writing to a string cannot fail"),
        }
    }
    output
}

/// The selected mapping scope for one inspection request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryScope {
    All,
    Mapping(PathBuf),
}

/// One read-only discovery request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryRequest {
    pub scope: DiscoveryScope,
}

/// Stable inventory record categories in their deterministic tie-break order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordCategory {
    Eligible,
    Ignored,
    DestinationOnly,
    UnsupportedSource,
    UnsafeDestinationCollision,
}

/// Stable observed node kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    File,
    Directory,
    Symlink,
    HardLink,
    SparseFile,
    Socket,
    Fifo,
    CharacterDevice,
    BlockDevice,
    Whiteout,
    UnknownSpecial,
    NestedMount,
}

/// One classified source, ignored, overlay, or collision path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiscoveryRecord {
    pub category: RecordCategory,
    pub mapping_kind: MappingKind,
    pub mapping_source: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative_path: Option<SafePath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<SafePath>,
    pub destination_path: SafePath,
    pub node_kind: NodeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<&'static str>,
    pub blocking: bool,
}

impl PartialOrd for DiscoveryRecord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DiscoveryRecord {
    fn cmp(&self, other: &Self) -> Ordering {
        self.mapping_source
            .cmp(&other.mapping_source)
            .then_with(|| self.relative_path.cmp(&other.relative_path))
            .then_with(|| self.category.cmp(&other.category))
    }
}

/// Exact metadata used only to compare two discovery passes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeEvidence {
    pub side: EvidenceSide,
    pub mapping_source: PathBuf,
    pub relative_bytes: Vec<u8>,
    pub device: u64,
    pub inode: u64,
    pub mode: u32,
    pub link_count: u64,
    pub size: u64,
    pub allocated_blocks: u64,
    pub modified_seconds: i64,
    pub modified_nanoseconds: i64,
    pub changed_seconds: i64,
    pub changed_nanoseconds: i64,
    pub child_names: Option<Vec<Vec<u8>>>,
}

/// Filesystem side associated with ephemeral node evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceSide {
    Source,
    Destination,
}

/// Exact policy bytes and metadata used for pass comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyEvidence {
    pub bytes: Vec<u8>,
    pub digest: [u8; 32],
    pub node: NodeEvidence,
}

/// One complete ephemeral inspection pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryPass {
    pub registry_bytes: Vec<u8>,
    pub records: Vec<DiscoveryRecord>,
    pub node_evidence: BTreeMap<(EvidenceSide, PathBuf, Vec<u8>), NodeEvidence>,
    pub policy_evidence: BTreeMap<Vec<u8>, PolicyEvidence>,
}

/// Derived totals for every stable record category.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct DiscoveryCounts {
    pub eligible: usize,
    pub ignored: usize,
    pub destination_only: usize,
    pub unsupported_source: usize,
    pub unsafe_destination_collision: usize,
}

/// The deterministic result of two equivalent complete passes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryInventory {
    pub scope: DiscoveryScope,
    pub records: Vec<DiscoveryRecord>,
    pub counts: DiscoveryCounts,
    pub blocking_count: usize,
}

impl DiscoveryInventory {
    /// Sort records and derive all public counts.
    pub fn new(scope: DiscoveryScope, mut records: Vec<DiscoveryRecord>) -> Self {
        records.sort();
        let mut counts = DiscoveryCounts::default();
        for record in &records {
            match record.category {
                RecordCategory::Eligible => counts.eligible += 1,
                RecordCategory::Ignored => counts.ignored += 1,
                RecordCategory::DestinationOnly => counts.destination_only += 1,
                RecordCategory::UnsupportedSource => counts.unsupported_source += 1,
                RecordCategory::UnsafeDestinationCollision => {
                    counts.unsafe_destination_collision += 1;
                }
            }
        }
        let blocking_count = records.iter().filter(|record| record.blocking).count();
        Self {
            scope,
            records,
            counts,
            blocking_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::MappingKind;
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::path::PathBuf;

    fn record(category: RecordCategory, relative: SafePath, blocking: bool) -> DiscoveryRecord {
        DiscoveryRecord {
            category,
            mapping_kind: MappingKind::Tree,
            mapping_source: PathBuf::from("/source"),
            relative_path: Some(relative.clone()),
            source_path: Some(relative.clone()),
            destination_path: relative,
            node_kind: NodeKind::File,
            reason: None,
            blocking,
        }
    }

    #[test]
    fn safe_path_preserves_non_utf8_bytes_without_loss() {
        let path = PathBuf::from(OsString::from_vec(vec![b'a', 0xff, b'\n']));
        let safe = SafePath::from_path(&path);
        assert_eq!(safe.display, "a\\xff\\n");
        assert_eq!(safe.raw_hex.as_deref(), Some("61ff0a"));
    }

    #[test]
    fn safe_path_preserves_printable_unicode_and_escapes_controls() {
        let safe = SafePath::from_bytes("café\n".as_bytes());
        assert_eq!(safe.display, "café\\n");
        assert_eq!(safe.raw_hex, None);
    }

    #[test]
    fn records_sort_by_mapping_relative_bytes_and_category() {
        let mut records = [
            record(RecordCategory::Ignored, SafePath::from_bytes(b"b"), false),
            record(
                RecordCategory::UnsupportedSource,
                SafePath::from_bytes(b"a"),
                true,
            ),
            record(RecordCategory::Eligible, SafePath::from_bytes(b"a"), false),
        ];
        records.sort();
        assert_eq!(records[0].category, RecordCategory::Eligible);
        assert_eq!(records[1].category, RecordCategory::UnsupportedSource);
        assert_eq!(records[2].relative_path.as_ref().unwrap().raw_bytes(), b"b");
    }

    #[test]
    fn inventory_derives_all_category_and_blocker_counts() {
        let records = vec![
            record(RecordCategory::Eligible, SafePath::from_bytes(b"a"), false),
            record(RecordCategory::Ignored, SafePath::from_bytes(b"b"), false),
            record(
                RecordCategory::UnsafeDestinationCollision,
                SafePath::from_bytes(b"c"),
                true,
            ),
        ];
        let inventory = DiscoveryInventory::new(DiscoveryScope::All, records);
        assert_eq!(inventory.counts.eligible, 1);
        assert_eq!(inventory.counts.ignored, 1);
        assert_eq!(inventory.counts.destination_only, 0);
        assert_eq!(inventory.counts.unsupported_source, 0);
        assert_eq!(inventory.counts.unsafe_destination_collision, 1);
        assert_eq!(inventory.blocking_count, 1);
    }
}
