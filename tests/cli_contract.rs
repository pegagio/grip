mod support;

use support::project::ProjectFixture;

#[test]
fn retained_commands_and_global_options_parse() {
    let root = tempfile::tempdir().unwrap();
    for command in [
        "init", "version", "add", "list", "remove", "status", "diff", "push", "pull", "sync",
    ] {
        let output = support::command(root.path(), &[command, "--help"]);
        assert!(output.status.success(), "{command}");
    }
    let version = support::command(root.path(), &["-o", "json", "version"]);
    assert!(version.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&version.stdout).unwrap()["details"]["version"],
        env!("CARGO_PKG_VERSION")
    );
}

#[test]
fn project_option_applies_to_project_commands_not_init_or_version() {
    let fixture = ProjectFixture::initialized();
    let root = fixture.project_root.to_str().unwrap();
    for arguments in [vec!["-p", root, "status"], vec!["status", "-p", root]] {
        let output = fixture.command_from(fixture.root.path(), &arguments);
        assert!(output.status.success(), "{arguments:?}");
    }
    for arguments in [vec!["-p", root, "init"], vec!["-p", root, "version"]] {
        let output = fixture.command_from(fixture.root.path(), &arguments);
        assert_eq!(output.status.code(), Some(2), "{arguments:?}");
    }
}

#[test]
fn default_output_is_human_and_human_is_not_an_output_value() {
    let root = tempfile::tempdir().unwrap();
    let human = support::command(root.path(), &["version"]);
    assert!(human.status.success());
    assert!(String::from_utf8_lossy(&human.stdout).starts_with("grip "));
    let invalid = support::command(root.path(), &["-o", "human", "version"]);
    assert_eq!(invalid.status.code(), Some(2));
}

#[test]
fn removed_commands_are_rejected_without_aliases() {
    let root = tempfile::tempdir().unwrap();
    for command in [
        "mapping", "validate", "fsck", "check", "show", "resolve", "delete", "retire", "accept",
        "recovery", "baseline",
    ] {
        let output = support::command(root.path(), &[command]);
        assert_eq!(output.status.code(), Some(2), "{command}");
    }
}
