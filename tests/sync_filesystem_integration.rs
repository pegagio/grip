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
fn exact_initial_match_sync_accepts_one_unbaselined_file_without_copying() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
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

    let before_source = fs::read(&source).unwrap();
    let before_destination = fs::read(&destination).unwrap();
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "sync", "source"],
    );
    assert!(output.status.success());
    let value = support::json(&output);
    assert_eq!(value["details"]["counts"]["actions"], 0);
    assert_eq!(value["details"]["counts"]["converged"], 1);
    assert_eq!(fs::read(&source).unwrap(), before_source);
    assert_eq!(fs::read(&destination).unwrap(), before_destination);
    let status = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "status", "source"],
    );
    assert_eq!(
        support::json(&status)["details"]["records"][0]["classification"],
        "synchronized"
    );
}

#[test]
fn selected_tree_child_initial_match_dry_run_then_accepts_only_that_child() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    for relative in ["nested/selected", "nested/sibling"] {
        let source_path = source.join(relative);
        let destination_path = destination.join(relative);
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        fs::create_dir_all(destination_path.parent().unwrap()).unwrap();
        fs::write(&source_path, relative).unwrap();
        fs::write(&destination_path, relative).unwrap();
        support::copy_complete_metadata(
            &source_path,
            &destination_path,
            grip::discovery::model::NodeKind::File,
        );
    }
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);

    let source_before = support::snapshot(&source);
    let destination_before = support::snapshot(&destination);
    let preview = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output",
            "json",
            "sync",
            "--dry-run",
            "source/nested/selected",
        ],
    );
    assert!(preview.status.success());
    let preview_json = support::json(&preview);
    assert_eq!(preview_json["details"]["result"], "planned");
    assert_eq!(preview_json["details"]["counts"]["actions"], 0);
    assert_eq!(preview_json["details"]["counts"]["converged"], 1);
    assert_eq!(support::snapshot(&source), source_before);
    assert_eq!(support::snapshot(&destination), destination_before);
    assert!(!metadata_dir.join("state/state.json").exists());

    let completed = support::project_command(
        root.path(),
        &metadata_dir,
        &["sync", "source/nested/selected"],
    );
    assert!(completed.status.success());
    assert_eq!(
        String::from_utf8(completed.stdout).unwrap(),
        "Established a baseline for 1 file(s).\n"
    );
    assert_eq!(support::snapshot(&source), source_before);
    assert_eq!(support::snapshot(&destination), destination_before);

    let selected_status = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "status", "source/nested/selected"],
    );
    assert_eq!(
        support::json(&selected_status)["details"]["records"][0]["classification"],
        "synchronized"
    );
    let sibling_status = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "status", "source/nested/sibling"],
    );
    assert_eq!(
        support::json(&sibling_status)["details"]["records"][0]["classification"],
        "initial_match"
    );
}

#[test]
fn broad_tree_initial_match_scopes_do_not_publish_baselines() {
    for selector in [None, Some("source"), Some("source/nested")] {
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let metadata_dir = support::initialize_project_metadata(root.path());
        let source = root.path().join("source");
        let destination = root.path().join("destination");
        for relative in ["nested/first", "nested/second"] {
            let source_path = source.join(relative);
            let destination_path = destination.join(relative);
            fs::create_dir_all(source_path.parent().unwrap()).unwrap();
            fs::create_dir_all(destination_path.parent().unwrap()).unwrap();
            fs::write(&source_path, relative).unwrap();
            fs::write(&destination_path, relative).unwrap();
            support::copy_complete_metadata(
                &source_path,
                &destination_path,
                grip::discovery::model::NodeKind::File,
            );
        }
        support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);
        let source_before = support::snapshot(&source);
        let destination_before = support::snapshot(&destination);
        let mut arguments = vec!["--output", "json", "sync"];
        if let Some(selector) = selector {
            arguments.push(selector);
        }

        let output = support::project_command(root.path(), &metadata_dir, &arguments);
        assert!(output.status.success());
        assert_eq!(support::json(&output)["details"]["result"], "no_op");
        assert!(!metadata_dir.join("state/state.json").exists());
        assert_eq!(support::snapshot(&source), source_before);
        assert_eq!(support::snapshot(&destination), destination_before);
    }
}

