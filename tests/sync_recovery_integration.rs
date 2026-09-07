mod support;

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};

#[test]
fn sync_preserves_the_losing_target_for_each_replacement() {
    let (root, grip_home, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "new source").unwrap();
    let output =
        support::command_with_grip_home(root.path(), &grip_home, &["--output", "json", "sync"]);
    assert!(output.status.success());
    let value = support::json(&output);
    let operation = value["details"]["operation_record"]["id"].as_str().unwrap();
    let recovery = value["details"]["actions"][0]["milestones"]["recovery_ref"]
        .as_str()
        .unwrap();
    assert_eq!(
        fs::read_to_string(
            grip_home
                .join("state/operations")
                .join(operation)
                .join(recovery)
        )
        .unwrap(),
        "accepted"
    );
    assert_eq!(fs::read_to_string(destination).unwrap(), "new source");
    let operation_directory = grip_home.join("state/operations").join(operation);
    let recovery_directory = operation_directory.join("recovery/00000000");
    assert_eq!(
        fs::metadata(&recovery_directory)
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        0o700
    );
    assert_eq!(
        fs::metadata(recovery_directory.join("payload"))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        0o600
    );
    let metadata: grip::mutation::recovery::RecoveryMetadataV1 =
        serde_json::from_slice(&fs::read(recovery_directory.join("metadata.json")).unwrap())
            .unwrap();
    assert_eq!(metadata.operation_id, operation);
    assert_eq!(metadata.action_index, 0);
    assert!(metadata.payload_present);
    assert!(metadata.verified);
    assert_eq!(
        fs::metadata(recovery_directory.join("metadata.json"))
            .unwrap()
            .mode()
            & 0o7777,
        0o600
    );
}
