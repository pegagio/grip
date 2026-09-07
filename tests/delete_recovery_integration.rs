mod support;
use std::fs;

#[test]
fn deletion_preserves_verified_bytes_before_removal() {
    let (root, grip_home, source, _destination) = support::accepted_file_fixture();
    fs::remove_file(&source).unwrap();
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output=json",
            "delete",
            "--source",
            source.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value = support::json(&output);
    let operation = value["details"]["operation_record"]["id"].as_str().unwrap();
    let recovery = grip_home
        .join("state/operations")
        .join(operation)
        .join("recovery/00000000");
    assert_eq!(fs::read(recovery.join("payload")).unwrap(), b"accepted");
    assert!(recovery.join("manifest.json").is_file());
}
