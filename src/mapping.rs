//! Mapping domain values and complete-registry ownership validation.

use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The two mapping extents supported by Grip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MappingKind {
    File,
    Tree,
}

impl MappingKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Tree => "tree",
        }
    }
}

/// A version-controllable mapping declaration with no machine-specific paths.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortableMapping {
    pub kind: MappingKind,
    pub source: crate::path_policy::ProjectRelativePath,
    pub destination: crate::path_policy::DestinationPath,
}

impl PortableMapping {
    /// Parse command-line mapping arguments into a normalized declaration.
    pub fn parse_cli(
        kind: MappingKind,
        source: &OsStr,
        destination: &OsStr,
    ) -> Result<Self, crate::GripError> {
        Ok(Self {
            kind,
            source: crate::path_policy::ProjectRelativePath::parse_cli(
                source,
                kind == MappingKind::Tree,
            )?,
            destination: crate::path_policy::DestinationPath::parse(destination)?,
        })
    }

    pub fn parse(
        kind: MappingKind,
        source: &OsStr,
        destination: &OsStr,
    ) -> Result<Self, crate::GripError> {
        Ok(Self {
            kind,
            source: crate::path_policy::ProjectRelativePath::parse(
                source,
                kind == MappingKind::Tree,
            )?,
            destination: crate::path_policy::DestinationPath::parse(destination)?,
        })
    }

    pub fn identity(&self) -> String {
        format!(
            "{}\0{}\0{}",
            self.source.as_str(),
            self.kind.as_str(),
            self.destination.as_str()
        )
    }

    pub fn resolve(
        &self,
        project_root: &Path,
        home_root: &Path,
        operation: &str,
    ) -> Result<ResolvedMapping, crate::GripError> {
        let source_path = self.source.resolve(project_root);
        let destination_path = self.destination.resolve(project_root, home_root);
        let source_evidence =
            crate::path_policy::inspect_durable_endpoint(&source_path, self.kind, true, operation)?;
        let destination_evidence = crate::path_policy::inspect_durable_destination_leaf_link(
            &destination_path,
            self.kind,
            operation,
        )?;
        Ok(ResolvedMapping {
            declaration: self.clone(),
            source: source_evidence.canonical.clone(),
            destination: destination_evidence.canonical.clone(),
            source_evidence,
            destination_evidence,
        })
    }
}

/// Runtime endpoints resolved from one portable declaration and command context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedMapping {
    pub declaration: PortableMapping,
    pub source: PathBuf,
    pub destination: PathBuf,
    pub source_evidence: crate::path_policy::PathEvidence,
    pub destination_evidence: crate::path_policy::PathEvidence,
}

impl ResolvedMapping {
    pub fn ownership_mapping(&self) -> Mapping {
        Mapping::new(
            self.declaration.kind,
            self.source.clone(),
            self.destination.clone(),
        )
    }
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

/// Component-aware relationship between a managed member and the path locating its source below
/// the destination root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedMemberRelation {
    Equal,
    Ancestor,
    Descendant,
    Disjoint,
}

impl ManagedMemberRelation {
    /// Return the stable machine-readable relationship name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Equal => "equal",
            Self::Ancestor => "ancestor",
            Self::Descendant => "descendant",
            Self::Disjoint => "disjoint",
        }
    }

    /// Return whether this relationship can make a contained-source mapping recursive.
    pub const fn is_recursive(self) -> bool {
        !matches!(self, Self::Disjoint)
    }
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

/// Derive the non-empty destination-relative path locating a contained tree source.
pub fn contained_source_prefix(mapping: &Mapping) -> Option<Vec<u8>> {
    use std::os::unix::ffi::OsStrExt;

    if mapping.kind != MappingKind::Tree || mapping.source == mapping.destination {
        return None;
    }
    mapping
        .source
        .strip_prefix(&mapping.destination)
        .ok()
        .map(|relative| relative.as_os_str().as_bytes().to_vec())
        .filter(|relative| !relative.is_empty())
}

/// Compare one non-empty managed relative identity with a contained source prefix.
pub fn managed_member_relation(member: &[u8], prefix: &[u8]) -> ManagedMemberRelation {
    if member == prefix {
        ManagedMemberRelation::Equal
    } else if raw_descendant(prefix, member) {
        ManagedMemberRelation::Ancestor
    } else if raw_descendant(member, prefix) {
        ManagedMemberRelation::Descendant
    } else {
        ManagedMemberRelation::Disjoint
    }
}

