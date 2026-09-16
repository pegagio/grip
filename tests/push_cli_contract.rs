mod support;

use clap::Parser;
use grip::cli::{Cli, Command};
use std::ffi::OsString;
use std::fs;
use support::project::ProjectFixture;

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
fn force_push_without_a_selector_is_an_aggregate_request() {
    let aggregate = Cli::try_parse_from(["grip", "push", "--force"]).unwrap();
    assert!(matches!(
        aggregate.command,
        Command::Push(ref args) if args.force && args.path.is_none() && !args.destination
    ));
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
    let human_text = String::from_utf8(human.stdout).unwrap();
    assert_eq!(
        human_text,
        format!(
            "Would push 1 file(s):\n  {} -> {}\n",
            fs::canonicalize(root.path())
                .unwrap()
                .join("source")
                .display(),
            fs::canonicalize(root.path())
                .unwrap()
                .join("destination")
                .display()
        )
    );
    for detail in [
        "add_file",
        "recovery=",
        "verified=",
        "durable=",
        "Baseline ",
        "Operation record ",
    ] {
        assert!(
            !human_text.contains(detail),
            "unexpected detail {detail}: {human_text}"
        );
    }
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
fn initial_source_authoritative_push_uses_the_concise_completed_push_transcript() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "source wins").unwrap();
    fs::write(&destination, "destination loses").unwrap();
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "source", "~/destination"],
        )
        .status
        .success()
    );

    let pushed = support::project_command(root.path(), &metadata_dir, &["push", "source"]);
    assert!(pushed.status.success());
    assert_eq!(
        String::from_utf8(pushed.stdout).unwrap(),
        format!(
            "Pushed 1 file(s):\n  {} -> {}\n",
            source.display(),
            destination.display()
        )
    );
}

#[test]
fn tree_push_counts_payload_files_not_directory_setup_actions() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source-tree");
    let destination = root.path().join("destination-tree");
    fs::create_dir(&source).unwrap();
    fs::create_dir(source.join("nested")).unwrap();
    fs::write(source.join("first.txt"), "first").unwrap();
    fs::write(source.join("nested/second.txt"), "second").unwrap();
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);

    let pushed = support::project_command(root.path(), &metadata_dir, &["push"]);
    assert!(pushed.status.success());
    assert_eq!(
        String::from_utf8(pushed.stdout).unwrap(),
        format!(
            "Pushed 2 file(s):\n  {} -> {}\n  {} -> {}\nCreated 3 directory(ies).\n",
            source.join("first.txt").display(),
            destination.join("first.txt").display(),
            source.join("nested/second.txt").display(),
            destination.join("nested/second.txt").display(),
        )
    );
}

#[test]
fn aggregate_force_lists_the_safety_blockers_that_prevent_execution() {
    use grip::discovery::model::SafePath;
    use grip::mutation::model::{MutationOperation, PlanBlocker};
    use grip::result::OutputMode;
    use std::path::Path;

    let mut plan = support::test_push_plan(0);
    plan.operation = MutationOperation::AggregateForcePush;
    plan.direction = None;
    plan.counts.selected = 1;
    plan.counts.blockers = 1;
    plan.blockers = vec![PlanBlocker {
        reason: "unsafe_symlink_ancestry".into(),
        paths: vec![
            SafePath::from_path(Path::new("/source-link")),
            SafePath::from_path(Path::new("/destination")),
        ],
    }];
    let outcome = grip::CommandOutcome::mutation_plan(&plan, "execute", None);
    let mut rendered = Vec::new();
    grip::render_command_result(outcome, OutputMode::Human, &mut rendered, None).unwrap();
    let text = String::from_utf8(rendered).unwrap();
    assert!(text.starts_with(
        "Error: Aggregate forced push blocked: 1 selected; 0 action(s); 1 blocker(s)\n"
    ));
    assert!(text.contains("unsafe_symlink_ancestry: /source-link -> /destination\n"));
    assert!(text.contains("Resolve the listed safety blockers, then run: grip status\n"));
}

