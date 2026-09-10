mod support;

use std::fs;
use support::project::ProjectFixture;

#[test]
fn project_root_tree_prunes_grip_metadata_before_ignore_evaluation() {
    let fixture = ProjectFixture::initialized();
    fs::write(fixture.project_root.join("payload"), "value").unwrap();
    fs::write(fixture.project_root.join(".gripignore"), "!.grip/**\n").unwrap();
    let added = fixture.command(&["add", ".", "~"]);
    assert!(
        added.status.success(),
        "{}",
        String::from_utf8_lossy(&added.stdout)
    );

    let inspected = fixture.command(&["--output=json", "status", "."]);
    assert!(inspected.status.success());
    let result: serde_json::Value = serde_json::from_slice(&inspected.stdout).unwrap();
    let records = result["details"]["records"].as_array().unwrap();
    assert!(records.iter().any(|record| {
        record["relative_path"]["display"] == "payload"
            && record["classification"] == "source_addition"
    }));
    assert!(!records.iter().any(|record| {
        record["relative_path"]["display"] == ".grip"
            || record["relative_path"]["display"]
                .as_str()
                .is_some_and(|path| path.starts_with(".grip/"))
    }));
}

#[test]
fn metadata_sources_and_selectors_are_rejected_without_descriptor_changes() {
    let fixture = ProjectFixture::initialized();
    let before = fs::read(fixture.descriptor_path()).unwrap();
    for source in [".grip", ".grip/config.toml", "payload/../.grip"] {
        let added = fixture.command(&["add", source, "~/target"]);
        assert!(!added.status.success(), "source {source} was accepted");
    }
    let inspected = fixture.command(&["status", ".grip"]);
    assert!(!inspected.status.success());
    assert_eq!(fs::read(fixture.descriptor_path()).unwrap(), before);
}