fn raw_descendant(candidate: &[u8], parent: &[u8]) -> bool {
    candidate
        .strip_prefix(parent)
        .is_some_and(|suffix| suffix.first() == Some(&b'/'))
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
        let path_relation = relation(&mapping.source, &mapping.destination);
        if path_relation != Relation::Disjoint {
            if mapping.kind == MappingKind::Tree
                && path_relation == Relation::Descendant
                && contained_source_prefix(mapping).is_some()
            {
                continue;
            }
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
            ) && !permits_exact_file_reservation(first, second)
            {
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
                    && !permits_cross_mapping_source(first, second, source_owner, destination_owner)
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

/// Return current-source conflicts for the one allowed exact-leaf reservation shape.
///
/// A contained tree destination may contain a separately sourced exact file destination, but only
/// while the tree source has no member that would map to the reserved leaf.
pub fn validate_contained_tree_reservations(mappings: &[Mapping]) -> Vec<OwnershipConflict> {
    let mut conflicts = Vec::new();
    for (index, first) in mappings.iter().enumerate() {
        for second in &mappings[index + 1..] {
            let Some((file, tree)) = exact_file_and_contained_tree(first, second) else {
                continue;
            };
            let relative = file
                .destination
                .strip_prefix(&tree.destination)
                .expect("validated contained destination relation");
            let tree_member = tree.source.join(relative);
            if std::fs::symlink_metadata(&tree_member).is_ok() {
                conflicts.push(OwnershipConflict {
                    reason: "reserved_destination_member",
                    first_mapping: file.source.clone(),
                    second_mapping: Some(tree.source.clone()),
                    first_path: tree_member,
                    second_path: file.destination.clone(),
                    relation: "equal",
                });
            }
        }
    }
    conflicts.sort();
    conflicts.dedup();
    conflicts
}

fn permits_exact_file_reservation(first: &Mapping, second: &Mapping) -> bool {
    exact_file_and_contained_tree(first, second).is_some()
}

fn permits_cross_mapping_source(
    first: &Mapping,
    second: &Mapping,
    source_owner: &Mapping,
    destination_owner: &Mapping,
) -> bool {
    exact_file_and_contained_tree(first, second).is_some()
        && source_owner.kind == MappingKind::File
        && destination_owner.kind == MappingKind::Tree
}

fn exact_file_and_contained_tree<'a>(
    first: &'a Mapping,
    second: &'a Mapping,
) -> Option<(&'a Mapping, &'a Mapping)> {
    [(first, second), (second, first)]
        .into_iter()
        .find(|(file, tree)| {
            file.kind == MappingKind::File
                && tree.kind == MappingKind::Tree
                && contained_source_prefix(tree).is_some()
                && relation(&file.destination, &tree.destination) == Relation::Descendant
                && relation(&file.source, &tree.source) == Relation::Disjoint
        })
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

    #[test]
    fn validate_ownership_allows_only_tree_source_beneath_destination() {
        assert!(
            validate_ownership(&[mapping(MappingKind::Tree, "/home/project/home", "/home")])
                .is_empty()
        );
        assert_eq!(
            validate_ownership(&[mapping(MappingKind::Tree, "/home", "/home/project/home")])[0]
                .reason,
            "recursive_topology"
        );
        assert_eq!(
            validate_ownership(&[mapping(MappingKind::File, "/home/project/file", "/home")])[0]
                .reason,
            "recursive_topology"
        );
    }

    #[test]
    fn managed_member_relations_use_component_boundaries() {
        let prefix = b"dotfiles/home";
        assert_eq!(
            managed_member_relation(b"dotfiles/home", prefix),
            ManagedMemberRelation::Equal
        );
        assert_eq!(
            managed_member_relation(b"dotfiles", prefix),
            ManagedMemberRelation::Ancestor
        );
        assert_eq!(
            managed_member_relation(b"dotfiles/home/nested", prefix),
            ManagedMemberRelation::Descendant
        );
        assert_eq!(
            managed_member_relation(b"dotfiles-home", prefix),
            ManagedMemberRelation::Disjoint
        );
    }
}
