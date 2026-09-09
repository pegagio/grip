mod support;

use std::fs;
use support::project::{PortableFixtureMapping, ProjectFixture};

fn prepare_bound_fixture() -> ProjectFixture {
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
    fixture
}

#[test]
fn copied_state_is_read_only_eligible_until_an_authorized_publication_rebinds_it() {
    let fixture = prepare_bound_fixture();
    let copied_root = fixture.copy_project("copied-project");
    let copied_home = fixture.root.path().join("copied-home");
    fs::create_dir(&copied_home).unwrap();
    let copied_source = copied_root.join("source");
    let copied_destination = copied_home.join("destination");
    fs::write(&copied_destination, "accepted").unwrap();
    support::copy_complete_metadata(
        &fixture.project_root.join("source"),
        &copied_source,
        grip::discovery::model::NodeKind::File,
    );
    support::copy_complete_metadata(
        &fixture.home_root.join("destination"),
        &copied_destination,
        grip::discovery::model::NodeKind::File,
    );

    let before = support::snapshot(fixture.root.path());
    let status = fixture
        .command_builder(&copied_root)
        .env("HOME", &copied_home)
        .args(["status"])
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&status.stdout),
        String::from_utf8_lossy(&status.stderr)
    );
    let status_json = fixture
        .command_builder(&copied_root)
        .env("HOME", &copied_home)
        .args(["--output=json", "status"])
        .output()
        .unwrap();
    let status_value: serde_json::Value = serde_json::from_slice(&status_json.stdout).unwrap();
    assert_eq!(
        status_value["details"]["project"]["state"],
        "rebind_eligible"
    );
    assert_eq!(support::snapshot(fixture.root.path()), before);

    let rebound = fixture
        .command_builder(&copied_root)
        .env("HOME", &copied_home)
        .args(["baseline", "accept"])
        .output()
        .unwrap();
    assert!(
        rebound.status.success(),
        "{}",
        String::from_utf8_lossy(&rebound.stdout)
    );
    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(copied_root.join(".grip/state/state.json")).unwrap())
            .unwrap();
    assert_eq!(
        state["payload"]["binding"]["project_root"],
        fs::canonicalize(&copied_root)
            .unwrap()
            .display()
            .to_string()
    );
    let rebound_status = fixture
        .command_builder(&copied_root)
        .env("HOME", &copied_home)
        .args(["--output=json", "status"])
        .output()
        .unwrap();
    let rebound_value: serde_json::Value = serde_json::from_slice(&rebound_status.stdout).unwrap();
    assert_eq!(rebound_value["details"]["project"]["state"], "bound");
    assert_eq!(
        state["payload"]["binding"]["user_home"],
        fs::canonicalize(&copied_home)
            .unwrap()
            .display()
            .to_string()
    );
}

