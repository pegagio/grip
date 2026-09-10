mod support;

use grip::operation::model::{ActionCheckpointPayloadV2, OperationSummaryPayloadV2};

#[test]
fn operation_record_v2_round_trips_portable_actions_and_diagnostic_paths() {
    let identity = grip::state::EntryIdentityV4 {
        mapping: grip::mapping::PortableMapping::parse(
            grip::mapping::MappingKind::File,
            std::ffi::OsStr::new("source"),
            std::ffi::OsStr::new("~/destination"),
        )
        .unwrap(),
        relative_path_hex: String::new(),
    };
    let record = grip::operation::model::OperationRecordV2::new(
        grip::operation::model::OperationRecordPayloadV2 {
            operation_id: "push-1".into(),
            operation: "push".into(),
            plan_id: "a".repeat(64),
            actions: vec![grip::operation::model::PortableActionV2 {
                index: 0,
                identity: Some(identity),
                endpoint_role: grip::state::EndpointRoleV1::Destination,
                diagnostic_path: Some(
                    grip::discovery::model::SafePath::from_path(std::path::Path::new(
                        "/diagnostic/only",
                    ))
                    .into(),
                ),
            }],
        },
    )
    .unwrap();
    let bytes = grip::operation::model::encode_operation_v2(&record).unwrap();
    assert_eq!(
        grip::operation::model::decode_operation_v2(&bytes).unwrap(),
        record
    );
}

