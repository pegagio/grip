//! Accepted-state retirement execution.

use crate::error::GripError;
use crate::mutation::model::BaselineOutcome;
use crate::observation::model::Selection;
use crate::retire::model::{RetirementPlan, RetirementResult};

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetirementFault {
    StatePublication,
}

/// Execute an unchanged state-only retirement plan.
pub fn execute(
    home: &crate::home::GripHome,
    expected_registry: &crate::registry::publication::RegistrySnapshot,
    expected_state: &crate::state::publication::StateSnapshot,
    selection: &Selection,
    plan: &RetirementPlan,
) -> Result<RetirementResult, GripError> {
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
    plan: &RetirementPlan,
    fault: Option<RetirementFault>,
) -> Result<RetirementResult, GripError> {
    let _mutation_guard = crate::state::mutation_lock::MutationLock::acquire(home, "retire")?;
    let registry = crate::registry::publication::load(home, false)
        .map_err(|error| error.for_mapping_operation("retire"))?;
    if registry.bytes != expected_registry.bytes || registry.registry != expected_registry.registry
    {
        return Err(GripError::discovery_operational(
            "retire",
            "stale_registry_evidence",
            Vec::new(),
            "registry changed before retirement",
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
    let rebuilt = crate::retire::plan::build(plan.scope.clone(), plan.force, records)?;
    if rebuilt.plan_id != plan.plan_id {
        return Err(GripError::discovery_operational(
            "retire",
            "stale_retirement_evidence",
            Vec::new(),
            "retirement plan changed before execution",
        ));
    }
    let mut receipt = crate::operation::publication::initialize_typed(
        home,
        "retire",
        &plan.plan_id,
        plan,
        plan.actions.len(),
    )?;
    let mut applied = plan.clone();
    let mut next = expected_state.accepted.clone();
    for action in &mut applied.actions {
        action.status = crate::mutation::model::ActionStatus::InProgress;
        action.milestones = checkpoint("not_visible", false);
        if let Err(error) =
            receipt.checkpoint_action(action.index, "in_progress", action.milestones.clone(), None)
        {
            return fail(&mut receipt, &mut applied, expected_state, error);
        }
        next.baselines.remove(&action.identity);
    }
    let state_directory = match crate::state::publication::prepare_directory(home) {
        Ok(value) => value,
        Err(error) => return fail(&mut receipt, &mut applied, expected_state, error),
    };
    let _state_guard =
        match crate::state::lock::PublicationLock::acquire(&state_directory.join("state.lock")) {
            Ok(value) => value,
            Err(error) => return fail(&mut receipt, &mut applied, expected_state, error),
        };
    if fault == Some(RetirementFault::StatePublication) {
        return fail(
            &mut receipt,
            &mut applied,
            expected_state,
            GripError::Internal("injected retirement state publication failure".into()),
        );
    }
    let generation =
        match crate::state::publication::publish_accepted_locked(home, expected_state, &next) {
            Ok(value) => value,
            Err(error) => return fail(&mut receipt, &mut applied, expected_state, error),
        };
    for action in &mut applied.actions {
        action.status = crate::mutation::model::ActionStatus::Completed;
        action.milestones = checkpoint("visible", true);
        if let Err(error) =
            receipt.checkpoint_action(action.index, "completed", action.milestones.clone(), None)
        {
            return fail_after_publication(&receipt, &applied, expected_state, generation, error);
        }
    }
    let baseline = BaselineOutcome {
        outcome: if generation.is_some() {
            "retired"
        } else {
            "unchanged"
        }
        .into(),
        prior_generation: expected_state.accepted.generation,
        published_generation: generation,
        authoritative_generation: generation.or(expected_state.accepted.generation),
        publication_visible: generation.is_some(),
        durability_confirmed: generation.is_some(),
    };
    if let Err(error) = receipt.checkpoint_summary(
        "completed",
        serde_json::to_value(&baseline).unwrap_or(serde_json::Value::Null),
        "prepared",
        None,
    ) {
        return fail_after_publication(&receipt, &applied, expected_state, generation, error);
    }
    Ok(RetirementResult {
        operation: "retire",
        mode: "execute".into(),
        completion: "complete".into(),
        result: if generation.is_some() {
            "applied"
        } else {
            "no_op"
        }
        .into(),
        plan: applied,
        operation_record: Some(receipt.operation_id().into()),
        baseline,
    })
}

fn fail_after_publication(
    receipt: &crate::operation::publication::OperationReceipt,
    plan: &RetirementPlan,
    expected_state: &crate::state::publication::StateSnapshot,
    generation: Option<u64>,
    error: GripError,
) -> Result<RetirementResult, GripError> {
    let baseline = BaselineOutcome {
        outcome: "retired".into(),
        prior_generation: expected_state.accepted.generation,
        published_generation: generation,
        authoritative_generation: generation.or(expected_state.accepted.generation),
        publication_visible: generation.is_some(),
        durability_confirmed: generation.is_some(),
    };
    let mut details = serde_json::to_value(plan)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    details.insert("mode".into(), "execute".into());
    details.insert("completion".into(), "complete".into());
    details.insert("result".into(), "failed".into());
    details.insert(
        "baseline".into(),
        serde_json::to_value(&baseline).unwrap_or(serde_json::Value::Null),
    );
    details.insert(
        "failure".into(),
        serde_json::json!({"phase":"operation_checkpoint","message":error.to_string()}),
    );
    Err(GripError::operation_lifecycle(
        "retire",
        "operation_checkpoint_failure",
        crate::error::ResultCategory::InternalError,
        receipt.operation_id(),
        details,
        baseline.publication_visible,
        "verified",
        baseline.durability_confirmed,
        format!("retirement completed but its terminal operation checkpoint failed: {error}"),
    ))
}

fn checkpoint(
    publication: &str,
    durable: bool,
) -> crate::operation::model::ActionCheckpointEvidenceV1 {
    crate::operation::model::ActionCheckpointEvidenceV1 {
        revalidation: "passed".into(),
        recovery: "not_required".into(),
        recovery_ref: None,
        staging: "not_attempted".into(),
        publication: publication.into(),
        verification: if durable { "verified" } else { "not_attempted" }.into(),
        durability_confirmed: durable,
    }
}

fn fail(
    receipt: &mut crate::operation::publication::OperationReceipt,
    plan: &mut RetirementPlan,
    expected_state: &crate::state::publication::StateSnapshot,
    error: GripError,
) -> Result<RetirementResult, GripError> {
    for action in &mut plan.actions {
        action.status = crate::mutation::model::ActionStatus::Failed;
        action.failure = Some(error.to_string());
        let _ = receipt.checkpoint_action(
            action.index,
            "failed",
            action.milestones.clone(),
            Some(error.to_string()),
        );
    }
    let baseline = BaselineOutcome {
        outcome: "not_published".into(),
        prior_generation: expected_state.accepted.generation,
        published_generation: None,
        authoritative_generation: expected_state.accepted.generation,
        publication_visible: false,
        durability_confirmed: false,
    };
    let _ = receipt.checkpoint_summary(
        "failed",
        serde_json::to_value(&baseline).unwrap_or(serde_json::Value::Null),
        "prepared",
        Some(error.to_string()),
    );
    let mut details = serde_json::to_value(&*plan)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    details.insert("mode".into(), "execute".into());
    details.insert("completion".into(), "failed".into());
    details.insert("result".into(), "failed".into());
    details.insert(
        "baseline".into(),
        serde_json::to_value(&baseline).unwrap_or(serde_json::Value::Null),
    );
    Err(GripError::operation_lifecycle(
        "retire",
        "state_publication_failure",
        crate::error::ResultCategory::InternalError,
        receipt.operation_id(),
        details,
        false,
        "not_attempted",
        false,
        format!("retirement state publication failed: {error}"),
    ))
}
