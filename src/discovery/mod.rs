//! Read-only source membership discovery and inventory orchestration.

pub mod filesystem;
pub mod ignore_policy;
pub mod model;

use crate::error::GripError;
use crate::home::GripHome;
use crate::mapping::{Mapping, MappingKind};
use crate::registry::publication::{self, RegistrySnapshot};
use filesystem::{Directory, evidence, metadata_at_path, unsupported_reason};
use model::{
    DiscoveryInventory, DiscoveryPass, DiscoveryRecord, DiscoveryScope, EvidenceSide, NodeKind,
    RecordCategory, SafePath,
};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

const OPERATION: &str = "mapping_inspect";

/// Produce a deterministic inventory from two equivalent complete inspections.
pub fn inspect(
    home: &GripHome,
    initial: &RegistrySnapshot,
    selected_source: Option<PathBuf>,
) -> Result<DiscoveryInventory, GripError> {
    inspect_with_between_pass(home, initial, selected_source, || {})
}

fn inspect_with_between_pass<F>(
    home: &GripHome,
    initial: &RegistrySnapshot,
    selected_source: Option<PathBuf>,
    between_passes: F,
) -> Result<DiscoveryInventory, GripError>
where
    F: FnOnce(),
{
    let scope = selected_source
        .clone()
        .map_or(DiscoveryScope::All, DiscoveryScope::Mapping);
    let selected = select_mappings(initial, selected_source.as_deref())?;
    let first = inspect_pass(&initial.bytes, selected)?;
    between_passes();

    let current = publication::load(home, false).map_err(|error| {
        stale(
            vec![home.path().join("config.toml")],
            &format!("Accepted registry changed during inspection: {error}"),
        )
    })?;
    if current.bytes != initial.bytes || current.registry != initial.registry {
        return Err(stale(
            vec![home.path().join("config.toml")],
            "Accepted registry changed during inspection",
        ));
    }
    let selected = select_mappings(&current, selected_source.as_deref())?;
    let second = inspect_pass(&current.bytes, selected)?;
    publication::revalidate_readonly(home, initial, OPERATION)?;
    if first != second {
        return Err(stale(
            Vec::new(),
            "Discovery evidence changed before the inventory was complete",
        ));
    }
    Ok(DiscoveryInventory::new(scope, first.records))
}

fn select_mappings<'a>(
    snapshot: &'a RegistrySnapshot,
    selected_source: Option<&Path>,
) -> Result<Vec<&'a Mapping>, GripError> {
    if let Some(source) = selected_source {
        let mapping = snapshot
            .registry
            .mappings()
            .iter()
            .find(|mapping| mapping.source == source)
            .ok_or_else(|| {
                GripError::discovery_invalid(
                    OPERATION,
                    "mapping_not_found",
                    vec![source.display().to_string()],
                    "Mapping not found",
                )
            })?;
        Ok(vec![mapping])
    } else {
        Ok(snapshot.registry.mappings().iter().collect())
    }
}

fn inspect_pass(
    registry_bytes: &[u8],
    mappings: Vec<&Mapping>,
) -> Result<DiscoveryPass, GripError> {
    let mut records = Vec::new();
    let mut node_evidence = BTreeMap::new();
    let mut policy_evidence = BTreeMap::new();
    for mapping in mappings {
        match mapping.kind {
            MappingKind::File => {
                inspect_file_mapping(mapping, &mut records, &mut node_evidence)?;
                inspect_file_destination(mapping, &mut records, &mut node_evidence)?;
            }
            MappingKind::Tree => inspect_tree_mapping(
                mapping,
                &mut records,
                &mut node_evidence,
                &mut policy_evidence,
            )
            .and_then(|()| inspect_tree_destination(mapping, &mut records, &mut node_evidence))?,
        }
    }
    records.sort();
    Ok(DiscoveryPass {
        registry_bytes: registry_bytes.to_vec(),
        records,
        node_evidence,
        policy_evidence,
    })
}

