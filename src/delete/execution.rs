//! Authorized-deletion execution.

use crate::delete::model::{DeletionPlan, DeletionResult};
use crate::error::GripError;
use crate::mutation::model::{ActionStatus, BaselineOutcome};
use crate::observation::model::Selection;

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeletionFault {
    Preservation,
    Unlink,
    DirectorySync,
    AbsenceVerification,
    FinalObservation,
    StatePublication,
}

/// Execute an unchanged deletion plan under Grip's writer coordination boundary.
pub fn execute(
    home: &crate::home::GripHome,
    expected_registry: &crate::registry::publication::RegistrySnapshot,
    expected_state: &crate::state::publication::StateSnapshot,
    selection: &Selection,
    plan: &DeletionPlan,
) -> Result<DeletionResult, GripError> {
    execute_with_fault(
        home,
        expected_registry,
        expected_state,
        selection,
        plan,
        None,
    )
}

#[doc(hidden)]
pub fn execute_with_fault(
    home: &crate::home::GripHome,
    expected_registry: &crate::registry::publication::RegistrySnapshot,
    expected_state: &crate::state::publication::StateSnapshot,
    selection: &Selection,
    plan: &DeletionPlan,
    fault: Option<DeletionFault>,
) -> Result<DeletionResult, GripError> {
    execute_with_hook(
        home,
        expected_registry,
        expected_state,
        selection,
        plan,
        fault,
        None,
    )
}

