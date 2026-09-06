#![allow(clippy::result_large_err)]

mod support;

use grip::classification;
use grip::classification::model::ClassificationScope;
use grip::observation::model::{PathSpace, Selection};
use grip::operation::model::{ActionCheckpointPayloadV1, OperationSummaryPayloadV1};
use grip::push::FaultPhase;
use std::fs;

fn fixture() -> (
    tempfile::TempDir,
    grip::home::GripHome,
    grip::registry::publication::RegistrySnapshot,
    grip::state::publication::StateSnapshot,
    Selection,
    grip::push::model::PushPlan,
) {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "payload").unwrap();
    support::write_registry(&grip_home, &[("file", &source, &destination)]);
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    let registry = grip::registry::publication::load(&home, false).unwrap();
    let state = grip::state::publication::load(&home).unwrap();
    let selection = Selection::All;
    let observed =
        grip::observation::inspect(&home, &registry, &state.accepted, &selection).unwrap();
    let records = observed
        .values()
        .map(|entry| classification::classify(entry, state.accepted.baselines.get(&entry.identity)))
        .collect();
    let plan = grip::push::plan::build_with_parent_requirements(
        ClassificationScope {
            kind: "all".into(),
            path_space: PathSpace::Source,
            selector: None,
            mapping_source: None,
        },
        records,
        registry.missing_destination_parents(),
    )
    .unwrap();
    (root, home, registry, state, selection, plan)
}

fn replacement_fixture() -> (
    tempfile::TempDir,
    grip::home::GripHome,
    grip::registry::publication::RegistrySnapshot,
    grip::state::publication::StateSnapshot,
    Selection,
    grip::push::model::PushPlan,
) {
    let (root, home, registry, state, selection, plan) = fixture();
    grip::push::execution::execute(&home, &registry, &state, &selection, &plan).unwrap();
    fs::write(root.path().join("source"), "replacement").unwrap();
    let registry = grip::registry::publication::load(&home, false).unwrap();
    let state = grip::state::publication::load(&home).unwrap();
    let observed =
        grip::observation::inspect(&home, &registry, &state.accepted, &selection).unwrap();
    let records = observed
        .values()
        .map(|entry| classification::classify(entry, state.accepted.baselines.get(&entry.identity)))
        .collect();
    let plan = grip::push::plan::build_with_parent_requirements(
        ClassificationScope {
            kind: "all".into(),
            path_space: PathSpace::Source,
            selector: None,
            mapping_source: None,
        },
        records,
        registry.missing_destination_parents(),
    )
    .unwrap();
    (root, home, registry, state, selection, plan)
}

#[test]
fn staging_failure_is_terminal_and_leaves_later_actions_unattempted() {
    let (root, home, registry, state, selection, plan) = fixture();
    let destination = root.path().join("destination");
    let error = grip::push::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        support::fail_push_at(FaultPhase::AfterStaging(0)),
    )
    .unwrap_err();
    let grip::GripError::PushFailed(failure) = error else {
        panic!("expected structured push failure");
    };
    assert_eq!(failure.plan.counts.failed, 1);
    assert_eq!(failure.plan.counts.completed, 0);
    assert!(!destination.exists());
    let operation = home
        .path()
        .join("state/operations")
        .join(&failure.operation_id);
    let summary = support::read_operation_component::<OperationSummaryPayloadV1>(
        &operation.join("operation.json"),
    );
    assert_eq!(summary.payload.state, "failed");
    assert_eq!(summary.payload.result_delivery, "prepared");
    let action = support::read_operation_component::<ActionCheckpointPayloadV1>(
        &operation.join("actions/00000000.json"),
    );
    assert_eq!(action.payload.status, "failed");
    assert_eq!(action.payload.milestones.staging, "verified");
}

#[test]
fn post_publication_failure_preserves_visible_payload_without_publishing_baseline() {
    let (root, home, registry, state, selection, plan) = fixture();
    let destination = root.path().join("destination");
    let error = grip::push::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        support::fail_push_at(FaultPhase::AfterPayloadPublication(0)),
    )
    .unwrap_err();
    let grip::GripError::PushFailed(failure) = error else {
        panic!("expected structured push failure");
    };
    assert_eq!(fs::read_to_string(destination).unwrap(), "payload");
    assert!(failure.plan.actions[0].milestones.durability_confirmed);
    assert_eq!(failure.baseline.outcome, "not_published");
    assert!(
        grip::state::publication::load(&home)
            .unwrap()
            .accepted
            .generation
            .is_none()
    );
}