fn inspect_file_destination(
    mapping: &Mapping,
    records: &mut Vec<DiscoveryRecord>,
    evidence_map: &mut BTreeMap<(EvidenceSide, PathBuf, Vec<u8>), model::NodeEvidence>,
) -> Result<(), GripError> {
    let metadata = match metadata_at_path(&mapping.destination) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(unavailable(&mapping.destination, "path_unavailable", error)),
    };
    let kind = metadata.classify(metadata.stat.st_dev as u64);
    evidence_map.insert(
        (
            EvidenceSide::Destination,
            mapping.source.clone(),
            Vec::new(),
        ),
        evidence(
            &metadata,
            EvidenceSide::Destination,
            &mapping.source,
            &[],
            None,
        ),
    );
    if kind != NodeKind::File {
        records.push(DiscoveryRecord {
            category: RecordCategory::UnsafeDestinationCollision,
            mapping_kind: mapping.kind,
            mapping_source: mapping.source.clone(),
            relative_path: None,
            source_path: Some(SafePath::from_path(&mapping.source)),
            destination_path: SafePath::from_path(&mapping.destination),
            node_kind: kind,
            reason: unsupported_reason(kind).or(Some("wrong_node_kind")),
            blocking: true,
        });
    }
    Ok(())
}

fn inspect_tree_destination(
    mapping: &Mapping,
    records: &mut Vec<DiscoveryRecord>,
    evidence_map: &mut BTreeMap<(EvidenceSide, PathBuf, Vec<u8>), model::NodeEvidence>,
) -> Result<(), GripError> {
    let root_metadata = match metadata_at_path(&mapping.destination) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(unavailable(&mapping.destination, "path_unavailable", error)),
    };
    if root_metadata.file_type() != rustix::fs::FileType::Directory {
        return Err(unavailable(
            &mapping.destination,
            "path_unavailable",
            std::io::Error::other("tree mapping destination root is not a directory"),
        ));
    }
    let root = Directory::open(&mapping.destination)
        .map_err(|error| unavailable(&mapping.destination, "directory_unreadable", error))?;
    let root_device = root.root_metadata().st_dev as u64;
    let names = root
        .child_names()
        .map_err(|error| unavailable(&mapping.destination, "directory_unreadable", error))?;
    evidence_map.insert(
        (
            EvidenceSide::Destination,
            mapping.source.clone(),
            Vec::new(),
        ),
        evidence(
            &root_metadata,
            EvidenceSide::Destination,
            &mapping.source,
            &[],
            Some(names.clone()),
        ),
    );
    let eligible: BTreeMap<Vec<u8>, NodeKind> = records
        .iter()
        .filter(|record| {
            record.mapping_source == mapping.source && record.category == RecordCategory::Eligible
        })
        .filter_map(|record| {
            record
                .relative_path
                .as_ref()
                .map(|path| (path.raw_bytes().to_vec(), record.node_kind))
        })
        .collect();
    walk_destination(
        mapping,
        &root,
        root_device,
        Vec::new(),
        names,
        &eligible,
        records,
        evidence_map,
    )
}

