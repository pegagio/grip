mod support;
use std::fs;

#[test]
fn converged_deletion_retirement_changes_state_only() {
    let (root, grip_home, source, destination) = support::accepted_file_fixture();
    fs::remove_file(&source).unwrap();
    fs::remove_file(&destination).unwrap();
    let before_registry = fs::read(grip_home.join("config.toml")).unwrap();
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "retire", source.to_str().unwrap()],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read(grip_home.join("config.toml")).unwrap(),
        before_registry
    );
    let state = fs::read_to_string(grip_home.join("state/state.json")).unwrap();
    assert!(!state.contains("relative_path_hex"));
}

#[test]
fn retirement_publication_failure_preserves_payloads_and_authoritative_generation() {
    let (_root, home, _registry, state, selection, deletion) =
        support::deletion_execution_fixture();
    let target = deletion.actions[0].target.clone();
    fs::remove_file(&target).unwrap();
    let registry = grip::registry::publication::load(&home, false).unwrap();
    let observed =
        grip::observation::inspect(&home, &registry, &state.accepted, &selection).unwrap();
    let records = observed
        .values()
        .map(|entry| {
            grip::classification::classify(entry, state.accepted.baselines.get(&entry.identity))
        })
        .collect();
    let plan = grip::retire::plan::build(
        grip::classification::model::ClassificationScope {
            kind: "all".into(),
            path_space: grip::observation::model::PathSpace::Source,
            selector: None,
            mapping_source: None,
        },
        false,
        records,
    )
    .unwrap();
    let result = grip::retire::execution::execute_with_fault(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        Some(grip::retire::execution::RetirementFault::StatePublication),
    );
    assert!(result.is_err());
    assert_eq!(
        grip::state::publication::load(&home)
            .unwrap()
            .accepted
            .generation,
        state.accepted.generation
    );
}

#[test]
fn removed_mapping_retirement_preserves_both_payloads_and_delivers_record() {
    let (root, grip_home, source, destination) = support::untracked_file_fixture();
    let source_bytes = fs::read(&source).unwrap();
    let destination_bytes = fs::read(&destination).unwrap();
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "retire", "--all"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(fs::read(source).unwrap(), source_bytes);
    assert_eq!(fs::read(destination).unwrap(), destination_bytes);
    let operation = support::json(&output)["details"]["operation_record"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let summary =
        support::read_operation_component::<grip::operation::model::OperationSummaryPayloadV1>(
            &grip_home
                .join("state/operations")
                .join(operation)
                .join("operation.json"),
        );
    assert_eq!(summary.payload.result_delivery, "delivered");
}

#[test]
fn retirement_execute_reports_contention_while_preview_remains_lock_free() {
    let (root, grip_home, source, destination) = support::accepted_file_fixture();
    fs::remove_file(source).unwrap();
    fs::remove_file(destination).unwrap();
    let home = grip::home::select(Some(grip_home.clone().into_os_string()), None).unwrap();
    let guard = grip::state::mutation_lock::MutationLock::acquire(&home, "test-owner").unwrap();
    assert!(
        support::command_with_grip_home(root.path(), &grip_home, &["retire", "-n", "--all"])
            .status
            .success()
    );
    assert_eq!(
        support::command_with_grip_home(root.path(), &grip_home, &["retire", "--all"])
            .status
            .code(),
        Some(13)
    );
    drop(guard);
}

#[test]
fn differing_untracked_survivors_require_force_and_force_changes_only_state() {
    let (root, grip_home, source, destination) = support::untracked_file_fixture();
    fs::write(&source, "changed-source").unwrap();
    let source_before = fs::read(&source).unwrap();
    let destination_before = fs::read(&destination).unwrap();
    let blocked = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "retire", "--all"],
    );
    assert_eq!(blocked.status.code(), Some(10));
    assert_eq!(
        support::json(&blocked)["details"]["blockers"][0]["reason"],
        "force_required"
    );
    let forced = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "retire", "--force", "--all"],
    );
    assert!(forced.status.success());
    assert_eq!(fs::read(source).unwrap(), source_before);
    assert_eq!(fs::read(destination).unwrap(), destination_before);
    assert_eq!(support::json(&forced)["details"]["force"], true);
}

#[test]
fn newly_ignored_entry_retires_without_mutating_either_payload() {
    let (root, grip_home, source, destination) = support::accepted_tree_fixture();
    fs::write(source.join(".gripignore"), "nested/file\n").unwrap();
    let source_before = fs::read(source.join("nested/file")).unwrap();
    let destination_before = fs::read(destination.join("nested/file")).unwrap();
    let ignored = source.join("nested/file");
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "retire", ignored.to_str().unwrap()],
    );
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read(source.join("nested/file")).unwrap(), source_before);
    assert_eq!(
        fs::read(destination.join("nested/file")).unwrap(),
        destination_before
    );
    assert!(
        support::json(&output)["details"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| action["reason"] == "newly_ignored")
    );
}
