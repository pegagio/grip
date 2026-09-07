mod support;
use std::fs;

#[test]
fn changed_surviving_peer_blocks_before_recovery_removal_or_state_retirement() {
    let (root, grip_home, source, destination) = support::accepted_file_fixture();
    fs::remove_file(&source).unwrap();
    fs::write(&destination, "changed").unwrap();
    let before = support::snapshot(root.path());
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["delete", "--source", source.to_str().unwrap()],
    );
    assert!(!output.status.success());
    support::assert_snapshot_unchanged(&before, root.path());
}

#[test]
fn injected_preservation_and_unlink_failures_leave_baseline_authoritative() {
    for fault in [
        grip::delete::execution::DeletionFault::Preservation,
        grip::delete::execution::DeletionFault::Unlink,
    ] {
        let (_root, home, registry, state, selection, plan) = support::deletion_execution_fixture();
        let generation = state.accepted.generation;
        let result = grip::delete::execution::execute_with_fault(
            &home,
            &registry,
            &state,
            &selection,
            &plan,
            Some(fault),
        );
        assert!(result.is_err());
        assert_eq!(
            grip::state::publication::load(&home)
                .unwrap()
                .accepted
                .generation,
            generation
        );
    }
}

#[test]
fn post_removal_failures_preserve_visible_effect_without_retiring_baseline() {
    for fault in [
        grip::delete::execution::DeletionFault::DirectorySync,
        grip::delete::execution::DeletionFault::AbsenceVerification,
        grip::delete::execution::DeletionFault::FinalObservation,
        grip::delete::execution::DeletionFault::StatePublication,
    ] {
        let (_root, home, registry, state, selection, plan) = support::deletion_execution_fixture();
        let target = plan.actions[0].target.clone();
        let generation = state.accepted.generation;
        let result = grip::delete::execution::execute_with_fault(
            &home,
            &registry,
            &state,
            &selection,
            &plan,
            Some(fault),
        );
        assert!(result.is_err());
        assert!(!target.exists());
        assert_eq!(
            grip::state::publication::load(&home)
                .unwrap()
                .accepted
                .generation,
            generation
        );
    }
}

#[test]
fn deletion_result_delivery_failure_does_not_revoke_visible_payload_or_state() {
    let (_root, home, registry, state, selection, plan) = support::deletion_execution_fixture();
    let target = plan.actions[0].target.clone();
    let result =
        grip::delete::execution::execute(&home, &registry, &state, &selection, &plan).unwrap();
    let operation = result.operation_record.clone().unwrap();
    let outcome = grip::CommandOutcome::deletion(
        &result.plan,
        &result.mode,
        result.operation_record.as_deref(),
        result.baseline,
    );
    assert!(
        grip::render_command_result(
            outcome,
            grip::result::OutputMode::Json,
            &mut support::FailingWriter,
            Some(&home),
        )
        .is_err()
    );
    assert!(!target.exists());
    let summary =
        support::read_operation_component::<grip::operation::model::OperationSummaryPayloadV1>(
            &home
                .path()
                .join("state/operations")
                .join(operation)
                .join("operation.json"),
        );
    assert_eq!(summary.payload.result_delivery, "failed");
}

#[test]
fn deletion_failure_human_and_json_results_expose_operation_actions_and_authority() {
    let (_root, home, registry, state, selection, plan) = support::deletion_execution_fixture();
    let error = grip::delete::execution::execute_with_fault(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        Some(grip::delete::execution::DeletionFault::Unlink),
    )
    .unwrap_err();
    let json_outcome = grip::CommandOutcome::failure(&error);
    let human_outcome = grip::CommandOutcome::failure(&error);
    let mut json = Vec::new();
    let mut human = Vec::new();
    grip::result::render(json_outcome, grip::result::OutputMode::Json, &mut json).unwrap();
    grip::result::render(human_outcome, grip::result::OutputMode::Human, &mut human).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&json).unwrap();
    assert_eq!(value["details"]["operation"], "delete");
    assert_eq!(value["details"]["actions"][0]["status"], "failed");
    assert!(value["details"]["operation_record"]["id"].is_string());
    assert_eq!(
        value["details"]["baseline"]["authoritative_generation"],
        state.accepted.generation.unwrap()
    );
    let human = String::from_utf8(human).unwrap();
    assert!(human.contains("failed"));
    assert!(human.contains("Operation record"));
    assert!(human.contains("Accepted state"));
}
