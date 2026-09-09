mod support;

use clap::Parser;
use grip::cli::{Cli, Command};
use std::ffi::OsString;
use std::fs;

#[test]
fn pull_parses_default_execute_selectors_and_dry_run_aliases() {
    let execute = Cli::try_parse_from(["grip", "pull"]).unwrap();
    assert!(matches!(
        execute.command,
        Command::Pull(ref args) if !args.dry_run && !args.destination && args.path.is_none()
    ));
    for alias in ["-n", "--dry-run"] {
        let preview = Cli::try_parse_from(["grip", "pull", alias]).unwrap();
        assert!(matches!(preview.command, Command::Pull(ref args) if args.dry_run));
    }
    let destination = Cli::try_parse_from(["grip", "pull", "--destination", "/target"]).unwrap();
    assert!(matches!(
        destination.command,
        Command::Pull(ref args)
            if args.destination && args.path == Some(OsString::from("/target"))
    ));
    let leading_dash = Cli::try_parse_from(["grip", "pull", "--", "-literal"]).unwrap();
    assert!(matches!(
        leading_dash.command,
        Command::Pull(ref args) if args.path == Some(OsString::from("-literal"))
    ));
    assert!(Cli::try_parse_from(["grip", "pull", "one", "two"]).is_err());
}

#[test]
fn pull_source_and_destination_selectors_resolve_the_same_managed_action() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "accepted").unwrap();
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
    assert!(
        support::project_command(root.path(), &metadata_dir, &["push"])
            .status
            .success()
    );
    fs::write(&destination, "changed").unwrap();

    let source_output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "pull", "--dry-run", "source"],
    );
    let destination_output = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output=json",
            "pull",
            "--dry-run",
            "--destination",
            "~/destination",
        ],
    );
    assert!(source_output.status.success());
    assert!(destination_output.status.success());
    let source_json = support::json(&source_output);
    let destination_json = support::json(&destination_output);
    assert_eq!(source_json["details"]["counts"]["actions"], 1);
    assert_eq!(destination_json["details"]["counts"]["actions"], 1);
    assert_eq!(
        source_json["details"]["actions"][0]["source_path"],
        destination_json["details"]["actions"][0]["source_path"]
    );
    assert_eq!(
        source_json["details"]["actions"][0]["destination_path"],
        destination_json["details"]["actions"][0]["destination_path"]
    );
    assert_eq!(source_json["details"]["scope"]["path_space"], "source");
    assert_eq!(
        destination_json["details"]["scope"]["path_space"],
        "destination"
    );
}

#[test]
fn pull_preview_uses_shared_schema_and_changes_nothing() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "accepted").unwrap();
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
    let pushed = support::project_command(root.path(), &metadata_dir, &["--output=json", "push"]);
    assert!(pushed.status.success());
    fs::write(&destination, "destination change").unwrap();
    let before = support::snapshot(root.path());

    let first = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "pull", "--dry-run"],
    );
    let second =
        support::project_command(root.path(), &metadata_dir, &["--output=json", "pull", "-n"]);
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    let value = support::json(&first);
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["details"]["operation"], "pull");
    assert_eq!(value["details"]["direction"], "pull");
    assert_eq!(value["details"]["mode"], "dry_run");
    assert_eq!(value["details"]["result"], "planned");
    assert_eq!(value["details"]["counts"]["actions"], 1);
    assert_eq!(value["details"]["actions"][0]["kind"], "replace_file");
    assert!(value["details"]["operation_record"].is_null());
    let human = support::project_command(root.path(), &metadata_dir, &["pull", "--dry-run"]);
    let text = String::from_utf8_lossy(&human.stdout);
    let arrow = format!("{} -> {}", destination.display(), source.display());
    assert!(
        text.contains(&arrow),
        "human output did not contain {arrow}: {text}"
    );
    assert_eq!(support::snapshot(root.path()), before);
}

#[test]
fn pull_reports_unmanaged_destination_content_without_importing_it() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source-tree");
    let destination = root.path().join("destination-tree");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("managed"), "accepted").unwrap();
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);
    assert!(
        support::project_command(root.path(), &metadata_dir, &["push"])
            .status
            .success()
    );
    fs::write(destination.join("managed"), "changed").unwrap();
    fs::write(destination.join("unmanaged"), "destination only").unwrap();
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "pull", "--dry-run"],
    );
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let entries = support::json(&output)["details"]["entries"]
        .as_array()
        .unwrap()
        .clone();
    assert!(entries.iter().any(|entry| {
        entry["classification"] == "destination_only_unmanaged"
            && entry["disposition"] == "no_action"
    }));
    assert!(!source.join("unmanaged").exists());
}

