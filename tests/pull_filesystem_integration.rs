mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;

#[test]
fn pull_replaces_source_preserves_destination_and_publishes_baseline() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "accepted").unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o600)).unwrap();
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
    assert!(
        support::project_command(root.path(), &metadata_dir, &["push"])
            .status
            .success()
    );
    fs::write(&destination, "destination winner").unwrap();
    fs::set_permissions(&destination, fs::Permissions::from_mode(0o640)).unwrap();
    let destination_before = fs::read(&destination).unwrap();

    let output = support::project_command(root.path(), &metadata_dir, &["--output=json", "pull"]);
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let value = support::json(&output);
    assert_eq!(value["details"]["direction"], "pull");
    assert_eq!(value["details"]["result"], "applied");
    assert_eq!(fs::read(&source).unwrap(), destination_before);
    assert_eq!(fs::read(&destination).unwrap(), destination_before);
    assert_eq!(
        fs::metadata(&source).unwrap().permissions().mode() & 0o7777,
        0o640
    );
    let operation_id = value["details"]["operation_record"]["id"].as_str().unwrap();
    assert!(operation_id.starts_with("pull-"));
    assert_eq!(
        fs::read_to_string(
            metadata_dir
                .join("state/operations")
                .join(operation_id)
                .join("recovery/00000000/payload")
        )
        .unwrap(),
        "accepted"
    );
    let status = support::project_command(root.path(), &metadata_dir, &["--output=json", "status"]);
    assert!(status.status.success());
    assert_eq!(support::json(&status)["details"]["attention_count"], 0);

    let before_noop = support::snapshot(root.path());
    let noop = support::project_command(root.path(), &metadata_dir, &["--output=json", "pull"]);
    assert!(noop.status.success());
    assert_eq!(support::json(&noop)["details"]["result"], "no_op");
    assert_eq!(support::snapshot(root.path()), before_noop);
}

#[test]
fn pull_updates_only_established_tree_members() {
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
    fs::write(destination.join("unmanaged"), "unmanaged").unwrap();
    let output = support::project_command(root.path(), &metadata_dir, &["pull"]);
    assert!(output.status.success());
    assert_eq!(
        fs::read_to_string(source.join("managed")).unwrap(),
        "changed"
    );
    assert!(!source.join("unmanaged").exists());
}

#[test]
fn pull_does_not_recreate_a_missing_source_or_parent() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source_parent = root.path().join("source-parent");
    let source = source_parent.join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source_parent).unwrap();
    fs::write(&source, "accepted").unwrap();
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
    assert!(
        support::project_command(root.path(), &metadata_dir, &["push"])
            .status
            .success()
    );
    fs::remove_file(&source).unwrap();
    fs::remove_dir(&source_parent).unwrap();
    fs::write(&destination, "changed").unwrap();
    let output = support::project_command(root.path(), &metadata_dir, &["--output=json", "pull"]);
    assert!(!output.status.success());
    assert!(!source_parent.exists());
    assert!(
        support::json(&output)["details"]["counts"]["blockers"]
            .as_u64()
            .unwrap()
            > 0
    );
}

#[test]
fn pull_blocks_symlink_substitution_on_either_mapping_side() {
    for replace_source in [false, true] {
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let metadata_dir = support::initialize_project_metadata(root.path());
        let source = root.path().join("source");
        let destination = root.path().join("destination");
        let link_target = root.path().join("link-target");
        fs::write(&source, "accepted").unwrap();
        support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
        assert!(
            support::project_command(root.path(), &metadata_dir, &["push"])
                .status
                .success()
        );
        fs::write(&link_target, "changed").unwrap();
        let substituted = if replace_source {
            &source
        } else {
            &destination
        };
        fs::remove_file(substituted).unwrap();
        symlink(&link_target, substituted).unwrap();
        let before = support::snapshot(root.path());
        let output =
            support::project_command(root.path(), &metadata_dir, &["--output=json", "pull"]);
        assert!(!output.status.success());
        assert_ne!(support::json(&output)["code"], "ok");
        assert_eq!(support::snapshot(root.path()), before);
    }
}
