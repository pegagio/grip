mod support;

use std::fs;
use std::os::unix::fs::symlink;

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

#[test]
fn destination_link_guidance_is_source_only_and_force_requires_the_exact_source() {
    let fixture = support::project::ProjectFixture::initialized();
    let source = fixture.project_root.join("source");
    let destination = fixture.home_destination("destination");
    let target = fixture.root.path().join("target");
    fs::write(&source, "managed\n").unwrap();
    fs::write(&target, "target remains unchanged\n").unwrap();
    symlink(&target, &destination).unwrap();
    assert!(
        fixture
            .command(&["add", "source", "~/destination"])
            .status
            .success()
    );

    let status_json = fixture.command(&["--output=json", "status"]);
    assert!(status_json.status.success());
    let status_json: serde_json::Value = serde_json::from_slice(&status_json.stdout).unwrap();
    assert_eq!(
        status_json["details"]["records"][0]["classification"],
        "unresolved_destination_link"
    );
    assert_eq!(
        status_json["details"]["records"][0]["destination_path"]["display"],
        destination.display().to_string()
    );

    let human_status = fixture.command(&["status"]);
    let human_status = String::from_utf8(human_status.stdout).unwrap();
    assert!(human_status.contains("Replace destination link: grip push --force source"));
    assert!(!human_status.contains("grip pull --force"));

    let ordinary = fixture.command(&["push", "--dry-run"]);
    assert!(!ordinary.status.success());
    let ordinary = String::from_utf8(ordinary.stdout).unwrap();
    assert!(
        ordinary.contains("Replace destination link: grip push --force source"),
        "{ordinary}"
    );
    assert!(
        fixture
            .command(&["push", "--force", "--dry-run", "source"])
            .status
            .success()
    );
    assert!(
        !fixture
            .command(&[
                "push",
                "--force",
                "--dry-run",
                "--destination",
                "~/destination",
            ])
            .status
            .success()
    );
    assert!(
        fs::symlink_metadata(&destination)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        fs::read_to_string(&target).unwrap(),
        "target remains unchanged\n"
    );
}
