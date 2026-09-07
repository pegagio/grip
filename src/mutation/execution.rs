//! Lock-held direction-neutral execution and accepted-state publication.

use crate::classification;
use crate::error::GripError;
use crate::home::GripHome;
use crate::mutation::model::{
    ActionKind, Disposition, MutationDirection, MutationOperation, MutationPlan,
};
use crate::observation::model::Selection;
use crate::operation::model::ActionCheckpointEvidenceV1;
use crate::registry::publication::RegistrySnapshot;
use crate::state::publication::StateSnapshot;
use std::collections::BTreeSet;

/// Successful execution evidence returned to command orchestration.
#[derive(Debug)]
pub struct ExecutionSuccess {
    pub plan: MutationPlan,
    pub operation_id: String,
    pub generation: u64,
    pub prior_generation: Option<u64>,
}

/// Execute one actionful, unblocked plan under the global writer lock.
pub fn execute(
    home: &GripHome,
    expected_registry: &RegistrySnapshot,
    expected_state: &StateSnapshot,
    selection: &Selection,
    initial_plan: &MutationPlan,
) -> Result<ExecutionSuccess, GripError> {
    execute_with_fault_hook(
        home,
        expected_registry,
        expected_state,
        selection,
        initial_plan,
        |_| Ok(()),
    )
}

/// Execute with deterministic test-only failures at typed mutation boundaries.
#[doc(hidden)]
pub fn execute_with_fault_hook<F>(
    home: &GripHome,
    expected_registry: &RegistrySnapshot,
    expected_state: &StateSnapshot,
    selection: &Selection,
    initial_plan: &MutationPlan,
    fault: F,
) -> Result<ExecutionSuccess, GripError>
where
    F: FnMut(crate::mutation::FaultPhase) -> Result<(), GripError>,
{
    execute_with_faults(
        home,
        expected_registry,
        expected_state,
        selection,
        initial_plan,
        fault,
        None,
    )
}

