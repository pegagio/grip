mod support;
use std::fs;

#[test]
fn new_payload_recovery_has_immutable_manifest_and_preserves_legacy_metadata() {
    let (root, grip_home, _, _, reference) = support::payload_recovery_fixture();
    let operation = reference.split(':').nth(1).unwrap();
    let directory = grip_home
        .join("state/operations")
        .join(operation)
        .join("recovery/00000000");
    let manifest = fs::read(directory.join("manifest.json")).unwrap();
    let metadata = fs::read(directory.join("metadata.json")).unwrap();
    let show =
        support::command_with_grip_home(root.path(), &grip_home, &["recovery", "show", &reference]);
    assert!(show.status.success());
    assert_eq!(fs::read(directory.join("manifest.json")).unwrap(), manifest);
    assert_eq!(fs::read(directory.join("metadata.json")).unwrap(), metadata);
}
