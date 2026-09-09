mod support;

use serde_json::Value;
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::os::unix::net::UnixListener;
use std::process::Command;

#[test]
fn inspection_includes_file_mapping_and_tree_members_without_mutation() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let file_source = root.path().join("single-source");
    let file_destination = root.path().join("single-destination");
    fs::write(&file_source, "single").unwrap();
    let tree_source = root.path().join("tree-source");
    let tree_destination = root.path().join("tree-destination");
    fs::create_dir_all(tree_source.join("nested/empty")).unwrap();
    fs::write(tree_source.join("nested/file.txt"), "payload").unwrap();
    support::write_descriptor(
        &metadata_dir,
        &[
            ("file", &file_source, &file_destination),
            ("tree", &tree_source, &tree_destination),
        ],
    );
    let before = support::snapshot(root.path());

    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "mapping", "inspect"],
    );
    assert_eq!(output.status.code(), Some(0));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    let records = result["details"]["records"].as_array().unwrap();
    assert_eq!(result["details"]["counts"]["eligible"], 4);
    assert_eq!(records[0]["mapping_kind"], "file");
    assert!(records[0].get("relative_path").is_none());
    assert_eq!(records[1]["relative_path"]["display"], "nested");
    assert_eq!(records[2]["relative_path"]["display"], "nested/empty");
    assert_eq!(records[3]["relative_path"]["display"], "nested/file.txt");
    assert_eq!(support::snapshot(root.path()), before);
}

#[test]
fn invalid_unselected_mapping_blocks_selected_inspection() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(
        metadata_dir.join("config.toml"),
        "schema_version = 2\n[[mappings]]\nkind = \"tree\"\nsource = \"/absolute\"\ndestination = \"~/destination\"\n",
    )
    .unwrap();
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "mapping", "inspect", "not-selected"],
    );
    assert_eq!(output.status.code(), Some(10));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["details"]["operation"], "project_selection");
    assert_eq!(result["details"]["reason"], "invalid_project_metadata");
}

#[test]
fn inspection_reports_source_blockers_destination_overlay_and_collisions() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(destination.join("managed.txt")).unwrap();
    fs::write(source.join("managed.txt"), "source").unwrap();
    fs::write(destination.join("local.txt"), "local").unwrap();
    symlink("local.txt", destination.join("local-link")).unwrap();
    symlink("managed.txt", source.join("source-link")).unwrap();
    let non_utf8 = OsString::from_vec(vec![b'n', 0xff]);
    let non_utf8_created = fs::write(source.join(&non_utf8), "raw").is_ok();
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);

    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "mapping", "inspect"],
    );
    assert_eq!(output.status.code(), Some(0));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        result["details"]["blocking_count"],
        if non_utf8_created { 3 } else { 2 }
    );
    let records = result["details"]["records"].as_array().unwrap();
    assert!(records.iter().any(|record| {
        record["category"] == "unsupported_source" && record["reason"] == "symlink"
    }));
    if non_utf8_created {
        assert!(records.iter().any(|record| {
            record["category"] == "unsupported_source"
                && record["reason"] == "non_utf8_path"
                && record["relative_path"]["raw_hex"] == "6eff"
        }));
    }
    assert!(records.iter().any(|record| {
        record["category"] == "unsafe_destination_collision"
            && record["relative_path"]["display"] == "managed.txt"
            && record["reason"] == "wrong_node_kind"
    }));
    assert!(records.iter().any(|record| {
        record["category"] == "destination_only"
            && record["relative_path"]["display"] == "local-link"
            && record["reason"] == "symlink"
            && record["blocking"] == false
    }));
}

#[test]
fn invalid_policy_fails_without_a_partial_inventory() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("visible"), "payload").unwrap();
    fs::write(source.join(".gripignore"), [0xff]).unwrap();
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);
    let before = support::snapshot(root.path());
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "mapping", "inspect"],
    );
    assert_eq!(output.status.code(), Some(10));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["details"]["reason"], "non_utf8_policy");
    assert!(
        result["details"]["paths"][0]["display"]
            .as_str()
            .unwrap()
            .ends_with("/.gripignore")
    );
    assert!(result["details"].get("records").is_none());
    assert_eq!(support::snapshot(root.path()), before);
}

