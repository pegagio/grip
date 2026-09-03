mod support;
use std::fs;

#[test]
fn help_and_version_flags_do_not_mutate_home() {
    let root = tempfile::tempdir().unwrap();
    let before = support::snapshot(root.path());
    for arg in ["--help", "--version"] {
        let output = support::command(root.path(), &[arg]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
    }
    assert_eq!(before, support::snapshot(root.path()));
}

#[test]
fn version_supports_human_and_json_results() {
    let root = tempfile::tempdir().unwrap();
    let human = support::command(root.path(), &["version"]);
    assert!(human.status.success());
    assert!(String::from_utf8_lossy(&human.stdout).starts_with("grip "));
    let json = support::command(root.path(), &["--output", "json", "version"]);
    let value: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "ok");
    assert_eq!(value["details"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(value.as_object().unwrap().len(), 5);
}

#[test]
fn validate_reports_success_and_public_error_categories() {
    let root = tempfile::tempdir().unwrap();
    let grip = support::minimal_home(root.path());
    let ok = support::command_with_grip_home(root.path(), &grip, &["--output=json", "validate"]);
    assert_eq!(ok.status.code(), Some(0));
    let value = serde_json::from_slice::<serde_json::Value>(&ok.stdout).unwrap();
    assert_eq!(value["code"], "ok");
    assert_eq!(value["details"]["grip_home"], grip.display().to_string());
    fs::write(
        grip.join("config.toml"),
        "schema_version = 2\nmappings = []\n",
    )
    .unwrap();
    let unsupported =
        support::command_with_grip_home(root.path(), &grip, &["--output=json", "validate"]);
    assert_eq!(unsupported.status.code(), Some(11));
    fs::write(grip.join("config.toml"), "bad").unwrap();
    let invalid =
        support::command_with_grip_home(root.path(), &grip, &["--output=json", "validate"]);
    assert_eq!(invalid.status.code(), Some(10));
}

#[test]
fn json_usage_errors_and_human_parser_errors_are_separated() {
    let root = tempfile::tempdir().unwrap();
    let json = support::command(root.path(), &["--output=json", "unknown"]);
    assert_eq!(json.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&json.stdout).unwrap()["code"],
        "invalid_usage"
    );
    let malformed = support::command(root.path(), &["--output", "wat", "version"]);
    assert_eq!(malformed.status.code(), Some(2));
    assert!(malformed.stdout.is_empty());
    assert!(!malformed.stderr.is_empty());
}

#[test]
fn diagnostics_are_opt_in_and_stderr_only() {
    let root = tempfile::tempdir().unwrap();
    let quiet = support::command(root.path(), &["version"]);
    assert!(quiet.stderr.is_empty());
    let verbose = support::command(root.path(), &["-v", "version"]);
    assert!(String::from_utf8_lossy(&verbose.stderr).contains("diagnostic:"));
    assert!(String::from_utf8_lossy(&verbose.stdout).starts_with("grip "));
}
