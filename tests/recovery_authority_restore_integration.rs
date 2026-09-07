mod support;
use std::fs;

fn reference_of_kind(value: &serde_json::Value, kind: &str) -> String {
    value["details"]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["kind"] == kind && entry["availability"] == "available")
        .map(|entry| match kind {
            "registry" => format!(
                "registry:sha256:{}",
                entry["reference"]["digest"].as_str().unwrap()
            ),
            "accepted_state" => format!(
                "state:generation:{}:sha256:{}",
                entry["reference"]["generation"].as_u64().unwrap(),
                entry["reference"]["digest"].as_str().unwrap()
            ),
            _ => unreachable!(),
        })
        .unwrap()
}

#[test]
fn exact_registry_recovery_restores_only_the_recorded_post_transition() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "payload").unwrap();
    let add = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "mapping",
            "add",
            "file",
            source.to_str().unwrap(),
            destination.to_str().unwrap(),
        ],
    );
    assert!(add.status.success());
    let list = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "list"],
    );
    let reference = reference_of_kind(&support::json(&list), "registry");
    let restore = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "restore", &reference],
    );
    assert!(
        restore.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&restore.stdout),
        String::from_utf8_lossy(&restore.stderr)
    );
    assert_eq!(
        fs::read_to_string(grip_home.join("config.toml")).unwrap(),
        "schema_version = 1\nmappings = []\n"
    );
}

#[test]
fn exact_state_recovery_restores_integrity_valid_prior_generation() {
    let (root, grip_home, _, _, _) = support::payload_recovery_fixture();
    let list = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "list"],
    );
    let reference = reference_of_kind(&support::json(&list), "accepted_state");
    let restore = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["recovery", "restore", &reference],
    );
    assert!(
        restore.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&restore.stdout),
        String::from_utf8_lossy(&restore.stderr)
    );
    assert_eq!(
        grip::state::decode_accepted(Some(&fs::read(grip_home.join("state/state.json")).unwrap()))
            .unwrap()
            .generation,
        Some(0)
    );
}

#[test]
fn registry_restore_repairs_missing_authority_but_blocks_newer_valid_authority() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "payload").unwrap();
    assert!(
        support::command_with_grip_home(
            root.path(),
            &grip_home,
            &[
                "mapping",
                "add",
                "file",
                source.to_str().unwrap(),
                destination.to_str().unwrap()
            ]
        )
        .status
        .success()
    );
    let list = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "list"],
    );
    let reference = reference_of_kind(&support::json(&list), "registry");
    fs::remove_file(grip_home.join("config.toml")).unwrap();
    assert!(
        support::command_with_grip_home(
            root.path(),
            &grip_home,
            &["recovery", "restore", &reference]
        )
        .status
        .success()
    );

    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "one").unwrap();
    assert!(
        support::command_with_grip_home(
            root.path(),
            &grip_home,
            &[
                "mapping",
                "add",
                "file",
                source.to_str().unwrap(),
                destination.to_str().unwrap()
            ]
        )
        .status
        .success()
    );
    let list = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "list"],
    );
    let reference = reference_of_kind(&support::json(&list), "registry");
    let other_source = root.path().join("other-source");
    let other_destination = root.path().join("other-destination");
    fs::write(&other_source, "two").unwrap();
    assert!(
        support::command_with_grip_home(
            root.path(),
            &grip_home,
            &[
                "mapping",
                "add",
                "file",
                other_source.to_str().unwrap(),
                other_destination.to_str().unwrap()
            ]
        )
        .status
        .success()
    );
    let before = fs::read(grip_home.join("config.toml")).unwrap();
    let blocked = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["recovery", "restore", &reference],
    );
    assert!(!blocked.status.success());
    assert_eq!(fs::read(grip_home.join("config.toml")).unwrap(), before);
}

