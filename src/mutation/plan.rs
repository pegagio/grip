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
    pub mapping: crate::observation::model::MappingSnapshot,
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
            dependencies: Vec::new(),
            status: ActionStatus::Unattempted,
            milestones: ActionEvidence::default(),
            failure: None,
        })
        .collect::<Vec<_>>();
    let mut blockers = Vec::new();

    for mut record in records {
        let direction = action_direction(operation, winner, record.classification);
        let mut disposition =
            disposition_for_operation(operation, winner, record.classification, record.blocking);
        if operation == MutationOperation::Resolve
            && disposition == Disposition::Action
            && (!record
                .source
                .as_ref()
                .is_some_and(|state| state.node_kind == NodeKind::File)
                || !record
                    .destination
                    .as_ref()
                    .is_some_and(|state| state.node_kind == NodeKind::File))
        {
            disposition = Disposition::Blocked;
            record.reasons = vec!["unsupported_resolution_transition".into()];
        }
        if direction == MutationDirection::Pull
            && disposition == Disposition::Action
            && record
                .destination
                .as_ref()
                .is_some_and(|state| state.node_kind == NodeKind::Directory)
        {
            disposition = Disposition::Blocked;
            record.reasons = vec!["unsupported_directory_metadata_transition".into()];
        }
        let mut action_indexes = Vec::new();
        if disposition == Disposition::Action {
            let (target, kind) = match direction {
                MutationDirection::Push => {
                    let source = record.source.clone().ok_or_else(|| {
                        GripError::Internal("actionable push entry has no source state".into())
                    })?;
                    let kind = match (record.classification, source.node_kind) {
                        (Classification::SourceAddition, NodeKind::Directory) => {
                            ActionKind::CreateDirectory
                        }
                        (Classification::SourceAddition, NodeKind::File) => ActionKind::AddFile,
                        (Classification::SourceOnlyChange, NodeKind::File) => {
                            ActionKind::ReplaceFile
                        }
                        (Classification::DivergentConflict, NodeKind::File)
                            if operation == MutationOperation::Resolve =>
                        {
                            ActionKind::ReplaceFile
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
                    if destination.node_kind != NodeKind::File {
                        return Err(GripError::Internal(
                            "actionable pull entry is not a regular file".into(),
                        ));
                    }
                    (record.identity.source_path(), ActionKind::ReplaceFile)
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
                dependencies: Vec::new(),
                status: ActionStatus::Unattempted,
                milestones: ActionEvidence::default(),
                failure: None,
            });
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
            if winner.is_some() && classification == Classification::DivergentConflict {
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
            NewlyIgnoredPendingRetirement,
            UntrackedPendingRetirement,
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
            NewlyIgnoredPendingRetirement,
            UntrackedPendingRetirement,
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
                if classification == DivergentConflict {
                    Disposition::Action
                } else {
                    Disposition::Blocked
                }
            );
        }
    }
}