#[test]
fn operation_record_v2_rejects_v1_unknown_fields_and_tampering() {
    assert_eq!(
        grip::operation::model::decode_operation_v2(br#"{"schema_version":1}"#)
            .unwrap_err()
            .category(),
        grip::ResultCategory::UnsupportedSchema
    );
    let payload = grip::operation::model::OperationRecordPayloadV2 {
        operation_id: "push-1".into(),
        operation: "push".into(),
        plan_id: "a".repeat(64),
        actions: Vec::new(),
    };
    let record = grip::operation::model::OperationRecordV2::new(payload).unwrap();
    let mut unknown = serde_json::to_value(&record).unwrap();
    unknown["unknown"] = true.into();
    assert!(
        grip::operation::model::decode_operation_v2(&serde_json::to_vec(&unknown).unwrap())
            .is_err()
    );
    let mut tampered = record;
    tampered.payload.operation = "pull".into();
    assert!(tampered.validate().is_err());
}

#[test]
fn operation_record_v2_accepts_all_mutation_operations() {
    for operation in ["push", "pull"] {
        let mut plan = support::test_push_plan(0);
        plan.operation = if operation == "push" {
            grip::mutation::model::MutationOperation::Push
        } else {
            grip::mutation::model::MutationOperation::Pull
        };
        plan.direction = Some(if operation == "push" {
            grip::mutation::model::MutationDirection::Push
        } else {
            grip::mutation::model::MutationDirection::Pull
        });
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let metadata_dir = support::initialize_project_metadata(root.path());
        let home = support::project_home(&metadata_dir);
        let receipt = grip::operation::publication::initialize(&home, &plan).unwrap();
        assert!(receipt.operation_id().starts_with(&format!("{operation}-")));
        let stored = grip::operation::model::decode_operation_v2(
            &std::fs::read(receipt.directory().join("record.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(stored.payload.operation, operation);
    }
}

#[test]
fn feature_eight_nonterminal_records_are_immutable_and_do_not_block_fresh_operations() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let home = support::project_home(&metadata_dir);
    let operation = "delete";
    let plan_id = "a".repeat(64);
    let plan = serde_json::json!({
        "operation": operation,
        "plan_id": plan_id,
        "actions": [{"index":0}]
    });
    let first =
        grip::operation::publication::initialize_typed(&home, operation, &plan_id, &plan, 1)
            .unwrap();
    let before = support::snapshot(first.directory());
    let second =
        grip::operation::publication::initialize_typed(&home, operation, &plan_id, &plan, 1)
            .unwrap();
    assert_ne!(first.operation_id(), second.operation_id());
    assert_eq!(support::snapshot(first.directory()), before);
}

#[test]
fn sync_operation_record_carries_operation_and_action_direction() {
    let (_root, home, registry, state, selection, plan) = support::sync_execution_fixture();
    let success =
        grip::mutation::execution::execute(&home, &registry, &state, &selection, &plan).unwrap();
    let stored = grip::operation::model::decode_operation_v2(
        &std::fs::read(
            home.path()
                .join("state/operations")
                .join(&success.operation_id)
                .join("record.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(stored.payload.operation, "sync");
    assert_eq!(
        stored.payload.actions[0].endpoint_role,
        grip::state::EndpointRoleV1::Destination
    );
}

#[test]
fn pull_result_delivery_finalizes_only_its_operation_record() {
    let (_root, home, registry, state, selection, plan) = support::pull_execution_fixture();
    let success =
        grip::mutation::execution::execute(&home, &registry, &state, &selection, &plan).unwrap();
    let completed = support::read_operation_component::<OperationSummaryPayloadV2>(
        &home
            .path()
            .join("state/operations")
            .join(&success.operation_id)
            .join("operation.json"),
    );
    assert_eq!(completed.payload.operation, "pull");
    assert_eq!(completed.payload.state, "completed");
    assert_eq!(completed.payload.result_delivery, "prepared");
    let action = support::read_operation_component::<ActionCheckpointPayloadV2>(
        &home
            .path()
            .join("state/operations")
            .join(&success.operation_id)
            .join("actions/00000000.json"),
    );
    assert_eq!(action.payload.status, "completed");
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
    let summary = support::read_operation_component::<OperationSummaryPayloadV2>(
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
        support::fail_mutation_at(grip::mutation::FaultPhase::BeforeStaging(0)),
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

#[test]
fn interrupted_and_failed_sync_records_remain_immutable_during_fresh_retries() {
    let (_root, home, registry, state, selection, plan) = support::sync_execution_fixture();
    let interrupted = grip::operation::publication::initialize(&home, &plan).unwrap();
    let interrupted_directory = interrupted.directory().to_path_buf();
    let interrupted_before = support::snapshot(&interrupted_directory);
    drop(interrupted);

    let failure = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        support::fail_mutation_at(grip::mutation::FaultPhase::BeforeStaging(0)),
    )
    .unwrap_err();
    let grip::GripError::MutationFailed(failure) = failure else {
        panic!("expected sync failure");
    };
    let failed_directory = home
        .path()
        .join("state/operations")
        .join(&failure.operation_id);
    let failed_before = support::snapshot(&failed_directory);

    let success =
        grip::mutation::execution::execute(&home, &registry, &state, &selection, &plan).unwrap();
    assert_ne!(success.operation_id, failure.operation_id);
    assert_eq!(
        support::snapshot(&interrupted_directory),
        interrupted_before
    );
    assert_eq!(support::snapshot(&failed_directory), failed_before);
}

#[test]
fn interrupted_resolution_record_does_not_block_a_fresh_resolution() {
    let (_root, home, registry, state, selection, plan) =
        support::resolution_execution_fixture(grip::mutation::model::ConflictWinner::Destination);
    let interrupted = grip::operation::publication::initialize(&home, &plan).unwrap();
    let interrupted_directory = interrupted.directory().to_path_buf();
    let before = support::snapshot(&interrupted_directory);
    drop(interrupted);
    let success =
        grip::mutation::execution::execute(&home, &registry, &state, &selection, &plan).unwrap();
    assert!(success.operation_id.starts_with("resolve-"));
    assert_eq!(support::snapshot(&interrupted_directory), before);
}

#[test]
fn mixed_sync_checkpoints_each_direction_and_terminal_baseline_once() {
    let (_root, home, registry, state, selection, plan) = support::mixed_sync_execution_fixture();
    let success =
        grip::mutation::execution::execute(&home, &registry, &state, &selection, &plan).unwrap();
    let directory = home
        .path()
        .join("state/operations")
        .join(&success.operation_id);
    let stored_plan = grip::operation::model::decode_operation_v2(
        &std::fs::read(directory.join("record.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        stored_plan.payload.actions[0].endpoint_role,
        grip::state::EndpointRoleV1::Destination
    );
    assert_eq!(
        stored_plan.payload.actions[1].endpoint_role,
        grip::state::EndpointRoleV1::Source
    );
    for index in 0..2 {
        let action = support::read_operation_component::<ActionCheckpointPayloadV2>(
            &directory.join(format!("actions/{index:08}.json")),
        );
        assert_eq!(action.payload.status, "completed");
        assert_eq!(action.payload.milestones.revalidation, "passed");
        assert_eq!(action.payload.milestones.publication, "visible");
        assert_eq!(action.payload.milestones.verification, "verified");
        assert!(action.payload.milestones.durability_confirmed);
    }
    let summary = support::read_operation_component::<OperationSummaryPayloadV2>(
        &directory.join("operation.json"),
    );
    assert_eq!(summary.payload.state, "completed");
    assert_eq!(summary.payload.baseline["outcome"], "published");
    assert_eq!(
        summary.payload.baseline["published_generation"],
        success.generation
    );
}

#[test]
fn failed_resolution_record_is_immutable_and_does_not_block_fresh_resolution() {
    let (_root, home, registry, state, selection, plan) =
        support::resolution_execution_fixture(grip::mutation::model::ConflictWinner::Source);
    let result = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        support::fail_mutation_at(grip::mutation::FaultPhase::BeforeStaging(0)),
    );
    let grip::GripError::MutationFailed(failure) = result.unwrap_err() else {
        panic!("expected resolution failure");
    };
    let failed_directory = home
        .path()
        .join("state/operations")
        .join(&failure.operation_id);
    let failed_before = support::snapshot(&failed_directory);
    let success =
        grip::mutation::execution::execute(&home, &registry, &state, &selection, &plan).unwrap();
    assert!(success.operation_id.starts_with("resolve-"));
    assert_ne!(success.operation_id, failure.operation_id);
    assert_eq!(support::snapshot(&failed_directory), failed_before);
}
