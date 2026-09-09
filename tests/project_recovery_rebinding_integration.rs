mod support;

use std::fs;
use support::project::{PortableFixtureMapping, ProjectFixture};

fn authority_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    fn walk(path: &std::path::Path, result: &mut Vec<std::path::PathBuf>) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            if metadata.is_dir() {
                walk(&path, result);
            } else if matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("record.json" | "manifest.json" | "recovery-v2.json" | "metadata-v3.json")
            ) {
                result.push(path);
            }
        }
    }
    let mut result = Vec::new();
    walk(root, &mut result);
    result
}

#[test]
fn copied_project_resolves_portable_operation_and_recovery_authority_in_new_context() {
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
    assert!(fixture.command(&["baseline", "accept"]).status.success());

    fs::write(&destination, "replacement").unwrap();
    let pulled = fixture.command(&["--output=json", "pull"]);
    assert!(pulled.status.success());
    let operation = support::json(&pulled)["details"]["operation_record"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let reference = format!("payload:{operation}:0");

    for path in authority_files(&fixture.state_dir()) {
        let value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let payload = &value["payload"];
        let identities = payload["actions"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|action| action.get("identity"))
            .chain(payload.get("identity"));
        for identity in identities {
            let mapping = &identity["mapping"];
            assert!(!mapping["source"].as_str().unwrap().starts_with('/'));
            assert!(mapping["destination"].as_str().unwrap().starts_with('~'));
        }
        if let Some(reference) = payload.get("private_ref").and_then(|value| value.as_str()) {
            assert!(!std::path::Path::new(reference).is_absolute());
        }
    }

    let copied_root = fixture.copy_project("recovery-copy");
    let copied_home = fixture.root.path().join("recovery-copy-home");
    fs::create_dir(&copied_home).unwrap();
    fs::copy(&destination, copied_home.join("destination")).unwrap();
    support::copy_complete_metadata(
        &source,
        &copied_root.join("source"),
        grip::discovery::model::NodeKind::File,
    );
    support::copy_complete_metadata(
        &destination,
        &copied_home.join("destination"),
        grip::discovery::model::NodeKind::File,
    );
    let rebound = fixture
        .command_builder(&copied_root)
        .env("HOME", &copied_home)
        .args(["baseline", "accept"])
        .output()
        .unwrap();
    assert!(
        rebound.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&rebound.stdout),
        String::from_utf8_lossy(&rebound.stderr)
    );

    let restored = fixture
        .command_builder(&copied_root)
        .env("HOME", &copied_home)
        .args(["recovery", "restore", &reference])
        .output()
        .unwrap();
    assert!(
        restored.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&restored.stdout),
        String::from_utf8_lossy(&restored.stderr)
    );
    assert_eq!(
        fs::read_to_string(copied_root.join("source")).unwrap(),
        "accepted"
    );
    assert_eq!(fs::read_to_string(&source).unwrap(), "replacement");

    fs::write(copied_root.join("source"), "unrelated").unwrap();
    let stale = fixture
        .command_builder(&copied_root)
        .env("HOME", &copied_home)
        .args(["recovery", "restore", &reference])
        .output()
        .unwrap();
    assert!(!stale.status.success());
    assert_eq!(
        fs::read_to_string(copied_root.join("source")).unwrap(),
        "unrelated"
    );
}
