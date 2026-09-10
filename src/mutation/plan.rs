//! Pure deterministic direction-aware mutation-plan construction.

use crate::classification::model::{Classification, ClassificationRecord, ClassificationScope};
use crate::discovery::model::NodeKind;
use crate::error::GripError;
use crate::mutation::model::{
    ActionEvidence, ActionKind, ActionStatus, ConflictWinner, Disposition, EntryDisposition,
    MutationAction, MutationCounts, MutationDirection, MutationOperation, MutationPlan,
    PlanBlocker,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

/// One absent parent captured while a mapping endpoint was validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParentRequirement {
    pub mapping: crate::observation::model::ResolvedMapping,
    pub path: PathBuf,
}

/// Build a plan using only classified records and previously validated parent evidence.
pub fn build_with_parent_requirements(
    scope: ClassificationScope,
    records: Vec<ClassificationRecord>,
    requirements: Vec<ParentRequirement>,
) -> Result<MutationPlan, GripError> {
    build_with_parent_requirements_for(MutationOperation::Push, scope, records, requirements)
}

/// Build one bidirectional sync plan while preserving push parent requirements.
pub fn build_sync_with_parent_requirements(
    scope: ClassificationScope,
    records: Vec<ClassificationRecord>,
    requirements: Vec<ParentRequirement>,
) -> Result<MutationPlan, GripError> {
    build_with_parent_requirements_for(MutationOperation::Sync, scope, records, requirements)
}

fn build_with_parent_requirements_for(
    operation: MutationOperation,
    scope: ClassificationScope,
    records: Vec<ClassificationRecord>,
    requirements: Vec<ParentRequirement>,
) -> Result<MutationPlan, GripError> {
    let managed_directories = records
        .iter()
        .filter(|record| {
            record.classification == Classification::SourceAddition
                && record
                    .source
                    .as_ref()
                    .is_some_and(|state| state.node_kind == NodeKind::Directory)
        })
        .map(|record| record.identity.destination_path())
        .collect::<BTreeSet<_>>();
    let mut parents: BTreeMap<PathBuf, BTreeSet<crate::observation::model::EntryIdentity>> =
        BTreeMap::new();
    for requirement in requirements {
        if managed_directories.contains(&requirement.path) {
            continue;
        }
        for record in &records {
            if disposition_for_operation(operation, None, record.classification, record.blocking)
                == Disposition::Action
                && action_direction(operation, None, record.classification)
                    == MutationDirection::Push
                && record.identity.mapping == requirement.mapping
                && record
                    .identity
                    .destination_path()
                    .starts_with(&requirement.path)
            {
                parents
                    .entry(requirement.path.clone())
                    .or_default()
                    .insert(record.identity.clone());
            }
        }
    }
    build_with_parents(operation, None, scope, records, parents)
}

/// Build a complete plan from already validated and classified evidence.
pub fn build(
    scope: ClassificationScope,
    records: Vec<ClassificationRecord>,
) -> Result<MutationPlan, GripError> {
    build_with_parents(
        MutationOperation::Push,
        None,
        scope,
        records,
        BTreeMap::new(),
    )
}

/// Build a complete plan for one mutation direction.
pub fn build_for(
    direction: MutationDirection,
    scope: ClassificationScope,
    records: Vec<ClassificationRecord>,
) -> Result<MutationPlan, GripError> {
    build_with_parents(
        MutationOperation::directional(direction),
        None,
        scope,
        records,
        BTreeMap::new(),
    )
}

/// Build one exact-entry conflict resolution plan for an explicit winner.
pub fn build_resolution(
    scope: ClassificationScope,
    records: Vec<ClassificationRecord>,
    winner: ConflictWinner,
) -> Result<MutationPlan, GripError> {
    build_with_parents(
        MutationOperation::Resolve,
        Some(winner),
        scope,
        records,
        BTreeMap::new(),
    )
}

