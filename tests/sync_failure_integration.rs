mod support;

use std::fs;

use grip::operation::model::{ActionCheckpointPayloadV2, OperationSummaryPayloadV2};

#[test]
fn sync_failure_stops_before_publication_and_preserves_baseline_authority() {
    let (root, home, registry, state, selection, plan) = support::sync_execution_fixture();
    let destination = root.path().join("destination");
    let before_state = fs::read(home.path().join("state/state.json")).unwrap();
    let error = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        support::fail_mutation_at(grip::mutation::FaultPhase::BeforeStaging(0)),
    )
    .unwrap_err();
    let grip::GripError::MutationFailed(failure) = error else {
        panic!("expected journaled sync failure");
    };
    assert_eq!(
        failure.plan.operation,
        grip::mutation::model::MutationOperation::Sync
    );
    assert_eq!(failure.reason, "staging_failure");
    assert_eq!(
        failure.plan.actions[0].direction,
        grip::mutation::model::MutationDirection::Push
    );
    assert_eq!(failure.baseline.outcome, "not_published");
    assert_eq!(fs::read_to_string(destination).unwrap(), "accepted");
    assert_eq!(
        fs::read(home.path().join("state/state.json")).unwrap(),
        before_state
    );
}

#[test]
#[allow(clippy::result_large_err)]
fn stale_sync_evidence_is_rejected_before_the_action() {
    let (root, home, registry, state, selection, plan) = support::sync_execution_fixture();
    let source = root.path().join("source");
    let error = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        |phase| {
            if phase == grip::mutation::FaultPhase::BeforeActionRevalidation(0) {
                fs::write(&source, "concurrent change").unwrap();
            }
            Ok(())
        },
    )
    .unwrap_err();
    let grip::GripError::MutationFailed(failure) = error else {
        panic!("expected stale sync failure");
    };
    assert_eq!(failure.reason, "stale_sync_evidence");
    assert_eq!(failure.failed_action_index, Some(0));
}

#[test]
fn first_mixed_sync_failure_leaves_later_actions_unattempted_and_baseline_unchanged() {
    let (root, home, registry, state, selection, plan) = support::mixed_sync_execution_fixture();
    let before_state = fs::read(home.path().join("state/state.json")).unwrap();
    assert_eq!(plan.actions.len(), 2);
    assert_ne!(plan.actions[0].direction, plan.actions[1].direction);
    let error = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        support::fail_mutation_at(grip::mutation::FaultPhase::BeforeStaging(1)),
    )
    .unwrap_err();
    let grip::GripError::MutationFailed(failure) = error else {
        panic!("expected mixed sync failure");
    };
    assert_eq!(failure.completion, "partial");
    assert_eq!(failure.plan.counts.completed, 1);
    assert_eq!(failure.plan.counts.failed, 1);
    assert_eq!(failure.plan.counts.unattempted, 0);
    assert_eq!(failure.baseline.outcome, "not_published");
    assert_eq!(
        fs::read(home.path().join("state/state.json")).unwrap(),
        before_state
    );
    assert_eq!(
        fs::read_to_string(root.path().join("destination-a")).unwrap(),
        "source change"
    );
    assert_eq!(
        fs::read_to_string(root.path().join("source-b")).unwrap(),
        "accepted-b"
    );
}