#[allow(clippy::too_many_arguments)]
fn walk_destination(
    mapping: &Mapping,
    directory: &Directory,
    root_device: u64,
    parent_relative: Vec<u8>,
    names: Vec<Vec<u8>>,
    eligible: &BTreeMap<Vec<u8>, NodeKind>,
    records: &mut Vec<DiscoveryRecord>,
    evidence_map: &mut BTreeMap<(EvidenceSide, PathBuf, Vec<u8>), model::NodeEvidence>,
) -> Result<(), GripError> {
    for name in names {
        let relative = join_relative(&parent_relative, &name);
        let destination_path = append_raw(&mapping.destination, &relative);
        let source_path = append_raw(&mapping.source, &relative);
        let metadata = directory
            .metadata(&name)
            .map_err(|error| unavailable(&destination_path, "path_unavailable", error))?;
        let kind = metadata.classify(root_device);
        let child_scan = if kind == NodeKind::Directory {
            let child = directory
                .open_child_directory(&name)
                .map_err(|error| unavailable(&destination_path, "directory_unreadable", error))?;
            let names = child
                .child_names()
                .map_err(|error| unavailable(&destination_path, "directory_unreadable", error))?;
            Some((child, names))
        } else {
            None
        };
        evidence_map.insert(
            (
                EvidenceSide::Destination,
                mapping.source.clone(),
                relative.clone(),
            ),
            evidence(
                &metadata,
                EvidenceSide::Destination,
                &mapping.source,
                &relative,
                child_scan.as_ref().map(|(_, names)| names.clone()),
            ),
        );

        match eligible.get(&relative) {
            Some(expected) if *expected == kind => {}
            Some(_) => records.push(DiscoveryRecord {
                category: RecordCategory::UnsafeDestinationCollision,
                mapping_kind: mapping.kind,
                mapping_source: mapping.source.clone(),
                relative_path: Some(SafePath::from_bytes(&relative)),
                source_path: Some(SafePath::from_path(&source_path)),
                destination_path: SafePath::from_path(&destination_path),
                node_kind: kind,
                reason: unsupported_reason(kind).or(Some("wrong_node_kind")),
                blocking: true,
            }),
            None => records.push(DiscoveryRecord {
                category: RecordCategory::DestinationOnly,
                mapping_kind: mapping.kind,
                mapping_source: mapping.source.clone(),
                relative_path: Some(SafePath::from_bytes(&relative)),
                source_path: None,
                destination_path: SafePath::from_path(&destination_path),
                node_kind: kind,
                reason: if std::str::from_utf8(&relative).is_err() {
                    Some("non_utf8_path")
                } else {
                    unsupported_reason(kind)
                },
                blocking: false,
            }),
        }

        if let Some((child, names)) = child_scan {
            walk_destination(
                mapping,
                &child,
                root_device,
                relative,
                names,
                eligible,
                records,
                evidence_map,
            )?;
        }
    }
    Ok(())
}

fn inspect_file_mapping(
    mapping: &Mapping,
    records: &mut Vec<DiscoveryRecord>,
    evidence_map: &mut BTreeMap<(EvidenceSide, PathBuf, Vec<u8>), model::NodeEvidence>,
) -> Result<(), GripError> {
    let metadata = match metadata_at_path(&mapping.source) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(unavailable(&mapping.source, "path_unavailable", error)),
    };
    let kind = metadata.classify(metadata.stat.st_dev as u64);
    evidence_map.insert(
        (EvidenceSide::Source, mapping.source.clone(), Vec::new()),
        evidence(&metadata, EvidenceSide::Source, &mapping.source, &[], None),
    );
    let (category, reason, blocking) = if kind == NodeKind::File {
        (RecordCategory::Eligible, None, false)
    } else {
        (
            RecordCategory::UnsupportedSource,
            unsupported_reason(kind),
            true,
        )
    };
    records.push(DiscoveryRecord {
        category,
        mapping_kind: mapping.kind,
        mapping_source: mapping.source.clone(),
        relative_path: None,
        source_path: Some(SafePath::from_path(&mapping.source)),
        destination_path: SafePath::from_path(&mapping.destination),
        node_kind: kind,
        reason,
        blocking,
    });
    Ok(())
}

fn inspect_tree_mapping(
    mapping: &Mapping,
    records: &mut Vec<DiscoveryRecord>,
    evidence_map: &mut BTreeMap<(EvidenceSide, PathBuf, Vec<u8>), model::NodeEvidence>,
    policy_evidence: &mut BTreeMap<Vec<u8>, model::PolicyEvidence>,
) -> Result<(), GripError> {
    let root = match Directory::open(&mapping.source) {
        Ok(root) => root,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(unavailable(&mapping.source, "directory_unreadable", error)),
    };
    let root_device = root.root_metadata().st_dev as u64;
    let names = root
        .child_names()
        .map_err(|error| unavailable(&mapping.source, "directory_unreadable", error))?;
    let root_metadata = filesystem::NodeMetadata {
        stat: *root.root_metadata(),
    };
    evidence_map.insert(
        (EvidenceSide::Source, mapping.source.clone(), Vec::new()),
        evidence(
            &root_metadata,
            EvidenceSide::Source,
            &mapping.source,
            &[],
            Some(names.clone()),
        ),
    );
    walk_source(
        mapping,
        &root,
        root_device,
        Vec::new(),
        names,
        records,
        evidence_map,
        policy_evidence,
        &[],
    )
}

