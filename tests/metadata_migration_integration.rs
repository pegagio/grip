mod support;

use grip::discovery::model::NodeKind;
use grip::mapping::MappingKind;
use grip::observation::model::{EntryIdentity, MappingSnapshot};
use std::collections::BTreeMap;

fn legacy_fixture(equal: bool) -> support::MetadataFixture {
    let fixture = support::MetadataFixture::file(b"same");
    let source =
        grip::observation::fingerprint::inspect_complete(&fixture.source, NodeKind::File).unwrap();
    grip::metadata::macos::apply_metadata_paths(
        &fixture.source,
        &fixture.destination,
        NodeKind::File,
        &source.state.metadata,
    )
    .unwrap();
    if !equal {
        support::set_fixture_mode(&fixture.destination, 0o600);
    }
    let identity = EntryIdentity::new(
        MappingSnapshot {
            kind: MappingKind::File,
            source: fixture.source.clone(),
            destination: fixture.destination.clone(),
        },
        Vec::new(),
    )
    .unwrap();
    support::write_v2_state(
        &fixture.grip_home,
        4,
        BTreeMap::from([(identity, support::supported_file_state(&fixture.source))]),
    );
    fixture
}

#[test]
fn equal_v2_copies_require_explicit_accept_and_read_only_commands_do_not_rewrite() {
    let fixture = legacy_fixture(true);
    let state_path = fixture.grip_home.join("state/state.json");
    let before = std::fs::read(&state_path).unwrap();
    let status = support::command_with_grip_home(
        fixture.root.path(),
        &fixture.grip_home,
        &["--output", "json", "status"],
    );
    assert!(status.status.success());
    assert_eq!(
        support::json(&status)["details"]["records"][0]["classification"],
        "metadata_migration_ready"
    );
    assert_eq!(std::fs::read(&state_path).unwrap(), before);

    let accept = support::command_with_grip_home(
        fixture.root.path(),
        &fixture.grip_home,
        &["--output", "json", "baseline", "accept"],
    );
    assert!(
        accept.status.success(),
        "{}",
        String::from_utf8_lossy(&accept.stderr)
    );
    let bytes = std::fs::read(&state_path).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["schema_version"],
        3
    );
}

#[test]
fn unequal_v2_copies_require_whole_entry_resolution_before_v3_publication() {
    let fixture = legacy_fixture(false);
    let status = support::command_with_grip_home(
        fixture.root.path(),
        &fixture.grip_home,
        &["--output", "json", "status"],
    );
    assert_eq!(
        support::json(&status)["details"]["records"][0]["classification"],
        "metadata_migration_conflict"
    );
    let blocked = support::command_with_grip_home(
        fixture.root.path(),
        &fixture.grip_home,
        &["baseline", "accept"],
    );
    assert!(!blocked.status.success());

    let resolved = support::command_with_grip_home(
        fixture.root.path(),
        &fixture.grip_home,
        &[
            "--output",
            "json",
            "resolve",
            fixture.source.to_str().unwrap(),
            "--source",
        ],
    );
    assert!(
        resolved.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&resolved.stdout),
        String::from_utf8_lossy(&resolved.stderr)
    );
    let state = std::fs::read(fixture.grip_home.join("state/state.json")).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&state).unwrap()["schema_version"],
        3
    );
    let source =
        grip::observation::fingerprint::inspect_complete(&fixture.source, NodeKind::File).unwrap();
    let destination =
        grip::observation::fingerprint::inspect_complete(&fixture.destination, NodeKind::File)
            .unwrap();
    assert_eq!(source.state, destination.state);
}

#[test]
fn destination_winner_migrates_v2_only_after_complete_transfer_and_verification() {
    let fixture = legacy_fixture(false);
    let expected =
        grip::observation::fingerprint::inspect_complete(&fixture.destination, NodeKind::File)
            .unwrap()
            .state;
    let resolved = support::command_with_grip_home(
        fixture.root.path(),
        &fixture.grip_home,
        &[
            "--output=json",
            "resolve",
            fixture.source.to_str().unwrap(),
            "--destination",
        ],
    );
    assert!(
        resolved.status.success(),
        "{}",
        String::from_utf8_lossy(&resolved.stdout)
    );
    assert_eq!(
        grip::observation::fingerprint::inspect_complete(&fixture.source, NodeKind::File)
            .unwrap()
            .state,
        expected
    );
    let state = std::fs::read(fixture.grip_home.join("state/state.json")).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&state).unwrap()["schema_version"],
        3
    );
}

#[test]
fn blocked_migration_keeps_exact_v2_authority_and_creates_no_operation() {
    let fixture = legacy_fixture(false);
    support::set_fixture_xattr(&fixture.source, "com.example.unknown", b"private-value");
    let state_path = fixture.grip_home.join("state/state.json");
    let state_before = std::fs::read(&state_path).unwrap();
    let operations = fixture.grip_home.join("state/operations");
    let output = support::command_with_grip_home(
        fixture.root.path(),
        &fixture.grip_home,
        &[
            "--output=json",
            "resolve",
            fixture.source.to_str().unwrap(),
            "--source",
        ],
    );
    assert!(!output.status.success());
    assert_eq!(std::fs::read(state_path).unwrap(), state_before);
    assert!(!operations.exists() || std::fs::read_dir(operations).unwrap().next().is_none());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("private-value"));
}

#[test]
fn migration_ready_status_is_byte_deterministic_and_never_rewrites_v2() {
    let fixture = legacy_fixture(true);
    let state_path = fixture.grip_home.join("state/state.json");
    let state_before = std::fs::read(&state_path).unwrap();
    let expected = support::command_with_grip_home(
        fixture.root.path(),
        &fixture.grip_home,
        &["--output=json", "status"],
    );
    for _ in 0..100 {
        let observed = support::command_with_grip_home(
            fixture.root.path(),
            &fixture.grip_home,
            &["--output=json", "status"],
        );
        assert_eq!(observed.status.code(), expected.status.code());
        assert_eq!(observed.stdout, expected.stdout);
        assert_eq!(observed.stderr, expected.stderr);
        assert_eq!(std::fs::read(&state_path).unwrap(), state_before);
    }
}