#[test]
fn state_restore_repairs_corrupt_authority_but_blocks_newer_valid_generation() {
    let (root, grip_home, _, _, _) = support::payload_recovery_fixture();
    let list = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "list"],
    );
    let reference = reference_of_kind(&support::json(&list), "accepted_state");
    fs::write(grip_home.join("state/state.json"), b"corrupt").unwrap();
    assert!(
        support::command_with_grip_home(
            root.path(),
            &grip_home,
            &["recovery", "restore", &reference]
        )
        .status
        .success()
    );

    let (root, grip_home, source, destination, _) = support::payload_recovery_fixture();
    let list = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "list"],
    );
    let reference = reference_of_kind(&support::json(&list), "accepted_state");
    fs::write(&source, "newer").unwrap();
    fs::write(&destination, "newer").unwrap();
    assert!(
        support::command_with_grip_home(root.path(), &grip_home, &["baseline", "accept"])
            .status
            .success()
    );
    let before = fs::read(grip_home.join("state/state.json")).unwrap();
    let blocked = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["recovery", "restore", &reference],
    );
    assert!(!blocked.status.success());
    assert_eq!(
        fs::read(grip_home.join("state/state.json")).unwrap(),
        before
    );
}

#[test]
fn registry_restore_preview_requires_valid_remaining_state_authority() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "payload").unwrap();
    assert!(
        support::command_with_grip_home(
            root.path(),
            &grip_home,
            &[
                "mapping",
                "add",
                "file",
                source.to_str().unwrap(),
                destination.to_str().unwrap(),
            ],
        )
        .status
        .success()
    );
    let list = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "list"],
    );
    let reference = reference_of_kind(&support::json(&list), "registry");
    fs::create_dir_all(grip_home.join("state")).unwrap();
    fs::write(grip_home.join("state/state.json"), b"corrupt").unwrap();
    let preview = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output=json",
            "recovery",
            "restore",
            "--dry-run",
            &reference,
        ],
    );
    assert_eq!(preview.status.code(), Some(12));
}

#[test]
fn registry_restore_publication_contention_finalizes_operation_failure() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "payload").unwrap();
    assert!(
        support::command_with_grip_home(
            root.path(),
            &grip_home,
            &[
                "mapping",
                "add",
                "file",
                source.to_str().unwrap(),
                destination.to_str().unwrap(),
            ],
        )
        .status
        .success()
    );
    let list = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "list"],
    );
    let reference: grip::recovery::model::RecoveryRef =
        reference_of_kind(&support::json(&list), "registry")
            .parse()
            .unwrap();
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    let plan = grip::recovery::restore::plan(&home, &reference).unwrap();
    let _registry_lock =
        grip::state::lock::PublicationLock::acquire(&home.path().join(".registry.lock")).unwrap();
    let error = grip::recovery::restore::execute(&home, &plan).unwrap_err();
    let grip::GripError::OperationLifecycle {
        operation_id,
        details,
        ..
    } = error
    else {
        panic!("expected operation-bound publication failure");
    };
    assert_eq!(details["plan"]["actions"][0]["status"], "failed");
    let summary =
        support::read_operation_component::<grip::operation::model::OperationSummaryPayloadV1>(
            &home
                .path()
                .join("state/operations")
                .join(operation_id)
                .join("operation.json"),
        );
    assert_eq!(summary.payload.state, "failed");
}

#[test]
fn state_restore_publication_fault_finalizes_operation_failure() {
    let (root, grip_home, _, _, _) = support::payload_recovery_fixture();
    let list = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "list"],
    );
    let reference: grip::recovery::model::RecoveryRef =
        reference_of_kind(&support::json(&list), "accepted_state")
            .parse()
            .unwrap();
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    let plan = grip::recovery::restore::plan(&home, &reference).unwrap();
    let error = grip::recovery::restore::execute_with_fault(
        &home,
        &plan,
        Some(grip::recovery::restore::RestoreFault::Publication),
    )
    .unwrap_err();
    let grip::GripError::OperationLifecycle {
        operation_id,
        details,
        ..
    } = error
    else {
        panic!("expected operation-bound state publication failure");
    };
    assert_eq!(details["plan"]["actions"][0]["status"], "failed");
    let summary =
        support::read_operation_component::<grip::operation::model::OperationSummaryPayloadV1>(
            &home
                .path()
                .join("state/operations")
                .join(operation_id)
                .join("operation.json"),
        );
    assert_eq!(summary.payload.state, "failed");
}
