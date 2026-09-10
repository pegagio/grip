mod support;

use std::fs;

fn json(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn add_list_and_remove_use_flat_commands_without_copying_payloads() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    fs::write(&source, "payload").unwrap();

    let add = support::project_command(
        root.path(),
        &metadata_dir,
        &["-o", "json", "add", "source", "~/destination"],
    );
    assert_eq!(add.status.code(), Some(0));
    assert_eq!(json(&add)["details"]["operation"], "add");
    assert!(!root.path().join("destination").exists());

    let list = support::project_command(
        root.path(),
        &metadata_dir,
        &["-o", "json", "list", "source"],
    );
    assert_eq!(list.status.code(), Some(0));
    assert_eq!(
        json(&list)["details"]["mappings"].as_array().unwrap().len(),
        1
    );

    let remove = support::project_command(
        root.path(),
        &metadata_dir,
        &["-o", "json", "remove", "source"],
    );
    assert_eq!(remove.status.code(), Some(0));
    assert_eq!(json(&remove)["details"]["operation"], "remove");
    assert_eq!(fs::read_to_string(source).unwrap(), "payload");
}

#[test]
fn add_requires_an_existing_compatible_endpoint() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let missing = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "missing", "~/also-missing"],
    );
    assert_ne!(missing.status.code(), Some(0));

    fs::write(root.path().join("file"), "payload").unwrap();
    fs::create_dir(root.path().join("directory")).unwrap();
    let incompatible =
        support::project_command(root.path(), &metadata_dir, &["add", "file", "~/directory"]);
    assert_ne!(incompatible.status.code(), Some(0));
}

#[test]
fn add_establishes_a_baseline_only_for_matching_existing_endpoints() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(root.path().join("source"), "same").unwrap();
    fs::write(root.path().join("destination"), "same").unwrap();
    support::copy_complete_metadata(
        &root.path().join("source"),
        &root.path().join("destination"),
        grip::discovery::model::NodeKind::File,
    );

    let added = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "source", "~/destination"],
    );
    assert_eq!(added.status.code(), Some(0));
    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(metadata_dir.join("state/state.json")).unwrap()).unwrap();
    assert_eq!(state["payload"]["baselines"].as_array().unwrap().len(), 1);
    assert!(state["payload"].get("pending_retirements").is_none());
}

#[test]
fn force_push_resolves_an_unbaselined_collision_from_destination_space() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(root.path().join("source"), "source wins").unwrap();
    fs::write(root.path().join("destination"), "destination loses").unwrap();

    assert_eq!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "source", "~/destination"],
        )
        .status
        .code(),
        Some(0)
    );
    let pushed = support::project_command(
        root.path(),
        &metadata_dir,
        &["push", "-f", "-d", "~/destination"],
    );
    assert_eq!(pushed.status.code(), Some(0), "{:?}", pushed);
    assert_eq!(
        fs::read_to_string(root.path().join("destination")).unwrap(),
        "source wins"
    );
}

#[test]
fn status_exit_code_only_escalates_attention() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(root.path().join("source"), "payload").unwrap();
    assert_eq!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "source", "~/destination"],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(
        support::project_command(root.path(), &metadata_dir, &["status"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(
        support::project_command(root.path(), &metadata_dir, &["status", "-e"])
            .status
            .code(),
        Some(1)
    );
}

#[test]
fn remove_prunes_baseline_and_readd_treats_existing_endpoints_as_new() {
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

    assert_eq!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "source", "~/destination"],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(
        support::project_command(root.path(), &metadata_dir, &["remove", "source"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(fs::read_to_string(&source).unwrap(), "same");
    assert_eq!(fs::read_to_string(&destination).unwrap(), "same");

    fs::write(&destination, "changed while untracked").unwrap();
    assert_eq!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "source", "~/destination"],
        )
        .status
        .code(),
        Some(0)
    );
    let status = support::project_command(root.path(), &metadata_dir, &["-o", "json", "status"]);
    assert_eq!(status.status.code(), Some(0));
    assert_eq!(
        json(&status)["details"]["records"][0]["classification"],
        "initial_collision"
    );
}

#[test]
fn force_push_propagates_a_selected_source_absence() {
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
    assert_eq!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "source", "~/destination"],
        )
        .status
        .code(),
        Some(0)
    );
    fs::remove_file(&source).unwrap();
    let forced = support::project_command(root.path(), &metadata_dir, &["push", "-f", "source"]);
    assert_eq!(forced.status.code(), Some(0), "{:?}", forced);
    assert!(!destination.exists());
    assert!(!metadata_dir.join("state/recovery").exists());
    let operations = metadata_dir.join("state/operations");
    for operation in fs::read_dir(operations).unwrap() {
        assert!(!operation.unwrap().path().join("recovery").exists());
    }
}