/// Execute with both pipeline and accepted-state publication fault injection.
#[doc(hidden)]
pub fn execute_with_faults<F>(
    home: &GripHome,
    expected_registry: &RegistrySnapshot,
    expected_state: &StateSnapshot,
    selection: &Selection,
    initial_plan: &MutationPlan,
    mut fault: F,
    state_fault: Option<crate::state::publication::PublicationFault>,
) -> Result<ExecutionSuccess, GripError>
where
    F: FnMut(crate::mutation::FaultPhase) -> Result<(), GripError>,
{
    let operation = initial_plan.operation;
    let operation_name = operation.as_str();
    let _mutation_guard = crate::state::mutation_lock::MutationLock::acquire(home, operation_name)?;
    fault(crate::mutation::FaultPhase::AfterMutationLock)?;
    let locked_registry = crate::registry::publication::load(home, false)
        .map_err(|error| error.for_mapping_operation(operation_name))?;
    let locked_state = crate::state::publication::load(home)?;
    if locked_registry.bytes != expected_registry.bytes
        || locked_registry.registry != expected_registry.registry
        || locked_state.bytes != expected_state.bytes
        || locked_state.accepted != expected_state.accepted
    {
        return Err(stale(
            operation,
            "registry or accepted state changed after mutation planning",
        ));
    }
    let locked_plan = rebuild_plan(
        home,
        &locked_registry,
        &locked_state,
        selection,
        initial_plan,
    )?;
    if &locked_plan != initial_plan {
        return Err(stale(
            operation,
            "mutation plan changed after lock acquisition",
        ));
    }
    let mut receipt = crate::operation::publication::initialize(home, &locked_plan)?;
    let mut plan = locked_plan;
    let mut actioned = plan
        .entries
        .iter()
        .filter(|entry| entry.disposition == Disposition::AcceptOnly)
        .map(|entry| entry.identity.clone())
        .collect::<BTreeSet<_>>();

    for index in 0..plan.actions.len() {
        let action = &mut plan.actions[index];
        action.status = crate::mutation::model::ActionStatus::InProgress;
        let mut failure_reason = "journal_failure";
        let attempted = (|| -> Result<(), GripError> {
            receipt.checkpoint_action(index, "in_progress", evidence(action), None)?;
            failure_reason = "revalidation_failure";
            fault(crate::mutation::FaultPhase::BeforeActionRevalidation(index))?;
            revalidate_action(home, &locked_registry, &locked_state, action, operation)?;
            action.milestones.revalidation = "passed".into();
            failure_reason = "journal_failure";
            receipt.checkpoint_action(index, "in_progress", evidence(action), None)?;
            let destination = action.destination.clone();
            match action.kind {
                ActionKind::CreateParentDirectory | ActionKind::CreateDirectory => {
                    failure_reason = "publication_failure";
                    crate::mutation::filesystem::create_directory(&destination)?;
                    action.milestones.publication = "visible".into();
                    action.milestones.verification = "verified".into();
                    action.milestones.durability_confirmed = true;
                }
                ActionKind::AddFile | ActionKind::ReplaceFile => {
                    let identity = action.identity.as_ref().ok_or_else(|| {
                        GripError::Internal("managed file action has no identity".into())
                    })?;
                    let direction = action.direction;
                    let (origin, expected_origin, expected_target) = match direction {
                        MutationDirection::Push => (
                            identity.source_path(),
                            action.expected_source.as_ref(),
                            action.expected_destination.as_ref(),
                        ),
                        MutationDirection::Pull => (
                            identity.destination_path(),
                            action.expected_destination.as_ref(),
                            action.expected_source.as_ref(),
                        ),
                    };
                    let expected_origin = expected_origin.ok_or_else(|| {
                        GripError::Internal("managed file action has no origin evidence".into())
                    })?;
                    if action.kind == ActionKind::ReplaceFile {
                        action.milestones.recovery = "planned".into();
                        failure_reason = "journal_failure";
                        receipt.checkpoint_action(index, "in_progress", evidence(action), None)?;
                        failure_reason = "recovery_failure";
                        fault(crate::mutation::FaultPhase::BeforeRecovery(index))?;
                        let recovery = crate::mutation::recovery::preserve_with_post(
                            &receipt,
                            index,
                            identity,
                            &destination,
                            expected_target.ok_or_else(|| {
                                GripError::Internal("replacement has no target evidence".into())
                            })?,
                            Some(expected_origin),
                        )?;
                        action.milestones.recovery = "preserved".into();
                        action.milestones.recovery_ref = Some(recovery.relative_ref);
                        failure_reason = "journal_failure";
                        receipt.checkpoint_action(index, "in_progress", evidence(action), None)?;
                        failure_reason = "recovery_failure";
                        fault(crate::mutation::FaultPhase::AfterRecovery(index))?;
                    }
                    failure_reason = "staging_failure";
                    fault(crate::mutation::FaultPhase::BeforeStaging(index))?;
                    let mut staged = crate::mutation::filesystem::stage_file_for(
                        direction,
                        &origin,
                        &destination,
                        expected_origin,
                    )?;
                    action.milestones.staging = "verified".into();
                    failure_reason = "journal_failure";
                    receipt.checkpoint_action(index, "in_progress", evidence(action), None)?;
                    failure_reason = "staging_failure";
                    fault(crate::mutation::FaultPhase::AfterStaging(index))?;
                    failure_reason = "publication_failure";
                    fault(crate::mutation::FaultPhase::BeforePayloadPublication(index))?;
                    if action.kind == ActionKind::AddFile {
                        crate::mutation::filesystem::publish_addition(&mut staged, &destination)?;
                    } else {
                        revalidate_action(
                            home,
                            &locked_registry,
                            &locked_state,
                            action,
                            operation,
                        )?;
                        crate::mutation::filesystem::publish_replacement(
                            &mut staged,
                            &destination,
                        )?;
                    }
                    action.milestones.publication = "visible".into();
                    action.milestones.durability_confirmed = true;
                    failure_reason = "journal_failure";
                    receipt.checkpoint_action(index, "in_progress", evidence(action), None)?;
                    failure_reason = "publication_failure";
                    fault(crate::mutation::FaultPhase::AfterPayloadPublication(index))?;
                    failure_reason = "verification_failure";
                    fault(crate::mutation::FaultPhase::BeforeTargetVerification(index))?;
                    fault(crate::mutation::FaultPhase::BeforeDestinationVerification(
                        index,
                    ))?;
                    crate::mutation::filesystem::verify_target(&destination, expected_origin)?;
                    action.milestones.verification = "verified".into();
                    fault(crate::mutation::FaultPhase::AfterTargetVerification(index))?;
                    failure_reason = "cleanup_failure";
                    drop(staged);
                    fault(crate::mutation::FaultPhase::AfterStagingCleanup(index))?;
                }
            }
            action.status = crate::mutation::model::ActionStatus::Completed;
            failure_reason = "journal_failure";
            receipt.checkpoint_action(index, "completed", evidence(action), None)?;
            Ok(())
        })();
        match attempted {
            Ok(()) => {
                if let Some(identity) = &action.identity {
                    actioned.insert(identity.clone());
                }
            }
            Err(error) => {
                let category = error.category();
                let reason = stable_reason(&error, failure_reason);
                let message = error.to_string();
                if let GripError::PushSideEffect {
                    reason: side_effect_reason,
                    publication_visible,
                    verification,
                    durability_confirmed,
                    ..
                }
                | GripError::MutationSideEffect {
                    reason: side_effect_reason,
                    publication_visible,
                    verification,
                    durability_confirmed,
                    ..
                } = &error
                {
                    if side_effect_reason == "recovery_failure" {
                        action.milestones.recovery = "failed".into();
                    } else {
                        action.milestones.publication = if *publication_visible {
                            "visible".into()
                        } else {
                            "not_visible".into()
                        };
                        action.milestones.verification = verification.clone();
                        action.milestones.durability_confirmed = *durability_confirmed;
                    }
                } else if failure_reason == "recovery_failure"
                    && action.milestones.recovery != "preserved"
                {
                    action.milestones.recovery = "failed".into();
                } else if failure_reason == "staging_failure"
                    && action.milestones.staging != "verified"
                {
                    action.milestones.staging = "failed".into();
                } else if failure_reason == "verification_failure"
                    && action.milestones.verification != "verified"
                {
                    action.milestones.verification = "failed".into();
                }
                action.status = crate::mutation::model::ActionStatus::Failed;
                action.failure = Some(reason.clone());
                plan.counts.completed = index;
                plan.counts.failed = 1;
                plan.counts.unattempted = plan.actions.len().saturating_sub(index + 1);
                let _ = receipt.checkpoint_action(
                    index,
                    "failed",
                    evidence(&plan.actions[index]),
                    Some(reason.clone()),
                );
                let baseline = crate::mutation::model::BaselineOutcome {
                    outcome: "not_published".into(),
                    prior_generation: locked_state.accepted.generation,
                    published_generation: None,
                    authoritative_generation: locked_state.accepted.generation,
                    publication_visible: false,
                    durability_confirmed: true,
                };
                let _ = receipt.checkpoint_summary(
                    "failed",
                    serde_json::to_value(&baseline).expect("baseline outcome serializes"),
                    "prepared",
                    Some(reason.clone()),
                );
                let paths = [
                    plan.actions[index].source_path.clone(),
                    Some(plan.actions[index].destination_path.clone()),
                ]
                .into_iter()
                .flatten()
                .collect();
                let completion =
                    if index > 0 || plan.actions[index].milestones.publication == "visible" {
                        "partial"
                    } else {
                        "failed"
                    };
                let expected_source = plan.actions[index].expected_source.clone();
                let expected_destination = plan.actions[index].expected_destination.clone();
                let (observed_source, observed_destination) = observe_action_state(
                    home,
                    &locked_registry,
                    &locked_state,
                    &plan.actions[index],
                );
                return Err(mutation_failure(
                    operation,
                    crate::error::MutationFailure {
                        operation_id: receipt.operation_id().to_owned(),
                        plan,
                        baseline,
                        reason,
                        phase: failure_reason.into(),
                        paths,
                        failed_action_index: Some(index),
                        expected_source,
                        expected_destination,
                        observed_source,
                        observed_destination,
                        completion: completion.into(),
                        category,
                        message,
                    },
                ));
            }
        }
    }

    macro_rules! finish_or_fail {
        ($expression:expr, $reason:literal) => {
            match $expression {
                Ok(value) => value,
                Err(error) => {
                    return terminal_prebaseline_failure(
                        receipt,
                        plan,
                        &locked_state,
                        error,
                        $reason,
                    );
                }
            }
        };
    }

    finish_or_fail!(
        fault(crate::mutation::FaultPhase::BeforeFinalObservation),
        "verification_failure"
    );
    let final_registry = finish_or_fail!(
        crate::registry::publication::load(home, false)
            .map_err(|error| error.for_mapping_operation(operation_name)),
        "coordination_failure"
    );
    if final_registry.bytes != locked_registry.bytes
        || final_registry.registry != locked_registry.registry
    {
        return terminal_prebaseline_failure(
            receipt,
            plan,
            &locked_state,
            stale(
                operation,
                "accepted registry changed during mutation execution",
            ),
            "revalidation_failure",
        );
    }
    let final_observed = finish_or_fail!(
        crate::observation::inspect(home, &final_registry, &locked_state.accepted, selection)
            .map_err(|error| error.for_operation(operation_name)),
        "verification_failure"
    );
    let final_records = final_observed
        .values()
        .map(|entry| {
            classification::classify(entry, locked_state.accepted.baselines.get(&entry.identity))
        })
        .collect::<Vec<_>>();
    let candidate = finish_or_fail!(
        crate::baseline::build_from_actioned(&locked_state.accepted, &final_records, &actioned),
        "baseline_candidate_failure"
    );
    finish_or_fail!(
        fault(crate::mutation::FaultPhase::BeforeFinalCoordination),
        "coordination_failure"
    );
    let _registry_guard = finish_or_fail!(
        crate::registry::publication::acquire_guard(home, operation_name),
        "coordination_failure"
    );
    let state_directory = finish_or_fail!(
        crate::state::publication::prepare_directory(home),
        "coordination_failure"
    );
    let _state_guard = finish_or_fail!(
        crate::state::lock::PublicationLock::acquire(&state_directory.join("state.lock")),
        "coordination_failure"
    );
    finish_or_fail!(
        revalidate_registry_bytes(home, &final_registry, operation),
        "revalidation_failure"
    );
    finish_or_fail!(
        crate::state::publication::revalidate(home, &locked_state),
        "revalidation_failure"
    );
    if let Err(error) = fault(crate::mutation::FaultPhase::BeforeBaselinePublication) {
        return terminal_baseline_failure(receipt, plan, &locked_state, error, false, None);
    }
    let generation = match crate::state::publication::publish_accepted_locked_with_fault(
        home,
        &locked_state,
        &candidate.next,
        state_fault,
    ) {
        Ok(Some(generation)) => generation,
        Ok(None) => {
            return terminal_baseline_failure(
                receipt,
                plan,
                &locked_state,
                GripError::Internal("actionful mutation published no accepted generation".into()),
                false,
                None,
            );
        }
        Err(error) => {
            let publication_visible = matches!(
                &error,
                GripError::Mapping {
                    publication_visible: true,
                    ..
                }
            );
            let candidate_generation = locked_state
                .accepted
                .generation
                .map_or(Some(0), |value| value.checked_add(1));
            let baseline = crate::mutation::model::BaselineOutcome {
                outcome: if publication_visible {
                    "publication_failed".into()
                } else {
                    "not_published".into()
                },
                prior_generation: locked_state.accepted.generation,
                published_generation: publication_visible
                    .then_some(candidate_generation)
                    .flatten(),
                authoritative_generation: if publication_visible {
                    candidate_generation
                } else {
                    locked_state.accepted.generation
                },
                publication_visible,
                durability_confirmed: !publication_visible,
            };
            let reason = "baseline_publication_failure".to_owned();
            let message = error.to_string();
            plan.counts.completed = plan.actions.len();
            plan.counts.unattempted = 0;
            let _ = receipt.checkpoint_summary(
                "failed",
                serde_json::to_value(&baseline).expect("baseline outcome serializes"),
                "prepared",
                Some(reason.clone()),
            );
            return Err(mutation_failure(
                operation,
                crate::error::MutationFailure {
                    operation_id: receipt.operation_id().to_owned(),
                    plan,
                    baseline,
                    reason,
                    phase: "baseline_publication".into(),
                    paths: Vec::new(),
                    failed_action_index: None,
                    expected_source: None,
                    expected_destination: None,
                    observed_source: None,
                    observed_destination: None,
                    completion: if publication_visible {
                        "failed"
                    } else {
                        "partial"
                    }
                    .into(),
                    category: error.category(),
                    message,
                },
            ));
        }
    };
    if let Err(error) = fault(crate::mutation::FaultPhase::AfterBaselinePublication) {
        return terminal_baseline_failure(
            receipt,
            plan,
            &locked_state,
            error,
            true,
            Some(generation),
        );
    }
    if let Err(error) = fault(crate::mutation::FaultPhase::BeforeTerminalSummary) {
        return terminal_baseline_failure(
            receipt,
            plan,
            &locked_state,
            error,
            true,
            Some(generation),
        );
    }
    if let Err(error) = receipt.checkpoint_summary(
        "completed",
        serde_json::json!({
            "outcome":"published",
            "prior_generation":locked_state.accepted.generation,
            "published_generation":generation,
            "authoritative_generation":generation,
            "publication_visible":true,
            "durability_confirmed":true
        }),
        "prepared",
        None,
    ) {
        return terminal_baseline_failure(
            receipt,
            plan,
            &locked_state,
            error,
            true,
            Some(generation),
        );
    }
    let operation_id = receipt.operation_id().to_owned();
    plan.counts.completed = plan.actions.len();
    plan.counts.unattempted = 0;
    Ok(ExecutionSuccess {
        plan,
        operation_id,
        generation,
        prior_generation: locked_state.accepted.generation,
    })
}

