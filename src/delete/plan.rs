//! Deterministic authorized-deletion planning.

use crate::classification::model::{Classification, ClassificationRecord, ClassificationScope};
use crate::delete::model::{
    DeletionAction, DeletionActionKind, DeletionAuthority, DeletionCounts, DeletionDisposition,
    DeletionEntryDisposition, DeletionEvidence, DeletionPlan,
};
use crate::discovery::model::NodeKind;
use crate::error::GripError;
use crate::mutation::model::{ActionStatus, PlanBlocker};
use sha2::{Digest, Sha256};

/// Build a complete deletion plan from classified evidence.
pub fn build(
    authority: DeletionAuthority,
    scope: ClassificationScope,
    mut records: Vec<ClassificationRecord>,
) -> Result<DeletionPlan, GripError> {
    records.sort_by(|first, second| first.identity.cmp(&second.identity));
    if records
        .windows(2)
        .any(|pair| pair[0].identity == pair[1].identity)
    {
        return Err(GripError::CorruptState(
            "deletion selection contains a duplicate managed identity".into(),
        ));
    }
    let eligible = match authority {
        DeletionAuthority::Source => Classification::SourceSideDeletion,
        DeletionAuthority::Destination => Classification::DestinationSideDeletion,
    };
    let mut entries = Vec::with_capacity(records.len());
    let mut actions = Vec::new();
    let mut blockers = Vec::new();

    for record in records {
        let disposition = if !record.blocking && record.classification == eligible {
            DeletionDisposition::Action
        } else if record.classification == Classification::ConvergedDeletion {
            DeletionDisposition::NoAction
        } else {
            DeletionDisposition::Blocked
        };
        let mut action_indexes = Vec::new();
        if disposition == DeletionDisposition::Action {
            let expected_target = match authority {
                DeletionAuthority::Source => record.destination.clone(),
                DeletionAuthority::Destination => record.source.clone(),
            }
            .ok_or_else(|| {
                GripError::Internal("actionable deletion has no remaining peer".into())
            })?;
            let expected_target_complete = match authority {
                DeletionAuthority::Source => record.destination_complete.clone(),
                DeletionAuthority::Destination => record.source_complete.clone(),
            };
            let authorized = if let Some(baseline) = record.baseline_complete.as_ref() {
                expected_target_complete.as_ref() == Some(baseline)
            } else {
                record.baseline.as_ref() == Some(&expected_target)
            };
            if !authorized {
                return Err(GripError::Internal(
                    "actionable deletion peer does not equal the accepted baseline".into(),
                ));
            }
            let target = match authority {
                DeletionAuthority::Source => record.identity.destination_path(),
                DeletionAuthority::Destination => record.identity.source_path(),
            };
            let index = actions.len();
            action_indexes.push(index);
            actions.push(DeletionAction {
                index,
                kind: match expected_target.node_kind {
                    NodeKind::File => DeletionActionKind::RemoveFile,
                    NodeKind::Directory => DeletionActionKind::RemoveDirectory,
                    _ => {
                        return Err(GripError::Internal(
                            "unsupported deletion action kind".into(),
                        ));
                    }
                },
                identity: record.identity.clone(),
                authority,
                target_side: match authority {
                    DeletionAuthority::Source => "destination",
                    DeletionAuthority::Destination => "source",
                }
                .into(),
                target_path: crate::discovery::model::SafePath::from_path(&target),
                target,
                expected_target,
                expected_target_complete,
                expected_absent_peer: authority.as_str().into(),
                expected_children: Vec::new(),
                dependencies: Vec::new(),
                status: ActionStatus::Unattempted,
                milestones: DeletionEvidence::default(),
                failure: None,
            });
        } else if disposition == DeletionDisposition::Blocked {
            blockers.push(PlanBlocker {
                reason: record
                    .reasons
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "deletion_not_authorized_for_classification".into()),
                paths: vec![record.source_path.clone(), record.destination_path.clone()],
            });
        }
        entries.push(DeletionEntryDisposition {
            identity: record.identity,
            classification: record.classification,
            source_path: record.source_path,
            destination_path: record.destination_path,
            disposition,
            action_indexes,
            reasons: record.reasons,
        });
    }

    actions.sort_by(|first, second| {
        first
            .identity
            .mapping
            .cmp(&second.identity.mapping)
            .then(depth(&second.identity.relative_path).cmp(&depth(&first.identity.relative_path)))
            .then(
                first
                    .identity
                    .relative_path
                    .cmp(&second.identity.relative_path),
            )
    });
    for (index, action) in actions.iter_mut().enumerate() {
        action.index = index;
    }
    for child_index in 0..actions.len() {
        for parent_index in 0..actions.len() {
            if child_index != parent_index
                && is_descendant(
                    &actions[child_index].identity.relative_path,
                    &actions[parent_index].identity.relative_path,
                )
            {
                let child_path = actions[child_index].target_path.clone();
                actions[parent_index].dependencies.push(child_index);
                actions[parent_index].expected_children.push(child_path);
            }
        }
    }
    for entry in &mut entries {
        entry.action_indexes = actions
            .iter()
            .filter(|action| action.identity == entry.identity)
            .map(|action| action.index)
            .collect();
    }
    let baseline_retirements = actions
        .iter()
        .map(|action| crate::discovery::model::SafePath::from_path(&action.identity.source_path()))
        .collect::<Vec<_>>();
    let counts = DeletionCounts {
        selected: entries.len(),
        actionable: actions.len(),
        no_action: entries
            .iter()
            .filter(|entry| entry.disposition == DeletionDisposition::NoAction)
            .count(),
        blockers: blockers.len(),
        completed: 0,
        failed: 0,
        unattempted: actions.len(),
    };
    let mut plan = DeletionPlan {
        operation: "delete",
        authority,
        plan_id: String::new(),
        scope,
        entries,
        actions,
        baseline_retirements,
        blockers,
        counts,
    };
    plan.plan_id = digest(&plan)?;
    Ok(plan)
}

