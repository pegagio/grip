mod support;

use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::{PermissionsExt, symlink};

fn classification(output: &std::process::Output) -> String {
    support::json(output)["details"]["records"][0]["classification"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn content_mode_timestamp_and_deletion_transitions_are_classified() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "accepted").unwrap();
    fs::write(&destination, "accepted").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
    assert_eq!(
        support::project_command(root.path(), &metadata_dir, &["baseline", "accept"])
            .status
            .code(),
        Some(0)
    );

    fs::write(&source, "accepted").unwrap();
    let timestamp_only =
        support::project_command(root.path(), &metadata_dir, &["--output", "json", "status"]);
    assert_eq!(classification(&timestamp_only), "source_only_change");
    assert_eq!(
        support::json(&timestamp_only)["details"]["records"][0]["changed_dimensions"]["source_to_baseline"],
        serde_json::json!(["modification_time"])
    );

    support::copy_complete_metadata(
        &destination,
        &source,
        grip::discovery::model::NodeKind::File,
    );
    fs::write(&source, "changed").unwrap();
    let content =
        support::project_command(root.path(), &metadata_dir, &["--output", "json", "status"]);
    assert_eq!(classification(&content), "source_only_change");
    assert_eq!(
        support::json(&content)["details"]["records"][0]["changed_dimensions"]["source_to_baseline"],
        serde_json::json!(["content", "modification_time"])
    );

    fs::write(&source, "accepted").unwrap();
    support::copy_complete_metadata(
        &destination,
        &source,
        grip::discovery::model::NodeKind::File,
    );
    fs::set_permissions(&source, fs::Permissions::from_mode(0o600)).unwrap();
    let mode =
        support::project_command(root.path(), &metadata_dir, &["--output", "json", "status"]);
    assert_eq!(classification(&mode), "source_only_change");
    assert_eq!(
        support::json(&mode)["details"]["records"][0]["changed_dimensions"]["source_to_baseline"],
        serde_json::json!(["permission_mode"])
    );

    fs::remove_file(&source).unwrap();
    let deleted =
        support::project_command(root.path(), &metadata_dir, &["--output", "json", "status"]);
    assert_eq!(classification(&deleted), "source_side_deletion");
}

#[test]
fn removed_mapping_baseline_is_untracked_without_reopening_payloads() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "accepted").unwrap();
    fs::write(&destination, "accepted").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
    support::project_command(root.path(), &metadata_dir, &["baseline", "accept"]);
    let removed =
        support::project_command(root.path(), &metadata_dir, &["mapping", "remove", "source"]);
    assert_eq!(removed.status.code(), Some(0));
    fs::remove_file(&source).unwrap();
    fs::remove_file(&destination).unwrap();
    let status =
        support::project_command(root.path(), &metadata_dir, &["--output", "json", "status"]);
    assert_eq!(classification(&status), "untracked_pending_retirement");
}

#[test]
fn newly_ignored_baseline_and_unsafe_destination_remain_visible() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&destination).unwrap();
    fs::write(source.join("file"), "accepted").unwrap();
    fs::write(destination.join("file"), "accepted").unwrap();
    support::copy_tree_entry_metadata(&source, &destination);
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);
    let accepted = support::project_command(root.path(), &metadata_dir, &["baseline", "accept"]);
    assert_eq!(
        accepted.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    fs::write(source.join(".gripignore"), "file\n").unwrap();
    let ignored =
        support::project_command(root.path(), &metadata_dir, &["--output", "json", "status"]);
    assert_eq!(classification(&ignored), "newly_ignored_pending_retirement");

    fs::remove_file(source.join(".gripignore")).unwrap();
    fs::remove_file(destination.join("file")).unwrap();
    symlink("missing", destination.join("file")).unwrap();
    let unsafe_result =
        support::project_command(root.path(), &metadata_dir, &["--output", "json", "status"]);
    assert_eq!(classification(&unsafe_result), "unsafe_collision");
    assert_eq!(
        support::json(&unsafe_result)["details"]["blocking_count"],
        1
    );
}

#[test]
fn mapped_root_deletion_and_non_utf8_source_are_reported_without_loss() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&destination).unwrap();
    fs::write(source.join("file"), "accepted").unwrap();
    fs::write(destination.join("file"), "accepted").unwrap();
    support::copy_tree_entry_metadata(&source, &destination);
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);
    assert_eq!(
        support::project_command(root.path(), &metadata_dir, &["baseline", "accept"])
            .status
            .code(),
        Some(0)
    );
    fs::remove_dir_all(&source).unwrap();
    let deleted =
        support::project_command(root.path(), &metadata_dir, &["--output", "json", "status"]);
    assert_eq!(deleted.status.code(), Some(0));
    assert_eq!(classification(&deleted), "source_side_deletion");

    fs::create_dir(&source).unwrap();
    let raw = std::ffi::OsString::from_vec(vec![b'r', 0xff, b'w']);
    let raw_created = fs::write(source.join(&raw), "unsupported").is_ok();
    let raw_status =
        support::project_command(root.path(), &metadata_dir, &["--output", "json", "status"]);
    let value = support::json(&raw_status);
    if raw_created {
        assert!(
            value["details"]["records"]
                .as_array()
                .unwrap()
                .iter()
                .any(|record| {
                    record["classification"] == "unsupported_managed"
                        && record["relative_path"]["raw_hex"] == "72ff77"
                })
        );
    }
}