fn rebuild_plan(
    home: &GripHome,
    registry: &RegistrySnapshot,
    state: &StateSnapshot,
    selection: &Selection,
    expected: &MutationPlan,
) -> Result<MutationPlan, GripError> {
    let operation = expected.operation;
    let operation_name = operation.as_str();
    let observed = crate::observation::inspect(home, registry, &state.accepted, selection)
        .map_err(|error| error.for_operation(operation_name))?;
    let records = observed
        .values()
        .map(|entry| classification::classify(entry, state.accepted.baselines.get(&entry.identity)))
        .collect();
    match operation {
        MutationOperation::Push => crate::mutation::plan::build_with_parent_requirements(
            expected.scope.clone(),
            records,
            registry.missing_destination_parents(),
        ),
        MutationOperation::Pull => crate::mutation::plan::build_for(
            MutationDirection::Pull,
            expected.scope.clone(),
            records,
        ),
        MutationOperation::Sync => crate::mutation::plan::build_sync_with_parent_requirements(
            expected.scope.clone(),
            records,
            registry.missing_destination_parents(),
        ),
        MutationOperation::Resolve => crate::mutation::plan::build_resolution(
            expected.scope.clone(),
            records,
            expected.winner.ok_or_else(|| {
                GripError::Internal("resolution plan has no explicit winner".into())
            })?,
        ),
    }
}

