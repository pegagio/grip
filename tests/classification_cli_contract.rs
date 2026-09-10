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