fn build_with_parents(
    operation: MutationOperation,
    winner: Option<ConflictWinner>,
    scope: ClassificationScope,
    mut records: Vec<ClassificationRecord>,
    parents: BTreeMap<PathBuf, BTreeSet<crate::observation::model::EntryIdentity>>,
) -> Result<MutationPlan, GripError> {
    records.sort_by(|first, second| first.identity.cmp(&second.identity));
    let mut entries = Vec::with_capacity(records.len());
    let mut parent_records = parents.into_iter().collect::<Vec<_>>();
    parent_records.sort_by(|(first, _), (second, _)| {
        first
            .components()
            .count()
            .cmp(&second.components().count())
            .then(
                first
                    .as_os_str()
                    .as_bytes()
                    .cmp(second.as_os_str().as_bytes()),
            )
    });
    let mut actions = parent_records
        .into_iter()
        .enumerate()
        .map(|(index, (path, identities))| MutationAction {
            index,
            direction: MutationDirection::Push,
            kind: ActionKind::CreateParentDirectory,
            identity: None,
            dependent_identities: identities
                .iter()
                .map(|identity| {
                    crate::discovery::model::SafePath::from_path(&identity.source_path())
                })
                .collect(),
            source_path: None,
            destination: path.clone(),
            destination_path: crate::discovery::model::SafePath::from_path(&path),
            expected_source: None,
            expected_destination: None,
            metadata: None,
            dependencies: Vec::new(),
            status: ActionStatus::Unattempted,
            milestones: ActionEvidence::default(),
            failure: None,
        })
        .collect::<Vec<_>>();
    let mut blockers = Vec::new();
    let mut finalizers = Vec::new();

    for mut record in records {
        let direction = action_direction(operation, winner, record.classification);
        let mut disposition =
            disposition_for_operation(operation, winner, record.classification, record.blocking);
        if disposition == Disposition::Action {
            let preflight = transition_preflight_blockers(&record, direction);
            if !preflight.is_empty() {
                record
                    .reasons
                    .extend(preflight.iter().map(|blocker| blocker.reason.clone()));
                blockers.extend(preflight);
                disposition = Disposition::Blocked;
            }
        }
        let mut action_indexes = Vec::new();
        if disposition == Disposition::Action {
            let (target, kind) = match direction {
                MutationDirection::Push => {
                    let source = record.source.clone().ok_or_else(|| {
                        GripError::Internal("actionable push entry has no source state".into())
                    })?;
                    let complete_kind = record
                        .source_complete
                        .as_ref()
                        .map(|state| state.node_kind)
                        .unwrap_or(source.node_kind);
                    let metadata_only = record
                        .source_complete
                        .as_ref()
                        .zip(record.destination_complete.as_ref())
                        .is_some_and(|(origin, target)| {
                            origin.node_kind == target.node_kind && origin.content == target.content
                        });
                    let kind = match (record.classification, complete_kind) {
                        (Classification::SourceAddition, NodeKind::Directory) => {
                            ActionKind::CreateDirectory
                        }
                        (Classification::SourceAddition, NodeKind::File) => ActionKind::AddFile,
                        (Classification::SourceOnlyChange, NodeKind::File) if metadata_only => {
                            ActionKind::ApplyMetadata
                        }
                        (Classification::SourceOnlyChange, NodeKind::File) => {
                            ActionKind::ReplaceFile
                        }
                        (Classification::SourceOnlyChange, NodeKind::Directory) => {
                            ActionKind::FinalizeDirectoryMetadata
                        }
                        (
                            Classification::InitialCollision
                            | Classification::DivergentConflict
                            | Classification::MetadataMigrationConflict,
                            NodeKind::File,
                        ) if operation == MutationOperation::Resolve => {
                            if metadata_only {
                                ActionKind::ApplyMetadata
                            } else {
                                ActionKind::ReplaceFile
                            }
                        }
                        (
                            Classification::InitialCollision
                            | Classification::DivergentConflict
                            | Classification::MetadataMigrationConflict,
                            NodeKind::Directory,
                        ) if operation == MutationOperation::Resolve => {
                            ActionKind::FinalizeDirectoryMetadata
                        }
                        _ => {
                            return Err(GripError::Internal(
                                "actionable classification has unsupported node state".into(),
                            ));
                        }
                    };
                    (record.identity.destination_path(), kind)
                }
                MutationDirection::Pull => {
                    let destination = record.destination.clone().ok_or_else(|| {
                        GripError::Internal("actionable pull entry has no destination state".into())
                    })?;
                    let complete_kind = record
                        .destination_complete
                        .as_ref()
                        .map(|state| state.node_kind)
                        .unwrap_or(destination.node_kind);
                    let metadata_only = record
                        .source_complete
                        .as_ref()
                        .zip(record.destination_complete.as_ref())
                        .is_some_and(|(target, origin)| {
                            origin.node_kind == target.node_kind && origin.content == target.content
                        });
                    let kind = match complete_kind {
                        NodeKind::File if metadata_only => ActionKind::ApplyMetadata,
                        NodeKind::File => ActionKind::ReplaceFile,
                        NodeKind::Directory => ActionKind::FinalizeDirectoryMetadata,
                        _ => {
                            return Err(GripError::Internal(
                                "actionable pull entry has unsupported node state".into(),
                            ));
                        }
                    };
                    (record.identity.source_path(), kind)
                }
            };
            let index = actions.len();
            action_indexes.push(index);
            actions.push(MutationAction {
                index,
                direction,
                kind,
                identity: Some(record.identity.clone()),
                dependent_identities: Vec::new(),
                source_path: Some(record.source_path.clone()),
                destination: target,
                destination_path: record.destination_path.clone(),
                expected_source: record.source.clone(),
                expected_destination: record.destination.clone(),
                metadata: metadata_action_evidence(&record, direction),
                dependencies: Vec::new(),
                status: ActionStatus::Unattempted,
                milestones: ActionEvidence::default(),
                failure: None,
            });
            if kind == ActionKind::FinalizeDirectoryMetadata {
                let mut finalizer = actions
                    .pop()
                    .expect("just-planned directory finalizer is present");
                finalizer.index = usize::MAX;
                finalizers.push(finalizer);
                action_indexes.clear();
            }
            if kind == ActionKind::CreateDirectory
                && let Some(expected_after) = record.source_complete.clone()
            {
                finalizers.push(MutationAction {
                    index: usize::MAX,
                    direction,
                    kind: ActionKind::FinalizeDirectoryMetadata,
                    identity: Some(record.identity.clone()),
                    dependent_identities: Vec::new(),
                    source_path: Some(record.source_path.clone()),
                    destination: record.identity.destination_path(),
                    destination_path: record.destination_path.clone(),
                    expected_source: record.source.clone(),
                    expected_destination: None,
                    metadata: Some(crate::mutation::model::MetadataActionEvidence {
                        expected_before: None,
                        expected_after,
                        changed_dimensions: all_metadata_dimensions(),
                        flags_to_clear: BTreeSet::new(),
                        capability_proofs: Vec::new(),
                    }),
                    dependencies: Vec::new(),
                    status: ActionStatus::Unattempted,
                    milestones: ActionEvidence::default(),
                    failure: None,
                });
            }
        } else if disposition == Disposition::Blocked {
            blockers.push(PlanBlocker {
                reason: record
                    .reasons
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "blocking_evidence".into()),
                paths: vec![record.source_path.clone(), record.destination_path.clone()],
            });
        }
        entries.push(EntryDisposition {
            identity: record.identity,
            classification: record.classification,
            source_path: record.source_path,
            destination_path: record.destination_path,
            disposition,
            action_indexes,
            reasons: record.reasons,
        });
    }

    finalizers.sort_by(|first, second| {
        second
            .destination
            .components()
            .count()
            .cmp(&first.destination.components().count())
            .then(first.destination.cmp(&second.destination))
    });
    for mut finalizer in finalizers {
        finalizer.index = actions.len();
        finalizer.dependencies = actions
            .iter()
            .filter(|action| {
                action.index != finalizer.index
                    && action.destination.starts_with(&finalizer.destination)
            })
            .map(|action| action.index)
            .collect();
        if let Some(identity) = &finalizer.identity
            && let Some(entry) = entries.iter_mut().find(|entry| &entry.identity == identity)
        {
            entry.action_indexes.push(finalizer.index);
        }
        actions.push(finalizer);
    }

    attach_directory_dependencies(&mut actions);
    let counts = MutationCounts {
        selected: entries.len(),
        actionable: actions.len(),
        no_action: entries
            .iter()
            .filter(|entry| entry.disposition == Disposition::NoAction)
            .count(),
        converged: entries
            .iter()
            .filter(|entry| entry.disposition == Disposition::AcceptOnly)
            .count(),
        blockers: blockers.len(),
        completed: 0,
        failed: 0,
        unattempted: actions.len(),
    };
    let mut plan = MutationPlan {
        operation,
        direction: match operation {
            MutationOperation::Push => Some(MutationDirection::Push),
            MutationOperation::Pull => Some(MutationDirection::Pull),
            MutationOperation::Sync | MutationOperation::Resolve => None,
        },
        winner,
        plan_id: String::new(),
        scope,
        acceptance_identities: entries
            .iter()
            .filter(|entry| {
                matches!(
                    entry.disposition,
                    Disposition::Action | Disposition::AcceptOnly
                )
            })
            .map(|entry| entry.source_path.clone())
            .collect(),
        entries,
        actions,
        blockers,
        counts,
    };
    plan.plan_id = plan_digest(&plan)?;
    Ok(plan)
}

