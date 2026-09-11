mod support;

use std::fs;

fn fixture() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let root = tempfile::tempdir().unwrap();
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
    let add = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "source", "~/destination"],
    );
    assert!(add.status.success());
    (root, metadata_dir, source)
}

#[test]
fn status_is_success_by_default_and_exit_code_reports_attention() {
    let (root, metadata_dir, source) = fixture();
    fs::write(&source, "changed").unwrap();

    let status = support::project_command(root.path(), &metadata_dir, &["-o", "json", "status"]);
    assert_eq!(status.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(
        value["details"]["records"][0]["classification"],
        "source_only_change"
    );

    let exit = support::project_command(root.path(), &metadata_dir, &["status", "-e"]);
    assert_eq!(exit.status.code(), Some(1));
}

#[test]
fn human_status_renders_a_compact_push_section() {
    let (root, metadata_dir, source) = fixture();
    fs::write(&source, "changed").unwrap();

    let status = support::project_command(root.path(), &metadata_dir, &["status"]);
    assert_eq!(status.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(status.stdout).unwrap(),
        format!(
            "Status: 1 entry checked; 0 current; 1 to push.\n\nChanges to push:\n  {} -> {}\n",
            fs::canonicalize(&source).unwrap().display(),
            fs::canonicalize(root.path().join("destination"))
                .unwrap()
                .display(),
        )
    );
}

#[test]
fn human_status_renders_initial_matches_as_needing_a_baseline() {
    let root = tempfile::tempdir().unwrap();
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

    let status = support::project_command(root.path(), &metadata_dir, &["status"]);
    assert_eq!(
        String::from_utf8(status.stdout).unwrap(),
        format!(
            "Status: 1 entry checked; 0 current; 1 needs baseline.\n\nNeeds baseline:\n  {} >-< {}\n",
            fs::canonicalize(&source).unwrap().display(),
            fs::canonicalize(&destination).unwrap().display(),
        )
    );
}

#[test]
fn human_status_distinguishes_clean_and_empty_scopes() {
    let (root, metadata_dir, _) = fixture();
    let clean = support::project_command(root.path(), &metadata_dir, &["status"]);
    assert_eq!(
        String::from_utf8(clean.stdout).unwrap(),
        "Status: 1 entry checked; 1 current; no action needed.\n"
    );

    let empty_root = tempfile::tempdir().unwrap();
    let empty_metadata_dir = support::initialize_project_metadata(empty_root.path());
    let empty_source = empty_root.path().join("source");
    let empty_destination = empty_root.path().join("destination");
    fs::create_dir(&empty_source).unwrap();
    fs::create_dir(&empty_destination).unwrap();
    support::write_descriptor(
        &empty_metadata_dir,
        &[("tree", &empty_source, &empty_destination)],
    );
    let empty = support::project_command(empty_root.path(), &empty_metadata_dir, &["status"]);
    assert_eq!(
        String::from_utf8(empty.stdout).unwrap(),
        "Status: no managed entries found.\n"
    );
}

#[test]
fn status_json_retains_classification_capabilities_and_exit_contracts() {
    let (root, metadata_dir, source) = fixture();
    fs::write(&source, "changed").unwrap();

    let status = support::project_command(root.path(), &metadata_dir, &["-o", "json", "status"]);
    assert_eq!(status.status.code(), Some(0));
    let value = support::json(&status);
    assert_eq!(
        value["details"]["records"][0]["classification"],
        "source_only_change"
    );
    assert_eq!(value["details"]["attention_count"], 1);
    assert_eq!(value["details"]["blocking_count"], 0);
    assert_eq!(
        value["details"]["endpoint_capabilities"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let exit =
        support::project_command(root.path(), &metadata_dir, &["-o", "json", "status", "-e"]);
    assert_eq!(exit.status.code(), Some(1));
    assert_eq!(support::json(&exit)["details"], value["details"]);
}

#[test]
fn diff_is_read_only_and_supports_destination_selection() {
    let (root, metadata_dir, source) = fixture();
    fs::write(&source, "changed").unwrap();
    let state = fs::read(metadata_dir.join("state/state.json")).unwrap();

    let diff = support::project_command(
        root.path(),
        &metadata_dir,
        &["-o", "json", "diff", "-d", "~/destination"],
    );
    assert_eq!(diff.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_slice(&diff.stdout).unwrap();
    assert_eq!(value["details"]["scope"]["path_space"], "destination");
    assert_eq!(
        fs::read(metadata_dir.join("state/state.json")).unwrap(),
        state
    );
}