fn revalidate_action(
    home: &GripHome,
    registry: &RegistrySnapshot,
    state: &StateSnapshot,
    action: &crate::mutation::model::MutationAction,
    operation: MutationOperation,
) -> Result<(), GripError> {
    let operation_name = operation.as_str();
    let current_registry = crate::registry::publication::load(home, false)
        .map_err(|error| error.for_mapping_operation(operation_name))?;
    if current_registry.bytes != registry.bytes || current_registry.registry != registry.registry {
        return Err(stale(
            operation,
            "accepted registry changed during mutation execution",
        ));
    }
    crate::state::publication::revalidate(home, state)?;
    let Some(identity) = &action.identity else {
        return Ok(());
    };
    let selected = Selection::Entry(identity.clone());
    let observed = crate::observation::inspect(home, &current_registry, &state.accepted, &selected)
        .map_err(|error| error.for_operation(operation_name))?;
    let entry = observed.get(identity).ok_or_else(|| {
        stale(
            operation,
            "managed entry disappeared during action revalidation",
        )
    })?;
    let record = classification::classify(entry, state.accepted.baselines.get(identity));
    let source_matches = record.source == action.expected_source;
    let destination_matches = record.destination == action.expected_destination;
    let classification_matches = operation != MutationOperation::Resolve
        || record.classification == crate::classification::model::Classification::DivergentConflict;
    if record.blocking && operation != MutationOperation::Resolve
        || !classification_matches
        || !source_matches
        || !destination_matches
    {
        return Err(stale(
            operation,
            "managed entry changed before mutation action",
        ));
    }
    Ok(())
}