fn all_metadata_dimensions() -> BTreeSet<crate::metadata::model::MetadataDimension> {
    BTreeSet::from([
        crate::metadata::model::MetadataDimension::PermissionMode,
        crate::metadata::model::MetadataDimension::Owner,
        crate::metadata::model::MetadataDimension::Group,
        crate::metadata::model::MetadataDimension::ModificationTime,
        crate::metadata::model::MetadataDimension::ExtendedAttribute,
        crate::metadata::model::MetadataDimension::AccessControlList,
        crate::metadata::model::MetadataDimension::BsdFlags,
    ])
}

fn transition_preflight_blockers(
    record: &ClassificationRecord,
    direction: MutationDirection,
) -> Vec<PlanBlocker> {
    use crate::metadata::model::{EndpointRole, Evidence, MetadataDimension};

    if record.source_complete.is_none() && record.destination_complete.is_none() {
        return Vec::new();
    }
    let target_role = match direction {
        MutationDirection::Push => EndpointRole::Destination,
        MutationDirection::Pull => EndpointRole::Source,
    };
    let paths = vec![record.source_path.clone(), record.destination_path.clone()];
    let mut blockers = Vec::new();
    let profile = record
        .endpoint_capabilities
        .iter()
        .find(|profile| profile.endpoint == target_role);
    let Some(profile) = profile else {
        blockers.push(PlanBlocker {
            reason: "target_capability_unavailable".into(),
            paths: paths.clone(),
        });
        return blockers;
    };
    if !matches!(&profile.filesystem_type, Evidence::Observed { value } if value == "apfs") {
        blockers.push(PlanBlocker {
            reason: "target_filesystem_not_apfs".into(),
            paths: paths.clone(),
        });
    }
    for dimension in [
        MetadataDimension::PermissionMode,
        MetadataDimension::Owner,
        MetadataDimension::Group,
        MetadataDimension::ModificationTime,
        MetadataDimension::ExtendedAttribute,
        MetadataDimension::AccessControlList,
        MetadataDimension::BsdFlags,
    ] {
        let capable = profile.capabilities.iter().any(|capability| {
            capability.dimension == dimension
                && matches!(capability.apply, Evidence::Observed { value: true })
                && matches!(capability.verify, Evidence::Observed { value: true })
        });
        if !capable {
            blockers.push(PlanBlocker {
                reason: format!(
                    "target_{}_capability_unavailable",
                    metadata_dimension_reason(dimension)
                ),
                paths: paths.clone(),
            });
        }
    }
    blockers.extend(
        record
            .compatibility_findings
            .iter()
            .filter(|finding| finding.endpoint == target_role && finding.blocking)
            .map(|finding| PlanBlocker {
                reason: format!("metadata_{}", compatibility_reason(finding.reason)),
                paths: paths.clone(),
            }),
    );
    blockers.sort_by(|first, second| first.reason.cmp(&second.reason));
    blockers.dedup_by(|first, second| first.reason == second.reason && first.paths == second.paths);
    blockers
}

