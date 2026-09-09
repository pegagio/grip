mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use support::project::{PROJECT_GITIGNORE, PortableFixtureMapping, ProjectFixture};

#[test]
fn read_only_and_dry_run_commands_do_not_create_project_state() {
    let fixture = ProjectFixture::initialized();
    fs::write(fixture.project_root.join("source"), "value").unwrap();
    fixture.write_descriptor(&[PortableFixtureMapping {
        kind: "file",
        source: "source",
        destination: "~/destination",
    }]);
    for arguments in [
        vec!["mapping", "list"],
        vec!["mapping", "inspect"],
        vec!["status"],
        vec!["push", "--dry-run"],
    ] {
        let output = fixture.command(&arguments);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(!fixture.state_dir().exists());
    }
    assert_eq!(
        fs::read_to_string(fixture.metadata_dir().join(".gitignore")).unwrap(),
        PROJECT_GITIGNORE
    );
}

#[test]
fn mutation_creates_only_private_project_local_state_and_locks() {
    let fixture = ProjectFixture::initialized();
    fs::write(fixture.project_root.join("source"), "value").unwrap();
    let output = fixture.command(&["mapping", "add", "file", "source", "~/destination"]);
    assert!(output.status.success());

    let state = fixture.state_dir();
    let locks = state.join("locks");
    assert_eq!(
        fs::metadata(&state).unwrap().permissions().mode() & 0o7777,
        0o700
    );
    assert_eq!(
        fs::metadata(&locks).unwrap().permissions().mode() & 0o7777,
        0o700
    );
    for entry in fs::read_dir(&locks).unwrap() {
        let metadata = entry.unwrap().metadata().unwrap();
        assert!(metadata.is_file());
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o600);
    }
    assert!(!fixture.metadata_dir().join(".mutation.lock").exists());
    assert!(!fixture.metadata_dir().join(".registry.lock").exists());
}

#[test]
fn baseline_publication_uses_state_v4_and_rejects_legacy_state() {
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
    fixture.write_descriptor(&[PortableFixtureMapping {
        kind: "file",
        source: "source",
        destination: "~/destination",
    }]);
    let accepted = fixture.command(&["baseline", "accept"]);
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stdout)
    );
    let state_path = fixture.state_dir().join("state.json");
    let state: serde_json::Value = serde_json::from_slice(&fs::read(&state_path).unwrap()).unwrap();
    assert_eq!(state["schema_version"], 4);
    assert_eq!(
        state["payload"]["baselines"][0]["identity"]["mapping"]["source"],
        "source"
    );
    assert_eq!(
        state["payload"]["baselines"][0]["identity"]["mapping"]["destination"],
        "~/destination"
    );

    fs::write(&state_path, r#"{"schema_version":3}"#).unwrap();
    let rejected = fixture.command(&["status"]);
    assert!(!rejected.status.success());
}