#[test]
fn selected_initial_collision_does_not_publish_a_baseline() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir_all(source.join("nested")).unwrap();
    fs::create_dir_all(destination.join("nested")).unwrap();
    fs::write(source.join("nested/file"), "source").unwrap();
    fs::write(destination.join("nested/file"), "destination").unwrap();
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);
    let source_before = support::snapshot(&source);
    let destination_before = support::snapshot(&destination);

    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "sync", "source/nested/file"],
    );
    assert_eq!(output.status.code(), Some(10));
    assert_eq!(support::json(&output)["details"]["result"], "blocked");
    assert!(!metadata_dir.join("state/state.json").exists());
    assert_eq!(support::snapshot(&source), source_before);
    assert_eq!(support::snapshot(&destination), destination_before);
}

#[test]
fn ignored_and_destination_only_tree_entries_do_not_publish_baselines() {
    let ignored_root = tempfile::tempdir_in("/private/tmp").unwrap();
    let ignored_metadata_dir = support::initialize_project_metadata(ignored_root.path());
    let ignored_source = ignored_root.path().join("source");
    let ignored_destination = ignored_root.path().join("destination");
    fs::create_dir_all(ignored_source.join("nested")).unwrap();
    fs::create_dir_all(ignored_destination.join("nested")).unwrap();
    fs::write(ignored_source.join("nested/file"), "same").unwrap();
    fs::write(ignored_destination.join("nested/file"), "same").unwrap();
    support::copy_complete_metadata(
        &ignored_source.join("nested/file"),
        &ignored_destination.join("nested/file"),
        grip::discovery::model::NodeKind::File,
    );
    fs::write(ignored_source.join(".gripignore"), "nested/file\n").unwrap();
    support::write_descriptor(
        &ignored_metadata_dir,
        &[("tree", &ignored_source, &ignored_destination)],
    );
    let ignored_before = support::snapshot(ignored_root.path());
    let ignored = support::project_command(
        ignored_root.path(),
        &ignored_metadata_dir,
        &["--output", "json", "sync", "source/nested/file"],
    );
    assert!(ignored.status.success());
    assert_eq!(support::json(&ignored)["details"]["result"], "no_op");
    assert!(!ignored_metadata_dir.join("state/state.json").exists());
    assert_eq!(support::snapshot(ignored_root.path()), ignored_before);

    let destination_only_root = tempfile::tempdir_in("/private/tmp").unwrap();
    let destination_only_metadata_dir =
        support::initialize_project_metadata(destination_only_root.path());
    let destination_only_source = destination_only_root.path().join("source");
    let destination_only_destination = destination_only_root.path().join("destination");
    fs::create_dir_all(&destination_only_source).unwrap();
    fs::create_dir_all(destination_only_destination.join("nested")).unwrap();
    fs::write(
        destination_only_destination.join("nested/file"),
        "destination only",
    )
    .unwrap();
    support::write_descriptor(
        &destination_only_metadata_dir,
        &[(
            "tree",
            &destination_only_source,
            &destination_only_destination,
        )],
    );
    let destination_only_before = support::snapshot(destination_only_root.path());
    let destination_only = support::project_command(
        destination_only_root.path(),
        &destination_only_metadata_dir,
        &["--output", "json", "sync", "source/nested/file"],
    );
    assert!(destination_only.status.success());
    assert_eq!(
        support::json(&destination_only)["details"]["result"],
        "no_op"
    );
    assert!(
        !destination_only_metadata_dir
            .join("state/state.json")
            .exists()
    );
    assert_eq!(
        support::snapshot(destination_only_root.path()),
        destination_only_before
    );

    let missing_peer_root = tempfile::tempdir_in("/private/tmp").unwrap();
    let missing_peer_metadata_dir = support::initialize_project_metadata(missing_peer_root.path());
    let missing_peer_source = missing_peer_root.path().join("source");
    let missing_peer_destination = missing_peer_root.path().join("destination");
    fs::create_dir_all(missing_peer_source.join("nested")).unwrap();
    fs::write(missing_peer_source.join("nested/file"), "source only").unwrap();
    support::write_descriptor(
        &missing_peer_metadata_dir,
        &[("tree", &missing_peer_source, &missing_peer_destination)],
    );
    let missing_peer = support::project_command(
        missing_peer_root.path(),
        &missing_peer_metadata_dir,
        &[
            "--output",
            "json",
            "sync",
            "--dry-run",
            "source/nested/file",
        ],
    );
    assert!(missing_peer.status.success());
    assert!(
        support::json(&missing_peer)["details"]["counts"]["actions"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert_eq!(
        support::json(&missing_peer)["details"]["counts"]["converged"],
        0
    );
    assert!(!missing_peer_metadata_dir.join("state/state.json").exists());
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
