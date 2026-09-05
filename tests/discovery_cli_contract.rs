mod support;

use serde_json::Value;
use std::fs;

#[test]
fn mapping_inspect_reports_an_empty_all_mapping_inventory() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output", "json", "mapping", "inspect"],
    );
    assert_eq!(output.status.code(), Some(0));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["details"]["operation"], "mapping_inspect");
    assert_eq!(result["details"]["scope"]["kind"], "all");
    assert_eq!(result["details"]["blocking_count"], 0);
    assert_eq!(result["details"]["counts"]["eligible"], 0);
    assert_eq!(result["details"]["records"], serde_json::json!([]));
}

#[test]
fn mapping_inspect_accepts_one_source_and_rejects_extra_selectors() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "payload").unwrap();
    support::write_registry(&grip_home, &[("file", &source, &destination)]);

    let selected = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output",
            "json",
            "mapping",
            "inspect",
            "--",
            source.to_str().unwrap(),
        ],
    );
    assert_eq!(selected.status.code(), Some(0));
    let result: Value = serde_json::from_slice(&selected.stdout).unwrap();
    assert_eq!(result["details"]["scope"]["kind"], "mapping");
    assert_eq!(
        result["details"]["scope"]["source"],
        fs::canonicalize(&source).unwrap().to_str().unwrap()
    );

    let extra = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["mapping", "inspect", source.to_str().unwrap(), "/extra"],
    );
    assert_eq!(extra.status.code(), Some(2));
}

#[test]
fn mapping_inspect_human_output_is_an_inventory_not_a_registry_listing() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let output = support::command_with_grip_home(root.path(), &grip_home, &["mapping", "inspect"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("Inspected 0 entries")
    );
}

#[test]
fn mapping_inspect_human_output_renders_stable_record_fields() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("visible"), "payload").unwrap();
    support::write_registry(&grip_home, &[("tree", &source, &destination)]);
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["mapping", "inspect", source.to_str().unwrap()],
    );
    assert_eq!(output.status.code(), Some(0));
    let human = String::from_utf8(output.stdout).unwrap();
    assert!(human.starts_with("Inspected 1 entries; 0 blocking finding(s)\n"));
    assert!(human.contains("eligible"));
    assert!(human.contains(" visible -> "));
    assert!(human.ends_with(" file\n"));
}

#[test]
fn mapping_inspect_keeps_json_results_and_diagnostics_on_separate_channels() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output", "json", "-v", "mapping", "inspect"],
    );
    assert_eq!(output.status.code(), Some(0));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["details"]["operation"], "mapping_inspect");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "diagnostic: command completed\n"
    );
}
