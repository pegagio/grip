mod support;

#[test]
fn confirmed_cleanup_removes_only_bytes_and_publishes_tombstone() {
    let (root, metadata_dir, _, _, reference) = support::payload_recovery_fixture();
    let accepted_registry = std::fs::read(metadata_dir.join("config.toml")).unwrap();
    let accepted_state = std::fs::read(metadata_dir.join("state/state.json")).unwrap();
    let preview = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output=json",
            "recovery",
            "remove",
            "-n",
            "--confirm",
            &reference,
        ],
    );
    assert!(preview.status.success());
    let execute = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output=json",
            "recovery",
            "remove",
            "--confirm",
            &reference,
        ],
    );
    assert!(
        execute.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&execute.stdout),
        String::from_utf8_lossy(&execute.stderr)
    );
    let operation = support::json(&execute)["details"]["operation_record"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let summary =
        support::read_operation_component::<grip::operation::model::OperationSummaryPayloadV2>(
            &metadata_dir
                .join("state/operations")
                .join(operation)
                .join("operation.json"),
        );
    assert_eq!(summary.payload.result_delivery, "delivered");
    assert_eq!(
        std::fs::read(metadata_dir.join("config.toml")).unwrap(),
        accepted_registry
    );
    assert_eq!(
        std::fs::read(metadata_dir.join("state/state.json")).unwrap(),
        accepted_state
    );
    let show = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "show", &reference],
    );
    assert_eq!(
        support::json(&show)["details"]["entry"]["availability"],
        "cleaned"
    );
}

#[test]
fn interrupted_tombstone_publication_is_retryable_but_not_reported_cleaned() {
    let (_root, metadata_dir, _, _, reference) = support::payload_recovery_fixture();
    let home = support::project_home(&metadata_dir);
    let reference: grip::recovery::model::RecoveryRef = reference.parse().unwrap();
    let plan = grip::recovery::cleanup::plan(&home, std::slice::from_ref(&reference)).unwrap();
    assert!(
        grip::recovery::cleanup::execute_with_fault(
            &home,
            &plan,
            Some(grip::recovery::cleanup::CleanupFault::TombstonePublication),
        )
        .is_err()
    );
    assert_eq!(
        grip::recovery::inventory::show(&home, &reference)
            .unwrap()
            .availability,
        grip::recovery::model::RecoveryAvailability::CleanupIncomplete
    );
    let retry = grip::recovery::cleanup::plan(&home, std::slice::from_ref(&reference)).unwrap();
    grip::recovery::cleanup::execute(&home, &retry).unwrap();
    assert_eq!(
        grip::recovery::inventory::show(&home, &reference)
            .unwrap()
            .availability,
        grip::recovery::model::RecoveryAvailability::Cleaned
    );
}

#[test]
fn cleanup_byte_removal_failure_preserves_available_bytes() {
    let (_root, metadata_dir, _, _, reference) = support::payload_recovery_fixture();
    let home = support::project_home(&metadata_dir);
    let reference: grip::recovery::model::RecoveryRef = reference.parse().unwrap();
    let plan = grip::recovery::cleanup::plan(&home, std::slice::from_ref(&reference)).unwrap();
    assert!(
        grip::recovery::cleanup::execute_with_fault(
            &home,
            &plan,
            Some(grip::recovery::cleanup::CleanupFault::ByteRemoval),
        )
        .is_err()
    );
    assert_eq!(
        grip::recovery::inventory::show(&home, &reference)
            .unwrap()
            .availability,
        grip::recovery::model::RecoveryAvailability::Available
    );
}

#[test]
fn multi_reference_cleanup_reports_completed_failed_and_unattempted_actions() {
    let (root, metadata_dir, source, _) = support::accepted_tree_fixture();
    std::fs::remove_dir_all(&source).unwrap();
    let deletion = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "delete", "--source", "source"],
    );
    assert!(deletion.status.success());
    let home = support::project_home(&metadata_dir);
    let references = grip::recovery::inventory::list(&home)
        .unwrap()
        .into_iter()
        .filter(|entry| {
            entry.reference.has_recoverable_bytes()
                && entry.availability == grip::recovery::model::RecoveryAvailability::Available
                && entry.provenance == "complete"
        })
        .map(|entry| entry.reference)
        .take(3)
        .collect::<Vec<_>>();
    assert_eq!(references.len(), 3);
    let plan = grip::recovery::cleanup::plan(&home, &references).unwrap();
    let error = grip::recovery::cleanup::execute_with_fault(
        &home,
        &plan,
        Some(grip::recovery::cleanup::CleanupFault::TombstonePublicationAt(1)),
    )
    .unwrap_err();
    let grip::GripError::OperationLifecycle { details, .. } = error else {
        panic!("expected operation-bound cleanup failure");
    };
    let actions = details["plan"]["actions"].as_array().unwrap();
    assert_eq!(actions[0]["status"], "completed");
    assert_eq!(actions[1]["status"], "failed");
    assert_eq!(actions[2]["status"], "unattempted");
}
