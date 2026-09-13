mod support;

#[test]
fn aggregate_tree_conflicts_preserve_force_scope_and_machine_contracts() {
    let (root, metadata_dir, source, destination) = support::aggregate_tree_collision_fixture();
    let before = support::snapshot(root.path());

    let status = support::project_command(root.path(), &metadata_dir, &["--output=json", "status"]);
    assert_eq!(status.status.code(), Some(0));
    let status_json = support::json(&status);
    assert_eq!(status_json["schema_version"], 1);
    assert_eq!(status_json["status"], "ok");
    assert!(
        status_json["details"]["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|record| record["classification"] == "initial_collision")
    );
    assert!(
        !serde_json::to_string(&status_json)
            .unwrap()
            .contains("grip diff"),
        "human guidance must not change JSON output"
    );

    let status_exit =
        support::project_command(root.path(), &metadata_dir, &["status", "--exit-code"]);
    assert_eq!(status_exit.status.code(), Some(1));

    let diff = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "diff", "source/nested/file"],
    );
    assert_eq!(diff.status.code(), Some(0));
    assert_eq!(support::json(&diff)["details"]["operation"], "diff");

    let forced_push = support::project_command(
        root.path(),
        &metadata_dir,
        &["push", "--force", "--dry-run", "source"],
    );
    assert_eq!(forced_push.status.code(), Some(10));
    assert!(
        String::from_utf8(forced_push.stdout)
            .unwrap()
            .contains("requires one exact established managed entry")
    );

    let forced_pull = support::project_command(
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
    assert_eq!(forced_pull.status.code(), Some(10));
    assert!(
        String::from_utf8(forced_pull.stdout)
            .unwrap()
            .contains("requires one exact established managed entry")
    );
    assert_eq!(support::snapshot(root.path()), before);
    assert!(source.exists());
}

#[test]
fn technical_blockers_do_not_receive_invented_guidance() {
    let mut outcome = grip::CommandOutcome::success("Resolution blocked");
    outcome.category = grip::ResultCategory::InvalidConfiguration;
    outcome.details = serde_json::json!({
        "operation": "push",
        "result": "blocked",
        "counts": {"selected": 1, "actions": 0, "blockers": 1},
        "blockers": [{"reason": "target_capability_unavailable", "paths": []}]
    })
    .as_object()
    .unwrap()
    .clone();
    let mut human = Vec::new();
    grip::result::render(outcome, grip::result::OutputMode::Human, &mut human).unwrap();
    let text = String::from_utf8(human).unwrap();
    assert!(text.contains("Run: grip status"));
    assert!(!text.contains("grip push --force"));
    assert!(!text.contains("grip pull --force"));
    assert!(!text.contains("grip diff"));
}