#[test]
fn accepted_state_faults_report_the_actual_authoritative_generation() {
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
        let (_root, home, registry, state, selection, plan) = fixture();
        let error = grip::push::execution::execute_with_faults(
            &home,
            &registry,
            &state,
            &selection,
            &plan,
            |_| Ok(()),
            Some(fault),
        )
        .unwrap_err();
        let grip::GripError::PushFailed(failure) = error else {
            panic!("expected structured push failure");
        };
        assert_eq!(failure.baseline.publication_visible, visible);
        assert_eq!(failure.baseline.durability_confirmed, !visible);
        assert_eq!(
            failure.baseline.authoritative_generation,
            visible.then_some(0)
        );
        assert_eq!(
            grip::state::publication::load(&home)
                .unwrap()
                .accepted
                .generation,
            visible.then_some(0)
        );
    }
}

#[test]
fn addition_fault_matrix_never_publishes_a_partial_baseline() {
    for phase in [
        FaultPhase::BeforeStaging(0),
        FaultPhase::AfterStaging(0),
        FaultPhase::BeforePayloadPublication(0),
        FaultPhase::AfterPayloadPublication(0),
        FaultPhase::BeforeDestinationVerification(0),
        FaultPhase::BeforeBaselinePublication,
    ] {
        let (_root, home, registry, state, selection, plan) = fixture();
        let result = grip::push::execution::execute_with_fault_hook(
            &home,
            &registry,
            &state,
            &selection,
            &plan,
            support::fail_push_at(phase),
        );
        assert!(result.is_err(), "phase {phase:?} must fail");
        assert!(
            grip::state::publication::load(&home)
                .unwrap()
                .accepted
                .generation
                .is_none()
        );
    }
}

#[test]
fn replacement_recovery_faults_preserve_prior_baseline_and_payload_evidence() {
    for phase in [FaultPhase::BeforeRecovery(0), FaultPhase::AfterRecovery(0)] {
        let (root, home, registry, state, selection, plan) = replacement_fixture();
        let result = grip::push::execution::execute_with_fault_hook(
            &home,
            &registry,
            &state,
            &selection,
            &plan,
            support::fail_push_at(phase),
        );
        let grip::GripError::PushFailed(failure) = result.unwrap_err() else {
            panic!("expected structured recovery failure");
        };
        assert_eq!(
            fs::read_to_string(root.path().join("destination")).unwrap(),
            "payload"
        );
        assert_eq!(
            grip::state::publication::load(&home)
                .unwrap()
                .accepted
                .generation,
            Some(0)
        );
        if phase == FaultPhase::AfterRecovery(0) {
            assert!(failure.plan.actions[0].milestones.recovery_ref.is_some());
        }
    }
}

#[test]
fn post_action_faults_are_structured_and_preserve_the_last_authoritative_baseline() {
    for (phase, reason, published) in [
        (
            FaultPhase::BeforeFinalObservation,
            "verification_failure",
            false,
        ),
        (
            FaultPhase::BeforeFinalCoordination,
            "coordination_failure",
            false,
        ),
        (
            FaultPhase::BeforeTerminalSummary,
            "baseline_publication_failure",
            true,
        ),
    ] {
        let (_root, home, registry, state, selection, plan) = fixture();
        let error = grip::push::execution::execute_with_fault_hook(
            &home,
            &registry,
            &state,
            &selection,
            &plan,
            support::fail_push_at(phase),
        )
        .unwrap_err();
        let grip::GripError::PushFailed(failure) = error else {
            panic!("expected structured post-action failure");
        };
        assert_eq!(failure.reason, reason);
        assert_eq!(failure.plan.counts.completed, failure.plan.actions.len());
        assert_eq!(failure.baseline.publication_visible, published);
        assert_eq!(
            grip::state::publication::load(&home)
                .unwrap()
                .accepted
                .generation
                .is_some(),
            published
        );
    }
}
