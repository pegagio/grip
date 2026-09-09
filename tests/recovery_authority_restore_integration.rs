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
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    fs::write(&source, "payload").unwrap();
    let add = support::project_command(
        root.path(),
        &metadata_dir,
        &["mapping", "add", "file", "source", "~/destination"],
    );
    assert!(add.status.success());
    let list = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "list"],
    );
    let reference = reference_of_kind(&support::json(&list), "registry");
    let restore = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "restore", &reference],
    );
    assert!(
        restore.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&restore.stdout),
        String::from_utf8_lossy(&restore.stderr)
    );
    assert_eq!(
        fs::read_to_string(metadata_dir.join("config.toml")).unwrap(),
        "schema_version = 2\nmappings = []\n"
    );
}

#[test]
fn exact_state_recovery_restores_integrity_valid_prior_generation() {
    let (root, metadata_dir, _, _, _) = support::payload_recovery_fixture();
    let list = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "list"],
    );
    let reference = reference_of_kind(&support::json(&list), "accepted_state");
    let restore = support::project_command(
        root.path(),
        &metadata_dir,
        &["recovery", "restore", &reference],
    );
    assert!(
        restore.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&restore.stdout),
        String::from_utf8_lossy(&restore.stderr)
    );
    assert_eq!(
        grip::state::decode_v4(&fs::read(metadata_dir.join("state/state.json")).unwrap())
            .unwrap()
            .generation,
        0
    );
}

#[test]
fn registry_restore_repairs_missing_authority_but_blocks_newer_valid_authority() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    fs::write(&source, "payload").unwrap();
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["mapping", "add", "file", "source", "~/destination"]
        )
        .status
        .success()
    );
    let list = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "list"],
    );
    let reference = reference_of_kind(&support::json(&list), "registry");
    fs::remove_file(metadata_dir.join("config.toml")).unwrap();
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["recovery", "restore", &reference]
        )
        .status
        .success()
    );

    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    fs::write(&source, "one").unwrap();
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["mapping", "add", "file", "source", "~/destination"]
        )
        .status
        .success()
    );
    let list = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "list"],
    );
    let reference = reference_of_kind(&support::json(&list), "registry");
    let other_source = root.path().join("other-source");
    fs::write(&other_source, "two").unwrap();
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &[
                "mapping",
                "add",
                "file",
                "other-source",
                "~/other-destination"
            ]
        )
        .status
        .success()
    );
    let before = fs::read(metadata_dir.join("config.toml")).unwrap();
    let blocked = support::project_command(
        root.path(),
        &metadata_dir,
        &["recovery", "restore", &reference],
    );
    assert!(!blocked.status.success());
    assert_eq!(fs::read(metadata_dir.join("config.toml")).unwrap(), before);
}

#[test]
fn state_restore_repairs_corrupt_authority_but_blocks_newer_valid_generation() {
    let (root, metadata_dir, _, _, _) = support::payload_recovery_fixture();
    let list = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "list"],
    );
    let reference = reference_of_kind(&support::json(&list), "accepted_state");
    fs::write(metadata_dir.join("state/state.json"), b"corrupt").unwrap();
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["recovery", "restore", &reference]
        )
        .status
        .success()
    );

    let (root, metadata_dir, source, destination, _) = support::payload_recovery_fixture();
    let list = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "list"],
    );
    let reference = reference_of_kind(&support::json(&list), "accepted_state");
    fs::write(&source, "newer").unwrap();
    fs::write(&destination, "newer").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    assert!(
        support::project_command(root.path(), &metadata_dir, &["baseline", "accept"])
            .status
            .success()
    );
    let before = fs::read(metadata_dir.join("state/state.json")).unwrap();
    let blocked = support::project_command(
        root.path(),
        &metadata_dir,
        &["recovery", "restore", &reference],
    );
    assert!(!blocked.status.success());
    assert_eq!(
        fs::read(metadata_dir.join("state/state.json")).unwrap(),
        before
    );
}

#[test]
fn registry_restore_preview_requires_valid_remaining_state_authority() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    fs::write(&source, "payload").unwrap();
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["mapping", "add", "file", "source", "~/destination",],
        )
        .status
        .success()
    );
    let list = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "list"],
    );
    let reference = reference_of_kind(&support::json(&list), "registry");
    fs::create_dir_all(metadata_dir.join("state")).unwrap();
    fs::write(metadata_dir.join("state/state.json"), b"corrupt").unwrap();
    let preview = support::project_command(
        root.path(),
        &metadata_dir,
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
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    fs::write(&source, "payload").unwrap();
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["mapping", "add", "file", "source", "~/destination",],
        )
        .status
        .success()
    );
    let list = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "list"],
    );
    let reference: grip::recovery::model::RecoveryRef =
        reference_of_kind(&support::json(&list), "registry")
            .parse()
            .unwrap();
    let home = support::project_home(&metadata_dir);
    let plan = grip::recovery::restore::plan(&home, &reference).unwrap();
    let _registry_lock = grip::state::lock::PublicationLock::acquire(
        &grip::state::lock::project_lock_path(&home, "registry.lock").unwrap(),
    )
    .unwrap();
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
        support::read_operation_component::<grip::operation::model::OperationSummaryPayloadV2>(
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
    let (root, metadata_dir, _, _, _) = support::payload_recovery_fixture();
    let list = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "list"],
    );
    let reference: grip::recovery::model::RecoveryRef =
        reference_of_kind(&support::json(&list), "accepted_state")
            .parse()
            .unwrap();
    let home = support::project_home(&metadata_dir);
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
        support::read_operation_component::<grip::operation::model::OperationSummaryPayloadV2>(
            &home
                .path()
                .join("state/operations")
                .join(operation_id)
                .join("operation.json"),
        );
    assert_eq!(summary.payload.state, "failed");
}
