mod support;

use grip::operation::model::{
    ActionCheckpointPayloadV1, OperationPlanPayloadV1, OperationSummaryPayloadV1,
};

#[test]
fn operation_record_v1_accepts_push_and_pull_but_rejects_unknown_operations() {
    for operation in ["push", "pull"] {
        let mut plan = support::test_push_plan(0);
        plan.direction = if operation == "push" {
            grip::mutation::model::MutationDirection::Push
        } else {
            grip::mutation::model::MutationDirection::Pull
        };
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let grip_home = support::minimal_home(root.path());
        let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
        let receipt = grip::operation::publication::initialize(&home, &plan).unwrap();
        assert!(receipt.operation_id().starts_with(&format!("{operation}-")));
        let stored = support::read_operation_component::<OperationPlanPayloadV1>(
            &receipt.directory().join("plan.json"),
        );
        assert_eq!(stored.payload.operation, operation);
    }

    let payload = OperationSummaryPayloadV1 {
        operation_id: "unknown-1".into(),
        operation: "sync".into(),
        state: "executing".into(),
        plan_id: "a".repeat(64),
        plan_ref: "plan.json".into(),
        baseline: serde_json::json!({"outcome":"not_attempted"}),
        result_delivery: "not_attempted".into(),
        failure: None,
    };
    assert!(grip::operation::model::OperationSummaryEnvelopeV1::new(payload).is_err());

    let mismatch = OperationPlanPayloadV1 {
        operation_id: "pull-1".into(),
        operation: "pull".into(),
        plan_id: "a".repeat(64),
        plan: serde_json::json!({
            "direction":"push",
            "plan_id":"a".repeat(64),
            "actions":[]
        }),
    };
    assert!(grip::operation::model::OperationPlanEnvelopeV1::new(mismatch).is_err());
}

#[test]
fn pull_result_delivery_finalizes_only_its_operation_record() {
    let (_root, home, registry, state, selection, plan) = support::pull_execution_fixture();
    let success =
        grip::mutation::execution::execute(&home, &registry, &state, &selection, &plan).unwrap();
    let completed = support::read_operation_component::<OperationSummaryPayloadV1>(
        &home
            .path()
            .join("state/operations")
            .join(&success.operation_id)
            .join("operation.json"),
    );
    assert_eq!(completed.payload.operation, "pull");
    assert_eq!(completed.payload.state, "completed");
    assert_eq!(completed.payload.result_delivery, "prepared");
    let action = support::read_operation_component::<ActionCheckpointPayloadV1>(
        &home
            .path()
            .join("state/operations")
            .join(&success.operation_id)
            .join("actions/00000000.json"),
    );
    assert_eq!(action.payload.status, "completed");
    assert_eq!(action.payload.milestones.recovery, "preserved");
    assert_eq!(action.payload.milestones.publication, "visible");
    assert_eq!(action.payload.milestones.verification, "verified");
    assert!(action.payload.milestones.durability_confirmed);
    let outcome = grip::CommandOutcome::mutation_applied(&success);
    assert!(
        grip::render_command_result(
            outcome,
            grip::result::OutputMode::Json,
            &mut support::FailingWriter,
            Some(&home),
        )
        .is_err()
    );
    let summary = support::read_operation_component::<OperationSummaryPayloadV1>(
        &home
            .path()
            .join("state/operations")
            .join(&success.operation_id)
            .join("operation.json"),
    );
    assert_eq!(summary.payload.operation, "pull");
    assert_eq!(summary.payload.state, "completed");
    assert_eq!(summary.payload.result_delivery, "failed");
}

#[test]
fn nonterminal_pull_record_is_immutable_and_does_not_block_a_fresh_pull() {
    let (_root, home, registry, state, selection, plan) = support::pull_execution_fixture();
    let interrupted = grip::operation::publication::initialize(&home, &plan).unwrap();
    let interrupted_path = interrupted.directory().join("operation.json");
    let before = std::fs::read(&interrupted_path).unwrap();
    drop(interrupted);

    let success =
        grip::mutation::execution::execute(&home, &registry, &state, &selection, &plan).unwrap();
    assert_ne!(
        success.operation_id,
        interrupted_path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
    );
    assert_eq!(std::fs::read(interrupted_path).unwrap(), before);
}

#[test]
fn failed_pull_record_is_immutable_and_does_not_block_a_fresh_pull() {
    let (_root, home, registry, state, selection, plan) = support::pull_execution_fixture();
    let error = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        support::fail_mutation_at(grip::mutation::FaultPhase::AfterRecovery(0)),
    )
    .unwrap_err();
    let grip::GripError::MutationFailed(failure) = error else {
        panic!("expected direction-neutral pull failure");
    };
    let failed_directory = home
        .path()
        .join("state/operations")
        .join(&failure.operation_id);
    let before = support::snapshot(&failed_directory);

    let success =
        grip::mutation::execution::execute(&home, &registry, &state, &selection, &plan).unwrap();
    assert_ne!(success.operation_id, failure.operation_id);
    assert_eq!(support::snapshot(&failed_directory), before);
}