fn metadata_dimension_reason(dimension: crate::metadata::model::MetadataDimension) -> &'static str {
    use crate::metadata::model::MetadataDimension;
    match dimension {
        MetadataDimension::Node => "node",
        MetadataDimension::Content => "content",
        MetadataDimension::PermissionMode => "permission_mode",
        MetadataDimension::Owner => "owner",
        MetadataDimension::Group => "group",
        MetadataDimension::ModificationTime => "modification_time",
        MetadataDimension::ExtendedAttribute => "extended_attribute",
        MetadataDimension::AccessControlList => "access_control_list",
        MetadataDimension::BsdFlags => "bsd_flags",
        MetadataDimension::Mount => "mount",
        MetadataDimension::Case => "case",
        MetadataDimension::Unicode => "unicode",
    }
}

fn compatibility_reason(reason: crate::metadata::model::CompatibilityReason) -> &'static str {
    use crate::metadata::model::CompatibilityReason;
    match reason {
        CompatibilityReason::Unavailable => "unavailable",
        CompatibilityReason::Unsupported => "unsupported",
        CompatibilityReason::Unreadable => "unreadable",
        CompatibilityReason::Unauthorized => "unauthorized",
        CompatibilityReason::UnknownXattr => "unknown_xattr",
        CompatibilityReason::ExcludedXattr => "excluded_xattr",
        CompatibilityReason::ProtectedFlag => "protected_flag",
        CompatibilityReason::ApfsCollision => "apfs_collision",
        CompatibilityReason::InconclusiveComparison => "inconclusive_comparison",
        CompatibilityReason::SparseFile => "sparse_file",
        CompatibilityReason::HardLink => "hard_link",
        CompatibilityReason::MountBoundary => "mount_boundary",
    }
}