/// Execute with a deterministic test hook immediately before each action revalidation.
#[doc(hidden)]
pub fn execute_with_hook(
    home: &crate::home::GripHome,
    expected_registry: &crate::registry::publication::RegistrySnapshot,
    expected_state: &crate::state::publication::StateSnapshot,
    selection: &Selection,
    plan: &DeletionPlan,
    fault: Option<DeletionFault>,
    mut before_action: Option<&mut dyn FnMut(usize)>,
) -> Result<DeletionResult, GripError> {
    let _mutation_guard = crate::state::mutation_lock::MutationLock::acquire(home, "delete")?;
    let registry = crate::registry::publication::load(home, false)
        .map_err(|error| error.for_mapping_operation("delete"))?;
    if registry.bytes != expected_registry.bytes || registry.registry != expected_registry.registry
    {
        return Err(crate::error::GripError::discovery_operational(
            "delete",
            "stale_registry_evidence",
            Vec::new(),
            "registry changed before deletion",
        ));
    }
    crate::state::publication::revalidate(home, expected_state)?;
    let observed =
        crate::observation::inspect(home, &registry, &expected_state.accepted, selection)?;
    let records = observed
        .values()
        .map(|entry| {
            crate::classification::classify(
                entry,
                expected_state.accepted.baselines.get(&entry.identity),
            )
        })
        .collect();
    let rebuilt = crate::delete::plan::build(plan.authority, plan.scope.clone(), records)?;
    if rebuilt.plan_id != plan.plan_id {
        return Err(crate::error::GripError::discovery_operational(
            "delete",
            "stale_deletion_evidence",
            Vec::new(),
            "deletion plan changed before execution",
        ));
    }
    let mut receipt = crate::operation::publication::initialize_typed(
        home,
        "delete",
        &plan.plan_id,
        plan,
        plan.actions.len(),
    )?;
    let mut applied = plan.clone();
    for index in 0..applied.actions.len() {
        if let Some(hook) = before_action.as_mut() {
            hook(index);
        }
        if let Err(error) =
            revalidate_action(home, expected_registry, expected_state, &applied, index)
        {
            return fail(
                &mut receipt,
                &mut applied,
                index,
                "revalidation",
                error,
                expected_state.accepted.generation,
            );
        }
        let action = &mut applied.actions[index];
        action.status = ActionStatus::InProgress;
        action.milestones.revalidation = "passed".into();
        if let Err(error) =
            receipt.checkpoint_action(index, "in_progress", checkpoint(action), None)
        {
            return fail(
                &mut receipt,
                &mut applied,
                index,
                "operation_checkpoint",
                error,
                expected_state.accepted.generation,
            );
        }
        let recovery = if fault == Some(DeletionFault::Preservation) {
            Err(GripError::Internal(
                "injected deletion preservation failure".into(),
            ))
        } else {
            crate::mutation::recovery::preserve_with_post(
                &receipt,
                index,
                &action.identity,
                &action.target,
                &action.expected_target,
                None,
            )
        };
        let recovery = match recovery {
            Ok(recovery) => recovery,
            Err(error) => {
                return fail(
                    &mut receipt,
                    &mut applied,
                    index,
                    "preservation",
                    error,
                    expected_state.accepted.generation,
                );
            }
        };
        action.milestones.recovery = "preserved".into();
        action.milestones.recovery_ref = Some(recovery.relative_ref);
        if let Err(error) =
            receipt.checkpoint_action(index, "in_progress", checkpoint(action), None)
        {
            return fail(
                &mut receipt,
                &mut applied,
                index,
                "operation_checkpoint",
                error,
                expected_state.accepted.generation,
            );
        }
        if fault == Some(DeletionFault::Unlink) {
            return fail(
                &mut receipt,
                &mut applied,
                index,
                "unlink",
                GripError::Internal("injected deletion unlink failure".into()),
                expected_state.accepted.generation,
            );
        }
        action.milestones.durability_confirmed = match crate::mutation::filesystem::remove_verified(
            &action.target,
            &action.expected_target,
        ) {
            Ok(value) => value,
            Err(error) => {
                return fail(
                    &mut receipt,
                    &mut applied,
                    index,
                    "unlink",
                    error,
                    expected_state.accepted.generation,
                );
            }
        };
        if matches!(
            fault,
            Some(DeletionFault::DirectorySync | DeletionFault::AbsenceVerification)
        ) {
            let phase = if fault == Some(DeletionFault::DirectorySync) {
                "directory_sync"
            } else {
                "absence_verification"
            };
            return fail(
                &mut receipt,
                &mut applied,
                index,
                phase,
                GripError::Internal(format!("injected deletion {phase} failure")),
                expected_state.accepted.generation,
            );
        }
        action.milestones.removal = "not_visible".into();
        action.milestones.verification = "verified".into();
        action.status = ActionStatus::Completed;
        applied.counts.completed += 1;
        applied.counts.unattempted -= 1;
        if let Err(error) = receipt.checkpoint_action(index, "completed", checkpoint(action), None)
        {
            return fail(
                &mut receipt,
                &mut applied,
                index,
                "operation_checkpoint",
                error,
                expected_state.accepted.generation,
            );
        }
    }
    if fault == Some(DeletionFault::FinalObservation) {
        let last = applied.actions.len().saturating_sub(1);
        return fail(
            &mut receipt,
            &mut applied,
            last,
            "final_observation",
            GripError::Internal("injected final observation failure".into()),
            expected_state.accepted.generation,
        );
    }
    let last = applied.actions.len().saturating_sub(1);
    let final_registry = match crate::registry::publication::load(home, false)
        .map_err(|error| error.for_mapping_operation("delete"))
    {
        Ok(value) => value,
        Err(error) => {
            return fail(
                &mut receipt,
                &mut applied,
                last,
                "final_observation",
                error,
                expected_state.accepted.generation,
            );
        }
    };
    let final_observed = match crate::observation::inspect(
        home,
        &final_registry,
        &expected_state.accepted,
        selection,
    ) {
        Ok(value) => value,
        Err(error) => {
            return fail(
                &mut receipt,
                &mut applied,
                last,
                "final_observation",
                error,
                expected_state.accepted.generation,
            );
        }
    };
    for action in &applied.actions {
        if final_observed
            .get(&action.identity)
            .is_none_or(|entry| entry.source.is_some() || entry.destination.is_some())
        {
            return fail(
                &mut receipt,
                &mut applied,
                last,
                "final_observation",
                GripError::Internal(
                    "final deletion observation did not prove both sides absent".into(),
                ),
                expected_state.accepted.generation,
            );
        }
    }
    let mut next = expected_state.accepted.clone();
    for action in &applied.actions {
        next.baselines.remove(&action.identity);
    }
    let state_directory = match crate::state::publication::prepare_directory(home) {
        Ok(value) => value,
        Err(error) => {
            return fail(
                &mut receipt,
                &mut applied,
                last,
                "state_publication",
                error,
                expected_state.accepted.generation,
            );
        }
    };
    let _state_guard =
        match crate::state::lock::PublicationLock::acquire(&state_directory.join("state.lock")) {
            Ok(value) => value,
            Err(error) => {
                return fail(
                    &mut receipt,
                    &mut applied,
                    last,
                    "state_publication",
                    error,
                    expected_state.accepted.generation,
                );
            }
        };
    if fault == Some(DeletionFault::StatePublication) {
        let last = applied.actions.len().saturating_sub(1);
        return fail(
            &mut receipt,
            &mut applied,
            last,
            "state_publication",
            GripError::Internal("injected state publication failure".into()),
            expected_state.accepted.generation,
        );
    }
    let generation =
        match crate::state::publication::publish_accepted_locked(home, expected_state, &next) {
            Ok(value) => value,
            Err(error) => {
                return fail(
                    &mut receipt,
                    &mut applied,
                    last,
                    "state_publication",
                    error,
                    expected_state.accepted.generation,
                );
            }
        };
    let authoritative = generation.or(expected_state.accepted.generation);
    let baseline = BaselineOutcome {
        outcome: "retired".into(),
        prior_generation: expected_state.accepted.generation,
        published_generation: generation,
        authoritative_generation: authoritative,
        publication_visible: generation.is_some(),
        durability_confirmed: generation.is_some(),
    };
    if let Err(error) = receipt.checkpoint_summary(
        "completed",
        serde_json::to_value(&baseline).unwrap_or(serde_json::Value::Null),
        "prepared",
        None,
    ) {
        return fail_after_publication(&receipt, &applied, &baseline, error);
    }
    Ok(DeletionResult {
        operation: "delete",
        authority: applied.authority,
        mode: "execute".into(),
        completion: "complete".into(),
        result: "applied".into(),
        plan: applied,
        operation_record: Some(receipt.operation_id().into()),
        baseline,
    })
}

