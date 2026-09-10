mod support;

use std::fs;

fn json(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn list_is_empty_without_writing_state() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());

    let output = support::project_command(root.path(), &metadata_dir, &["-o", "json", "list"]);

    assert!(output.status.success());
    assert_eq!(
        json(&output)["details"]["mappings"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert!(!metadata_dir.join("state").exists());
}

#[test]
fn list_is_sorted_by_source_and_selects_an_exact_source() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(root.path().join("z-source"), "z").unwrap();
    fs::write(root.path().join("a-source"), "a").unwrap();

    for (source, destination) in [
        ("z-source", "~/z-destination"),
        ("a-source", "~/a-destination"),
    ] {
        let output =
            support::project_command(root.path(), &metadata_dir, &["add", source, destination]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let list = support::project_command(root.path(), &metadata_dir, &["-o", "json", "list"]);
    let mappings = json(&list)["details"]["mappings"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(mappings.len(), 2);
    assert!(
        mappings[0]["resolved"]["source"]["display"]
            .as_str()
            .unwrap()
            .ends_with("a-source")
    );
    let selected = support::project_command(
        root.path(),
        &metadata_dir,
        &["-o", "json", "list", "z-source"],
    );
    assert_eq!(
        json(&selected)["details"]["mappings"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn remove_only_changes_mapping_declaration() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "source").unwrap();
    fs::write(&destination, "destination").unwrap();
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "source", "~/destination"]
        )
        .status
        .success()
    );

    let output = support::project_command(root.path(), &metadata_dir, &["remove", "source"]);

    assert!(output.status.success());
    assert_eq!(fs::read_to_string(source).unwrap(), "source");
    assert_eq!(fs::read_to_string(destination).unwrap(), "destination");
    assert_eq!(
        json(&support::project_command(
            root.path(),
            &metadata_dir,
            &["-o", "json", "list"]
        ))["details"]["mappings"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn invalid_direct_add_reports_a_mapping_error() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());

    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["-o", "json", "add", "../escape", "~/destination"],
    );

    assert!(!output.status.success());
    assert_eq!(json(&output)["code"], "invalid_configuration");
}
