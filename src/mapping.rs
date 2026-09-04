//! Mapping domain values and complete-registry ownership validation.

use serde::Serialize;
use std::path::{Path, PathBuf};

/// The two mapping extents supported by Grip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MappingKind {
    File,
    Tree,
}

/// One canonical source-to-destination ownership declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mapping {
    pub kind: MappingKind,
    pub source: PathBuf,
    pub destination: PathBuf,
}

/// One exact or prospective-tree path extent owned by a mapping endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Namespace {
    pub root: PathBuf,
    pub kind: MappingKind,
}

impl Namespace {
    pub fn new(root: PathBuf, kind: MappingKind) -> Self {
        Self { root, kind }
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        namespaces_overlap(&self.root, self.kind, &other.root, other.kind).is_some()
    }
}

impl Mapping {
    pub fn new(kind: MappingKind, source: PathBuf, destination: PathBuf) -> Self {
        Self {
            kind,
            source,
            destination,
        }
    }
}

/// A stable explanation of an ownership conflict.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct OwnershipConflict {
    pub reason: &'static str,
    pub first_mapping: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_mapping: Option<PathBuf>,
    pub first_path: PathBuf,
    pub second_path: PathBuf,
    pub relation: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Relation {
    Equal,
    Ancestor,
    Descendant,
    Disjoint,
}

fn relation(first: &Path, second: &Path) -> Relation {
    if first == second {
        Relation::Equal
    } else if second.starts_with(first) {
        Relation::Ancestor
    } else if first.starts_with(second) {
        Relation::Descendant
    } else {
        Relation::Disjoint
    }
}

fn relation_name(value: Relation) -> &'static str {
    match value {
        Relation::Equal => "equal",
        Relation::Ancestor => "ancestor",
        Relation::Descendant => "descendant",
        Relation::Disjoint => "disjoint",
    }
}

fn namespaces_overlap(
    first_path: &Path,
    first_kind: MappingKind,
    second_path: &Path,
    second_kind: MappingKind,
) -> Option<Relation> {
    let relation = relation(first_path, second_path);
    let overlaps = matches!(
        (first_kind, second_kind, relation),
        (_, _, Relation::Equal)
            | (MappingKind::Tree, _, Relation::Ancestor)
            | (_, MappingKind::Tree, Relation::Descendant)
    );
    overlaps.then_some(relation)
}

fn conflict(
    reason: &'static str,
    first: &Mapping,
    second: Option<&Mapping>,
    first_path: &Path,
    second_path: &Path,
    relation: Relation,
) -> OwnershipConflict {
    OwnershipConflict {
        reason,
        first_mapping: first.source.clone(),
        second_mapping: second.map(|mapping| mapping.source.clone()),
        first_path: first_path.to_path_buf(),
        second_path: second_path.to_path_buf(),
        relation: relation_name(relation),
    }
}

/// Return every deterministic ownership conflict in a complete registry.
pub fn validate_ownership(mappings: &[Mapping]) -> Vec<OwnershipConflict> {
    let mut conflicts = Vec::new();
    for mapping in mappings {
        if let Some(path_relation) = namespaces_overlap(
            &mapping.source,
            mapping.kind,
            &mapping.destination,
            mapping.kind,
        ) {
            let reason = if path_relation == Relation::Equal {
                "equal_endpoints"
            } else {
                "recursive_topology"
            };
            conflicts.push(conflict(
                reason,
                mapping,
                None,
                &mapping.source,
                &mapping.destination,
                path_relation,
            ));
        }
    }
    for (index, first) in mappings.iter().enumerate() {
        for second in &mappings[index + 1..] {
            if first.source == second.source {
                conflicts.push(conflict(
                    if first == second {
                        "duplicate_tuple"
                    } else {
                        "duplicate_source"
                    },
                    first,
                    Some(second),
                    &first.source,
                    &second.source,
                    Relation::Equal,
                ));
            } else if let Some(path_relation) =
                namespaces_overlap(&first.source, first.kind, &second.source, second.kind)
            {
                conflicts.push(conflict(
                    "source_overlap",
                    first,
                    Some(second),
                    &first.source,
                    &second.source,
                    path_relation,
                ));
            }
            if let Some(path_relation) = namespaces_overlap(
                &first.destination,
                first.kind,
                &second.destination,
                second.kind,
            ) {
                conflicts.push(conflict(
                    "destination_overlap",
                    first,
                    Some(second),
                    &first.destination,
                    &second.destination,
                    path_relation,
                ));
            }
            for (
                source_owner,
                source_path,
                source_kind,
                destination_owner,
                destination_path,
                destination_kind,
            ) in [
                (
                    first,
                    &first.source,
                    first.kind,
                    second,
                    &second.destination,
                    second.kind,
                ),
                (
                    second,
                    &second.source,
                    second.kind,
                    first,
                    &first.destination,
                    first.kind,
                ),
            ] {
                if let Some(path_relation) =
                    namespaces_overlap(source_path, source_kind, destination_path, destination_kind)
                {
                    conflicts.push(conflict(
                        "cross_mapping_recursion",
                        source_owner,
                        Some(destination_owner),
                        source_path,
                        destination_path,
                        path_relation,
                    ));
                }
            }
        }
    }
    conflicts.sort();
    conflicts.dedup();
    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mapping(kind: MappingKind, source: &str, destination: &str) -> Mapping {
        Mapping::new(kind, PathBuf::from(source), PathBuf::from(destination))
    }

    #[test]
    fn validate_ownership_allows_component_boundary_siblings() {
        let mappings = vec![
            mapping(MappingKind::Tree, "/source/a", "/destination/a"),
            mapping(MappingKind::Tree, "/source/ab", "/destination/ab"),
        ];
        assert!(validate_ownership(&mappings).is_empty());
    }

    #[test]
    fn validate_ownership_reports_nested_source() {
        let mappings = vec![
            mapping(MappingKind::Tree, "/source/a", "/destination/a"),
            mapping(MappingKind::File, "/source/a/file", "/destination/b"),
        ];
        assert_eq!(validate_ownership(&mappings)[0].reason, "source_overlap");
    }
}