fn evidence(action: &crate::mutation::model::MutationAction) -> ActionCheckpointEvidenceV1 {
    ActionCheckpointEvidenceV1 {
        revalidation: action.milestones.revalidation.clone(),
        recovery: action.milestones.recovery.clone(),
        recovery_ref: action.milestones.recovery_ref.clone(),
        staging: action.milestones.staging.clone(),
        publication: action.milestones.publication.clone(),
        verification: action.milestones.verification.clone(),
        durability_confirmed: action.milestones.durability_confirmed,
    }
}

fn stale(operation: MutationOperation, message: &str) -> GripError {
    GripError::discovery_operational(
        operation.as_str(),
        match operation {
            MutationOperation::Push => "stale_push_evidence",
            MutationOperation::Pull => "stale_pull_evidence",
            MutationOperation::Sync => "stale_sync_evidence",
            MutationOperation::Resolve => "stale_resolution_evidence",
        },
        Vec::new(),
        message,
    )
}

fn stable_reason(error: &GripError, fallback: &str) -> String {
    match error {
        GripError::PushSideEffect { reason, .. }
        | GripError::MutationSideEffect { reason, .. }
        | GripError::Mapping { reason, .. } => reason.clone(),
        GripError::CorruptState(_) => "corrupt_state".into(),
        GripError::StateContention | GripError::MutationContention { .. } => {
            "state_contention".into()
        }
        _ => fallback.into(),
    }
}