#[test]
fn copied_state_with_payload_drift_blocks_mutation() {
    let fixture = prepare_bound_fixture();
    let copied_root = fixture.copy_project("drifted-project");
    let copied_home = fixture.root.path().join("drifted-home");
    fs::create_dir(&copied_home).unwrap();
    fs::write(copied_home.join("destination"), "accepted").unwrap();
    fs::write(copied_root.join("source"), "changed").unwrap();
    let output = fixture
        .command_builder(&copied_root)
        .env("HOME", &copied_home)
        .args(["push"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let value: serde_json::Value = serde_json::from_slice(
        &fixture
            .command_builder(&copied_root)
            .env("HOME", &copied_home)
            .args(["--output=json", "push"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    assert_eq!(value["details"]["project"]["state"], "rebind_blocked");
    assert_eq!(
        fs::read_to_string(copied_home.join("destination")).unwrap(),
        "accepted"
    );
}

fn copied_project(
    fixture: &ProjectFixture,
    name: &str,
) -> (std::path::PathBuf, std::path::PathBuf) {
    let copied_root = fixture.copy_project(name);
    let copied_home = fixture.root.path().join(format!("{name}-home"));
    fs::create_dir(&copied_home).unwrap();
    fs::copy(
        fixture.home_root.join("destination"),
        copied_home.join("destination"),
    )
    .unwrap();
    (copied_root, copied_home)
}

fn status_json(
    fixture: &ProjectFixture,
    root: &std::path::Path,
    home: &std::path::Path,
) -> serde_json::Value {
    let output = fixture
        .command_builder(root)
        .env("HOME", home)
        .args(["--output=json", "status"])
        .output()
        .unwrap();
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn descriptor_drift_and_missing_mapping_block_rebinding() {
    let fixture = prepare_bound_fixture();
    let (copied_root, copied_home) = copied_project(&fixture, "missing-mapping");
    fs::write(
        copied_root.join(".grip/config.toml"),
        support::project::EMPTY_DESCRIPTOR_V2,
    )
    .unwrap();
    let value = status_json(&fixture, &copied_root, &copied_home);
    assert_eq!(value["details"]["project"]["state"], "rebind_blocked");
    assert!(
        value["details"]["project"]["blockers"]
            .as_array()
            .unwrap()
            .contains(&"missing_mapping".into())
    );
}

#[test]
fn destination_drift_blocks_rebinding() {
    let fixture = prepare_bound_fixture();
    let (copied_root, copied_home) = copied_project(&fixture, "destination-drift");
    fs::write(copied_home.join("destination"), "changed").unwrap();
    let value = status_json(&fixture, &copied_root, &copied_home);
    assert_eq!(value["details"]["project"]["state"], "rebind_blocked");
    assert!(
        value["details"]["project"]["blockers"]
            .as_array()
            .unwrap()
            .contains(&"destination_drift".into())
    );
}

#[test]
fn corrupt_binding_and_incomplete_baseline_are_rejected() {
    let fixture = prepare_bound_fixture();
    let state_path = fixture.state_dir().join("state.json");
    let mut state = grip::state::decode_v4(&fs::read(&state_path).unwrap()).unwrap();
    state.binding.project_root = "relative".into();
    fs::write(&state_path, grip::state::encode_v4(&state).unwrap()).unwrap();
    assert!(!fixture.command(&["status"]).status.success());

    let fixture = prepare_bound_fixture();
    let state_path = fixture.state_dir().join("state.json");
    let mut state = grip::state::decode_v4(&fs::read(&state_path).unwrap()).unwrap();
    state.baselines.values_mut().next().unwrap().content = None;
    fs::write(&state_path, grip::state::encode_v4(&state).unwrap()).unwrap();
    assert!(!fixture.command(&["status"]).status.success());
}

#[test]
fn contradictory_accepted_evidence_blocks_rebinding() {
    let fixture = prepare_bound_fixture();
    let state_path = fixture.state_dir().join("state.json");
    let mut state = grip::state::decode_v4(&fs::read(&state_path).unwrap()).unwrap();
    let identity = state.baselines.keys().next().unwrap().clone();
    state
        .pending_retirements
        .push(grip::state::PendingRetirementV4 { identity });
    fs::write(&state_path, grip::state::encode_v4(&state).unwrap()).unwrap();
    let value = status_json(&fixture, &fixture.project_root, &fixture.home_root);
    assert_eq!(value["details"]["project"]["state"], "rebind_blocked");
    assert!(
        value["details"]["project"]["blockers"]
            .as_array()
            .unwrap()
            .contains(&"contradictory_pending_retirement".into())
    );
}

#[test]
fn ambiguous_mapping_and_unsafe_ancestry_are_rejected_before_rebinding() {
    let fixture = prepare_bound_fixture();
    let descriptor = fs::read_to_string(fixture.descriptor_path()).unwrap();
    fs::write(
        fixture.descriptor_path(),
        format!("{descriptor}{descriptor}"),
    )
    .unwrap();
    assert!(!fixture.command(&["status"]).status.success());

    let fixture = prepare_bound_fixture();
    let real = fixture.root.path().join("real-home");
    fs::create_dir(&real).unwrap();
    let linked = fixture.root.path().join("linked-home");
    std::os::unix::fs::symlink(&real, &linked).unwrap();
    let output = fixture
        .command_builder(&fixture.project_root)
        .env("HOME", linked)
        .arg("status")
        .output()
        .unwrap();
    assert!(!output.status.success());
}
