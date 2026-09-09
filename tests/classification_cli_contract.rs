mod support;

use std::fs;
use std::process::{Command, Stdio};

fn file_fixture() -> (
    tempfile::TempDir,
    std::path::PathBuf,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "same").unwrap();
    fs::write(&destination, "same").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
    (root, metadata_dir, source, destination)
}

#[test]
fn status_check_diff_and_baseline_share_the_versioned_result_contract() {
    let (root, metadata_dir, _, _) = file_fixture();
    let status =
        support::project_command(root.path(), &metadata_dir, &["--output", "json", "status"]);
    assert_eq!(status.status.code(), Some(0));
    let status_json = support::json(&status);
    assert_eq!(
        status_json["details"]["records"][0]["classification"],
        "initial_match"
    );
    assert_eq!(
        status_json["details"]["counts"].as_object().unwrap().len(),
        20
    );

    let check = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "check", "source"],
    );
    assert_eq!(check.status.code(), Some(1));
    assert_eq!(support::json(&check)["code"], "attention_required");

    let accept = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(accept.status.code(), Some(0));
    assert_eq!(support::json(&accept)["details"]["generation"], 0);

    let diff = support::project_command(root.path(), &metadata_dir, &["--output", "json", "diff"]);
    let diff_json = support::json(&diff);
    assert_eq!(
        diff_json["details"]["records"][0]["classification"],
        "synchronized"
    );
    assert_eq!(
        diff_json["details"]["records"][0]["changed_dimensions"]["source_to_destination"],
        serde_json::json!([])
    );

    let validate = support::project_command(root.path(), &metadata_dir, &["validate"]);
    assert_eq!(validate.status.code(), Some(0));
}

#[test]
fn baseline_noop_preserves_exact_state_bytes_and_generation() {
    let (root, metadata_dir, _, _) = file_fixture();
    let first = support::project_command(root.path(), &metadata_dir, &["baseline", "accept"]);
    assert_eq!(first.status.code(), Some(0));
    let path = metadata_dir.join("state/state.json");
    let before = fs::read(&path).unwrap();
    let second = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(
        support::json(&second)["details"]["result"],
        "already_current"
    );
    assert_eq!(fs::read(path).unwrap(), before);
    assert!(!metadata_dir.join("state/recovery/generation-0").exists());
}

#[test]
fn grammar_and_selector_boundaries_are_stable() {
    let (root, metadata_dir, source, destination) = file_fixture();
    let selected = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "status", "--", "source"],
    );
    assert_eq!(selected.status.code(), Some(0));
    assert_eq!(
        support::json(&selected)["details"]["scope"]["kind"],
        "entry"
    );

    let destination_selected = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output",
            "json",
            "status",
            "--destination",
            "~/destination",
        ],
    );
    assert_eq!(destination_selected.status.code(), Some(0));

    let outside = root.path().join("outside");
    let invalid = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "status", outside.to_str().unwrap()],
    );
    assert_eq!(invalid.status.code(), Some(10));

    let extra = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output",
            "json",
            "status",
            source.to_str().unwrap(),
            destination.to_str().unwrap(),
        ],
    );
    assert_eq!(extra.status.code(), Some(2));
}

#[test]
fn ignored_and_destination_only_selectors_return_explicit_unmanaged_records() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&destination).unwrap();
    fs::write(source.join("ignored"), "source").unwrap();
    fs::write(source.join(".gripignore"), "ignored\n").unwrap();
    fs::write(destination.join("unmanaged"), "destination").unwrap();
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);

    let ignored = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "status", "source/ignored"],
    );
    assert_eq!(ignored.status.code(), Some(0));
    assert_eq!(
        support::json(&ignored)["details"]["records"][0]["reasons"][0],
        "ignored"
    );

    let unmanaged = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output",
            "json",
            "status",
            "--destination",
            "~/destination/unmanaged",
        ],
    );
    assert_eq!(unmanaged.status.code(), Some(0));
    assert_eq!(
        support::json(&unmanaged)["details"]["records"][0]["classification"],
        "destination_only_unmanaged"
    );
}