fn fail_after_publication(
    receipt: &crate::operation::publication::OperationReceipt,
    plan: &DeletionPlan,
    baseline: &BaselineOutcome,
    error: GripError,
) -> Result<DeletionResult, GripError> {
    let mut details = serde_json::to_value(plan)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    details.insert("mode".into(), "execute".into());
    details.insert("completion".into(), "complete".into());
    details.insert("result".into(), "failed".into());
    details.insert(
        "baseline".into(),
        serde_json::to_value(baseline).unwrap_or(serde_json::Value::Null),
    );
    details.insert(
        "failure".into(),
        serde_json::json!({"phase":"operation_checkpoint","message":error.to_string()}),
    );
    Err(GripError::operation_lifecycle(
        "delete",
        "operation_checkpoint_failure",
        crate::error::ResultCategory::InternalError,
        receipt.operation_id(),
        details,
        baseline.publication_visible,
        "verified",
        baseline.durability_confirmed,
        format!("deletion completed but its terminal operation checkpoint failed: {error}"),
    ))
}

fn revalidate_action(
    home: &crate::home::GripHome,
    expected_registry: &crate::registry::publication::RegistrySnapshot,
    expected_state: &crate::state::publication::StateSnapshot,
    applied: &DeletionPlan,
    index: usize,
) -> Result<(), GripError> {
    let action = applied
        .actions
        .get(index)
        .ok_or_else(|| GripError::Internal("deletion action index is invalid".into()))?;
    if action
        .dependencies
        .iter()
        .any(|dependency| applied.actions[*dependency].status != ActionStatus::Completed)
    {
        return Err(GripError::Internal(
            "deletion action dependency is not complete".into(),
        ));
    }
    let registry = crate::registry::publication::load(home, false)
        .map_err(|error| error.for_mapping_operation("delete"))?;
    if registry.bytes != expected_registry.bytes || registry.registry != expected_registry.registry
    {
        return Err(GripError::discovery_operational(
            "delete",
            "stale_registry_evidence",
            Vec::new(),
            "registry changed before deletion action",
        ));
    }
    crate::state::publication::revalidate(home, expected_state)?;
    if expected_state.accepted.baselines.get(&action.identity) != Some(&action.expected_target) {
        return Err(GripError::CorruptState(
            "accepted deletion membership or baseline changed".into(),
        ));
    }
    let authoritative_path = match action.authority {
        crate::delete::model::DeletionAuthority::Source => action.identity.source_path(),
        crate::delete::model::DeletionAuthority::Destination => action.identity.destination_path(),
    };
    match std::fs::symlink_metadata(&authoritative_path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Ok(_) => {
            return Err(GripError::InvalidConfiguration(
                "authoritative deletion side is no longer absent".into(),
            ));
        }
        Err(error) => {
            return Err(GripError::from_io(
                "could not revalidate authoritative deletion absence",
                error,
            ));
        }
    }
    crate::mutation::filesystem::verify_removal_ready(&action.target, &action.expected_target)
}

