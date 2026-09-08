//! Deterministic accepted-state retirement planning.

use crate::classification::model::{Classification, ClassificationRecord, ClassificationScope};
use crate::error::GripError;
use crate::mutation::model::PlanBlocker;
use crate::retire::model::{
    RetirementAction, RetirementDisposition, RetirementEntryDisposition, RetirementPlan,
};
use sha2::{Digest, Sha256};

/// Build a complete state-only retirement plan.
pub fn build(
    scope: ClassificationScope,
    force: bool,
    mut records: Vec<ClassificationRecord>,
) -> Result<RetirementPlan, GripError> {
    records.sort_by(|first, second| first.identity.cmp(&second.identity));
    if records
        .windows(2)
        .any(|pair| pair[0].identity == pair[1].identity)
    {
        return Err(GripError::CorruptState(
            "retirement selection contains a duplicate managed identity".into(),
        ));
    }
    let mut entries = Vec::with_capacity(records.len());
    let mut actions = Vec::new();
    let mut blockers = Vec::new();
    for record in records {
        let pending = matches!(
            record.classification,
            Classification::NewlyIgnoredPendingRetirement
                | Classification::UntrackedPendingRetirement
        );
        let converged = record.classification == Classification::ConvergedDeletion;
        let differences = if pending {
            surviving_differences(&record)
        } else {
            Vec::new()
        };
        let disposition = if record.blocking {
            RetirementDisposition::Blocked
        } else if pending && !differences.is_empty() && !force {
            RetirementDisposition::ForceRequired
        } else if pending || converged {
            RetirementDisposition::Retire
        } else {
            RetirementDisposition::Blocked
        };
        let path = record.source_path.clone();
        if disposition == RetirementDisposition::Retire {
            actions.push(RetirementAction {
                index: actions.len(),
                identity: record.identity.clone(),
                path: path.clone(),
                reason: retirement_reason(record.classification).into(),
                force: force && !differences.is_empty(),
                status: crate::mutation::model::ActionStatus::Unattempted,
                milestones: crate::operation::model::ActionCheckpointEvidenceV1 {
                    revalidation: "planned".into(),
                    recovery: "not_required".into(),
                    recovery_ref: None,
                    staging: "not_required".into(),
                    publication: "not_attempted".into(),
                    verification: "not_attempted".into(),
                    durability_confirmed: false,
                },
                failure: None,
            });
        } else if matches!(
            disposition,
            RetirementDisposition::Blocked | RetirementDisposition::ForceRequired
        ) {
            blockers.push(PlanBlocker {
                reason: if disposition == RetirementDisposition::ForceRequired {
                    "force_required".into()
                } else {
                    record
                        .reasons
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "retirement_not_eligible".into())
                },
                paths: vec![path.clone()],
            });
        }
        entries.push(RetirementEntryDisposition {
            identity: record.identity,
            classification: record.classification,
            path,
            disposition,
            differences,
            reasons: record.reasons,
        });
    }
    let retired_identities = actions.iter().map(|action| action.path.clone()).collect();
    let mut plan = RetirementPlan {
        operation: "retire",
        plan_id: String::new(),
        scope,
        force,
        entries,
        actions,
        retired_identities,
        blockers,
    };
    plan.plan_id = digest(&plan)?;
    Ok(plan)
}

fn surviving_differences(record: &ClassificationRecord) -> Vec<String> {
    let mut differences = Vec::new();
    let source_to_baseline = record
        .source_complete
        .as_ref()
        .zip(record.baseline_complete.as_ref())
        .map_or_else(
            || {
                record
                    .source
                    .as_ref()
                    .zip(record.baseline.as_ref())
                    .is_some_and(|(current, baseline)| current != baseline)
            },
            |(current, baseline)| current != baseline,
        );
    if source_to_baseline {
        differences.push("source_to_baseline".into());
    }
    let destination_to_baseline = record
        .destination_complete
        .as_ref()
        .zip(record.baseline_complete.as_ref())
        .map_or_else(
            || {
                record
                    .destination
                    .as_ref()
                    .zip(record.baseline.as_ref())
                    .is_some_and(|(current, baseline)| current != baseline)
            },
            |(current, baseline)| current != baseline,
        );
    if destination_to_baseline {
        differences.push("destination_to_baseline".into());
    }
    let source_to_destination = record
        .source_complete
        .as_ref()
        .zip(record.destination_complete.as_ref())
        .map_or_else(
            || {
                record
                    .source
                    .as_ref()
                    .zip(record.destination.as_ref())
                    .is_some_and(|(source, destination)| source != destination)
            },
            |(source, destination)| source != destination,
        );
    if source_to_destination {
        differences.push("source_to_destination".into());
    }
    differences
}

fn retirement_reason(classification: Classification) -> &'static str {
    match classification {
        Classification::NewlyIgnoredPendingRetirement => "newly_ignored",
        Classification::UntrackedPendingRetirement => "mapping_removed",
        Classification::ConvergedDeletion => "converged_deletion",
        _ => "ineligible",
    }
}

fn digest(plan: &RetirementPlan) -> Result<String, GripError> {
    let bytes = serde_json::to_vec(plan).map_err(|error| {
        GripError::Internal(format!("could not encode retirement plan: {error}"))
    })?;
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

    fn state(digest: char) -> SupportedState {
        SupportedState {
            node_kind: crate::discovery::model::NodeKind::File,
            content: Some(ContentFingerprint {
                algorithm: "sha256".into(),
                digest: digest.to_string().repeat(64),
                length: 1,
            }),
            permission_mode: Some("0644".into()),
        }
    }
    fn record(classification: Classification, changed: bool) -> ClassificationRecord {
        let mapping = MappingSnapshot {
            kind: MappingKind::File,
            source: PathBuf::from("/source"),
            destination: PathBuf::from("/destination"),
        };
        let identity = EntryIdentity::new(mapping.clone(), Vec::new()).unwrap();
        ClassificationRecord {
            identity,
            classification,
            mapping_kind: MappingKind::File,
            mapping_source: mapping.source,
            relative_path: None,
            source_path: SafePath::from_path(std::path::Path::new("/source")),
            destination_path: SafePath::from_path(std::path::Path::new("/destination")),
            source: Some(state(if changed { 'b' } else { 'a' })),
            destination: Some(state('a')),
            baseline: Some(state('a')),
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
    fn retirement_classification_matrix_is_exhaustive_and_force_is_explicit() {
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
        for classification in variants {
            let plan = build(scope(), false, vec![record(classification, false)]).unwrap();
            let eligible = matches!(
                classification,
                Classification::ConvergedDeletion
                    | Classification::NewlyIgnoredPendingRetirement
                    | Classification::UntrackedPendingRetirement
            );
            assert_eq!(!plan.actions.is_empty(), eligible, "{classification:?}");
        }
        let blocked = build(
            scope(),
            false,
            vec![record(Classification::NewlyIgnoredPendingRetirement, true)],
        )
        .unwrap();
        assert_eq!(blocked.blockers[0].reason, "force_required");
        let forced = build(
            scope(),
            true,
            vec![record(Classification::NewlyIgnoredPendingRetirement, true)],
        )
        .unwrap();
        assert!(forced.actions[0].force);
    }

    #[test]
    fn retirement_plan_is_input_order_independent() {
        let first = record(Classification::ConvergedDeletion, false);
        let mut second = first.clone();
        second.identity.relative_path = b"child".to_vec();
        let forward = build(scope(), false, vec![first.clone(), second.clone()]).unwrap();
        let reverse = build(scope(), false, vec![second, first]).unwrap();
        assert_eq!(forward, reverse);
    }
}
