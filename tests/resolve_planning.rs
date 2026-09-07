mod support;

use std::fs;

#[test]
fn resolution_winner_determines_transfer_direction_and_plan_identity() {
    let (root, grip_home, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "source change").unwrap();
    fs::write(&destination, "destination change").unwrap();
    let source_plan = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output",
            "json",
            "resolve",
            "-n",
            "--source",
            "--",
            source.to_str().unwrap(),
        ],
    );
    let destination_plan = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output",
            "json",
            "resolve",
            "-n",
            "--destination",
            "--",
            source.to_str().unwrap(),
        ],
    );
    let source_plan = support::json(&source_plan);
    let destination_plan = support::json(&destination_plan);
    assert_eq!(source_plan["details"]["actions"][0]["direction"], "push");
    assert_eq!(
        destination_plan["details"]["actions"][0]["direction"],
        "pull"
    );
    assert_ne!(
        source_plan["details"]["plan_id"],
        destination_plan["details"]["plan_id"]
    );
}

#[test]
fn resolution_rejects_a_non_conflicting_entry() {
    let (root, grip_home, source, _) = support::accepted_file_fixture();
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output",
            "json",
            "resolve",
            "--source",
            "--",
            source.to_str().unwrap(),
        ],
    );
    assert_eq!(output.status.code(), Some(10));
    assert_eq!(support::json(&output)["details"]["result"], "blocked");
}

#[test]
fn resolution_rejects_each_supported_non_conflict_change_shape() {
    for shape in ["source_only", "destination_only", "converged"] {
        let (root, grip_home, source, destination) = support::accepted_file_fixture();
        match shape {
            "source_only" => fs::write(&source, "source change").unwrap(),
            "destination_only" => fs::write(&destination, "destination change").unwrap(),
            "converged" => {
                fs::write(&source, "same change").unwrap();
                fs::write(&destination, "same change").unwrap();
            }
            _ => unreachable!(),
        }
        let output = support::command_with_grip_home(
            root.path(),
            &grip_home,
            &[
                "--output=json",
                "resolve",
                "--source",
                "--",
                source.to_str().unwrap(),
            ],
        );
        assert_eq!(output.status.code(), Some(10), "shape={shape}");
        assert_eq!(support::json(&output)["details"]["result"], "blocked");
    }
}

#[test]
fn resolution_rejects_a_mapping_or_subtree_that_is_not_one_exact_entry() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source-tree");
    let destination = root.path().join("destination-tree");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("one"), "one").unwrap();
    fs::write(source.join("two"), "two").unwrap();
    support::write_registry(&grip_home, &[("tree", &source, &destination)]);
    assert!(
        support::command_with_grip_home(root.path(), &grip_home, &["push"])
            .status
            .success()
    );
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output=json",
            "resolve",
            "--source",
            "--",
            source.to_str().unwrap(),
        ],
    );
    assert_eq!(output.status.code(), Some(10));
    assert_eq!(
        support::json(&output)["details"]["reason"],
        "resolution_requires_exact_entry"
    );
}
