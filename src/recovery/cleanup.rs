//! Explicit exact-reference recovery cleanup.

use crate::error::GripError;
use crate::project::ProjectPaths;
use crate::recovery::model::{
    CleanupAction, CleanupPlan, CleanupTombstonePayloadV1, RecoveryAvailability,
    RecoveryEnvelopeV1, RecoveryRef,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupFault {
    ByteRemoval,
    TombstonePublication,
    ByteRemovalAt(usize),
    TombstonePublicationAt(usize),
}

pub fn plan(home: &ProjectPaths, references: &[RecoveryRef]) -> Result<CleanupPlan, GripError> {
    let mut unique = BTreeSet::new();
    let mut actions = Vec::new();
    let mut blockers = Vec::new();
    for reference in references {
        if !unique.insert(reference.clone()) {
            return Err(GripError::InvalidConfiguration(
                "recovery cleanup references must be unique".into(),
            ));
        }
        if !reference.has_recoverable_bytes() {
            return Err(GripError::InvalidConfiguration(
                "operation recovery references cannot be removed".into(),
            ));
        }
    }
    for reference in unique {
        let entry = crate::recovery::inventory::show(home, &reference)?;
        if let RecoveryRef::Payload { operation_id, .. } = &reference {
            let summary_path = home
                .path()
                .join("state/operations")
                .join(operation_id)
                .join("operation.json");
            let bytes = crate::mutation::filesystem::read_private_file(&summary_path)?;
            let summary = crate::operation::model::decode::<
                crate::operation::model::OperationSummaryPayloadV2,
            >(&bytes)?;
            if summary.payload.state == "executing" {
                blockers.push(format!("{reference}: active_operation"));
            }
        }
        match entry.availability {
            RecoveryAvailability::Available | RecoveryAvailability::CleanupIncomplete => {
                actions.push(CleanupAction {
                    index: actions.len(),
                    reference,
                    byte_count: entry.byte_count.unwrap_or(0),
                    status: "unattempted".into(),
                });
            }
            RecoveryAvailability::Cleaned => blockers.push(format!("{reference}: already_cleaned")),
            RecoveryAvailability::Missing => blockers.push(format!("{reference}: missing")),
        }
    }
    let projection = serde_json::to_vec(&(
        actions
            .iter()
            .map(|a| (&a.reference, a.byte_count))
            .collect::<Vec<_>>(),
        &blockers,
    ))
    .map_err(|error| GripError::Internal(format!("could not encode cleanup plan: {error}")))?;
    Ok(CleanupPlan {
        operation: "recovery_remove",
        plan_id: format!("{:x}", Sha256::digest(projection)),
        actions,
        blockers,
        operation_record: None,
    })
}

pub fn execute(home: &ProjectPaths, expected: &CleanupPlan) -> Result<CleanupPlan, GripError> {
    execute_with_fault(home, expected, None)
}

#[doc(hidden)]
pub fn execute_with_fault(
    home: &ProjectPaths,
    expected: &CleanupPlan,
    fault: Option<CleanupFault>,
) -> Result<CleanupPlan, GripError> {
    let _guard = crate::state::mutation_lock::MutationLock::acquire(home, "recovery_remove")?;
    let refs = expected
        .actions
        .iter()
        .map(|action| action.reference.clone())
        .collect::<Vec<_>>();
    let rebuilt = plan(home, &refs)?;
    if rebuilt.plan_id != expected.plan_id || !rebuilt.blockers.is_empty() {
        return Err(GripError::InvalidConfiguration(
            "recovery cleanup plan changed before execution".into(),
        ));
    }
    let mut receipt = crate::operation::publication::initialize_typed(
        home,
        "recovery_remove",
        &expected.plan_id,
        expected,
        expected.actions.len(),
    )?;
    let mut result = expected.clone();
    for index in 0..result.actions.len() {
        let action = &result.actions[index];
        if let Err(error) = receipt.checkpoint_action(
            action.index,
            "in_progress",
            checkpoint("not_attempted", false),
            None,
        ) {
            return cleanup_fail(
                &mut receipt,
                &mut result,
                index,
                "operation_checkpoint",
                error,
            );
        }
        let located = match crate::recovery::inventory::locate_manifest(home, &action.reference) {
            Ok(value) => value,
            Err(error) => {
                return cleanup_fail(&mut receipt, &mut result, index, "revalidation", error);
            }
        };
        if located.payload.byte_count != action.byte_count {
            return cleanup_fail(
                &mut receipt,
                &mut result,
                index,
                "revalidation",
                GripError::CorruptState("cleanup byte count changed".into()),
            );
        }
        if located.payload_path.exists() {
            if fault == Some(CleanupFault::ByteRemoval)
                || fault == Some(CleanupFault::ByteRemovalAt(index))
            {
                return cleanup_fail(
                    &mut receipt,
                    &mut result,
                    index,
                    "byte_removal",
                    GripError::Internal("injected recovery byte removal failure".into()),
                );
            }
            if let Err(error) = crate::mutation::filesystem::remove_private_file(
                &located.payload_path,
                action.byte_count,
            ) {
                return cleanup_fail(&mut receipt, &mut result, index, "byte_removal", error);
            }
        }
        let tombstone = match RecoveryEnvelopeV1::new(CleanupTombstonePayloadV1 {
            reference: action.reference.clone(),
            cleaned_at: timestamp(),
            removed_byte_count: action.byte_count,
            cleanup_operation_id: receipt.operation_id().into(),
        }) {
            Ok(value) => value,
            Err(error) => {
                return cleanup_fail(
                    &mut receipt,
                    &mut result,
                    index,
                    "tombstone_publication",
                    error,
                );
            }
        };
        if fault == Some(CleanupFault::TombstonePublication)
            || fault == Some(CleanupFault::TombstonePublicationAt(index))
        {
            return cleanup_fail(
                &mut receipt,
                &mut result,
                index,
                "tombstone_publication",
                GripError::Internal("injected cleanup tombstone publication failure".into()),
            );
        }
        if let Err(error) = crate::operation::publication::publish_new_component(
            &located.directory,
            "cleaned.json",
            &match crate::recovery::model::encode(&tombstone) {
                Ok(value) => value,
                Err(error) => {
                    return cleanup_fail(
                        &mut receipt,
                        &mut result,
                        index,
                        "tombstone_publication",
                        error,
                    );
                }
            },
        ) {
            return cleanup_fail(
                &mut receipt,
                &mut result,
                index,
                "tombstone_publication",
                error,
            );
        }
        let action = &mut result.actions[index];
        action.status = "completed".into();
        if let Err(error) =
            receipt.checkpoint_action(action.index, "completed", checkpoint("visible", true), None)
        {
            return cleanup_fail(
                &mut receipt,
                &mut result,
                index,
                "operation_checkpoint",
                error,
            );
        }
    }
    if let Err(error) = receipt.checkpoint_summary(
        "completed",
        serde_json::json!({"outcome":"unchanged"}),
        "prepared",
        None,
    ) {
        let last = result.actions.len().saturating_sub(1);
        return cleanup_fail(
            &mut receipt,
            &mut result,
            last,
            "operation_checkpoint",
            error,
        );
    }
    result.operation_record = Some(receipt.operation_id().into());
    Ok(result)
}

fn cleanup_fail(
    receipt: &mut crate::operation::publication::OperationReceipt,
    plan: &mut CleanupPlan,
    index: usize,
    phase: &str,
    error: GripError,
) -> Result<CleanupPlan, GripError> {
    let action = &mut plan.actions[index];
    let action_completed = action.status == "completed";
    if !action_completed {
        action.status = "failed".into();
    }
    let message = format!("recovery cleanup failed during {phase}: {error}");
    let _ = receipt.checkpoint_action(
        action.index,
        "failed",
        checkpoint("not_attempted", false),
        Some(message.clone()),
    );
    let _ = receipt.checkpoint_summary(
        "failed",
        serde_json::json!({"outcome":"unchanged"}),
        "prepared",
        Some(message.clone()),
    );
    let publication_visible = action_completed || phase == "tombstone_publication";
    let mut details = serde_json::Map::new();
    details.insert("mode".into(), "execute".into());
    details.insert("completion".into(), "partial".into());
    details.insert("result".into(), "failed".into());
    details.insert(
        "plan".into(),
        serde_json::to_value(&*plan).unwrap_or(serde_json::Value::Null),
    );
    details.insert(
        "failure".into(),
        serde_json::json!({"phase":phase,"message":error.to_string()}),
    );
    Err(GripError::operation_lifecycle(
        "recovery_remove",
        &format!("{phase}_failure"),
        crate::error::ResultCategory::InternalError,
        receipt.operation_id(),
        details,
        publication_visible,
        if publication_visible {
            "failed"
        } else {
            "not_attempted"
        },
        false,
        message,
    ))
}

fn checkpoint(
    publication: &str,
    durable: bool,
) -> crate::operation::model::ActionCheckpointEvidenceV2 {
    crate::operation::model::ActionCheckpointEvidenceV2 {
        revalidation: "passed".into(),
        recovery: "not_required".into(),
        recovery_ref: None,
        staging: "not_attempted".into(),
        publication: publication.into(),
        verification: if durable { "verified" } else { "not_attempted" }.into(),
        durability_confirmed: durable,
    }
}

fn timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| format!("{}Z", value.as_secs()))
        .unwrap_or_else(|_| "0Z".into())
}
