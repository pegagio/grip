mod support;

use clap::Parser;
use grip::cli::{Cli, Command};
use std::ffi::OsString;
use std::fs;

#[test]
fn push_parses_default_execute_and_both_dry_run_aliases() {
    let execute = Cli::try_parse_from(["grip", "push"]).unwrap();
    assert!(matches!(
        execute.command,
        Command::Push(ref args) if !args.dry_run && !args.destination && args.path.is_none()
    ));

    for alias in ["-n", "--dry-run"] {
        let preview = Cli::try_parse_from(["grip", "push", alias]).unwrap();
        assert!(matches!(
            preview.command,
            Command::Push(ref args) if args.dry_run
        ));
    }
}

#[test]
fn output_failure_finalizes_delivery_without_rewriting_published_state() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let home = support::project_home(&metadata_dir);
    let mut receipt =
        grip::operation::publication::initialize(&home, &support::test_push_plan(0)).unwrap();
    receipt
        .checkpoint_summary(
            "completed",
            serde_json::json!({"outcome":"published","authoritative_generation":0}),
            "prepared",
            None,
        )
        .unwrap();
    let state_path = home.path().join("state/state.json");
    fs::write(&state_path, b"published-state-sentinel").unwrap();
    let before = fs::read(&state_path).unwrap();
    let mut outcome = grip::CommandOutcome::success("Push applied");
    outcome.details.insert(
        "operation_record".into(),
        serde_json::json!({"available":true,"id":receipt.operation_id()}),
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
    assert_eq!(fs::read(state_path).unwrap(), before);
    let summary = support::read_operation_component::<
        grip::operation::model::OperationSummaryPayloadV2,
    >(&receipt.directory().join("operation.json"));
    assert_eq!(summary.payload.result_delivery, "failed");
    assert_eq!(summary.payload.state, "completed");
}

#[test]
fn push_parses_destination_selector_and_option_terminator() {
    let destination = Cli::try_parse_from(["grip", "push", "--destination", "/target"]).unwrap();
    assert!(matches!(
        destination.command,
        Command::Push(ref args)
            if args.destination && args.path == Some(OsString::from("/target"))
    ));

    let leading_dash = Cli::try_parse_from(["grip", "push", "--", "-literal"]).unwrap();
    assert!(matches!(
        leading_dash.command,
        Command::Push(ref args) if args.path == Some(OsString::from("-literal"))
    ));
}

#[test]
fn push_rejects_extra_selectors() {
    assert!(Cli::try_parse_from(["grip", "push", "one", "two"]).is_err());
}

#[test]
fn dry_run_reports_deterministic_action_and_changes_nothing() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "payload").unwrap();
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
    let before = support::snapshot(root.path());

    let first = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "push", "--dry-run"],
    );
    let second =
        support::project_command(root.path(), &metadata_dir, &["--output=json", "push", "-n"]);
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    let value = support::json(&first);
    assert_eq!(value["details"]["result"], "planned");
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "ok");
    assert_eq!(value["code"], "ok");
    assert_eq!(value["details"]["counts"]["actions"], 1);
    assert_eq!(value["details"]["entries"].as_array().unwrap().len(), 1);
    assert_eq!(value["details"]["actions"].as_array().unwrap().len(), 1);
    assert_eq!(value["details"]["blockers"].as_array().unwrap().len(), 0);
    assert_eq!(value["details"]["plan_id"]["algorithm"], "sha256");
    assert_eq!(
        value["details"]["plan_id"]["digest"]
            .as_str()
            .unwrap()
            .len(),
        64
    );
    assert!(value["details"]["actions"][0]["destination_path"]["display"].is_string());
    let human = support::project_command(root.path(), &metadata_dir, &["push", "-n"]);
    assert!(human.status.success());
    let human_text = String::from_utf8_lossy(&human.stdout);
    assert!(human_text.contains("add_file"));
    assert!(human_text.contains(&destination.display().to_string()));
    assert_eq!(value["details"]["actions"][0]["kind"], "add_file");
    assert_eq!(value["details"]["baseline"]["outcome"], "not_attempted");
    assert!(
        value["details"]["baseline"]
            .get("prior_generation")
            .is_some()
    );
    assert_eq!(support::snapshot(root.path()), before);
}

#[test]
fn push_exit_categories_cover_usage_schema_corruption_and_operational_failure() {
    let usage = Cli::try_parse_from(["grip", "push", "one", "two"]).unwrap_err();
    assert_eq!(usage.exit_code(), 2);

    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(
        metadata_dir.join("config.toml"),
        "schema_version = 99\nmappings = []\n",
    )
    .unwrap();
    let unsupported = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "push", "--dry-run"],
    );
    assert_eq!(unsupported.status.code(), Some(11));

    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::create_dir(metadata_dir.join("state")).unwrap();
    let corrupt = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "push", "--dry-run"],
    );
    assert_eq!(corrupt.status.code(), Some(12));
    assert_eq!(grip::ResultCategory::InternalError.exit_code(), 20);
}

#[test]
fn dry_run_reports_all_blockers_and_starts_no_action() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source_a = root.path().join("source-a");
    let source_b = root.path().join("source-b");
    let destination_a = root.path().join("destination-a");
    let destination_b = root.path().join("destination-b");
    fs::write(&source_a, "source-a").unwrap();
    fs::write(&source_b, "source-b").unwrap();
    fs::write(&destination_a, "destination-a").unwrap();
    fs::write(&destination_b, "destination-b").unwrap();
    support::write_descriptor(
        &metadata_dir,
        &[
            ("file", &source_a, &destination_a),
            ("file", &source_b, &destination_b),
        ],
    );
    let before = support::snapshot(root.path());
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "push", "--dry-run"],
    );
    assert_eq!(output.status.code(), Some(10));
    let value = support::json(&output);
    assert_eq!(value["details"]["completion"], "blocked");
    assert_eq!(value["details"]["counts"]["blockers"], 2);
    assert_eq!(support::snapshot(root.path()), before);
}
