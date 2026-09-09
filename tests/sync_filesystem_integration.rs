mod support;

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};

#[test]
fn sync_executes_mixed_directions_and_publishes_one_generation() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source_a = root.path().join("source-a");
    let destination_a = root.path().join("destination-a");
    let source_b = root.path().join("source-b");
    let destination_b = root.path().join("destination-b");
    fs::write(&source_a, "accepted-a").unwrap();
    fs::write(&source_b, "accepted-b").unwrap();
    support::write_descriptor(
        &metadata_dir,
        &[
            ("file", &source_a, &destination_a),
            ("file", &source_b, &destination_b),
        ],
    );
    assert!(
        support::project_command(root.path(), &metadata_dir, &["push"])
            .status
            .success()
    );
    fs::write(&source_a, "source change").unwrap();
    fs::write(&destination_b, "destination change").unwrap();

    let output =
        support::project_command(root.path(), &metadata_dir, &["--output", "json", "sync"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value = support::json(&output);
    assert_eq!(value["details"]["operation"], "sync");
    assert_eq!(value["details"]["baseline"]["published_generation"], 1);
    assert_eq!(fs::read_to_string(destination_a).unwrap(), "source change");
    assert_eq!(fs::read_to_string(source_b).unwrap(), "destination change");
}

#[test]
fn converged_only_sync_records_and_accepts_state_then_becomes_noop() {
    let (root, metadata_dir, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "converged").unwrap();
    fs::write(&destination, "converged").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    let first = support::project_command(root.path(), &metadata_dir, &["--output", "json", "sync"]);
    assert!(first.status.success());
    let value = support::json(&first);
    assert_eq!(value["details"]["counts"]["actions"], 0);
    assert_eq!(value["details"]["counts"]["converged"], 1);
    assert_eq!(value["details"]["baseline"]["published_generation"], 1);
    assert!(
        value["details"]["operation_record"]["available"]
            .as_bool()
            .unwrap()
    );
    let operations_before = fs::read_dir(metadata_dir.join("state/operations"))
        .unwrap()
        .count();
    let state_before = fs::read(metadata_dir.join("state/state.json")).unwrap();

    let second =
        support::project_command(root.path(), &metadata_dir, &["--output", "json", "sync"]);
    assert!(second.status.success());
    assert_eq!(support::json(&second)["details"]["result"], "no_op");
    assert_eq!(
        fs::read(metadata_dir.join("state/state.json")).unwrap(),
        state_before
    );
    assert_eq!(
        fs::read_dir(metadata_dir.join("state/operations"))
            .unwrap()
            .count(),
        operations_before
    );
}

#[test]
fn sync_adds_a_nested_source_entry_with_private_parent_creation() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source-tree");
    let destination = root.path().join("destination-tree");
    fs::create_dir(&source).unwrap();
    fs::create_dir(source.join("nested")).unwrap();
    fs::write(source.join("nested/config"), "source addition").unwrap();
    fs::set_permissions(
        source.join("nested/config"),
        fs::Permissions::from_mode(0o640),
    )
    .unwrap();
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);

    let output = support::project_command(root.path(), &metadata_dir, &["--output=json", "sync"]);
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(destination.join("nested/config")).unwrap(),
        "source addition"
    );
    assert_eq!(
        fs::metadata(destination.join("nested/config"))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        0o640
    );
    let value = support::json(&output);
    let actions = value["details"]["actions"].as_array().unwrap();
    assert!(
        actions
            .iter()
            .any(|action| action["kind"] == "create_parent_directory")
    );
    assert!(actions.iter().any(|action| action["kind"] == "add_file"));
    assert_eq!(value["details"]["baseline"]["published_generation"], 0);
}

#[test]
fn sync_blocks_a_symlink_target_without_following_or_changing_it() {
    let (root, metadata_dir, source, destination) = support::accepted_file_fixture();
    let sentinel = root.path().join("sentinel");
    fs::write(&source, "source change").unwrap();
    fs::write(&sentinel, "outside mapping").unwrap();
    fs::remove_file(&destination).unwrap();
    symlink(&sentinel, &destination).unwrap();
    let before_state = fs::read(metadata_dir.join("state/state.json")).unwrap();

    let output = support::project_command(root.path(), &metadata_dir, &["--output=json", "sync"]);
    assert_eq!(output.status.code(), Some(10));
    let value = support::json(&output);
    assert_eq!(value["status"], "error");
    assert_eq!(value["code"], "invalid_configuration");
    assert!(
        fs::symlink_metadata(&destination)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read_to_string(&sentinel).unwrap(), "outside mapping");
    assert_eq!(
        fs::read(metadata_dir.join("state/state.json")).unwrap(),
        before_state
    );
}
