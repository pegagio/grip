mod support;

use std::fs;
use support::project::{EMPTY_DESCRIPTOR_V2, PROJECT_GITIGNORE, ProjectFixture};

#[test]
fn init_uses_cwd_and_creates_only_commit_eligible_metadata() {
    let fixture = ProjectFixture::new();
    let output = fixture.command(&["--output", "json", "init"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["details"]["initialization"], "initialized");
    assert_eq!(
        fs::read_to_string(fixture.descriptor_path()).unwrap(),
        EMPTY_DESCRIPTOR_V2
    );
    assert_eq!(
        fs::read_to_string(fixture.metadata_dir().join(".gitignore")).unwrap(),
        PROJECT_GITIGNORE
    );
    assert!(!fixture.state_dir().exists());
    assert!(!fixture.project_root.join(".git").exists());
}

#[test]
fn init_accepts_an_explicit_existing_path_and_reinitializes_as_noop() {
    let fixture = ProjectFixture::new();
    let project = fixture.project_root.to_str().unwrap();
    let first = fixture.command_from(fixture.root.path(), &["init", project]);
    assert!(first.status.success());
    let before = fixture.snapshot_all();
    let second = fixture.command(&["--output=json", "init"]);
    assert!(second.status.success());
    let value: serde_json::Value = serde_json::from_slice(&second.stdout).unwrap();
    assert_eq!(value["details"]["initialization"], "already_initialized");
    assert_eq!(fixture.snapshot_all(), before);
}

#[test]
fn project_option_is_invalid_for_init_and_application_version() {
    let fixture = ProjectFixture::new();
    for command in [
        vec!["--project", ".", "init"],
        vec!["--project", ".", "version"],
    ] {
        let output = fixture.command(&command);
        assert_eq!(output.status.code(), Some(2));
    }
}