#[test]
fn check_has_distinct_clean_attention_usage_configuration_schema_and_corruption_exits() {
    let (root, metadata_dir, source, destination) = file_fixture();
    let attention =
        support::project_command(root.path(), &metadata_dir, &["--output", "json", "check"]);
    assert_eq!(attention.status.code(), Some(1));
    assert_eq!(support::json(&attention)["status"], "ok");
    assert_eq!(support::json(&attention)["code"], "attention_required");
    assert!(
        support::project_command(root.path(), &metadata_dir, &["baseline", "accept"])
            .status
            .success()
    );
    let clean = support::project_command(root.path(), &metadata_dir, &["check"]);
    assert_eq!(clean.status.code(), Some(0));

    let usage = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "check",
            source.to_str().unwrap(),
            destination.to_str().unwrap(),
        ],
    );
    assert_eq!(usage.status.code(), Some(2));
    let invalid = support::project_command(
        root.path(),
        &metadata_dir,
        &["check", root.path().join("outside").to_str().unwrap()],
    );
    assert_eq!(invalid.status.code(), Some(10));

    let state_path = metadata_dir.join("state/state.json");
    fs::write(&state_path, br#"{"schema_version":99}"#).unwrap();
    let unsupported = support::project_command(root.path(), &metadata_dir, &["check"]);
    assert_eq!(unsupported.status.code(), Some(11));
    fs::write(&state_path, b"{}").unwrap();
    let corrupt = support::project_command(root.path(), &metadata_dir, &["check"]);
    assert_eq!(corrupt.status.code(), Some(12));
}

#[test]
fn diff_reports_three_safe_comparisons_without_payload_content() {
    let (root, metadata_dir, source, destination) = file_fixture();
    assert!(
        support::project_command(root.path(), &metadata_dir, &["baseline", "accept"])
            .status
            .success()
    );
    let secret = "do-not-render-this-payload";
    fs::write(&source, secret).unwrap();
    fs::write(&destination, secret).unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    let output =
        support::project_command(root.path(), &metadata_dir, &["--output", "json", "diff"]);
    assert_eq!(output.status.code(), Some(0));
    let value = support::json(&output);
    let changed = &value["details"]["records"][0]["changed_dimensions"];
    assert_eq!(
        changed["source_to_baseline"],
        serde_json::json!(["content", "modification_time"])
    );
    assert_eq!(
        changed["destination_to_baseline"],
        serde_json::json!(["content", "modification_time"])
    );
    assert_eq!(changed["source_to_destination"], serde_json::json!([]));
    let rendered = String::from_utf8(output.stdout).unwrap();
    assert!(!rendered.contains(secret));
    assert!(rendered.contains("sha256"));

    let human = support::project_command(root.path(), &metadata_dir, &["diff"]);
    let human_text = String::from_utf8(human.stdout).unwrap();
    assert!(human_text.contains("source_to_baseline content"));
    assert!(!human_text.contains(secret));
}

#[test]
fn closed_result_channel_returns_operational_failure() {
    let (root, _, _, _) = file_fixture();
    let mut child = Command::new(env!("CARGO_BIN_EXE_grip"))
        .env_clear()
        .env("HOME", root.path())
        .arg("status")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    assert_eq!(child.wait().unwrap().code(), Some(20));
}

