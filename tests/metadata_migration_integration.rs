mod support;

use grip::discovery::model::NodeKind;

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
    support::write_unsupported_state_v2(&fixture.metadata_dir);
    fixture
}

fn assert_strict_v4_rejection(fixture: &support::MetadataFixture, arguments: &[&str]) {
    let state_path = fixture.metadata_dir.join("state/state.json");
    let before = support::snapshot(fixture.root.path());
    let state_before = std::fs::read(&state_path).unwrap();
    let output = support::project_command(fixture.root.path(), &fixture.metadata_dir, arguments);
    assert_eq!(output.status.code(), Some(11));
    let value = support::json(&output);
    assert_eq!(value["code"], "unsupported_schema");
    assert_eq!(value["details"]["project"]["state"], "rebind_blocked");
    assert_eq!(
        value["details"]["project"]["blockers"],
        serde_json::json!(["state_unavailable"])
    );
    assert_eq!(std::fs::read(state_path).unwrap(), state_before);
    assert_eq!(support::snapshot(fixture.root.path()), before);
}

#[test]
fn state_v2_is_rejected_without_implicit_migration_for_every_command_family() {
    for arguments in [
        &["--output=json", "status"][..],
        &["--output=json", "baseline", "accept"][..],
        &["--output=json", "push", "--dry-run"][..],
        &["--output=json", "resolve", "source", "--source"][..],
    ] {
        let fixture = legacy_fixture(true);
        assert_strict_v4_rejection(&fixture, arguments);
    }
}

#[test]
fn divergent_state_v2_is_rejected_before_payload_or_operation_mutation() {
    let fixture = legacy_fixture(false);
    let source_before = std::fs::read(&fixture.source).unwrap();
    let destination_before = std::fs::read(&fixture.destination).unwrap();
    assert_strict_v4_rejection(
        &fixture,
        &["--output=json", "resolve", "source", "--destination"],
    );
    assert_eq!(std::fs::read(&fixture.source).unwrap(), source_before);
    assert_eq!(
        std::fs::read(&fixture.destination).unwrap(),
        destination_before
    );
    assert!(!fixture.metadata_dir.join("state/operations").exists());
}

#[test]
fn unsupported_state_rejection_is_byte_deterministic() {
    let fixture = legacy_fixture(true);
    let state_path = fixture.metadata_dir.join("state/state.json");
    let state_before = std::fs::read(&state_path).unwrap();
    let expected = support::project_command(
        fixture.root.path(),
        &fixture.metadata_dir,
        &["--output=json", "status"],
    );
    for _ in 0..100 {
        let observed = support::project_command(
            fixture.root.path(),
            &fixture.metadata_dir,
            &["--output=json", "status"],
        );
        assert_eq!(observed.status.code(), expected.status.code());
        assert_eq!(observed.stdout, expected.stdout);
        assert_eq!(observed.stderr, expected.stderr);
        assert_eq!(std::fs::read(&state_path).unwrap(), state_before);
    }
}
