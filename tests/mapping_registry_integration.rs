mod support;

use std::fs;
use support::project::ProjectFixture;

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
fn remove_retains_the_exact_declaration_of_an_unrelated_relative_destination() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(root.path().join("retained"), "retained").unwrap();
    fs::write(root.path().join("removed"), "removed").unwrap();
    let destination_root = root.path().join("..").join("grip-destination");
    fs::create_dir_all(&destination_root).unwrap();
    fs::write(destination_root.join("retained"), "retained").unwrap();
    fs::write(root.path().join("removed-destination"), "removed").unwrap();
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "retained", "../grip-destination/./retained"],
        )
        .status
        .success()
    );
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "removed", "removed-destination"],
        )
        .status
        .success()
    );

    let output = support::project_command(root.path(), &metadata_dir, &["remove", "removed"]);

    assert!(output.status.success());
    let descriptor = fs::read_to_string(metadata_dir.join("config.toml")).unwrap();
    assert!(descriptor.contains("destination = \"../grip-destination/./retained\""));
    assert!(!descriptor.contains("source = \"removed\""));
}

#[test]
fn remove_retains_absolute_home_and_relative_declarations_of_unrelated_mappings() {
    let fixture = ProjectFixture::initialized();
    let absolute_destination = fixture.absolute_destination("retained-absolute");
    fs::create_dir_all(absolute_destination.parent().unwrap()).unwrap();
    for (source, destination) in [
        (
            "retained-relative",
            fixture.project_root.join("relative-destination"),
        ),
        ("retained-home", fixture.home_root.join("retained-home")),
        ("retained-absolute", absolute_destination.clone()),
        ("removed", fixture.project_root.join("removed-destination")),
    ] {
        fs::write(fixture.project_root.join(source), source).unwrap();
        fs::write(destination, source).unwrap();
    }
    let absolute_declaration = absolute_destination.to_str().unwrap();
    for (source, destination) in [
        ("retained-relative", "./relative-destination"),
        ("retained-home", "~/retained-home"),
        ("retained-absolute", absolute_declaration),
        ("removed", "removed-destination"),
    ] {
        assert!(
            fixture
                .command(&["add", source, destination])
                .status
                .success()
        );
    }

    let output = fixture.command(&["--output=json", "remove", "removed"]);
    assert!(output.status.success());
    let removed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        removed["details"]["mapping"]["declared"]["destination"],
        "removed-destination"
    );

    let descriptor = fs::read_to_string(fixture.descriptor_path()).unwrap();
    assert!(descriptor.contains("destination = \"./relative-destination\""));
    assert!(descriptor.contains("destination = \"~/retained-home\""));
    assert!(descriptor.contains(&format!("destination = \"{absolute_declaration}\"")));
}

#[test]
fn remove_reports_the_matching_declaration_with_mixed_mapping_kinds() {
    let fixture = ProjectFixture::initialized();
    fs::create_dir(fixture.project_root.join("a-tree")).unwrap();
    fs::write(fixture.project_root.join("z-file"), "file").unwrap();
    assert!(
        fixture
            .command(&["add", "a-tree", "./a-tree-destination"])
            .status
            .success()
    );
    assert!(
        fixture
            .command(&["add", "z-file", "./z-file-destination"])
            .status
            .success()
    );

    let output = fixture.command(&["--output=json", "remove", "a-tree"]);

    assert!(output.status.success());
    let removed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(removed["details"]["mapping"]["declared"]["kind"], "tree");
    assert_eq!(
        removed["details"]["mapping"]["declared"]["source"],
        "a-tree"
    );
    assert_eq!(
        removed["details"]["mapping"]["declared"]["destination"],
        "./a-tree-destination"
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
