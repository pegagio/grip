mod support;

use std::fs;

#[test]
fn sync_parses_execute_selectors_and_preview_aliases() {
    let (root, grip_home, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "source change").unwrap();

    for preview in [["sync", "-n"], ["sync", "--dry-run"]] {
        let output = support::command_with_grip_home(root.path(), &grip_home, &preview);
        assert!(output.status.success());
    }
    let selected = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["sync", "--destination", "--", destination.to_str().unwrap()],
    );
    assert!(selected.status.success());
    assert_eq!(fs::read_to_string(source).unwrap(), "source change");
    assert_eq!(fs::read_to_string(destination).unwrap(), "source change");
}

#[test]
fn sync_preview_reports_operation_and_per_action_direction_without_mutation() {
    let (root, grip_home, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "source change").unwrap();
    let before = support::snapshot(root.path());
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output", "json", "sync", "--dry-run"],
    );
    assert!(output.status.success());
    let value = support::json(&output);
    assert_eq!(value["details"]["operation"], "sync");
    assert!(value["details"].get("direction").is_none());
    assert_eq!(value["details"]["actions"][0]["direction"], "push");
    assert_eq!(value["details"]["result"], "planned");
    assert_eq!(support::snapshot(root.path()), before);
    assert_eq!(fs::read_to_string(destination).unwrap(), "accepted");
}

#[test]
fn sync_rejects_extra_selectors() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let output = support::command(root.path(), &["sync", "one", "two"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn sync_human_output_uses_each_actions_direction() {
    let (root, grip_home, source, destination) = support::accepted_file_fixture();
    fs::write(&destination, "destination change").unwrap();
    let output = support::command_with_grip_home(root.path(), &grip_home, &["sync", "-n"]);
    assert!(output.status.success());
    let human = String::from_utf8(output.stdout).unwrap();
    assert!(human.contains(&format!(
        "{} -> {}",
        destination.display(),
        source.display()
    )));
}

#[test]
fn sync_output_failure_finalizes_only_its_operation_record() {
    let (_root, home, registry, state, selection, plan) = support::sync_execution_fixture();
    let success =
        grip::mutation::execution::execute(&home, &registry, &state, &selection, &plan).unwrap();
    let operation_id = success.operation_id.clone();
    let outcome = grip::result::CommandOutcome::mutation_applied(&success);
    assert!(
        grip::render_command_result(
            outcome,
            grip::result::OutputMode::Human,
            &mut support::FailingWriter,
            Some(&home),
        )
        .is_err()
    );
    let summary =
        support::read_operation_component::<grip::operation::model::OperationSummaryPayloadV1>(
            &home
                .path()
                .join("state/operations")
                .join(operation_id)
                .join("operation.json"),
        );
    assert_eq!(summary.payload.result_delivery, "failed");
}

#[test]
fn sync_human_and_json_blocked_results_expose_equivalent_evidence() {
    let (json_root, json_home, json_source, json_destination) = support::accepted_file_fixture();
    fs::write(&json_source, "source conflict").unwrap();
    fs::write(&json_destination, "destination conflict").unwrap();
    let json_output =
        support::command_with_grip_home(json_root.path(), &json_home, &["--output=json", "sync"]);
    assert_eq!(json_output.status.code(), Some(10));
    let json = support::json(&json_output);
    assert_eq!(json["details"]["operation"], "sync");
    assert_eq!(json["details"]["result"], "blocked");
    assert_eq!(json["details"]["counts"]["blockers"], 1);

    let (human_root, human_home, human_source, human_destination) =
        support::accepted_file_fixture();
    fs::write(&human_source, "source conflict").unwrap();
    fs::write(&human_destination, "destination conflict").unwrap();
    let human_output = support::command_with_grip_home(human_root.path(), &human_home, &["sync"]);
    assert_eq!(human_output.status.code(), Some(10));
    let human = String::from_utf8(human_output.stdout).unwrap();
    for evidence in ["Sync blocked", "1 blocker(s)", "divergent_change"] {
        assert!(human.contains(evidence), "missing {evidence:?} in {human}");
    }
}

#[test]
fn sync_and_resolve_use_stable_terminal_exit_categories() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    assert_eq!(
        support::command(root.path(), &["sync", "one", "two"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        support::command(root.path(), &["resolve", "--source"])
            .status
            .code(),
        Some(2)
    );

    let (blocked_root, blocked_home, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "source conflict").unwrap();
    fs::write(&destination, "destination conflict").unwrap();
    assert_eq!(
        support::command_with_grip_home(blocked_root.path(), &blocked_home, &["sync"])
            .status
            .code(),
        Some(10)
    );
    assert_eq!(
        support::command_with_grip_home(
            blocked_root.path(),
            &blocked_home,
            &["resolve", "--source", "--", source.to_str().unwrap()],
        )
        .status
        .code(),
        Some(0)
    );
}
