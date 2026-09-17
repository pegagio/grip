//! Lock-held direction-neutral execution and accepted-state publication.

use crate::classification;
use crate::error::GripError;
use crate::mutation::model::{
    ActionKind, Disposition, MutationDirection, MutationOperation, MutationPlan,
};
use crate::observation::model::Selection;
use crate::operation::model::ActionCheckpointEvidenceV2;
use crate::project::ProjectPaths;
use crate::registry::publication::RegistrySnapshot;
use crate::state::publication::StateSnapshot;
use std::collections::{BTreeMap, BTreeSet};

/// Successful execution evidence returned to command orchestration.
#[derive(Debug)]
pub struct ExecutionSuccess {
    pub plan: MutationPlan,
    pub operation_id: String,
    pub generation: u64,
    pub prior_generation: Option<u64>,
}

/// Immutable operation evidence required while revalidating one planned action.
struct RevalidationContext<'a> {
    operation: MutationOperation,
    use_modification_time: bool,
    expected_adoption_policy_digest: Option<&'a str>,
    adoption_target: Option<&'a crate::observation::model::EntryIdentity>,
}

/// Execute one actionful, unblocked plan under the selected project's writer lock.
pub fn execute(
    home: &ProjectPaths,
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
    home: &ProjectPaths,
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
    home: &ProjectPaths,
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
    crate::revalidate_project_for_mutation()?;
    fault(crate::mutation::FaultPhase::AfterMutationLock)?;
    let locked_registry = crate::registry::publication::load(home, false)
        .map_err(|error| error.for_mapping_operation(operation_name))?;
    let mut locked_state = crate::state::publication::load(home)?;
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
    let mut aggregate_publication_generations = BTreeMap::new();
    let adoption_target = (operation == MutationOperation::Adopt)
        .then(|| {
            plan.entries
                .iter()
                .max_by_key(|entry| entry.identity.relative_path.len())
                .map(|entry| entry.identity.clone())
        })
        .flatten();

    for index in 0..plan.actions.len() {
        let adoption_policy_digest = plan.adoption_policy_digest.as_deref();
        let revalidation = RevalidationContext {
            operation,
            use_modification_time: plan.use_modification_time,
            expected_adoption_policy_digest: adoption_policy_digest,
            adoption_target: adoption_target.as_ref(),
        };
        let action = &mut plan.actions[index];
        action.status = crate::mutation::model::ActionStatus::InProgress;
        let mut failure_reason = "journal_failure";
        let attempted = (|| -> Result<(), GripError> {
            receipt.checkpoint_action(index, "in_progress", evidence(action), None)?;
            failure_reason = "revalidation_failure";
            fault(crate::mutation::FaultPhase::BeforeActionRevalidation(index))?;
            revalidate_action(home, &locked_registry, &locked_state, action, &revalidation)?;
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
                ActionKind::AddFile
                | ActionKind::ReplaceFile
                | ActionKind::ReplaceDestinationLinkFile => {
                    let identity = action.identity.as_ref().ok_or_else(|| {
                        GripError::Internal("managed file action has no identity".into())
                    })?;
                    let direction = action.direction;
                    let (origin, expected_origin, _expected_target) = match direction {
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
                            &revalidation,
                        )?;
                        crate::mutation::filesystem::publish_replacement(
                            &mut staged,
                            &destination,
                        )?;
                    }
                    action.milestones.publication = "visible".into();
                    action.milestones.durability_confirmed = true;
                    if let Some(metadata) = action.metadata.as_ref() {
                        crate::metadata::macos::apply_metadata_paths_with_options_and_hook(
                            &origin,
                            &destination,
                            metadata.expected_after.node_kind,
                            &metadata.expected_after.metadata,
                            metadata.changed_dimensions.contains(
                                &crate::metadata::model::MetadataDimension::ModificationTime,
                            ),
                            |phase| metadata_fault(&mut fault, index, phase),
                        )
                        .map_err(|error| {
                            GripError::from_io("could not apply complete file metadata", error)
                        })?;
                    }
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
                    if let Some(metadata) = action.metadata.as_ref() {
                        let complete = crate::observation::fingerprint::inspect_complete(
                            &destination,
                            metadata.expected_after.node_kind,
                        )?;
                        if !crate::classification::complete_equivalent_with_options(
                            &complete.state,
                            &metadata.expected_after,
                            crate::classification::ComparisonOptions {
                                use_modification_time: metadata.changed_dimensions.contains(
                                    &crate::metadata::model::MetadataDimension::ModificationTime,
                                ),
                            },
                        ) {
                            return Err(GripError::Internal(
                                "complete file verification failed".into(),
                            ));
                        }
                    }
                    action.milestones.verification = "verified".into();
                    fault(crate::mutation::FaultPhase::AfterTargetVerification(index))?;
                    failure_reason = "cleanup_failure";
                    drop(staged);
                    fault(crate::mutation::FaultPhase::AfterStagingCleanup(index))?;
                }
                ActionKind::ReplaceDestinationLinkDirectory => {
                    let identity = action.identity.as_ref().ok_or_else(|| {
                        GripError::Internal("managed directory action has no identity".into())
                    })?;
                    let expected = action.expected_source.as_ref().ok_or_else(|| {
                        GripError::Internal(
                            "managed directory action has no source evidence".into(),
                        )
                    })?;
                    if expected.node_kind != crate::discovery::model::NodeKind::Directory {
                        return Err(GripError::Internal(
                            "destination-link directory action has non-directory source evidence"
                                .into(),
                        ));
                    }
                    failure_reason = "publication_failure";
                    fault(crate::mutation::FaultPhase::BeforePayloadPublication(index))?;
                    revalidate_action(
                        home,
                        &locked_registry,
                        &locked_state,
                        action,
                        &revalidation,
                    )?;
                    crate::mutation::filesystem::replace_link_with_empty_directory(&destination)?;
                    action.milestones.publication = "visible".into();
                    action.milestones.durability_confirmed = true;
                    if let Some(metadata) = action.metadata.as_ref() {
                        crate::metadata::macos::apply_metadata_paths_with_options_and_hook(
                            &identity.source_path(),
                            &destination,
                            crate::discovery::model::NodeKind::Directory,
                            &metadata.expected_after.metadata,
                            false,
                            |phase| metadata_fault(&mut fault, index, phase),
                        )
                        .map_err(|error| {
                            GripError::from_io("could not apply complete directory metadata", error)
                        })?;
                    }
                    failure_reason = "verification_failure";
                    crate::mutation::filesystem::verify_target(&destination, expected)?;
                    action.milestones.verification = "verified".into();
                }
                ActionKind::ApplyMetadata | ActionKind::FinalizeDirectoryMetadata => {
                    let identity = action.identity.as_ref().ok_or_else(|| {
                        GripError::Internal("metadata action has no managed identity".into())
                    })?;
                    let metadata = action.metadata.clone().ok_or_else(|| {
                        GripError::Internal(
                            "metadata action is missing complete transition evidence".into(),
                        )
                    })?;
                    let (origin, _expected_target_legacy) = match action.direction {
                        MutationDirection::Push => {
                            (identity.source_path(), action.expected_destination.as_ref())
                        }
                        MutationDirection::Pull => {
                            (identity.destination_path(), action.expected_source.as_ref())
                        }
                    };
                    failure_reason = "publication_failure";
                    action.milestones.publication = "visible".into();
                    crate::metadata::macos::apply_metadata_paths_with_options_and_hook(
                        &origin,
                        &destination,
                        metadata.expected_after.node_kind,
                        &metadata.expected_after.metadata,
                        metadata
                            .changed_dimensions
                            .contains(&crate::metadata::model::MetadataDimension::ModificationTime),
                        |phase| metadata_fault(&mut fault, index, phase),
                    )
                    .map_err(|error| {
                        GripError::from_io("could not apply complete metadata", error)
                    })?;
                    action.milestones.publication = "visible".into();
                    action.milestones.durability_confirmed = true;
                    failure_reason = "verification_failure";
                    let verified = crate::observation::fingerprint::inspect_complete(
                        &destination,
                        metadata.expected_after.node_kind,
                    )?;
                    if !crate::classification::complete_equivalent_with_options(
                        &verified.state,
                        &metadata.expected_after,
                        crate::classification::ComparisonOptions {
                            use_modification_time: metadata.changed_dimensions.contains(
                                &crate::metadata::model::MetadataDimension::ModificationTime,
                            ),
                        },
                    ) {
                        return Err(GripError::Internal(
                            "complete metadata verification failed".into(),
                        ));
                    }
                    action.milestones.verification = "verified".into();
                }
                ActionKind::RemoveDestination => {
                    let expected = action.expected_destination.as_ref().ok_or_else(|| {
                        GripError::Internal(
                            "aggregate destination removal has no destination evidence".into(),
                        )
                    })?;
                    failure_reason = "publication_failure";
                    fault(crate::mutation::FaultPhase::BeforePayloadPublication(index))?;
                    action.milestones.durability_confirmed =
                        crate::mutation::filesystem::remove_verified(&destination, expected)?;
                    action.milestones.publication = "not_visible".into();
                    failure_reason = "verification_failure";
                    if std::fs::symlink_metadata(&destination).is_ok() {
                        return Err(GripError::Internal(
                            "aggregate destination removal remained visible".into(),
                        ));
                    }
                    action.milestones.verification = "verified".into();
                }
            }
            action.status = crate::mutation::model::ActionStatus::Completed;
            failure_reason = "journal_failure";
            receipt.checkpoint_action(index, "completed", evidence(action), None)?;
            Ok(())
        })();
        match attempted {
            Ok(()) => {
                let completed_identity = action.identity.clone();
                if let Some(identity) = completed_identity {
                    actioned.insert(identity.clone());
                    if operation == MutationOperation::AggregateForcePush
                        && entry_actions_completed(&plan, &identity)
                    {
                        let published = publish_aggregate_entry(
                            home,
                            &locked_registry,
                            &locked_state,
                            &identity,
                            state_fault,
                        )?;
                        if let Some(generation) = published.accepted.generation {
                            aggregate_publication_generations.insert(identity, generation);
                        }
                        locked_state = published;
                    }
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
                    if side_effect_reason != "recovery_failure" {
                        action.milestones.publication = if *publication_visible {
                            "visible".into()
                        } else {
                            "not_visible".into()
                        };
                        action.milestones.verification = verification.clone();
                        action.milestones.durability_confirmed = *durability_confirmed;
                    }
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
                let aggregate_publication_visible = operation
                    == MutationOperation::AggregateForcePush
                    && locked_state.accepted.generation != expected_state.accepted.generation;
                let baseline = crate::mutation::model::BaselineOutcome {
                    outcome: if aggregate_publication_visible {
                        "partially_published".into()
                    } else {
                        "not_published".into()
                    },
                    prior_generation: expected_state.accepted.generation,
                    published_generation: aggregate_publication_visible
                        .then_some(locked_state.accepted.generation)
                        .flatten(),
                    authoritative_generation: locked_state.accepted.generation,
                    publication_visible: aggregate_publication_visible,
                    durability_confirmed: true,
                };
                let _ = if operation == MutationOperation::AggregateForcePush {
                    receipt.checkpoint_aggregate_summary(
                        "failed",
                        serde_json::to_value(&baseline).expect("baseline outcome serializes"),
                        "prepared",
                        Some(reason.clone()),
                        aggregate_entry_outcomes(home, &plan, &aggregate_publication_generations)?,
                    )
                } else {
                    receipt.checkpoint_summary(
                        "failed",
                        serde_json::to_value(&baseline).expect("baseline outcome serializes"),
                        "prepared",
                        Some(reason.clone()),
                    )
                };
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

    if operation == MutationOperation::AggregateForcePush {
        let generation = locked_state.accepted.generation.ok_or_else(|| {
            GripError::Internal(
                "aggregate force completed without accepted state generation".into(),
            )
        })?;
        receipt.checkpoint_aggregate_summary(
            "completed",
            serde_json::json!({
                "outcome":"published_per_entry",
                "prior_generation":expected_state.accepted.generation,
                "published_generation":generation,
                "authoritative_generation":generation,
                "publication_visible":true,
                "durability_confirmed":true
            }),
            "prepared",
            None,
            aggregate_entry_outcomes(home, &plan, &aggregate_publication_generations)?,
        )?;
        plan.counts.completed = plan.actions.len();
        plan.counts.unattempted = 0;
        return Ok(ExecutionSuccess {
            plan,
            operation_id: receipt.operation_id().to_owned(),
            generation,
            prior_generation: expected_state.accepted.generation,
        });
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
        if operation == MutationOperation::Adopt {
            let crate::observation::model::Selection::Entries(identities) = selection else {
                return terminal_prebaseline_failure(
                    receipt,
                    plan,
                    &locked_state,
                    GripError::Internal("adoption execution lost exact selected identities".into()),
                    "verification_failure",
                );
            };
            crate::observation::inspect_adoption(
                home,
                &final_registry,
                &locked_state.accepted,
                identities,
            )
            .map_err(|error| error.for_operation(operation_name))
        } else {
            crate::observation::inspect(home, &final_registry, &locked_state.accepted, selection)
                .map_err(|error| error.for_operation(operation_name))
        },
        "verification_failure"
    );
    let final_records = final_observed
        .values()
        .map(|entry| classification::classify_accepted(entry, &locked_state.accepted))
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
    let state_lock = finish_or_fail!(
        crate::state::lock::project_lock_path(home, "state.lock"),
        "coordination_failure"
    );
    let _state_guard = finish_or_fail!(
        crate::state::lock::PublicationLock::acquire(&state_lock),
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
    let generation =
        match crate::state::publication::publish_complete_after_action_locked_with_fault(
            home,
            &locked_state,
            &candidate.next.complete_baselines,
            state_fault,
        ) {
            Ok(generation) => generation,
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

fn aggregate_entry_outcomes(
    home: &ProjectPaths,
    plan: &MutationPlan,
    publication_generations: &BTreeMap<crate::observation::model::EntryIdentity, u64>,
) -> Result<Vec<crate::operation::model::AggregateEntryOutcomeV2>, GripError> {
    plan.entries
        .iter()
        .map(|entry| {
            let actions = entry
                .action_indexes
                .iter()
                .filter_map(|index| plan.actions.get(*index))
                .collect::<Vec<_>>();
            let failed = actions
                .iter()
                .find(|action| action.status == crate::mutation::model::ActionStatus::Failed);
            let completed = !actions.is_empty()
                && actions
                    .iter()
                    .all(|action| action.status == crate::mutation::model::ActionStatus::Completed);
            Ok(crate::operation::model::AggregateEntryOutcomeV2 {
                identity: crate::state::portable_identity_from_runtime(home, &entry.identity)?,
                status: if completed {
                    "completed"
                } else if failed.is_some() {
                    "failed"
                } else if actions.is_empty() {
                    "unchanged"
                } else {
                    "unattempted"
                }
                .into(),
                publication_generation: completed
                    .then(|| publication_generations.get(&entry.identity).copied())
                    .flatten(),
                failure: failed.and_then(|action| action.failure.clone()),
            })
        })
        .collect()
}

fn entry_actions_completed(
    plan: &MutationPlan,
    identity: &crate::observation::model::EntryIdentity,
) -> bool {
    plan.entries
        .iter()
        .find(|entry| &entry.identity == identity)
        .is_some_and(|entry| {
            !entry.action_indexes.is_empty()
                && entry.action_indexes.iter().all(|index| {
                    plan.actions.get(*index).is_some_and(|action| {
                        action.status == crate::mutation::model::ActionStatus::Completed
                    })
                })
        })
}

/// Publish one completely verified aggregate entry, then return a freshly loaded snapshot for
/// subsequent action revalidation. Source absence intentionally retires the entry's baseline.
fn publish_aggregate_entry(
    home: &ProjectPaths,
    registry: &RegistrySnapshot,
    state: &StateSnapshot,
    identity: &crate::observation::model::EntryIdentity,
    fault: Option<crate::state::publication::PublicationFault>,
) -> Result<StateSnapshot, GripError> {
    let current_registry = crate::registry::publication::load(home, false)
        .map_err(|error| error.for_mapping_operation("aggregate_force_push"))?;
    if current_registry.bytes != registry.bytes {
        return Err(stale(
            MutationOperation::AggregateForcePush,
            "accepted registry changed during aggregate execution",
        ));
    }
    crate::state::publication::revalidate(home, state)?;
    let selection = Selection::Entry(identity.clone());
    let observed =
        crate::observation::inspect(home, &current_registry, &state.accepted, &selection)
            .map_err(|error| error.for_operation("aggregate_force_push"))?;
    let observed_entry = observed.get(identity).ok_or_else(|| {
        GripError::Internal("completed aggregate entry disappeared before publication".into())
    })?;
    let record = classification::classify_accepted(observed_entry, &state.accepted);
    if record.blocking {
        return Err(stale(
            MutationOperation::AggregateForcePush,
            "completed aggregate entry became blocking before publication",
        ));
    }
    let mut next = state.accepted.clone();
    match (&record.source_complete, &record.destination_complete) {
        (Some(source), Some(destination))
            if crate::classification::complete_equivalent(source, destination) =>
        {
            next.complete_baselines
                .insert(identity.clone(), source.clone());
        }
        (None, None) => {
            next.complete_baselines.remove(identity);
        }
        _ => {
            return Err(GripError::BaselineNotAcceptable {
                records: vec![record.clone()],
            });
        }
    }
    let state_lock = crate::state::lock::project_lock_path(home, "state.lock")?;
    let _state_guard = crate::state::lock::PublicationLock::acquire(&state_lock)?;
    revalidate_registry_bytes(home, registry, MutationOperation::AggregateForcePush)?;
    let _generation = crate::state::publication::publish_complete_after_action_locked_with_fault(
        home,
        state,
        &next.complete_baselines,
        fault,
    )?;
    crate::state::publication::load(home)
}

fn metadata_fault<F>(
    fault: &mut F,
    index: usize,
    phase: crate::metadata::macos::MetadataApplyPhase,
) -> std::io::Result<()>
where
    F: FnMut(crate::mutation::FaultPhase) -> Result<(), GripError>,
{
    use crate::metadata::macos::MetadataApplyPhase;
    use crate::mutation::FaultPhase;

    let phase = match phase {
        MetadataApplyPhase::AfterProtectedFlagsCleared => {
            FaultPhase::AfterMetadataProtectedFlagsCleared(index)
        }
        MetadataApplyPhase::AfterOwnership => FaultPhase::AfterMetadataOwnership(index),
        MetadataApplyPhase::AfterAcl => FaultPhase::AfterMetadataAcl(index),
        MetadataApplyPhase::AfterExtendedAttributes => {
            FaultPhase::AfterMetadataExtendedAttributes(index)
        }
        MetadataApplyPhase::AfterPermissionMode => FaultPhase::AfterMetadataPermissionMode(index),
        MetadataApplyPhase::AfterModificationTime => {
            FaultPhase::AfterMetadataModificationTime(index)
        }
        MetadataApplyPhase::AfterBsdFlags => FaultPhase::AfterMetadataBsdFlags(index),
        MetadataApplyPhase::AfterDurability => FaultPhase::AfterMetadataDurability(index),
        MetadataApplyPhase::BeforeVerification => FaultPhase::BeforeMetadataVerification(index),
        MetadataApplyPhase::AfterVerification => FaultPhase::AfterMetadataVerification(index),
    };
    fault(phase).map_err(|error| std::io::Error::other(error.to_string()))
}

fn rebuild_plan(
    home: &ProjectPaths,
    registry: &RegistrySnapshot,
    state: &StateSnapshot,
    selection: &Selection,
    expected: &MutationPlan,
) -> Result<MutationPlan, GripError> {
    let operation = expected.operation;
    let operation_name = operation.as_str();
    let mut observed = if operation == MutationOperation::Adopt {
        let identities = match selection {
            Selection::Entries(identities) => identities,
            _ => {
                return Err(GripError::Internal(
                    "adoption plan has non-exact selection".into(),
                ));
            }
        };
        crate::observation::inspect_adoption(home, registry, &state.accepted, identities)
            .map_err(|error| error.for_operation(operation_name))?
    } else {
        crate::observation::inspect(home, registry, &state.accepted, selection)
            .map_err(|error| error.for_operation(operation_name))?
    };
    let destination_link_directory = expected.actions.iter().find_map(|action| {
        (action.kind == ActionKind::ReplaceDestinationLinkDirectory)
            .then_some(action.identity.as_ref())
            .flatten()
    });
    if let Some(identity) = destination_link_directory {
        crate::observation::model::expose_destination_link_descendants_for_replacement(
            &mut observed,
            identity,
        );
    }
    let records = observed
        .values()
        .map(|entry| {
            classification::classify_accepted_with_options(
                entry,
                &state.accepted,
                classification::ComparisonOptions {
                    use_modification_time: expected.use_modification_time,
                },
            )
        })
        .collect();
    let mut plan = match operation {
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
        MutationOperation::Adopt => {
            crate::mutation::plan::build_adopt(expected.scope.clone(), records).and_then(
                |mut plan| {
                    let identities = match selection {
                        Selection::Entries(identities) => identities,
                        _ => unreachable!("adoption selection was checked above"),
                    };
                    let target = identities
                        .last()
                        .expect("adoption selection has a target identity");
                    let decision = crate::discovery::ignore_policy::adoption_decision(
                        home.project_root()?,
                        &target.mapping.source,
                        &target.source_path(),
                    )?;
                    plan.adoption_policy_digest = Some(decision.policy_digest);
                    Ok(plan)
                },
            )
        }
        MutationOperation::Sync => crate::mutation::plan::build_sync_with_parent_requirements(
            expected.scope.clone(),
            records,
            registry.missing_destination_parents(),
        ),
        MutationOperation::Resolve => {
            let winner = expected.winner.ok_or_else(|| {
                GripError::Internal("resolution plan has no explicit winner".into())
            })?;
            if let Some(identity) = destination_link_directory {
                if winner != crate::mutation::model::ConflictWinner::Source {
                    return Err(GripError::Internal(
                        "destination-link directory plan has destination winner".into(),
                    ));
                }
                crate::mutation::plan::build_source_winner_destination_link_subtree_resolution(
                    expected.scope.clone(),
                    records,
                    identity,
                )
            } else {
                crate::mutation::plan::build_resolution(expected.scope.clone(), records, winner)
            }
        }
        MutationOperation::AggregateForcePush => crate::mutation::plan::build_aggregate_force_push(
            expected.scope.clone(),
            records,
            registry.missing_destination_parents(),
        ),
    }?;
    crate::mutation::plan::configure_modification_time(&mut plan, expected.use_modification_time)?;
    Ok(plan)
}

fn revalidate_action(
    home: &ProjectPaths,
    registry: &RegistrySnapshot,
    state: &StateSnapshot,
    action: &crate::mutation::model::MutationAction,
    context: &RevalidationContext<'_>,
) -> Result<(), GripError> {
    let operation = context.operation;
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
    if operation == MutationOperation::Adopt {
        let expected = context.expected_adoption_policy_digest.ok_or_else(|| {
            GripError::Internal("adoption action has no ignore-policy evidence".into())
        })?;
        let target = context.adoption_target.ok_or_else(|| {
            GripError::Internal("adoption action has no exact target identity".into())
        })?;
        let current = crate::discovery::ignore_policy::adoption_decision(
            home.project_root()?,
            &target.mapping.source,
            &target.source_path(),
        )?;
        if current.policy_digest != expected {
            return Err(stale(
                operation,
                "effective .gripignore policy changed before mutation action",
            ));
        }
    }
    let selected = Selection::Entry(identity.clone());
    let observed = if operation == MutationOperation::Adopt {
        crate::observation::inspect_adoption(
            home,
            &current_registry,
            &state.accepted,
            std::slice::from_ref(identity),
        )
        .map_err(|error| error.for_operation(operation_name))?
    } else {
        crate::observation::inspect(home, &current_registry, &state.accepted, &selected)
            .map_err(|error| error.for_operation(operation_name))?
    };
    if observed.values().any(|entry| {
        entry
            .unsupported
            .iter()
            .any(|reason| reason == "destination:recursive_member_topology")
    }) {
        return Err(stale(
            operation,
            "recursive member topology appeared during action revalidation",
        ));
    }
    let entry = observed.get(identity).ok_or_else(|| {
        stale(
            operation,
            "managed entry disappeared during action revalidation",
        )
    })?;
    let record = classification::classify_accepted_with_options(
        entry,
        &state.accepted,
        classification::ComparisonOptions {
            use_modification_time: context.use_modification_time,
        },
    );
    if action.expected_destination_link != record.destination_link {
        return Err(stale(
            operation,
            "destination link object changed before mutation action",
        ));
    }
    let created_directory_finalizer = action.kind == ActionKind::FinalizeDirectoryMetadata
        && action
            .metadata
            .as_ref()
            .is_some_and(|metadata| metadata.expected_before.is_none());
    let descendant_directory_finalizer = action.kind == ActionKind::FinalizeDirectoryMetadata
        && !action.dependencies.is_empty()
        && action
            .metadata
            .as_ref()
            .is_some_and(|metadata| metadata.expected_before.is_some());
    let complete_finalizer_matches = if descendant_directory_finalizer {
        action.metadata.as_ref().is_some_and(|metadata| {
            let (origin, target) = match action.direction {
                MutationDirection::Push => (
                    record.source_complete.as_ref(),
                    record.destination_complete.as_ref(),
                ),
                MutationDirection::Pull => (
                    record.destination_complete.as_ref(),
                    record.source_complete.as_ref(),
                ),
            };
            let target_matches = metadata.expected_before.as_ref().is_some_and(|expected| {
                target.is_some_and(|actual| {
                    let mut normalized = actual.clone();
                    normalized.metadata.modified_time = expected.metadata.modified_time;
                    normalized == *expected
                })
            });
            origin.is_some_and(|actual| {
                crate::classification::complete_equivalent(actual, &metadata.expected_after)
            }) && target_matches
        })
    } else {
        true
    };
    let endpoint_matches = if created_directory_finalizer {
        match action.direction {
            MutationDirection::Push => {
                record.source == action.expected_source
                    && record.destination.as_ref().is_some_and(|destination| {
                        destination.node_kind == crate::discovery::model::NodeKind::Directory
                    })
                    && action.metadata.as_ref().is_some_and(|metadata| {
                        record.source_complete.as_ref().is_some_and(|actual| {
                            crate::classification::complete_equivalent(
                                actual,
                                &metadata.expected_after,
                            )
                        })
                    })
            }
            MutationDirection::Pull => {
                record.destination == action.expected_destination
                    && record.source.as_ref().is_some_and(|source| {
                        source.node_kind == crate::discovery::model::NodeKind::Directory
                    })
                    && action.metadata.as_ref().is_some_and(|metadata| {
                        record.destination_complete.as_ref().is_some_and(|actual| {
                            crate::classification::complete_equivalent(
                                actual,
                                &metadata.expected_after,
                            )
                        })
                    })
            }
        }
    } else {
        record.source == action.expected_source && record.destination == action.expected_destination
    };
    let replacement_descendant_is_still_absent = operation == MutationOperation::Resolve
        && action.expected_destination.is_none()
        && matches!(
            action.kind,
            ActionKind::AddFile | ActionKind::CreateDirectory
        )
        && record.classification == classification::model::Classification::SourceAddition;
    let classification_matches = !matches!(
        operation,
        MutationOperation::Resolve | MutationOperation::AggregateForcePush
    ) || created_directory_finalizer
        || replacement_descendant_is_still_absent
        || (operation == MutationOperation::Resolve
            && crate::mutation::plan::resolution_is_actionable(
                match action.direction {
                    MutationDirection::Push => crate::mutation::model::ConflictWinner::Source,
                    MutationDirection::Pull => crate::mutation::model::ConflictWinner::Destination,
                },
                record.classification,
            ))
        || (operation == MutationOperation::AggregateForcePush
            && crate::mutation::plan::aggregate_force_is_actionable(record.classification));
    if record.blocking
        && !matches!(
            operation,
            MutationOperation::Resolve | MutationOperation::AggregateForcePush
        )
        && !created_directory_finalizer
        && !descendant_directory_finalizer
        || !classification_matches
        || !endpoint_matches
        || !complete_finalizer_matches
    {
        return Err(stale(
            operation,
            "managed entry changed before mutation action",
        ));
    }
    Ok(())
}

fn evidence(action: &crate::mutation::model::MutationAction) -> ActionCheckpointEvidenceV2 {
    ActionCheckpointEvidenceV2 {
        revalidation: action.milestones.revalidation.clone(),
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
            MutationOperation::Adopt => "stale_adopt_evidence",
            MutationOperation::Sync => "stale_sync_evidence",
            MutationOperation::Resolve => "stale_resolution_evidence",
            MutationOperation::AggregateForcePush => "stale_aggregate_force_push_evidence",
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
    home: &ProjectPaths,
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
        MutationOperation::Pull
        | MutationOperation::Adopt
        | MutationOperation::Sync
        | MutationOperation::Resolve
        | MutationOperation::AggregateForcePush => GripError::MutationFailed(Box::new(failure)),
    }
}

fn observe_action_state(
    home: &ProjectPaths,
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
