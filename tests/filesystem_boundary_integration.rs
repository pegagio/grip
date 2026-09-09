mod support;

use grip::discovery::model::NodeKind;
use serde_json::Value;
use std::ffi::CString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixListener;

#[test]
fn non_following_discovery_reports_special_nodes_and_preserves_symlink_referent() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&destination).unwrap();

    let sentinel = root.path().join("sentinel");
    fs::write(&sentinel, b"must remain unread and unchanged").unwrap();
    std::os::unix::fs::symlink(&sentinel, source.join("link")).unwrap();
    let socket = UnixListener::bind(source.join("socket")).ok();
    let fifo = source.join("fifo");
    let fifo_name = CString::new(fifo.as_os_str().as_bytes()).unwrap();
    // SAFETY: fifo_name is a valid, NUL-terminated path inside the disposable fixture.
    assert_eq!(unsafe { libc::mkfifo(fifo_name.as_ptr(), 0o600) }, 0);
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);

    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "inspect"],
    );
    assert!(output.status.success());
    let value: Value = support::json(&output);
    let records = value["details"]["records"].as_array().unwrap();
    let mut expected = vec![("link", "symlink"), ("fifo", "fifo")];
    if socket.is_some() {
        expected.push(("socket", "socket"));
    }
    for (name, reason) in expected {
        assert!(records.iter().any(|record| {
            record["relative_path"]["display"] == name
                && record["category"] == "unsupported_source"
                && record["reason"] == reason
                && record["blocking"] == true
        }));
    }
    assert_eq!(
        fs::read(&sentinel).unwrap(),
        b"must remain unread and unchanged"
    );
}

#[test]
fn raw_node_classification_covers_uncreatable_special_nodes_and_mount_boundaries() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let file = root.path().join("file");
    fs::write(&file, b"payload").unwrap();
    let metadata = grip::discovery::filesystem::metadata_at_path(&file).unwrap();
    let mut stat = metadata.stat;
    let device = stat.st_dev as u64;

    for (mode, expected) in [
        (libc::S_IFCHR as _, NodeKind::CharacterDevice),
        (libc::S_IFBLK as _, NodeKind::BlockDevice),
        (0, NodeKind::UnknownSpecial),
    ] {
        stat.st_mode = mode;
        assert_eq!(
            grip::discovery::filesystem::classify(&stat, device),
            expected
        );
    }
    stat.st_mode = libc::S_IFDIR as _;
    assert_eq!(
        grip::discovery::filesystem::classify(&stat, device.saturating_add(1)),
        NodeKind::NestedMount
    );
    #[cfg(target_os = "macos")]
    {
        stat.st_mode = 0o160000;
        assert_eq!(
            grip::discovery::filesystem::classify(&stat, device),
            NodeKind::Whiteout
        );
    }
}

#[test]
fn hard_link_and_sparse_evidence_are_authoritative_and_blocking() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&destination).unwrap();
    fs::write(source.join("hard-a"), b"payload").unwrap();
    fs::hard_link(source.join("hard-a"), source.join("hard-b")).unwrap();
    let sparse = fs::File::create(source.join("sparse")).unwrap();
    sparse.set_len(1024 * 1024).unwrap();
    drop(sparse);
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);

    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "inspect"],
    );
    let records = support::json(&output)["details"]["records"]
        .as_array()
        .unwrap()
        .clone();
    assert!(
        records
            .iter()
            .filter(|record| record["reason"] == "hard_link")
            .count()
            == 2
    );
    assert!(
        records
            .iter()
            .any(|record| record["reason"] == "sparse_file")
    );
    assert!(
        records
            .iter()
            .filter(|record| record["blocking"] == true)
            .count()
            >= 3
    );
}

#[test]
#[allow(clippy::result_large_err)]
fn post_plan_link_count_drift_stops_before_payload_mutation() {
    let (root, home, registry, state, selection, plan) = support::pull_execution_fixture();
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    let source_before = fs::read(&source).unwrap();
    let alias = root.path().join("destination-alias");
    let result = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        |phase| {
            if phase == grip::mutation::FaultPhase::BeforeActionRevalidation(0) {
                fs::hard_link(&destination, &alias).unwrap();
            }
            Ok(())
        },
    );
    assert!(result.is_err());
    assert_eq!(fs::read(source).unwrap(), source_before);
    assert_eq!(fs::metadata(destination).unwrap().nlink(), 2);
}

#[test]
fn filesystem_identity_change_is_classified_as_mount_status_drift() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let directory = root.path().join("directory");
    fs::create_dir(&directory).unwrap();
    let metadata = grip::discovery::filesystem::metadata_at_path(&directory).unwrap();
    let mut moved_to_other_filesystem = metadata.stat;
    let inspected_device = moved_to_other_filesystem.st_dev as u64;
    assert_eq!(
        grip::discovery::filesystem::classify(&moved_to_other_filesystem, inspected_device),
        NodeKind::Directory
    );
    moved_to_other_filesystem.st_dev = moved_to_other_filesystem.st_dev.saturating_add(1);
    assert_eq!(
        grip::discovery::filesystem::classify(&moved_to_other_filesystem, inspected_device),
        NodeKind::NestedMount
    );
}

#[test]
#[allow(clippy::result_large_err)]
fn post_plan_filesystem_binding_drift_stops_before_payload_mutation() {
    let (root, home, registry, state, selection, plan) = support::pull_execution_fixture();
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    let destination_before = fs::read(&destination).unwrap();
    let result = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        |phase| {
            if phase == grip::mutation::FaultPhase::BeforeActionRevalidation(0) {
                fs::remove_file(&source).unwrap();
                std::os::unix::fs::symlink(&destination, &source).unwrap();
            }
            Ok(())
        },
    );
    let grip::GripError::MutationFailed(failure) = result.unwrap_err() else {
        panic!("filesystem binding drift must produce a journaled mutation failure");
    };
    assert_eq!(failure.phase, "revalidation_failure");
    assert_eq!(failure.failed_action_index, Some(0));
    assert!(
        fs::symlink_metadata(&source)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read(&destination).unwrap(), destination_before);
}