#[test]
fn push_noop_uses_a_concise_terminal_transcript() {
    let (root, metadata_dir, _source, _destination) = support::accepted_file_fixture();
    let output = support::project_command(root.path(), &metadata_dir, &["push"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Nothing to push.\n"
    );
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

#[test]
fn blocked_push_shows_force_choices_for_an_initial_collision() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "source version").unwrap();
    fs::write(&destination, "destination version").unwrap();
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
    let before = support::snapshot(root.path());

    let output = support::project_command(root.path(), &metadata_dir, &["push", "source"]);
    assert_eq!(output.status.code(), Some(10));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "Error: Push blocked: 1 selected; 0 action(s); 1 blocker(s)\n  source <-> {}\n    Keep source: grip push --force source\n    Keep destination: grip pull --force --destination {}\n",
            fs::canonicalize(&destination).unwrap().display(),
            fs::canonicalize(&destination).unwrap().display(),
        )
    );
    let json = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "push", "source"],
    );
    assert_eq!(json.status.code(), Some(10));
    assert!(support::json(&json)["details"]["baseline"].is_object());
    assert_eq!(support::snapshot(root.path()), before);

    let force_push = support::project_command(
        root.path(),
        &metadata_dir,
        &["push", "--force", "--dry-run", "source"],
    );
    assert!(force_push.status.success());
    let force_pull = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "pull",
            "--force",
            "--dry-run",
            "--destination",
            destination.to_str().unwrap(),
        ],
    );
    assert!(force_pull.status.success());
}

#[test]
fn blocked_push_uses_diff_for_an_aggregate_tree_conflict() {
    let (root, metadata_dir, _source, destination) = support::aggregate_tree_collision_fixture();

    let output =
        support::project_command(root.path(), &metadata_dir, &["push", "source/nested/file"]);
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
fn forced_push_blocked_by_a_technical_condition_uses_a_public_direction_and_status_guidance() {
    let mut outcome = grip::CommandOutcome::success("Resolution blocked");
    outcome.category = grip::ResultCategory::InvalidConfiguration;
    outcome.details = serde_json::json!({
        "operation": "resolve",
        "winner": "source",
        "result": "blocked",
        "counts": {"selected": 1, "actions": 0, "blockers": 1},
        "blockers": [{"reason": "blocking_evidence", "paths": []}],
        "baseline": {"outcome": "not_attempted", "authoritative_generation": 1},
        "operation_record": {"available": true, "id": "internal-record"}
    })
    .as_object()
    .unwrap()
    .clone();
    let mut human = Vec::new();
    grip::result::render(outcome, grip::result::OutputMode::Human, &mut human).unwrap();
    assert_eq!(
        String::from_utf8(human).unwrap(),
        "Error: Push blocked: 1 selected; 0 action(s); 1 blocker(s)\n  Grip cannot safely continue. Run: grip status\n"
    );
}

#[test]
fn blocked_push_guidance_uses_a_source_selector_from_the_invocation_directory() {
    let fixture = ProjectFixture::initialized();
    let app = fixture.project_root.join("app");
    let source = app.join("main.py");
    let destination = fixture.home_destination("workspace/app/main.py");
    fs::create_dir_all(&app).unwrap();
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    fs::write(&source, "accepted").unwrap();
    fs::write(&destination, "accepted").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    assert!(
        fixture
            .command(&["add", "app/main.py", "~/workspace/app/main.py"])
            .status
            .success()
    );
    fs::write(&source, "source change").unwrap();
    fs::write(&destination, "destination change").unwrap();

    let blocked = fixture.command_from(&app, &["push", "main.py"]);
    assert_eq!(blocked.status.code(), Some(10));
    assert!(
        String::from_utf8(blocked.stdout)
            .unwrap()
            .contains(&format!(
                "Keep source: grip push --force main.py\n    Keep destination: grip pull --force --destination {}",
                destination.display()
            ))
    );
}
