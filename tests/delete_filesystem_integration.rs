mod support;
use std::fs;

#[test]
fn deletion_classifies_a_cross_device_directory_as_a_mount_boundary() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let directory = root.path().join("managed");
    fs::create_dir(&directory).unwrap();
    let metadata = grip::discovery::filesystem::metadata_at_path(&directory).unwrap();
    let mut cross_device = metadata.stat;
    cross_device.st_dev = cross_device.st_dev.saturating_add(1);

    let kind = grip::discovery::filesystem::classify(&cross_device, metadata.stat.st_dev as u64);
    assert_eq!(kind, grip::discovery::model::NodeKind::NestedMount);
    assert_eq!(
        grip::discovery::filesystem::unsupported_reason(kind),
        Some("nested_mount")
    );
}

#[test]
fn deletion_removes_only_the_authorized_unchanged_peer() {
    let (root, metadata_dir, source, destination) = support::accepted_file_fixture();
    let neighbor = destination.with_extension("neighbor");
    fs::write(&neighbor, "keep").unwrap();
    fs::remove_file(&source).unwrap();
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "delete", "--source", "source"],
    );
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!destination.exists());
    assert_eq!(fs::read_to_string(neighbor).unwrap(), "keep");
}

#[test]
fn directory_deletion_blocks_unmanaged_descendants_then_removes_child_first() {
    let (root, metadata_dir, source, destination) = support::accepted_tree_fixture();
    fs::remove_dir_all(&source).unwrap();
    let unmanaged = destination.join("unmanaged");
    fs::write(&unmanaged, "keep").unwrap();
    let before = support::snapshot(root.path());
    let blocked = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "delete", "--source", "source"],
    );
    assert!(!blocked.status.success());
    support::assert_snapshot_unchanged(&before, root.path());
    fs::remove_file(unmanaged).unwrap();
    let applied = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "delete", "--source", "source"],
    );
    assert!(
        applied.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&applied.stdout),
        String::from_utf8_lossy(&applied.stderr)
    );
    assert!(destination.exists());
    assert!(
        !destination.join("nested").exists(),
        "{}",
        String::from_utf8_lossy(&applied.stdout)
    );
    let operation = support::json(&applied)["details"]["operation_record"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let recovery = metadata_dir
        .join("state/operations")
        .join(operation)
        .join("recovery");
    assert!(recovery.join("00000000/payload").is_file());
    assert!(recovery.join("00000001/manifest.json").is_file());
    assert!(!recovery.join("00000001/payload").exists());
}