fn metadata_action_evidence(
    record: &ClassificationRecord,
    direction: MutationDirection,
) -> Option<crate::mutation::model::MetadataActionEvidence> {
    let (before, after) = match direction {
        MutationDirection::Push => (
            record.destination_complete.as_ref(),
            record.source_complete.as_ref()?,
        ),
        MutationDirection::Pull => (
            record.source_complete.as_ref(),
            record.destination_complete.as_ref()?,
        ),
    };
    let changed_dimensions = before.map_or_else(all_metadata_dimensions, |before| {
        crate::classification::changed_dimensions_complete(Some(before), Some(after))
            .unwrap_or_default()
            .into_iter()
            .map(|dimension| match dimension {
                crate::classification::model::ChangedDimension::NodeKind => {
                    crate::metadata::model::MetadataDimension::Node
                }
                crate::classification::model::ChangedDimension::Content => {
                    crate::metadata::model::MetadataDimension::Content
                }
                crate::classification::model::ChangedDimension::PermissionMode => {
                    crate::metadata::model::MetadataDimension::PermissionMode
                }
                crate::classification::model::ChangedDimension::Owner => {
                    crate::metadata::model::MetadataDimension::Owner
                }
                crate::classification::model::ChangedDimension::Group => {
                    crate::metadata::model::MetadataDimension::Group
                }
                crate::classification::model::ChangedDimension::ModificationTime => {
                    crate::metadata::model::MetadataDimension::ModificationTime
                }
                crate::classification::model::ChangedDimension::ExtendedAttribute => {
                    crate::metadata::model::MetadataDimension::ExtendedAttribute
                }
                crate::classification::model::ChangedDimension::AccessControlList => {
                    crate::metadata::model::MetadataDimension::AccessControlList
                }
                crate::classification::model::ChangedDimension::BsdFlags => {
                    crate::metadata::model::MetadataDimension::BsdFlags
                }
            })
            .collect()
    });
    let flags_to_clear = before.map_or_else(BTreeSet::new, |before| {
        before
            .metadata
            .bsd_flags
            .iter()
            .filter(|flag| {
                matches!(
                    flag,
                    crate::metadata::model::BsdFlag::Immutable
                        | crate::metadata::model::BsdFlag::Append
                )
            })
            .copied()
            .collect()
    });
    Some(crate::mutation::model::MetadataActionEvidence {
        expected_before: before.cloned(),
        expected_after: after.clone(),
        changed_dimensions,
        flags_to_clear,
        capability_proofs: Vec::new(),
    })
}