#[test]
fn pull_honors_hierarchical_source_side_gripignore_without_importing() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source-tree");
    let destination = root.path().join("destination-tree");
    fs::create_dir_all(source.join("nested")).unwrap();
    fs::write(source.join("nested/.gripignore"), "*.secret\n").unwrap();
    fs::write(source.join("nested/managed"), "accepted").unwrap();
    fs::write(source.join("nested/private.secret"), "source private").unwrap();
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);
    assert!(
        support::project_command(root.path(), &metadata_dir, &["push"])
            .status
            .success()
    );
    fs::write(destination.join("nested/managed"), "changed").unwrap();
    fs::write(
        destination.join("nested/private.secret"),
        "destination private",
    )
    .unwrap();
    let private_before = fs::read(source.join("nested/private.secret")).unwrap();

    let output = support::project_command(root.path(), &metadata_dir, &["--output=json", "pull"]);
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(source.join("nested/managed")).unwrap(),
        "changed"
    );
    assert_eq!(
        fs::read(source.join("nested/private.secret")).unwrap(),
        private_before
    );
}

#[test]
fn pull_blocks_divergent_changes_before_mutation() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "accepted").unwrap();
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
    assert!(
        support::project_command(root.path(), &metadata_dir, &["push"])
            .status
            .success()
    );
    fs::write(&source, "source change").unwrap();
    fs::write(&destination, "destination change").unwrap();
    let before = support::snapshot(root.path());
    let output = support::project_command(root.path(), &metadata_dir, &["--output=json", "pull"]);
    assert_eq!(output.status.code(), Some(10));
    let value = support::json(&output);
    assert_eq!(value["details"]["completion"], "blocked");
    assert_eq!(value["details"]["counts"]["actions"], 0);
    assert_eq!(
        value["details"]["operation_record"],
        serde_json::Value::Null
    );
    assert_eq!(support::snapshot(root.path()), before);
}

#[test]
fn pull_failure_has_equivalent_human_and_machine_evidence() {
    let (_root, home, registry, state, selection, plan) = support::pull_execution_fixture();
    let error = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        support::fail_push_at(grip::mutation::FaultPhase::AfterPayloadPublication(0)),
    )
    .unwrap_err();
    let outcome = grip::CommandOutcome::failure(&error);
    let mut json_bytes = Vec::new();
    grip::result::render(
        outcome.clone(),
        grip::result::OutputMode::Json,
        &mut json_bytes,
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&json_bytes).unwrap();
    assert_eq!(value["details"]["direction"], "pull");
    assert_eq!(value["details"]["completion"], "partial");
    assert_eq!(value["details"]["failure"]["phase"], "publication_failure");
    assert_eq!(
        value["details"]["failure"]["category"],
        "operational_failure"
    );
    assert_eq!(value["details"]["failure"]["action_index"], 0);
    assert!(value["details"]["failure"]["expected_source"].is_object());
    assert!(value["details"]["failure"]["expected_destination"].is_object());
    assert_eq!(
        value["details"]["failure"]["observed_source"],
        value["details"]["failure"]["expected_destination"]
    );
    assert!(value["details"]["failure"]["guidance"].is_string());
    assert_eq!(value["details"]["actions"][0]["status"], "failed");
    assert_eq!(
        value["details"]["actions"][0]["milestones"]["publication"],
        "visible"
    );
    assert!(value["details"]["operation_record"]["id"].is_string());
    let mut human = Vec::new();
    grip::result::render(outcome, grip::result::OutputMode::Human, &mut human).unwrap();
    let text = String::from_utf8(human).unwrap();
    assert!(text.contains("Pull failed"));
    assert!(text.contains("visible=yes verified=no"));
    assert!(text.contains("Operation record"));
    assert!(text.contains("Baseline not published"));
}

#[test]
fn pull_human_results_cover_apply_noop_and_blocked_outcomes() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "accepted").unwrap();
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
    assert!(
        support::project_command(root.path(), &metadata_dir, &["push"])
            .status
            .success()
    );
    fs::write(&destination, "destination change").unwrap();

    let applied = support::project_command(root.path(), &metadata_dir, &["pull"]);
    assert!(applied.status.success());
    let applied_text = String::from_utf8(applied.stdout).unwrap();
    assert!(applied_text.contains("Pull applied"));
    assert!(applied_text.contains("Baseline published"));

    let noop = support::project_command(root.path(), &metadata_dir, &["pull"]);
    assert!(noop.status.success());
    assert!(
        String::from_utf8(noop.stdout)
            .unwrap()
            .contains("Pull complete")
    );

    fs::write(&source, "source divergence").unwrap();
    fs::write(&destination, "destination divergence").unwrap();
    let blocked = support::project_command(root.path(), &metadata_dir, &["pull"]);
    assert_eq!(blocked.status.code(), Some(10));
    let blocked_text = String::from_utf8(blocked.stdout).unwrap();
    assert!(blocked_text.contains("Pull blocked"));
    assert!(
        blocked_text.contains("blocked divergent_change"),
        "blocked output omitted blocker detail: {blocked_text}"
    );
}
