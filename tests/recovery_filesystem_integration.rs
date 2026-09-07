mod support;
use std::fs;
use std::os::unix::fs::symlink;

#[test]
fn private_recovery_reader_rejects_symlink_substitution_without_touching_referent() {
    let (root, grip_home, _, _, reference) = support::payload_recovery_fixture();
    let operation = reference.split(':').nth(1).unwrap();
    let payload = grip_home
        .join("state/operations")
        .join(operation)
        .join("recovery/00000000/payload");
    let outside = root.path().join("outside");
    fs::write(&outside, "outside").unwrap();
    fs::remove_file(&payload).unwrap();
    symlink(&outside, &payload).unwrap();
    let show =
        support::command_with_grip_home(root.path(), &grip_home, &["recovery", "show", &reference]);
    assert!(!show.status.success());
    assert_eq!(fs::read_to_string(outside).unwrap(), "outside");
}

#[test]
fn cleanup_unlinks_only_the_selected_private_payload() {
    let (root, grip_home, _, _, reference) = support::payload_recovery_fixture();
    let operation = reference.split(':').nth(1).unwrap();
    let directory = grip_home
        .join("state/operations")
        .join(operation)
        .join("recovery/00000000");
    let metadata = fs::read(directory.join("metadata.json")).unwrap();
    let cleanup = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["recovery", "remove", "--confirm", &reference],
    );
    assert!(cleanup.status.success());
    assert!(!directory.join("payload").exists());
    assert_eq!(fs::read(directory.join("metadata.json")).unwrap(), metadata);
}