/// Map one inherited classification to its push behavior.
pub fn disposition_for(classification: Classification, blocking: bool) -> Disposition {
    disposition_for_direction(MutationDirection::Push, classification, blocking)
}

/// Map one inherited classification to direction-specific mutation behavior.
pub fn disposition_for_direction(
    direction: MutationDirection,
    classification: Classification,
    blocking: bool,
) -> Disposition {
    if blocking {
        return Disposition::Blocked;
    }
    match direction {
        MutationDirection::Push => match classification {
            Classification::SourceAddition | Classification::SourceOnlyChange => {
                Disposition::Action
            }
            _ => Disposition::NoAction,
        },
        MutationDirection::Pull => match classification {
            Classification::DestinationOnlyChange => Disposition::Action,
            _ => Disposition::NoAction,
        },
    }
}

/// Map an inherited classification to one operation-specific behavior.
pub fn disposition_for_operation(
    operation: MutationOperation,
    winner: Option<ConflictWinner>,
    classification: Classification,
    blocking: bool,
) -> Disposition {
    match operation {
        MutationOperation::Push => {
            disposition_for_direction(MutationDirection::Push, classification, blocking)
        }
        MutationOperation::Pull => {
            disposition_for_direction(MutationDirection::Pull, classification, blocking)
        }
        MutationOperation::Sync => {
            if blocking {
                return Disposition::Blocked;
            }
            match classification {
                Classification::SourceAddition | Classification::SourceOnlyChange => {
                    Disposition::Action
                }
                Classification::DestinationOnlyChange => Disposition::Action,
                Classification::ConvergedTwoSidedChange => Disposition::AcceptOnly,
                _ => Disposition::NoAction,
            }
        }
        MutationOperation::Resolve => {
            if winner.is_some()
                && matches!(
                    classification,
                    Classification::InitialCollision
                        | Classification::DivergentConflict
                        | Classification::MetadataMigrationConflict
                )
            {
                Disposition::Action
            } else {
                Disposition::Blocked
            }
        }
    }
}

fn action_direction(
    operation: MutationOperation,
    winner: Option<ConflictWinner>,
    classification: Classification,
) -> MutationDirection {
    match operation {
        MutationOperation::Push => MutationDirection::Push,
        MutationOperation::Pull => MutationDirection::Pull,
        MutationOperation::Sync => {
            if classification == Classification::DestinationOnlyChange {
                MutationDirection::Pull
            } else {
                MutationDirection::Push
            }
        }
        MutationOperation::Resolve => match winner {
            Some(ConflictWinner::Source) => MutationDirection::Push,
            Some(ConflictWinner::Destination) => MutationDirection::Pull,
            None => MutationDirection::Push,
        },
    }
}