#[test]
fn policy_node_and_parse_failures_are_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("target"), "payload").unwrap();
    symlink("target", source.join(".gripignore")).unwrap();
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);
    let symlinked = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "mapping", "inspect"],
    );
    assert_eq!(symlinked.status.code(), Some(10));
    let result: Value = serde_json::from_slice(&symlinked.stdout).unwrap();
    assert_eq!(result["details"]["reason"], "invalid_policy");

    fs::remove_file(source.join(".gripignore")).unwrap();
    fs::write(source.join(".gripignore"), "dangling\\\n").unwrap();
    let malformed = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "mapping", "inspect"],
    );
    assert_eq!(malformed.status.code(), Some(10));
    let result: Value = serde_json::from_slice(&malformed.stdout).unwrap();
    assert_eq!(result["details"]["reason"], "invalid_policy");
}

#[test]
fn unreadable_and_hard_linked_policies_prevent_a_complete_inventory() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    let policy = source.join(".gripignore");
    fs::write(&policy, "ignored\n").unwrap();
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);

    fs::set_permissions(&policy, fs::Permissions::from_mode(0o000)).unwrap();
    let unreadable = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "mapping", "inspect"],
    );
    assert_eq!(unreadable.status.code(), Some(20));
    let result: Value = serde_json::from_slice(&unreadable.stdout).unwrap();
    assert_eq!(result["details"]["reason"], "unreadable_policy");

    fs::set_permissions(&policy, fs::Permissions::from_mode(0o600)).unwrap();
    fs::hard_link(&policy, source.join("policy-copy")).unwrap();
    let hard_linked = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "mapping", "inspect"],
    );
    assert_eq!(hard_linked.status.code(), Some(10));
    let result: Value = serde_json::from_slice(&hard_linked.stdout).unwrap();
    assert_eq!(result["details"]["reason"], "invalid_policy");

    fs::remove_file(source.join("policy-copy")).unwrap();
    fs::remove_file(&policy).unwrap();
    let sparse = fs::File::create(&policy).unwrap();
    sparse.set_len(1024 * 1024).unwrap();
    drop(sparse);
    let sparse_policy = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "mapping", "inspect"],
    );
    assert_eq!(sparse_policy.status.code(), Some(10));
    let result: Value = serde_json::from_slice(&sparse_policy.stdout).unwrap();
    assert_eq!(result["details"]["reason"], "invalid_policy");
}

#[test]
fn special_source_and_destination_nodes_are_classified_without_opening_them() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&destination).unwrap();
    fs::write(source.join("hard-a"), "payload").unwrap();
    fs::hard_link(source.join("hard-a"), source.join("hard-b")).unwrap();
    let sparse = fs::File::create(source.join("sparse")).unwrap();
    sparse.set_len(1024 * 1024).unwrap();
    drop(sparse);
    let fifo_status = Command::new("mkfifo")
        .arg(source.join("fifo"))
        .status()
        .unwrap();
    assert!(fifo_status.success());
    fs::write(source.join("paired"), "source").unwrap();
    symlink("local", destination.join("paired")).unwrap();
    fs::write(destination.join("hard-a"), "overlay").unwrap();
    fs::hard_link(destination.join("hard-a"), destination.join("hard-b")).unwrap();
    let destination_sparse = fs::File::create(destination.join("sparse")).unwrap();
    destination_sparse.set_len(1024 * 1024).unwrap();
    drop(destination_sparse);
    fs::write(destination.join(".gripignore"), "ignored\n").unwrap();
    let socket = UnixListener::bind(destination.join("overlay.socket")).ok();
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);

    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "mapping", "inspect"],
    );
    assert_eq!(output.status.code(), Some(0));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    let records = result["details"]["records"].as_array().unwrap();
    for reason in ["hard_link", "sparse_file", "fifo"] {
        assert!(
            records.iter().any(|record| {
                record["category"] == "unsupported_source" && record["reason"] == reason
            }),
            "missing {reason}"
        );
    }
    if socket.is_some() {
        assert!(records.iter().any(|record| {
            record["category"] == "destination_only"
                && record["reason"] == "socket"
                && record["blocking"] == false
        }));
    }
    for reason in ["hard_link", "sparse_file"] {
        assert!(records.iter().any(|record| {
            record["category"] == "destination_only" && record["reason"] == reason
        }));
    }
    assert!(records.iter().any(|record| {
        record["category"] == "destination_only"
            && record["relative_path"]["display"] == ".gripignore"
    }));
    assert!(records.iter().any(|record| {
        record["category"] == "unsafe_destination_collision"
            && record["relative_path"]["display"] == "paired"
            && record["reason"] == "symlink"
    }));
}
