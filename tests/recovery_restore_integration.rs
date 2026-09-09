mod support;
use std::fs;

#[test]
fn payload_restore_preserves_displaced_post_state_and_retains_source_recovery() {
    let (root, metadata_dir, source, _, reference) = support::payload_recovery_fixture();
    assert_eq!(fs::read_to_string(&source).unwrap(), "replacement");
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "restore", &reference],
    );
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(&source).unwrap(), "accepted");
    let operation = support::json(&output)["details"]["operation_record"]["id"]
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
    let shown = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "show", &reference],
    );
    assert_eq!(
        support::json(&shown)["details"]["entry"]["availability"],
        "available"
    );
}

#[test]
fn payload_restore_accepts_absent_target_without_accepting_a_new_baseline() {
    let (root, metadata_dir, source, _, reference) = support::payload_recovery_fixture();
    let before_state = fs::read(metadata_dir.join("state/state.json")).unwrap();
    fs::remove_file(&source).unwrap();
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["recovery", "restore", &reference],
    );
    assert!(output.status.success());
    assert_eq!(fs::read_to_string(source).unwrap(), "accepted");
    assert_eq!(
        fs::read(metadata_dir.join("state/state.json")).unwrap(),
        before_state
    );
}

#[test]
fn payload_restore_blocks_different_occupied_target_without_mutation() {
    let (root, metadata_dir, source, _, reference) = support::payload_recovery_fixture();
    fs::write(&source, "unrelated-current-value").unwrap();
    let before = support::snapshot(root.path());
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["recovery", "restore", &reference],
    );
    assert!(!output.status.success());
    support::assert_snapshot_unchanged(&before, root.path());
}

#[test]
fn payload_restore_reports_mutation_contention_and_succeeds_after_release() {
    let (root, metadata_dir, source, _, reference) = support::payload_recovery_fixture();
    fs::remove_file(&source).unwrap();
    let home = support::project_home(&metadata_dir);
    let guard = grip::state::mutation_lock::MutationLock::acquire(&home, "test-owner").unwrap();
    let blocked = support::project_command(
        root.path(),
        &metadata_dir,
        &["recovery", "restore", &reference],
    );
    assert_eq!(blocked.status.code(), Some(13));
    drop(guard);
    let retry = support::project_command(
        root.path(),
        &metadata_dir,
        &["recovery", "restore", &reference],
    );
    assert!(retry.status.success());
}

#[test]
fn injected_restore_failures_report_prepublication_and_visible_effects_truthfully() {
    for fault in [
        grip::recovery::restore::RestoreFault::Staging,
        grip::recovery::restore::RestoreFault::Publication,
    ] {
        let (_root, metadata_dir, source, _, reference) = support::payload_recovery_fixture();
        fs::remove_file(&source).unwrap();
        let home = support::project_home(&metadata_dir);
        let reference = reference.parse().unwrap();
        let plan = grip::recovery::restore::plan(&home, &reference).unwrap();
        assert!(grip::recovery::restore::execute_with_fault(&home, &plan, Some(fault)).is_err());
        assert!(!source.exists());
    }
    let (_root, metadata_dir, source, _, reference) = support::payload_recovery_fixture();
    fs::remove_file(&source).unwrap();
    let home = support::project_home(&metadata_dir);
    let reference = reference.parse().unwrap();
    let plan = grip::recovery::restore::plan(&home, &reference).unwrap();
    let error = grip::recovery::restore::execute_with_fault(
        &home,
        &plan,
        Some(grip::recovery::restore::RestoreFault::Verification),
    )
    .unwrap_err();
    assert!(source.exists());
    assert!(matches!(
        error,
        grip::GripError::OperationLifecycle {
            publication_visible: true,
            ..
        }
    ));
}

#[test]
fn directory_recovery_composes_parent_before_child_without_baseline_acceptance() {
    let (root, metadata_dir, source, destination) = support::accepted_tree_fixture();
    fs::remove_dir_all(&source).unwrap();
    let deletion = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "delete", "--source", "source"],
    );
    assert!(deletion.status.success());
    let operation = support::json(&deletion)["details"]["operation_record"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let state = fs::read(metadata_dir.join("state/state.json")).unwrap();
    let directory_ref = format!("payload:{operation}:1");
    let file_ref = format!("payload:{operation}:0");
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["recovery", "restore", &directory_ref]
        )
        .status
        .success()
    );
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["recovery", "restore", &file_ref]
        )
        .status
        .success()
    );
    assert_eq!(
        fs::read_to_string(destination.join("nested/file")).unwrap(),
        "accepted"
    );
    assert_eq!(
        fs::read(metadata_dir.join("state/state.json")).unwrap(),
        state
    );
}