fn revalidate_registry_bytes(
    home: &GripHome,
    expected: &RegistrySnapshot,
    operation: MutationOperation,
) -> Result<(), GripError> {
    let current = crate::registry::publication::load(home, false)
        .map_err(|error| error.for_mapping_operation(operation.as_str()))?;
    if current.bytes != expected.bytes || current.registry != expected.registry {
        return Err(stale(
            operation,
            "accepted registry changed during mutation execution",
        ));
    }
    Ok(())
}

fn terminal_baseline_failure(
    mut receipt: crate::operation::publication::OperationReceipt,
    mut plan: MutationPlan,
    state: &StateSnapshot,
    error: GripError,
    publication_visible: bool,
    candidate_generation: Option<u64>,
) -> Result<ExecutionSuccess, GripError> {
    let operation = plan.operation;
    let baseline = crate::mutation::model::BaselineOutcome {
        outcome: if publication_visible {
            "publication_failed".into()
        } else {
            "not_published".into()
        },
        prior_generation: state.accepted.generation,
        published_generation: publication_visible
            .then_some(candidate_generation)
            .flatten(),
        authoritative_generation: if publication_visible {
            candidate_generation
        } else {
            state.accepted.generation
        },
        publication_visible,
        durability_confirmed: true,
    };
    let reason = "baseline_publication_failure".to_owned();
    let category = error.category();
    let message = error.to_string();
    plan.counts.completed = plan.actions.len();
    plan.counts.unattempted = 0;
    let _ = receipt.checkpoint_summary(
        "failed",
        serde_json::to_value(&baseline).expect("baseline outcome serializes"),
        "prepared",
        Some(reason.clone()),
    );
    Err(mutation_failure(
        operation,
        crate::error::MutationFailure {
            operation_id: receipt.operation_id().to_owned(),
            plan,
            baseline,
            reason,
            phase: "baseline_publication".into(),
            paths: Vec::new(),
            failed_action_index: None,
            expected_source: None,
            expected_destination: None,
            observed_source: None,
            observed_destination: None,
            completion: if publication_visible {
                "failed"
            } else {
                "partial"
            }
            .into(),
            category,
            message,
        },
    ))
}

