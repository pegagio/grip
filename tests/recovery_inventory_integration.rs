mod support;

#[test]
fn recovery_inventory_is_deterministic_metadata_only_and_show_is_exact() {
    let (root, metadata_dir, _, _, reference) = support::payload_recovery_fixture();
    let first = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "list"],
    );
    let second = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "list"],
    );
    assert_eq!(first.stdout, second.stdout);
    let inventory = support::json(&first);
    for entry in inventory["details"]["entries"].as_array().unwrap() {
        assert!(entry.get("payload").is_none());
        assert!(entry.get("content").is_none());
    }
    let show = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "show", &reference],
    );
    assert!(show.status.success());
    assert_eq!(
        support::json(&show)["details"]["entry"]["reference"]["kind"],
        "payload"
    );
}

fn payload_directory(metadata_dir: &std::path::Path, reference: &str) -> std::path::PathBuf {
    metadata_dir
        .join("state/operations")
        .join(reference.split(':').nth(1).unwrap())
        .join("recovery/00000000")
}

#[test]
fn recovery_manifest_v1_is_rejected_without_rewrite() {
    let (root, metadata_dir, _, _, reference) = support::payload_recovery_fixture();
    let directory = payload_directory(&metadata_dir, &reference);
    let manifest_path = directory.join("manifest.json");
    let manifest = std::fs::read(&manifest_path).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&manifest).unwrap();
    value["schema_version"] = 1.into();
    let old_schema = serde_json::to_vec(&value).unwrap();
    std::fs::write(&manifest_path, &old_schema).unwrap();
    let show = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "show", &reference],
    );
    assert_eq!(show.status.code(), Some(11));
    assert_eq!(std::fs::read(manifest_path).unwrap(), old_schema);
}

#[test]
fn corrupt_and_incomplete_entries_fail_closed_or_report_incomplete() {
    let (root, metadata_dir, _, _, reference) = support::payload_recovery_fixture();
    let directory = payload_directory(&metadata_dir, &reference);
    let payload = directory.join("payload");
    std::fs::remove_file(&payload).unwrap();
    let incomplete = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "show", &reference],
    );
    assert_eq!(
        support::json(&incomplete)["details"]["entry"]["availability"],
        "cleanup_incomplete"
    );
    std::fs::write(directory.join("manifest.json"), b"corrupt").unwrap();
    let corrupt = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "show", &reference],
    );
    assert_eq!(corrupt.status.code(), Some(12));
}

#[test]
fn incomplete_and_tombstoned_cleaned_availability_are_distinct() {
    let (root, metadata_dir, _, _, reference) = support::payload_recovery_fixture();
    let directory = payload_directory(&metadata_dir, &reference);
    std::fs::remove_file(directory.join("payload")).unwrap();
    let missing = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "show", &reference],
    );
    assert_eq!(
        support::json(&missing)["details"]["entry"]["availability"],
        "cleanup_incomplete"
    );

    let (other_root, other_home, _, _, other_reference) = support::payload_recovery_fixture();
    let cleaned = support::project_command(
        other_root.path(),
        &other_home,
        &["recovery", "remove", "--confirm", &other_reference],
    );
    assert!(cleaned.status.success());
    let show = support::project_command(
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
    let (root, metadata_dir, _, _, reference) = support::payload_recovery_fixture();
    let operation_id = reference.split(':').nth(1).unwrap();
    let valid_operation = format!("operation:{operation_id}");
    let valid_show = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "show", &valid_operation],
    );
    assert!(valid_show.status.success());
    let valid = &support::json(&valid_show)["details"]["entry"];
    assert_eq!(valid["integrity"], "verified");
    assert!(valid["created_at"].as_str().is_some());
    assert!(valid["byte_count"].as_u64().unwrap() > 0);

    let corrupt = metadata_dir.join("state/operations/corrupt-1-2-3");
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
    let selected = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "recovery", "show", &reference],
    );
    assert!(selected.status.success());
    let home = support::project_home(&metadata_dir);
    let selected_reference: grip::recovery::model::RecoveryRef = reference.parse().unwrap();
    assert!(grip::recovery::restore::plan(&home, &selected_reference).is_ok());
    assert!(grip::recovery::cleanup::plan(&home, &[selected_reference]).is_ok());
    let inventory = support::project_command(
        root.path(),
        &metadata_dir,
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