#[allow(clippy::too_many_arguments)]
fn walk_source(
    mapping: &Mapping,
    directory: &Directory,
    root_device: u64,
    parent_relative: Vec<u8>,
    names: Vec<Vec<u8>>,
    records: &mut Vec<DiscoveryRecord>,
    evidence_map: &mut BTreeMap<(EvidenceSide, PathBuf, Vec<u8>), model::NodeEvidence>,
    policy_evidence: &mut BTreeMap<Vec<u8>, model::PolicyEvidence>,
    inherited_policies: &[ignore_policy::GripignorePolicy],
) -> Result<(), GripError> {
    let directory_path = append_raw(&mapping.source, &parent_relative);
    let mut policies = inherited_policies.to_vec();
    if names.iter().any(|name| name == b".gripignore") {
        let (policy, key, evidence) = ignore_policy::load(
            directory,
            &mapping.source,
            &directory_path,
            &parent_relative,
            root_device,
        )?;
        policies.push(policy);
        policy_evidence.insert(key, evidence);
    }
    for name in names {
        if name == b".gripignore" {
            continue;
        }
        let relative = join_relative(&parent_relative, &name);
        let source_path = append_raw(&mapping.source, &relative);
        let destination_path = append_raw(&mapping.destination, &relative);
        let metadata = directory
            .metadata(&name)
            .map_err(|error| unavailable(&source_path, "path_unavailable", error))?;
        let kind = metadata.classify(root_device);
        let invalid_utf8 = std::str::from_utf8(&relative).is_err();
        let policy_ignored = policies
            .iter()
            .rev()
            .find_map(|policy| policy.decision(&source_path, kind == NodeKind::Directory))
            .unwrap_or(false);
        let (category, reason, blocking) = if policy_ignored {
            (RecordCategory::Ignored, None, false)
        } else if invalid_utf8 {
            (
                RecordCategory::UnsupportedSource,
                Some("non_utf8_path"),
                true,
            )
        } else if let Some(reason) = unsupported_reason(kind) {
            (RecordCategory::UnsupportedSource, Some(reason), true)
        } else {
            (RecordCategory::Eligible, None, false)
        };
        let child_scan = if category == RecordCategory::Eligible && kind == NodeKind::Directory {
            let child = directory
                .open_child_directory(&name)
                .map_err(|error| unavailable(&source_path, "directory_unreadable", error))?;
            let names = child
                .child_names()
                .map_err(|error| unavailable(&source_path, "directory_unreadable", error))?;
            Some((child, names))
        } else {
            None
        };
        evidence_map.insert(
            (
                EvidenceSide::Source,
                mapping.source.clone(),
                relative.clone(),
            ),
            evidence(
                &metadata,
                EvidenceSide::Source,
                &mapping.source,
                &relative,
                child_scan.as_ref().map(|(_, names)| names.clone()),
            ),
        );
        records.push(DiscoveryRecord {
            category,
            mapping_kind: mapping.kind,
            mapping_source: mapping.source.clone(),
            relative_path: Some(SafePath::from_bytes(&relative)),
            source_path: Some(SafePath::from_path(&source_path)),
            destination_path: SafePath::from_path(&destination_path),
            node_kind: kind,
            reason,
            blocking,
        });

        if let Some((child, names)) = child_scan {
            walk_source(
                mapping,
                &child,
                root_device,
                relative,
                names,
                records,
                evidence_map,
                policy_evidence,
                &policies,
            )?;
        }
    }
    Ok(())
}

fn join_relative(parent: &[u8], name: &[u8]) -> Vec<u8> {
    let mut relative =
        Vec::with_capacity(parent.len() + usize::from(!parent.is_empty()) + name.len());
    relative.extend_from_slice(parent);
    if !parent.is_empty() {
        relative.push(b'/');
    }
    relative.extend_from_slice(name);
    relative
}