#[test]
fn sync_fault_matrix_records_every_pipeline_boundary_truthfully() {
    use grip::mutation::FaultPhase;
    let phases = [
        FaultPhase::BeforeActionRevalidation(0),
        FaultPhase::BeforeRecovery(0),
        FaultPhase::AfterRecovery(0),
        FaultPhase::BeforeStaging(0),
        FaultPhase::AfterStaging(0),
        FaultPhase::BeforePayloadPublication(0),
        FaultPhase::AfterMetadataProtectedFlagsCleared(0),
        FaultPhase::AfterMetadataOwnership(0),
        FaultPhase::AfterMetadataAcl(0),
        FaultPhase::AfterMetadataExtendedAttributes(0),
        FaultPhase::AfterMetadataPermissionMode(0),
        FaultPhase::AfterMetadataModificationTime(0),
        FaultPhase::AfterMetadataBsdFlags(0),
        FaultPhase::AfterMetadataDurability(0),
        FaultPhase::BeforeMetadataVerification(0),
        FaultPhase::AfterMetadataVerification(0),
        FaultPhase::AfterPayloadPublication(0),
        FaultPhase::BeforeTargetVerification(0),
        FaultPhase::BeforeDestinationVerification(0),
        FaultPhase::AfterTargetVerification(0),
        FaultPhase::AfterStagingCleanup(0),
        FaultPhase::BeforeFinalObservation,
        FaultPhase::BeforeFinalCoordination,
        FaultPhase::BeforeBaselinePublication,
        FaultPhase::AfterBaselinePublication,
        FaultPhase::BeforeTerminalSummary,
    ];
    for phase in phases {
        let (_root, home, registry, state, selection, plan) = support::sync_execution_fixture();
        let result = grip::mutation::execution::execute_with_fault_hook(
            &home,
            &registry,
            &state,
            &selection,
            &plan,
            support::fail_mutation_at(phase),
        );
        let grip::GripError::MutationFailed(failure) = result.unwrap_err() else {
            panic!("phase {phase:?} must produce a journaled sync failure");
        };
        assert_eq!(
            failure.plan.operation,
            grip::mutation::model::MutationOperation::Sync
        );
        assert_eq!(
            failure.plan.counts.completed + failure.plan.counts.failed,
            1
        );
        assert_eq!(failure.plan.counts.unattempted, 0);
        let operation_directory = home
            .path()
            .join("state/operations")
            .join(&failure.operation_id);
        let summary = support::read_operation_component::<OperationSummaryPayloadV2>(
            &operation_directory.join("operation.json"),
        );
        assert_eq!(summary.payload.operation, "sync");
        assert_eq!(summary.payload.state, "failed");
        assert_eq!(
            summary.payload.baseline["publication_visible"],
            failure.baseline.publication_visible
        );
        let action = support::read_operation_component::<ActionCheckpointPayloadV2>(
            &operation_directory.join("actions/00000000.json"),
        );
        assert!(matches!(
            action.payload.status.as_str(),
            "completed" | "failed"
        ));
        if phase == FaultPhase::AfterTargetVerification(0) {
            assert_eq!(action.payload.milestones.verification, "verified");
        }
        if matches!(
            phase,
            FaultPhase::AfterBaselinePublication | FaultPhase::BeforeTerminalSummary
        ) {
            assert!(failure.baseline.publication_visible);
            assert!(failure.baseline.durability_confirmed);
            assert!(failure.baseline.authoritative_generation.is_some());
        } else {
            assert!(!failure.baseline.publication_visible);
            assert_eq!(
                failure.baseline.authoritative_generation,
                state.accepted.generation
            );
        }
    }
}

#[test]
fn sync_fault_after_outer_lock_fails_before_operation_or_payload_mutation() {
    let (root, home, registry, state, selection, plan) = support::sync_execution_fixture();
    let operations = home.path().join("state/operations");
    let operation_count = fs::read_dir(&operations).unwrap().count();
    let destination_before = fs::read(root.path().join("destination")).unwrap();
    let result = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        support::fail_mutation_at(grip::mutation::FaultPhase::AfterMutationLock),
    );
    assert!(matches!(result, Err(grip::GripError::Internal(_))));
    assert_eq!(fs::read_dir(operations).unwrap().count(), operation_count);
    assert_eq!(
        fs::read(root.path().join("destination")).unwrap(),
        destination_before
    );
}

#[test]
#[allow(clippy::result_large_err)]
fn sync_state_publication_faults_report_baseline_visibility_and_authority() {
    for (fault, visible) in [
        (
            grip::state::publication::PublicationFault::BeforeV2StateRename,
            false,
        ),
        (
            grip::state::publication::PublicationFault::AfterV2StateRename,
            true,
        ),
    ] {
        let (_root, home, registry, state, selection, plan) = support::sync_execution_fixture();
        let result = grip::mutation::execution::execute_with_faults(
            &home,
            &registry,
            &state,
            &selection,
            &plan,
            |_| Ok(()),
            Some(fault),
        );
        let grip::GripError::MutationFailed(failure) = result.unwrap_err() else {
            panic!("accepted-state fault must produce a sync failure");
        };
        assert_eq!(failure.reason, "baseline_publication_failure");
        assert_eq!(failure.baseline.publication_visible, visible);
        assert_eq!(
            failure.baseline.authoritative_generation,
            if visible {
                state
                    .accepted
                    .generation
                    .map_or(Some(0), |value| value.checked_add(1))
            } else {
                state.accepted.generation
            }
        );
    }
}