#[test]
fn read_only_commands_are_byte_deterministic_and_mutation_free_for_100_runs() {
    let (root, metadata_dir, _, _) = file_fixture();
    assert!(
        support::project_command(root.path(), &metadata_dir, &["baseline", "accept"])
            .status
            .success()
    );
    let before = support::snapshot(root.path());
    for operation in ["status", "check", "diff"] {
        let expected =
            support::project_command(root.path(), &metadata_dir, &["--output", "json", operation]);
        assert_eq!(expected.status.code(), Some(0));
        for _ in 0..100 {
            let actual = support::project_command(
                root.path(),
                &metadata_dir,
                &["--output", "json", operation],
            );
            assert_eq!(actual.status.code(), Some(0));
            assert_eq!(actual.stdout, expected.stdout);
            assert_eq!(actual.stderr, expected.stderr);
        }
    }
    assert_eq!(support::snapshot(root.path()), before);
}

#[test]
fn human_classification_output_includes_dimensions_and_blocking_reasons() {
    let (root, metadata_dir, source, _) = file_fixture();
    assert!(
        support::project_command(root.path(), &metadata_dir, &["baseline", "accept"])
            .status
            .success()
    );
    fs::write(&source, "changed").unwrap();
    let status = support::project_command(root.path(), &metadata_dir, &["status"]);
    let status_text = String::from_utf8(status.stdout).unwrap();
    assert!(status_text.contains("source_to_baseline content"));
    assert!(status_text.contains("reasons source_changed"));

    let tree_root = tempfile::tempdir().unwrap();
    let tree_home = support::initialize_project_metadata(tree_root.path());
    let tree_source = tree_root.path().join("source");
    let tree_destination = tree_root.path().join("destination");
    fs::create_dir(&tree_source).unwrap();
    fs::create_dir(&tree_destination).unwrap();
    fs::write(tree_source.join("entry"), "same").unwrap();
    fs::write(tree_destination.join("entry"), "same").unwrap();
    support::copy_tree_entry_metadata(&tree_source, &tree_destination);
    support::write_descriptor(&tree_home, &[("tree", &tree_source, &tree_destination)]);
    assert!(
        support::project_command(tree_root.path(), &tree_home, &["baseline", "accept"])
            .status
            .success()
    );
    fs::remove_file(tree_destination.join("entry")).unwrap();
    std::os::unix::fs::symlink("missing", tree_destination.join("entry")).unwrap();
    let check = support::project_command(tree_root.path(), &tree_home, &["check"]);
    let check_text = String::from_utf8(check.stdout).unwrap();
    let check_json =
        support::project_command(tree_root.path(), &tree_home, &["--output", "json", "check"]);
    let check_value = support::json(&check_json);
    let reasons = check_value["details"]["records"]
        .as_array()
        .unwrap_or_else(|| panic!("missing classification records: {check_value}"))
        .iter()
        .flat_map(|record| record["reasons"].as_array().into_iter().flatten())
        .collect::<Vec<_>>();
    assert!(!reasons.is_empty());
    for reason in reasons {
        assert!(check_text.contains(reason.as_str().unwrap()));
    }
    assert!(check_text.contains("source_to_baseline none"));
}

#[test]
fn read_only_commands_reject_unsafe_state_directories() {
    for kind in ["mode", "file", "symlink"] {
        let root = tempfile::tempdir().unwrap();
        let metadata_dir = support::initialize_project_metadata(root.path());
        let state_path = metadata_dir.join("state");
        match kind {
            "mode" => {
                fs::create_dir(&state_path).unwrap();
                fs::set_permissions(
                    &state_path,
                    std::os::unix::fs::PermissionsExt::from_mode(0o755),
                )
                .unwrap();
            }
            "file" => fs::write(&state_path, b"not a directory").unwrap(),
            "symlink" => {
                let outside = tempfile::tempdir().unwrap();
                std::os::unix::fs::symlink(outside.path(), &state_path).unwrap();
                let output = support::project_command(
                    root.path(),
                    &metadata_dir,
                    &["--output", "json", "status"],
                );
                assert_eq!(output.status.code(), Some(12));
                continue;
            }
            _ => unreachable!(),
        }
        let output =
            support::project_command(root.path(), &metadata_dir, &["--output", "json", "status"]);
        assert_eq!(output.status.code(), Some(12));
    }
}
