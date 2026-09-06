//! Pure deterministic push-plan construction.

use crate::classification::model::{Classification, ClassificationRecord, ClassificationScope};
use crate::discovery::model::NodeKind;
use crate::error::GripError;
use crate::push::model::{
    ActionEvidence, ActionKind, ActionStatus, Disposition, EntryDisposition, PlanBlocker,
    PushAction, PushCounts, PushPlan,
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
) -> Result<PushPlan, GripError> {
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
            if disposition_for(record.classification, record.blocking) == Disposition::Action
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
    build_with_parents(scope, records, parents)
}

/// Build a complete plan from already validated and classified evidence.
pub fn build(
    scope: ClassificationScope,
    records: Vec<ClassificationRecord>,
) -> Result<PushPlan, GripError> {
    build_with_parents(scope, records, BTreeMap::new())
}

fn build_with_parents(
    scope: ClassificationScope,
    mut records: Vec<ClassificationRecord>,
    parents: BTreeMap<PathBuf, BTreeSet<crate::observation::model::EntryIdentity>>,
) -> Result<PushPlan, GripError> {
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
        .map(|(index, (path, identities))| PushAction {
            index,
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

    for record in records {
        let disposition = disposition_for(record.classification, record.blocking);
        let mut action_indexes = Vec::new();
        if disposition == Disposition::Action {
            let source = record.source.clone().ok_or_else(|| {
                GripError::Internal("actionable push entry has no source state".into())
            })?;
            let kind = match (record.classification, source.node_kind) {
                (Classification::SourceAddition, NodeKind::Directory) => {
                    ActionKind::CreateDirectory
                }
                (Classification::SourceAddition, NodeKind::File) => ActionKind::AddFile,
                (Classification::SourceOnlyChange, NodeKind::File) => ActionKind::ReplaceFile,
                _ => {
                    return Err(GripError::Internal(
                        "actionable classification has unsupported node state".into(),
                    ));
                }
            };
            let index = actions.len();
            action_indexes.push(index);
            actions.push(PushAction {
                index,
                kind,
                identity: Some(record.identity.clone()),
                dependent_identities: Vec::new(),
                source_path: Some(record.source_path.clone()),
                destination: record.identity.destination_path(),
                destination_path: record.destination_path.clone(),
                expected_source: Some(source),
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
    let counts = PushCounts {
        selected: entries.len(),
        actionable: actions.len(),
        no_action: entries
            .iter()
            .filter(|entry| entry.disposition == Disposition::NoAction)
            .count(),
        blockers: blockers.len(),
        completed: 0,
        failed: 0,
        unattempted: actions.len(),
    };
    let mut plan = PushPlan {
        plan_id: String::new(),
        scope,
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
    if blocking {
        return Disposition::Blocked;
    }
    match classification {
        Classification::SourceAddition | Classification::SourceOnlyChange => Disposition::Action,
        _ => Disposition::NoAction,
    }
}

fn attach_directory_dependencies(actions: &mut [PushAction]) {
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
    scope: &'a ClassificationScope,
    entries: &'a [EntryDisposition],
    actions: &'a [PushAction],
    blockers: &'a [PlanBlocker],
    counts: &'a PushCounts,
}

fn plan_digest(plan: &PushPlan) -> Result<String, GripError> {
    let bytes = serde_json::to_vec(&PlanIdentity {
        scope: &plan.scope,
        entries: &plan.entries,
        actions: &plan.actions,
        blockers: &plan.blockers,
        counts: &plan.counts,
    })
    .map_err(|error| GripError::Internal(format!("could not encode push plan: {error}")))?;
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
}
