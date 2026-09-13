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
fn mapping_add_accepts_and_persists_normalized_relative_source_spellings() {
    let fixture = ProjectFixture::initialized();
    fs::create_dir(fixture.project_root.join("app")).unwrap();
    assert!(
        fixture
            .command(&["add", "./app/", "~/app"])
            .status
            .success()
    );
    assert!(
        fs::read_to_string(fixture.descriptor_path())
            .unwrap()
            .contains("source = \"app\"")
    );
}

#[test]
fn mapping_add_rejects_invalid_sources_and_destination_forms() {
    let fixture = ProjectFixture::initialized();
    fs::write(fixture.project_root.join("file"), "payload").unwrap();
    for (source, destination) in [
        ("/absolute", "~/x"),
        ("../escape", "~/x"),
        (".grip", "~/x"),
        ("nested/../.grip", "~/x"),
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
fn mapping_add_preserves_project_relative_destination_and_uses_project_root_for_selectors() {
    let fixture = ProjectFixture::initialized();
    fs::write(fixture.project_root.join("file"), "payload").unwrap();
    let destination_root = fixture.project_root.parent().unwrap().join("grip-dst");
    fs::create_dir_all(&destination_root).unwrap();
    assert!(
        fixture
            .command(&["add", "file", "../grip-dst/./file"])
            .status
            .success()
    );
    assert!(
        fs::read_to_string(fixture.descriptor_path())
            .unwrap()
            .contains("destination = \"../grip-dst/./file\"")
    );
    let nested = fixture.project_root.join("nested");
    fs::create_dir(&nested).unwrap();
    let selected = fixture
        .command_builder(&nested)
        .args(["status", "--destination", "../grip-dst/file"])
        .output()
        .unwrap();
    assert!(selected.status.success(), "{selected:?}");
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
    assert!(fixture.command(&["list", "./source/"]).status.success());
    for arguments in [
        vec!["status", "./source/"],
        vec!["status", "--destination", "~/destination"],
        vec!["diff", "./source/"],
        vec!["push", "--dry-run", "./source/"],
        vec!["pull", "--dry-run", "./source/"],
        vec!["pull", "--dry-run", "--destination", "~/destination"],
        vec!["sync", "--dry-run", "./source/"],
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
    assert!(fixture.command(&["remove", "./source/"]).status.success());
}

#[test]
fn push_resolves_status_source_labels_from_the_invocation_directory() {
    let fixture = ProjectFixture::initialized();
    let app = fixture.project_root.join("app");
    let shared = fixture.project_root.join("shared");
    fs::create_dir_all(&app).unwrap();
    fs::create_dir_all(&shared).unwrap();
    add_changed_file_mapping(
        &fixture,
        "app/main.py",
        "workspace/app/main.py",
        "main changed",
    );
    add_changed_file_mapping(
        &fixture,
        "shared/config.yml",
        "workspace/shared/config.yml",
        "config changed",
    );

    let status = fixture.command_from(&app, &["status"]);
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stdout)
    );
    let stdout = String::from_utf8(status.stdout).unwrap();
    assert!(stdout.contains("  main.py -> "));
    assert!(stdout.contains("  ../shared/config.yml -> "));

    let normal = fixture.command_from(&app, &["push", "main.py"]);
    assert!(
        normal.status.success(),
        "{}",
        String::from_utf8_lossy(&normal.stdout)
    );
    assert_eq!(
        fs::read(fixture.home_destination("workspace/app/main.py")).unwrap(),
        b"main changed"
    );
    let dry_run = fixture.command_from(&app, &["push", "--dry-run", "../shared/config.yml"]);
    assert!(
        dry_run.status.success(),
        "{}",
        String::from_utf8_lossy(&dry_run.stdout)
    );
    let forced = fixture.command_from(
        &app,
        &["push", "--force", "--dry-run", "../shared/config.yml"],
    );
    assert!(!forced.status.success());
    assert!(
        String::from_utf8(forced.stdout)
            .unwrap()
            .starts_with("Error: Push blocked:"),
    );

    let external = fixture.explicit_project_command(&["status"]);
    assert!(external.status.success());
    let external_label = format!(
        "{}/shared/config.yml",
        fixture.project_root.file_name().unwrap().to_string_lossy()
    );
    assert!(
        String::from_utf8(external.stdout)
            .unwrap()
            .contains(&format!("  {external_label} -> "))
    );
    let explicit_push = fixture.explicit_project_command(&["push", "--dry-run", &external_label]);
    assert!(
        explicit_push.status.success(),
        "{}",
        String::from_utf8_lossy(&explicit_push.stdout)
    );
}

#[test]
fn cwd_relative_push_rejects_absolute_and_escaping_selectors_without_mutation() {
    let fixture = ProjectFixture::initialized();
    let app = fixture.project_root.join("app");
    fs::create_dir_all(&app).unwrap();
    add_changed_file_mapping(
        &fixture,
        "app/main.py",
        "workspace/app/main.py",
        "main changed",
    );
    let outside = fixture.root.path().join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("secret"), "secret").unwrap();
    std::os::unix::fs::symlink(&outside, app.join("escape")).unwrap();
    let state_before = fs::read(fixture.state_dir().join("state.json")).unwrap();

    for selector in [
        fixture
            .project_root
            .join("app/main.py")
            .to_string_lossy()
            .into_owned(),
        "../../outside/secret".into(),
        "escape/secret".into(),
    ] {
        let output = fixture.command_from(&app, &["push", "--dry-run", &selector]);
        assert!(
            !output.status.success(),
            "selector={selector} output={}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert_eq!(
            fs::read(fixture.state_dir().join("state.json")).unwrap(),
            state_before
        );
    }
}

fn add_changed_file_mapping(
    fixture: &ProjectFixture,
    source_relative: &str,
    destination_relative: &str,
    changed_content: &str,
) {
    let source = fixture.project_root.join(source_relative);
    let destination = fixture.home_destination(destination_relative);
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    fs::write(&source, "same").unwrap();
    fs::write(&destination, "same").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    assert!(
        fixture
            .command(&["add", source_relative, &format!("~/{destination_relative}"),])
            .status
            .success()
    );
    fs::write(source, changed_content).unwrap();
}