fn terminal_prebaseline_failure(
    mut receipt: crate::operation::publication::OperationReceipt,
    mut plan: MutationPlan,
    state: &StateSnapshot,
    error: GripError,
    reason: &str,
) -> Result<ExecutionSuccess, GripError> {
    let operation = plan.operation;
    let baseline = crate::mutation::model::BaselineOutcome {
        outcome: "not_published".into(),
        prior_generation: state.accepted.generation,
        published_generation: None,
        authoritative_generation: state.accepted.generation,
        publication_visible: false,
        durability_confirmed: true,
    };
    let category = error.category();
    plan.counts.completed = plan.actions.len();
    plan.counts.failed = 0;
    plan.counts.unattempted = 0;
    let _ = receipt.checkpoint_summary(
        "failed",
        serde_json::to_value(&baseline).expect("baseline outcome serializes"),
        "prepared",
        Some(reason.into()),
    );
    Err(mutation_failure(
        operation,
        crate::error::MutationFailure {
            operation_id: receipt.operation_id().to_owned(),
            plan,
            baseline,
            reason: reason.into(),
            phase: reason.into(),
            paths: Vec::new(),
            failed_action_index: None,
            expected_source: None,
            expected_destination: None,
            observed_source: None,
            observed_destination: None,
            completion: "partial".into(),
            category,
            message: error.to_string(),
        },
    ))
}

fn mutation_failure(
    operation: MutationOperation,
    failure: crate::error::MutationFailure,
) -> GripError {
    match operation {
        MutationOperation::Push => GripError::PushFailed(Box::new(failure)),
        MutationOperation::Pull | MutationOperation::Sync | MutationOperation::Resolve => {
            GripError::MutationFailed(Box::new(failure))
        }
    }
}

fn observe_action_state(
    home: &GripHome,
    _registry: &RegistrySnapshot,
    state: &StateSnapshot,
    action: &crate::mutation::model::MutationAction,
) -> (
    Option<crate::observation::model::SupportedState>,
    Option<crate::observation::model::SupportedState>,
) {
    let Some(identity) = &action.identity else {
        return (None, None);
    };
    let selected = Selection::Entry(identity.clone());
    crate::registry::publication::load(home, false)
        .and_then(|registry| {
            crate::observation::inspect(home, &registry, &state.accepted, &selected)
        })
        .ok()
        .and_then(|observed| observed.get(identity).cloned())
        .map_or((None, None), |entry| (entry.source, entry.destination))
}
