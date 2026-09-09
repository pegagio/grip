mod support;
use std::fs;
use std::os::unix::fs::symlink;

#[test]
fn private_recovery_reader_rejects_symlink_substitution_without_touching_referent() {
    let (root, metadata_dir, _, _, reference) = support::payload_recovery_fixture();
    let operation = reference.split(':').nth(1).unwrap();
    let payload = metadata_dir
        .join("state/operations")
        .join(operation)
        .join("recovery/00000000/payload");
    let outside = root.path().join("outside");
    fs::write(&outside, "outside").unwrap();
    fs::remove_file(&payload).unwrap();
    symlink(&outside, &payload).unwrap();
    let show = support::project_command(
        root.path(),
        &metadata_dir,
        &["recovery", "show", &reference],
    );
    assert!(!show.status.success());
    assert_eq!(fs::read_to_string(outside).unwrap(), "outside");
}

#[test]
fn cleanup_unlinks_only_the_selected_private_payload() {
    let (root, metadata_dir, _, _, reference) = support::payload_recovery_fixture();
    let operation = reference.split(':').nth(1).unwrap();
    let directory = metadata_dir
        .join("state/operations")
        .join(operation)
        .join("recovery/00000000");
    let recovery = fs::read(directory.join("recovery-v2.json")).unwrap();
    let metadata = fs::read(directory.join("metadata-v3.json")).unwrap();
    let cleanup = support::project_command(
        root.path(),
        &metadata_dir,
        &["recovery", "remove", "--confirm", &reference],
    );
    assert!(cleanup.status.success());
    assert!(!directory.join("payload").exists());
    assert_eq!(
        fs::read(directory.join("recovery-v2.json")).unwrap(),
        recovery
    );
    assert_eq!(
        fs::read(directory.join("metadata-v3.json")).unwrap(),
        metadata
    );
}
