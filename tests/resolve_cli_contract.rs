mod support;

use std::fs;

#[test]
fn resolve_requires_exactly_one_winner_and_one_path() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    for args in [
        vec!["resolve", "--source"],
        vec!["resolve", "/tmp/path"],
        vec!["resolve", "--source", "--destination", "/tmp/path"],
        vec!["resolve", "--source", "--source", "/tmp/path"],
        vec!["resolve", "--source", "/tmp/one", "/tmp/two"],
    ] {
        assert_eq!(support::command(root.path(), &args).status.code(), Some(2));
    }
}

#[test]
fn resolve_preview_is_source_space_only_and_non_mutating() {
    let (root, grip_home, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "source change").unwrap();
    fs::write(&destination, "destination change").unwrap();
    let before = support::snapshot(root.path());
    for preview in ["-n", "--dry-run"] {
        let output = support::command_with_grip_home(
            root.path(),
            &grip_home,
            &[
                "--output",
                "json",
                "resolve",
                preview,
                "--source",
                "--",
                source.to_str().unwrap(),
            ],
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let value = support::json(&output);
        assert_eq!(value["details"]["operation"], "resolve");
        assert_eq!(value["details"]["winner"], "source");
        assert_eq!(value["details"]["actions"][0]["direction"], "push");
    }
    assert_eq!(support::snapshot(root.path()), before);

    let rejected = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["resolve", "--source", "--", destination.to_str().unwrap()],
    );
    assert_eq!(rejected.status.code(), Some(10));
}

#[test]
fn resolve_human_output_names_winner_and_directional_transfer() {
    let (root, grip_home, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "source change").unwrap();
    fs::write(&destination, "destination change").unwrap();
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "resolve",
            "-n",
            "--destination",
            "--",
            source.to_str().unwrap(),
        ],
    );
    assert!(output.status.success());
    let human = String::from_utf8(output.stdout).unwrap();
    assert!(human.contains("Winner destination"));
    assert!(human.contains(&format!(
        "{} -> {}",
        destination.display(),
        source.display()
    )));
}

#[test]
fn resolve_accepts_the_documented_path_then_winner_form() {
    let (root, grip_home, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "source change").unwrap();
    fs::write(&destination, "destination change").unwrap();
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["resolve", source.to_str().unwrap(), "--source", "--dry-run"],
    );
    assert!(output.status.success());
}

#[test]
fn resolve_human_and_json_results_expose_equivalent_winner_and_acceptance() {
    let (json_root, json_home, json_source, json_destination) = support::accepted_file_fixture();
    fs::write(&json_source, "winner").unwrap();
    fs::write(&json_destination, "loser").unwrap();
    let json_output = support::command_with_grip_home(
        json_root.path(),
        &json_home,
        &[
            "--output=json",
            "resolve",
            "--source",
            "--",
            json_source.to_str().unwrap(),
        ],
    );
    assert!(json_output.status.success());
    let json = support::json(&json_output);
    assert_eq!(json["details"]["winner"], "source");
    assert_eq!(json["details"]["result"], "applied");
    assert_eq!(json["details"]["actions"][0]["direction"], "push");
    assert_eq!(
        json["details"]["actions"][0]["milestones"]["recovery"],
        "preserved"
    );
    assert_eq!(json["details"]["baseline"]["outcome"], "published");

    let (human_root, human_home, human_source, human_destination) =
        support::accepted_file_fixture();
    fs::write(&human_source, "winner").unwrap();
    fs::write(&human_destination, "loser").unwrap();
    let human_output = support::command_with_grip_home(
        human_root.path(),
        &human_home,
        &["resolve", "--source", "--", human_source.to_str().unwrap()],
    );
    assert!(human_output.status.success());
    let human = String::from_utf8(human_output.stdout).unwrap();
    for evidence in [
        "Resolution applied",
        "Winner source",
        "recovery=preserved",
        "Baseline published",
    ] {
        assert!(human.contains(evidence), "missing {evidence:?} in {human}");
    }
}

#[test]
fn resolve_output_failure_preserves_the_published_baseline_and_record() {
    let (_root, home, registry, state, selection, plan) =
        support::resolution_execution_fixture(grip::mutation::model::ConflictWinner::Destination);
    let success =
        grip::mutation::execution::execute(&home, &registry, &state, &selection, &plan).unwrap();
    let generation = success.generation;
    let operation_id = success.operation_id.clone();
    let outcome = grip::result::CommandOutcome::mutation_applied(&success);
    assert!(
        grip::render_command_result(
            outcome,
            grip::result::OutputMode::Json,
            &mut support::FailingWriter,
            Some(&home),
        )
        .is_err()
    );
    let current = grip::state::publication::load(&home).unwrap();
    assert_eq!(current.accepted.generation, Some(generation));
    let summary =
        support::read_operation_component::<grip::operation::model::OperationSummaryPayloadV1>(
            &home
                .path()
                .join("state/operations")
                .join(operation_id)
                .join("operation.json"),
        );
    assert_eq!(summary.payload.operation, "resolve");
    assert_eq!(summary.payload.state, "completed");
    assert_eq!(summary.payload.result_delivery, "failed");
}