fn fail(
    receipt: &mut crate::operation::publication::OperationReceipt,
    plan: &mut DeletionPlan,
    index: usize,
    phase: &str,
    error: GripError,
    prior_generation: Option<u64>,
) -> Result<DeletionResult, GripError> {
    if let Some(action) = plan.actions.get_mut(index)
        && action.status != ActionStatus::Completed
    {
        action.status = ActionStatus::Failed;
        action.failure = Some(error.to_string());
        plan.counts.failed += 1;
        plan.counts.unattempted = plan.counts.unattempted.saturating_sub(1);
        let _ =
            receipt.checkpoint_action(index, "failed", checkpoint(action), Some(error.to_string()));
    }
    let baseline = BaselineOutcome {
        outcome: "not_published".into(),
        prior_generation,
        published_generation: None,
        authoritative_generation: prior_generation,
        publication_visible: false,
        durability_confirmed: false,
    };
    let _ = receipt.checkpoint_summary(
        "failed",
        serde_json::to_value(&baseline).unwrap_or(serde_json::Value::Null),
        "prepared",
        Some(format!("{phase}: {error}")),
    );
    let mut details = serde_json::to_value(&*plan)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    details.insert("mode".into(), "execute".into());
    details.insert("completion".into(), "partial".into());
    details.insert("result".into(), "failed".into());
    details.insert(
        "baseline".into(),
        serde_json::to_value(&baseline).unwrap_or(serde_json::Value::Null),
    );
    details.insert(
        "failure".into(),
        serde_json::json!({"phase":phase,"message":error.to_string()}),
    );
    Err(GripError::operation_lifecycle(
        "delete",
        &format!("{phase}_failure"),
        crate::error::ResultCategory::InternalError,
        receipt.operation_id(),
        details,
        plan.actions
            .iter()
            .any(|action| action.milestones.removal == "not_visible"),
        if plan.actions.iter().all(|action| {
            action.status == ActionStatus::Unattempted
                || action.milestones.verification == "verified"
        }) {
            "verified"
        } else {
            "failed"
        },
        plan.actions
            .iter()
            .filter(|action| action.status == ActionStatus::Completed)
            .all(|action| action.milestones.durability_confirmed),
        format!("deletion failed during {phase}: {error}"),
    ))
}

fn checkpoint(
    action: &crate::delete::model::DeletionAction,
) -> crate::operation::model::ActionCheckpointEvidenceV1 {
    crate::operation::model::ActionCheckpointEvidenceV1 {
        revalidation: action.milestones.revalidation.clone(),
        recovery: action.milestones.recovery.clone(),
        recovery_ref: action.milestones.recovery_ref.clone(),
        staging: "not_attempted".into(),
        publication: action.milestones.removal.clone(),
        verification: action.milestones.verification.clone(),
        durability_confirmed: action.milestones.durability_confirmed,
    }
}
