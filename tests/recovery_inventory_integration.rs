mod support;

#[test]
fn recovery_inventory_is_deterministic_metadata_only_and_show_is_exact() {
    let (root, grip_home, _, _, reference) = support::payload_recovery_fixture();
    let first = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "list"],
    );
    let second = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "list"],
    );
    assert_eq!(first.stdout, second.stdout);
    let inventory = support::json(&first);
    for entry in inventory["details"]["entries"].as_array().unwrap() {
        assert!(entry.get("payload").is_none());
        assert!(entry.get("content").is_none());
    }
    let show = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "show", &reference],
    );
    assert!(show.status.success());
    assert_eq!(
        support::json(&show)["details"]["entry"]["reference"]["kind"],
        "payload"
    );
}

fn payload_directory(grip_home: &std::path::Path, reference: &str) -> std::path::PathBuf {
    grip_home
        .join("state/operations")
        .join(reference.split(':').nth(1).unwrap())
        .join("recovery/00000000")
}

#[test]
fn legacy_payload_is_projected_conservatively_without_rewrite() {
    let (root, grip_home, _, _, reference) = support::payload_recovery_fixture();
    let directory = payload_directory(&grip_home, &reference);
    std::fs::remove_file(directory.join("manifest.json")).unwrap();
    let metadata = std::fs::read(directory.join("metadata.json")).unwrap();
    let show = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "show", &reference],
    );
    assert!(show.status.success());
    assert_eq!(
        support::json(&show)["details"]["entry"]["provenance"],
        "legacy_incomplete"
    );
    assert_eq!(
        std::fs::read(directory.join("metadata.json")).unwrap(),
        metadata
    );
    assert!(!directory.join("manifest.json").exists());
}

#[test]
fn corrupt_and_incomplete_entries_fail_closed_or_report_incomplete() {
    let (root, grip_home, _, _, reference) = support::payload_recovery_fixture();
    let directory = payload_directory(&grip_home, &reference);
    let payload = directory.join("payload");
    std::fs::remove_file(&payload).unwrap();
    let incomplete = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "show", &reference],
    );
    assert_eq!(
        support::json(&incomplete)["details"]["entry"]["availability"],
        "cleanup_incomplete"
    );
    std::fs::write(directory.join("manifest.json"), b"corrupt").unwrap();
    let corrupt = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "show", &reference],
    );
    assert_eq!(corrupt.status.code(), Some(12));
}

#[test]
fn legacy_missing_and_tombstoned_cleaned_availability_are_distinct() {
    let (root, grip_home, _, _, reference) = support::payload_recovery_fixture();
    let directory = payload_directory(&grip_home, &reference);
    std::fs::remove_file(directory.join("manifest.json")).unwrap();
    std::fs::remove_file(directory.join("payload")).unwrap();
    let missing = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "show", &reference],
    );
    assert_eq!(
        support::json(&missing)["details"]["entry"]["availability"],
        "missing"
    );

    let (other_root, other_home, _, _, other_reference) = support::payload_recovery_fixture();
    let cleaned = support::command_with_grip_home(
        other_root.path(),
        &other_home,
        &["recovery", "remove", "--confirm", &other_reference],
    );
    assert!(cleaned.status.success());
    let show = support::command_with_grip_home(
        other_root.path(),
        &other_home,
        &["--output=json", "recovery", "show", &other_reference],
    );
    assert_eq!(
        support::json(&show)["details"]["entry"]["availability"],
        "cleaned"
    );
}

#[test]
fn operation_inventory_verifies_records_and_exact_show_ignores_unrelated_corruption() {
    let (root, grip_home, _, _, reference) = support::payload_recovery_fixture();
    let operation_id = reference.split(':').nth(1).unwrap();
    let valid_operation = format!("operation:{operation_id}");
    let valid_show = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "show", &valid_operation],
    );
    assert!(valid_show.status.success());
    let valid = &support::json(&valid_show)["details"]["entry"];
    assert_eq!(valid["integrity"], "verified");
    assert!(valid["created_at"].as_str().is_some());
    assert!(valid["byte_count"].as_u64().unwrap() > 0);

    let corrupt = grip_home.join("state/operations/corrupt-1-2-3");
    std::fs::create_dir(&corrupt).unwrap();
    std::fs::set_permissions(
        &corrupt,
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .unwrap();
    std::fs::write(corrupt.join("operation.json"), b"corrupt").unwrap();
    std::fs::set_permissions(
        corrupt.join("operation.json"),
        std::os::unix::fs::PermissionsExt::from_mode(0o600),
    )
    .unwrap();
    let selected = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "show", &reference],
    );
    assert!(selected.status.success());
    let home = grip::home::select(Some(grip_home.clone().into_os_string()), None).unwrap();
    let selected_reference: grip::recovery::model::RecoveryRef = reference.parse().unwrap();
    assert!(grip::recovery::restore::plan(&home, &selected_reference).is_ok());
    assert!(grip::recovery::cleanup::plan(&home, &[selected_reference]).is_ok());
    let inventory = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "list"],
    );
    assert!(inventory.status.success());
    let value = support::json(&inventory);
    let corrupt_entry = value["details"]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["reference"]["operation_id"] == "corrupt-1-2-3")
        .unwrap();
    assert_ne!(corrupt_entry["integrity"], "verified");
    assert_eq!(corrupt_entry["availability"], "missing");
}
