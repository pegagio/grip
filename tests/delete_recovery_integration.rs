mod support;
use std::fs;

#[test]
fn deletion_preserves_verified_bytes_before_removal() {
    let (root, metadata_dir, source, _destination) = support::accepted_file_fixture();
    fs::remove_file(&source).unwrap();
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "delete", "--source", "source"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value = support::json(&output);
    let operation = value["details"]["operation_record"]["id"].as_str().unwrap();
    let recovery = metadata_dir
        .join("state/operations")
        .join(operation)
        .join("recovery/00000000");
    assert_eq!(fs::read(recovery.join("payload")).unwrap(), b"accepted");
    assert!(recovery.join("manifest.json").is_file());
}