fn depth(relative: &[u8]) -> usize {
    if relative.is_empty() {
        0
    } else {
        relative.split(|byte| *byte == b'/').count()
    }
}

fn is_descendant(candidate: &[u8], parent: &[u8]) -> bool {
    parent.is_empty() && !candidate.is_empty()
        || candidate
            .strip_prefix(parent)
            .is_some_and(|suffix| suffix.first() == Some(&b'/'))
}

fn digest(plan: &DeletionPlan) -> Result<String, GripError> {
    let bytes = serde_json::to_vec(plan)
        .map_err(|error| GripError::Internal(format!("could not encode deletion plan: {error}")))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classification::model::{ChangedDimensions, Direction};
    use crate::discovery::model::SafePath;
    use crate::mapping::MappingKind;
    use crate::observation::model::{
        ContentFingerprint, EntryIdentity, MappingSnapshot, PathSpace, SupportedState,
    };
    use std::path::PathBuf;

    fn state() -> SupportedState {
        SupportedState {
            node_kind: NodeKind::File,
            content: Some(ContentFingerprint {
                algorithm: "sha256".into(),
                digest: "a".repeat(64),
                length: 1,
            }),
            permission_mode: Some("0644".into()),
        }
    }
    fn record(classification: Classification, relative: &[u8]) -> ClassificationRecord {
        let mapping = MappingSnapshot {
            kind: MappingKind::Tree,
            source: PathBuf::from("/source"),
            destination: PathBuf::from("/destination"),
        };
        let identity = EntryIdentity::new(mapping.clone(), relative.to_vec()).unwrap();
        let baseline = Some(state());
        let (source, destination) = match classification {
            Classification::SourceSideDeletion => (None, baseline.clone()),
            Classification::DestinationSideDeletion => (baseline.clone(), None),
            Classification::ConvergedDeletion => (None, None),
            _ => (baseline.clone(), baseline.clone()),
        };
        ClassificationRecord {
            identity: identity.clone(),
            classification,
            mapping_kind: MappingKind::Tree,
            mapping_source: mapping.source,
            relative_path: identity.relative_safe_path(),
            source_path: SafePath::from_path(&identity.source_path()),
            destination_path: SafePath::from_path(&identity.destination_path()),
            source,
            destination,
            baseline,
            source_complete: None,
            destination_complete: None,
            baseline_complete: None,
            compatibility_findings: Vec::new(),
            endpoint_capabilities: Vec::new(),
            prospective_direction: Direction::None,
            changed_dimensions: ChangedDimensions {
                source_to_baseline: None,
                destination_to_baseline: None,
                source_to_destination: None,
            },
            attention: false,
            blocking: false,
            reasons: Vec::new(),
        }
    }
    fn scope() -> ClassificationScope {
        ClassificationScope {
            kind: "all".into(),
            path_space: PathSpace::Source,
            selector: None,
            mapping_source: None,
        }
    }

    #[test]
    fn deletion_eligibility_matrix_is_authority_bound_and_accumulates_blockers() {
        let variants = [
            Classification::SourceAddition,
            Classification::InitialMatch,
            Classification::InitialCollision,
            Classification::DestinationOnlyUnmanaged,
            Classification::Synchronized,
            Classification::SourceOnlyChange,
            Classification::DestinationOnlyChange,
            Classification::ConvergedTwoSidedChange,
            Classification::DivergentConflict,
            Classification::SourceSideDeletion,
            Classification::DestinationSideDeletion,
            Classification::DeleteChangeConflict,
            Classification::ChangeDeleteConflict,
            Classification::ConvergedDeletion,
            Classification::NewlyIgnoredPendingRetirement,
            Classification::UntrackedPendingRetirement,
            Classification::UnsupportedManaged,
            Classification::UnsafeCollision,
        ];
        for authority in [DeletionAuthority::Source, DeletionAuthority::Destination] {
            let records = variants
                .into_iter()
                .enumerate()
                .map(|(index, classification)| {
                    record(classification, format!("entry-{index}").as_bytes())
                })
                .collect();
            let plan = build(authority, scope(), records).unwrap();
            assert_eq!(plan.actions.len(), 1);
            assert_eq!(plan.counts.no_action, 1);
            assert_eq!(plan.blockers.len(), variants.len() - 2);
        }
    }

    #[test]
    fn deletion_orders_children_before_parents_and_is_input_deterministic() {
        let parent = record(Classification::SourceSideDeletion, b"parent");
        let child = record(Classification::SourceSideDeletion, b"parent/child");
        let forward = build(
            DeletionAuthority::Source,
            scope(),
            vec![parent.clone(), child.clone()],
        )
        .unwrap();
        let reverse = build(DeletionAuthority::Source, scope(), vec![child, parent]).unwrap();
        assert_eq!(forward, reverse);
        assert_eq!(forward.actions[0].identity.relative_path, b"parent/child");
        assert_eq!(forward.actions[1].dependencies, vec![0]);
        assert!(
            build(
                DeletionAuthority::Source,
                scope(),
                vec![
                    record(Classification::SourceSideDeletion, b"same"),
                    record(Classification::SourceSideDeletion, b"same"),
                ],
            )
            .is_err()
        );
    }
}
