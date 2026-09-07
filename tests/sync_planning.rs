mod support;

use std::fs;

#[test]
fn sync_plan_is_deterministic_and_contains_both_directions() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let source_a = root.path().join("source-a");
    let destination_a = root.path().join("destination-a");
    let source_b = root.path().join("source-b");
    let destination_b = root.path().join("destination-b");
    fs::write(&source_a, "accepted-a").unwrap();
    fs::write(&source_b, "accepted-b").unwrap();
    support::write_registry(
        &grip_home,
        &[
            ("file", &source_b, &destination_b),
            ("file", &source_a, &destination_a),
        ],
    );
    assert!(
        support::command_with_grip_home(root.path(), &grip_home, &["push"])
            .status
            .success()
    );
    fs::write(&source_a, "source change").unwrap();
    fs::write(&destination_b, "destination change").unwrap();

    let first = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output", "json", "sync", "-n"],
    );
    let second = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output", "json", "sync", "--dry-run"],
    );
    let first = support::json(&first);
    let second = support::json(&second);
    assert_eq!(first["details"]["plan_id"], second["details"]["plan_id"]);
    assert_eq!(first["details"]["actions"], second["details"]["actions"]);
    let directions = first["details"]["actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|action| action["direction"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(directions, ["push", "pull"]);
}

#[test]
fn one_conflict_blocks_the_complete_mixed_plan() {
    let (root, grip_home, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "source change").unwrap();
    fs::write(&destination, "destination change").unwrap();
    let output =
        support::command_with_grip_home(root.path(), &grip_home, &["--output", "json", "sync"]);
    assert_eq!(output.status.code(), Some(10));
    let value = support::json(&output);
    assert_eq!(value["details"]["result"], "blocked");
    assert_eq!(value["details"]["counts"]["blockers"], 1);
    assert_eq!(fs::read_to_string(source).unwrap(), "source change");
    assert_eq!(
        fs::read_to_string(destination).unwrap(),
        "destination change"
    );
}

#[test]
fn unmanaged_destination_content_remains_a_non_action() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source-tree");
    let destination = root.path().join("destination-tree");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("managed"), "accepted").unwrap();
    support::write_registry(&grip_home, &[("tree", &source, &destination)]);
    assert!(
        support::command_with_grip_home(root.path(), &grip_home, &["push"])
            .status
            .success()
    );
    fs::write(destination.join("unmanaged"), "destination only").unwrap();
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "sync", "--dry-run"],
    );
    assert!(output.status.success());
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
