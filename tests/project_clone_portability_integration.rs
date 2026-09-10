mod support;

use std::fs;
use support::project::{PortableFixtureMapping, ProjectFixture};

#[test]
fn copied_descriptor_is_byte_identical_but_resolves_to_each_root_and_home() {
    let first = ProjectFixture::initialized();
    fs::create_dir_all(first.project_root.join("payload")).unwrap();
    first.write_descriptor(&[PortableFixtureMapping {
        kind: "tree",
        source: "payload",
        destination: "~/.config/payload",
    }]);
    let descriptor = fs::read(first.descriptor_path()).unwrap();
    let second_root = first.copy_project("second-project");
    let second_home = first.root.path().join("second-home");
    fs::create_dir(&second_home).unwrap();

    let first_result = first.command(&["--output=json", "list"]);
    let second_result = first
        .command_builder(&second_root)
        .env("HOME", &second_home)
        .args(["--output=json", "list"])
        .output()
        .unwrap();
    assert!(first_result.status.success());
    assert!(second_result.status.success());
    assert_eq!(
        fs::read(second_root.join(".grip/config.toml")).unwrap(),
        descriptor
    );

    let first_json: serde_json::Value = serde_json::from_slice(&first_result.stdout).unwrap();
    let second_json: serde_json::Value = serde_json::from_slice(&second_result.stdout).unwrap();
    assert_eq!(
        first_json["details"]["mappings"][0]["declared"],
        second_json["details"]["mappings"][0]["declared"]
    );
    assert_ne!(
        first_json["details"]["mappings"][0]["resolved"],
        second_json["details"]["mappings"][0]["resolved"]
    );
    assert_eq!(fs::read(first.descriptor_path()).unwrap(), descriptor);
}