fn attach_directory_dependencies(actions: &mut [MutationAction]) {
    let directories = actions
        .iter()
        .filter(|action| {
            matches!(
                action.kind,
                ActionKind::CreateParentDirectory | ActionKind::CreateDirectory
            )
        })
        .map(|action| (action.destination.clone(), action.index))
        .collect::<BTreeMap<_, _>>();
    for action in actions {
        let Some(parent) = action.destination.parent() else {
            continue;
        };
        if let Some(index) = directories.get(parent)
            && *index < action.index
        {
            action.dependencies.push(*index);
        }
    }
}

#[derive(Serialize)]
struct PlanIdentity<'a> {
    operation: MutationOperation,
    direction: Option<MutationDirection>,
    winner: Option<ConflictWinner>,
    scope: &'a ClassificationScope,
    entries: &'a [EntryDisposition],
    acceptance_identities: &'a [crate::discovery::model::SafePath],
    actions: &'a [MutationAction],
    blockers: &'a [PlanBlocker],
    counts: &'a MutationCounts,
}

fn plan_digest(plan: &MutationPlan) -> Result<String, GripError> {
    let bytes = serde_json::to_vec(&PlanIdentity {
        operation: plan.operation,
        direction: plan.direction,
        winner: plan.winner,
        scope: &plan.scope,
        entries: &plan.entries,
        acceptance_identities: &plan.acceptance_identities,
        actions: &plan.actions,
        blockers: &plan.blockers,
        counts: &plan.counts,
    })
    .map_err(|error| GripError::Internal(format!("could not encode mutation plan: {error}")))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disposition_for_covers_all_inherited_classifications() {
        use Classification::*;
        let classifications = [
            SourceAddition,
            InitialMatch,
            InitialCollision,
            DestinationOnlyUnmanaged,
            Synchronized,
            SourceOnlyChange,
            DestinationOnlyChange,
            ConvergedTwoSidedChange,
            DivergentConflict,
            SourceSideDeletion,
            DestinationSideDeletion,
            DeleteChangeConflict,
            ChangeDeleteConflict,
            ConvergedDeletion,
            UnsupportedManaged,
            UnsafeCollision,
        ];
        for classification in classifications {
            let expected = if matches!(classification, SourceAddition | SourceOnlyChange) {
                Disposition::Action
            } else {
                Disposition::NoAction
            };
            assert_eq!(disposition_for(classification, false), expected);
            assert_eq!(disposition_for(classification, true), Disposition::Blocked);
        }
    }

    #[test]
    fn sync_and_resolution_policies_cover_all_inherited_classifications() {
        use Classification::*;
        let classifications = [
            SourceAddition,
            InitialMatch,
            InitialCollision,
            DestinationOnlyUnmanaged,
            Synchronized,
            SourceOnlyChange,
            DestinationOnlyChange,
            ConvergedTwoSidedChange,
            DivergentConflict,
            SourceSideDeletion,
            DestinationSideDeletion,
            DeleteChangeConflict,
            ChangeDeleteConflict,
            ConvergedDeletion,
            UnsupportedManaged,
            UnsafeCollision,
        ];
        for classification in classifications {
            let expected = match classification {
                SourceAddition | SourceOnlyChange | DestinationOnlyChange => Disposition::Action,
                ConvergedTwoSidedChange => Disposition::AcceptOnly,
                _ => Disposition::NoAction,
            };
            assert_eq!(
                disposition_for_operation(MutationOperation::Sync, None, classification, false),
                expected
            );
            assert_eq!(
                disposition_for_operation(MutationOperation::Sync, None, classification, true),
                Disposition::Blocked
            );
            assert_eq!(
                disposition_for_operation(
                    MutationOperation::Resolve,
                    Some(ConflictWinner::Source),
                    classification,
                    true,
                ),
                if matches!(classification, InitialCollision | DivergentConflict) {
                    Disposition::Action
                } else {
                    Disposition::Blocked
                }
            );
        }
    }
}
