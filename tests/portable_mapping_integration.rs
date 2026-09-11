mod support;

use std::fs;
use support::project::ProjectFixture;

#[test]
fn mapping_add_stores_only_portable_declarations_and_resolves_per_context() {
    let fixture = ProjectFixture::initialized();
    fs::create_dir_all(fixture.project_root.join("home/editor")).unwrap();
    let output = fixture.command(&["--output=json", "add", "home/editor", "~/.config/editor"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let descriptor = fs::read_to_string(fixture.descriptor_path()).unwrap();
    assert!(descriptor.contains("source = \"home/editor\""));
    assert!(descriptor.contains("destination = \"~/.config/editor\""));
    assert!(!descriptor.contains(fixture.project_root.to_str().unwrap()));
    assert!(!descriptor.contains(fixture.home_root.to_str().unwrap()));

    let listed = fixture.command(&["--output=json", "list"]);
    assert!(listed.status.success());
    let inspected = fixture.command(&["--output=json", "status", "home/editor"]);
    assert!(inspected.status.success());
}

#[test]
fn descriptor_bytes_are_deterministic_across_declaration_order() {
    let first = ProjectFixture::initialized();
    let second = ProjectFixture::initialized();
    for fixture in [&first, &second] {
        fs::write(fixture.project_root.join("a"), "a").unwrap();
        fs::write(fixture.project_root.join("z"), "z").unwrap();
    }
    for source in ["z", "a"] {
        assert!(
            first
                .command(&["add", source, &format!("~/{source}")])
                .status
                .success()
        );
    }
    for source in ["a", "z"] {
        assert!(
            second
                .command(&["add", source, &format!("~/{source}")])
                .status
                .success()
        );
    }
    assert_eq!(
        fs::read(first.descriptor_path()).unwrap(),
        fs::read(second.descriptor_path()).unwrap()
    );
}

#[test]
fn mapping_commands_use_project_relative_source_identity() {
    let fixture = ProjectFixture::initialized();
    fs::write(fixture.project_root.join("file"), "payload").unwrap();
    assert!(fixture.command(&["add", "file", "~/file"]).status.success());
    assert!(fixture.command(&["list", "file"]).status.success());
    assert!(fixture.command(&["remove", "file"]).status.success());
    assert!(
        !fs::read_to_string(fixture.descriptor_path())
            .unwrap()
            .contains("[[mappings]]")
    );
}

#[test]
fn mapping_add_rejects_invalid_sources_and_destination_forms() {
    let fixture = ProjectFixture::initialized();
    fs::write(fixture.project_root.join("file"), "payload").unwrap();
    for (source, destination) in [
        ("/absolute", "~/x"),
        ("../escape", "~/x"),
        ("file", "relative"),
        ("file", "./relative"),
        ("file", "../relative"),
        ("file", "${HOME}/x"),
        ("file", "~other/x"),
    ] {
        assert!(
            !fixture
                .command(&["add", source, destination])
                .status
                .success()
        );
    }
}

#[test]
fn mapping_add_preserves_non_normalized_home_destination_and_supports_selectors() {
    let fixture = ProjectFixture::initialized();
    fs::write(fixture.project_root.join("file"), "payload").unwrap();
    let destination = fixture.home_destination(".config/grip/../editor");
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    assert!(
        fixture
            .command(&["add", "file", "~/.config/grip/../editor"])
            .status
            .success()
    );
    assert!(
        fs::read_to_string(fixture.descriptor_path())
            .unwrap()
            .contains("destination = \"~/.config/grip/../editor\"")
    );
    assert!(fixture.command(&["list"]).status.success());
    assert!(
        fixture
            .command(&["status", "--destination", "~/.config//editor"])
            .status
            .success()
    );
}

#[test]
fn mapping_add_accepts_and_persists_standalone_home_destination() {
    let fixture = ProjectFixture::initialized();
    fs::create_dir(fixture.project_root.join("source-tree")).unwrap();
    assert!(
        fixture
            .command(&["add", "source-tree", "~"])
            .status
            .success()
    );
    assert!(
        fs::read_to_string(fixture.descriptor_path())
            .unwrap()
            .contains("destination = \"~\"")
    );
    assert!(fixture.command(&["list"]).status.success());
    assert!(
        fixture
            .command(&["status", "--destination", "~"])
            .status
            .success()
    );
}

#[test]
fn mapping_add_preserves_absolute_destination_and_supports_destination_selector() {
    let fixture = ProjectFixture::initialized();
    fs::write(fixture.project_root.join("file"), "payload").unwrap();
    let destination = fixture.absolute_destination("editor");
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    let destination_text = destination.to_str().unwrap();
    assert!(
        fixture
            .command(&["add", "file", destination_text])
            .status
            .success()
    );
    assert!(
        fs::read_to_string(fixture.descriptor_path())
            .unwrap()
            .contains(&format!("destination = \"{destination_text}\""))
    );
    assert!(fixture.command(&["list"]).status.success());
    assert!(
        fixture
            .command(&["status", "--destination", destination_text])
            .status
            .success()
    );
}

#[test]
fn absolute_destination_declaration_survives_state_publication_and_removal() {
    let fixture = ProjectFixture::initialized();
    let source = fixture.project_root.join("file");
    let destination = fixture.absolute_destination("editor");
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    fs::write(&source, "payload").unwrap();
    fs::write(&destination, "payload").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    let destination_text = destination.to_str().unwrap();
    assert!(
        fixture
            .command(&["add", "file", destination_text])
            .status
            .success()
    );
    let state = fs::read_to_string(fixture.state_dir().join("state.json")).unwrap();
    assert!(state.contains(destination_text));
    assert!(fixture.command(&["remove", "file"]).status.success());
}

#[test]
fn project_commands_resolve_source_and_destination_selectors_in_portable_spaces() {
    let fixture = ProjectFixture::initialized();
    let source = fixture.project_root.join("source");
    let destination = fixture.home_root.join("destination");
    fs::write(&source, "accepted").unwrap();
    fs::write(&destination, "accepted").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    assert!(
        fixture
            .command(&["add", "source", "~/destination"])
            .status
            .success()
    );
    for arguments in [
        vec!["status", "source"],
        vec!["status", "--destination", "~/destination"],
        vec!["push", "--dry-run", "source"],
        vec!["pull", "--dry-run", "--destination", "~/destination"],
        vec!["sync", "--dry-run", "source"],
    ] {
        let output = fixture.command(&arguments);
        assert!(
            output.status.success(),
            "arguments={arguments:?} output={}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
    assert!(
        !fixture
            .command(&["status", source.to_str().unwrap()])
            .status
            .success()
    );
    assert!(
        fixture
            .command(&["status", "--destination", destination.to_str().unwrap()])
            .status
            .success()
    );
}
