mod support;

use std::fs;

#[test]
fn sync_parses_execute_selectors_and_preview_aliases() {
    let (root, metadata_dir, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "source change").unwrap();

    for preview in [["sync", "-n"], ["sync", "--dry-run"]] {
        let output = support::project_command(root.path(), &metadata_dir, &preview);
        assert!(output.status.success());
    }
    let selected = support::project_command(
        root.path(),
        &metadata_dir,
        &["sync", "--destination", "--", "~/destination"],
    );
    assert!(selected.status.success());
    assert_eq!(fs::read_to_string(source).unwrap(), "source change");
    assert_eq!(fs::read_to_string(destination).unwrap(), "source change");
}

#[test]
fn sync_preview_reports_operation_and_per_action_direction_without_mutation() {
    let (root, metadata_dir, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "source change").unwrap();
    let before = support::snapshot(root.path());
    let output = support::project_command(
        root.path(),
        &metadata_dir,
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
    let (root, metadata_dir, source, destination) = support::accepted_file_fixture();
    fs::write(&destination, "destination change").unwrap();
    let output = support::project_command(root.path(), &metadata_dir, &["sync", "-n"]);
    assert!(output.status.success());
    let human = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        human,
        format!(
            "Would synchronize 1 file(s):\n  {} <- {}\n",
            source.display(),
            destination.display()
        )
    );
}

#[test]
fn mixed_sync_preview_and_apply_render_one_heading_with_both_directions() {
    let (root, home, registry, state, selection, plan) = support::mixed_sync_execution_fixture();
    let preview = grip::CommandOutcome::mutation_plan(&plan, "dry_run", state.accepted.generation);
    let mut preview_human = Vec::new();
    grip::result::render(preview, grip::result::OutputMode::Human, &mut preview_human).unwrap();
    let root = root.path();
    let preview_text = String::from_utf8(preview_human).unwrap();
    assert_eq!(
        preview_text,
        format!(
            "Would synchronize 2 file(s):\n  {} -> {}\n  {} <- {}\n",
            root.join("source-a").display(),
            root.join("destination-a").display(),
            root.join("source-b").display(),
            root.join("destination-b").display(),
        )
    );
    for detail in [
        "replace_file",
        "recovery=",
        "verified=",
        "durable=",
        "Baseline ",
        "Operation record ",
    ] {
        assert!(
            !preview_text.contains(detail),
            "unexpected detail {detail}: {preview_text}"
        );
    }

    let applied =
        grip::mutation::execution::execute(&home, &registry, &state, &selection, &plan).unwrap();
    let mut applied_human = Vec::new();
    grip::result::render(
        grip::CommandOutcome::mutation_applied(&applied),
        grip::result::OutputMode::Human,
        &mut applied_human,
    )
    .unwrap();
    assert_eq!(
        String::from_utf8(applied_human).unwrap(),
        format!(
            "Synchronized 2 file(s):\n  {} -> {}\n  {} <- {}\n",
            root.join("source-a").display(),
            root.join("destination-a").display(),
            root.join("source-b").display(),
            root.join("destination-b").display(),
        )
    );
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
        support::read_operation_component::<grip::operation::model::OperationSummaryPayloadV2>(
            &home
                .path()
                .join("state/operations")
                .join(operation_id)
                .join("operation.json"),
        );
    assert_eq!(summary.payload.result_delivery, "failed");
}

#[test]
fn sync_human_blocked_result_gives_force_choices_while_json_retains_evidence() {
    let (json_root, json_home, json_source, json_destination) = support::accepted_file_fixture();
    fs::write(&json_source, "source conflict").unwrap();
    fs::write(&json_destination, "destination conflict").unwrap();
    let json_output =
        support::project_command(json_root.path(), &json_home, &["--output=json", "sync"]);
    assert_eq!(json_output.status.code(), Some(10));
    let json = support::json(&json_output);
    assert_eq!(json["details"]["operation"], "sync");
    assert_eq!(json["details"]["result"], "blocked");
    assert_eq!(json["details"]["counts"]["blockers"], 1);

    let (human_root, human_home, human_source, human_destination) =
        support::accepted_file_fixture();
    fs::write(&human_source, "source conflict").unwrap();
    fs::write(&human_destination, "destination conflict").unwrap();
    let human_output = support::project_command(human_root.path(), &human_home, &["sync"]);
    assert_eq!(human_output.status.code(), Some(10));
    let human = String::from_utf8(human_output.stdout).unwrap();
    assert_eq!(
        human,
        format!(
            "Error: Sync blocked: 1 selected; 0 action(s); 1 blocker(s)\n  source <-> {}\n    Keep source: grip push --force source\n    Keep destination: grip pull --force {}\n",
            human_destination.display(),
            human_destination.display(),
        )
    );

    let force_push = support::project_command(
        human_root.path(),
        &human_home,
        &["push", "--force", "--dry-run", "source"],
    );
    assert!(force_push.status.success());
    let force_pull = support::project_command(
        human_root.path(),
        &human_home,
        &[
            "pull",
            "--force",
            "--dry-run",
            human_destination.to_str().unwrap(),
        ],
    );
    assert!(force_pull.status.success());
}

#[test]
fn blocked_sync_uses_diff_for_an_aggregate_tree_conflict() {
    let (root, metadata_dir, _source, destination) = support::aggregate_tree_collision_fixture();

    let output =
        support::project_command(root.path(), &metadata_dir, &["sync", "source/nested/file"]);
    assert_eq!(output.status.code(), Some(10));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains(&format!(
        "  source/nested/file <-> {}/nested/file\n    Run: grip diff source/nested/file\n",
        destination.display(),
    )));
    assert!(!text.contains("grip push --force"));
    assert!(!text.contains("grip pull --force"));
}

#[test]
fn sync_noop_uses_a_concise_terminal_transcript() {
    let (root, metadata_dir, _source, _destination) = support::accepted_file_fixture();
    let output = support::project_command(root.path(), &metadata_dir, &["sync"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Nothing to synchronize.\n"
    );
}

#[test]
fn baseline_only_mutation_uses_a_concise_terminal_transcript() {
    let mut outcome = grip::CommandOutcome::success("Sync preview complete");
    outcome.details = serde_json::json!({
        "operation": "sync",
        "result": "planned",
        "actions": [],
        "counts": {"converged": 2}
    })
    .as_object()
    .unwrap()
    .clone();
    let mut human = Vec::new();
    grip::result::render(outcome, grip::result::OutputMode::Human, &mut human).unwrap();
    assert_eq!(
        String::from_utf8(human).unwrap(),
        "Would establish a baseline for 2 file(s).\n"
    );
}

#[test]
fn sync_and_forced_push_use_stable_terminal_exit_categories() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    assert_eq!(
        support::command(root.path(), &["sync", "one", "two"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        support::command(root.path(), &["push", "--not-an-option"])
            .status
            .code(),
        Some(2)
    );

    let (blocked_root, blocked_home, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "source conflict").unwrap();
    fs::write(&destination, "destination conflict").unwrap();
    assert_eq!(
        support::project_command(blocked_root.path(), &blocked_home, &["sync"])
            .status
            .code(),
        Some(10)
    );
    assert_eq!(
        support::project_command(
            blocked_root.path(),
            &blocked_home,
            &["push", "-f", "source"],
        )
        .status
        .code(),
        Some(0)
    );
}