fn append_raw(root: &Path, relative: &[u8]) -> PathBuf {
    let mut result = root.to_path_buf();
    if !relative.is_empty() {
        result.push(OsString::from_vec(relative.to_vec()));
    }
    result
}

fn unavailable(path: &Path, reason: &str, error: std::io::Error) -> GripError {
    GripError::discovery_operational(
        OPERATION,
        reason,
        vec![SafePath::from_path(path).display],
        &format!(
            "Could not inspect {}: {error}",
            SafePath::from_path(path).display
        ),
    )
}

fn stale(paths: Vec<PathBuf>, message: &str) -> GripError {
    GripError::discovery_operational(
        OPERATION,
        "stale_discovery_evidence",
        paths
            .iter()
            .map(|path| SafePath::from_path(path).display)
            .collect(),
        message,
    )
}

#[cfg(test)]
mod stale_tests {
    use super::*;
    use std::fs;

    fn tree_fixture() -> (tempfile::TempDir, GripHome, PathBuf) {
        let root = tempfile::tempdir().unwrap();
        let grip_home_path = root.path().join("grip-home");
        let source = root.path().join("source");
        let destination = root.path().join("destination");
        fs::create_dir(&grip_home_path).unwrap();
        fs::create_dir(&source).unwrap();
        fs::write(source.join("existing"), "value").unwrap();
        let source = fs::canonicalize(source).unwrap();
        let mut destination_canonical = fs::canonicalize(root.path()).unwrap();
        destination_canonical.push(destination.file_name().unwrap());
        fs::write(
            grip_home_path.join("config.toml"),
            format!(
                "schema_version = 1\n[[mappings]]\nkind = \"tree\"\nsource = {:?}\ndestination = {:?}\n",
                source.display().to_string(),
                destination_canonical.display().to_string()
            ),
        )
        .unwrap();
        let home = crate::home::select(Some(grip_home_path.into_os_string()), None).unwrap();
        (root, home, source)
    }

    #[test]
    fn second_pass_rejects_directory_enumeration_addition() {
        let (_root, home, source) = tree_fixture();
        let snapshot = publication::load(&home, false).unwrap();
        let error = inspect_with_between_pass(&home, &snapshot, None, || {
            fs::write(source.join("added"), "new").unwrap();
        })
        .unwrap_err();
        assert!(matches!(
            error,
            GripError::Mapping { ref reason, .. } if reason == "stale_discovery_evidence"
        ));
    }

    #[test]
    fn second_pass_rejects_directory_enumeration_removal() {
        let (_root, home, source) = tree_fixture();
        let snapshot = publication::load(&home, false).unwrap();
        let error = inspect_with_between_pass(&home, &snapshot, None, || {
            fs::remove_file(source.join("existing")).unwrap();
        })
        .unwrap_err();
        assert!(matches!(
            error,
            GripError::Mapping { ref reason, .. } if reason == "stale_discovery_evidence"
        ));
    }

    #[test]
    fn second_pass_rejects_entry_replacement() {
        let (_root, home, source) = tree_fixture();
        let snapshot = publication::load(&home, false).unwrap();
        let error = inspect_with_between_pass(&home, &snapshot, None, || {
            fs::remove_file(source.join("existing")).unwrap();
            fs::create_dir(source.join("existing")).unwrap();
        })
        .unwrap_err();
        assert!(matches!(
            error,
            GripError::Mapping { ref reason, .. } if reason == "stale_discovery_evidence"
        ));
    }

    #[test]
    fn second_pass_rejects_policy_byte_drift() {
        let (_root, home, source) = tree_fixture();
        fs::write(source.join(".gripignore"), "first\n").unwrap();
        let snapshot = publication::load(&home, false).unwrap();
        let error = inspect_with_between_pass(&home, &snapshot, None, || {
            fs::write(source.join(".gripignore"), "second\n").unwrap();
        })
        .unwrap_err();
        assert!(matches!(
            error,
            GripError::Mapping { ref reason, .. } if reason == "stale_discovery_evidence"
        ));
    }
}
