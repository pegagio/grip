mod support;

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};

#[test]
fn sync_preserves_the_losing_target_for_each_replacement() {
    let (root, metadata_dir, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "new source").unwrap();
    let output =
        support::project_command(root.path(), &metadata_dir, &["--output", "json", "sync"]);
    assert!(output.status.success());
    let value = support::json(&output);
    let operation = value["details"]["operation_record"]["id"].as_str().unwrap();
    let recovery = value["details"]["actions"][0]["milestones"]["recovery_ref"]
        .as_str()
        .unwrap();
    let operation_directory = metadata_dir.join("state/operations").join(operation);
    let recovery_v3 = grip::recovery::model::decode_metadata_v3(
        &fs::read(operation_directory.join(recovery)).unwrap(),
    )
    .unwrap();
    let payload_ref = recovery_v3.payload.payload_ref.as_deref().unwrap();
    assert_eq!(
        fs::read_to_string(
            operation_directory
                .join("recovery/00000000")
                .join(payload_ref)
        )
        .unwrap(),
        "accepted"
    );
    assert_eq!(fs::read_to_string(destination).unwrap(), "new source");
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
    let metadata = grip::mutation::recovery::decode_mutation_recovery_v2(
        &fs::read(recovery_directory.join("recovery-v2.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(metadata.payload.operation_id, operation);
    assert_eq!(metadata.payload.action_index, 0);
    assert_eq!(metadata.payload.private_ref, "payload");
    assert_eq!(
        fs::metadata(recovery_directory.join("recovery-v2.json"))
            .unwrap()
            .mode()
            & 0o7777,
        0o600
    );
}
