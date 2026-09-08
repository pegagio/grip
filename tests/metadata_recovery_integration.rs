mod support;

use grip::discovery::model::NodeKind;

#[test]
fn metadata_only_recovery_restores_the_exact_prior_complete_state() {
    let (root, grip_home, source, destination) = support::accepted_file_fixture();
    let prior = grip::observation::fingerprint::inspect_complete(&destination, NodeKind::File)
        .unwrap()
        .state;
    support::set_fixture_mode(&source, 0o600);
    support::set_fixture_modified_time(&source, 1_600_000_123, 456_789_012);
    support::set_fixture_xattr(&source, "com.apple.TextEncoding", b"utf-8");

    let pushed =
        support::command_with_grip_home(root.path(), &grip_home, &["--output=json", "push"]);
    assert!(
        pushed.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&pushed.stdout),
        String::from_utf8_lossy(&pushed.stderr)
    );
    let operation = support::json(&pushed)["details"]["operation_record"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let reference = format!("payload:{operation}:0");
    let metadata_path = grip_home
        .join("state/operations")
        .join(&operation)
        .join("recovery/00000000/metadata-v2.json");
    assert!(metadata_path.is_file());

    let restored = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "restore", &reference],
    );
    assert!(
        restored.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&restored.stdout),
        String::from_utf8_lossy(&restored.stderr)
    );
    assert_eq!(
        grip::observation::fingerprint::inspect_complete(&destination, NodeKind::File)
            .unwrap()
            .state,
        prior
    );
}

#[test]
fn directory_metadata_recovery_restores_in_place_without_removing_children() {
    let (root, grip_home, source, destination) = support::accepted_tree_fixture();
    let source_directory = source.join("nested");
    let destination_directory = destination.join("nested");
    let prior = grip::observation::fingerprint::inspect_complete(
        &destination_directory,
        NodeKind::Directory,
    )
    .unwrap()
    .state;
    support::set_fixture_mode(&source_directory, 0o700);
    support::set_fixture_modified_time(&source_directory, 1_600_000_222, 987_654_321);

    let pushed = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "push", source_directory.to_str().unwrap()],
    );
    assert!(
        pushed.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&pushed.stdout),
        String::from_utf8_lossy(&pushed.stderr)
    );
    let value = support::json(&pushed);
    let operation = value["details"]["operation_record"]["id"].as_str().unwrap();
    let action_index = value["details"]["actions"][0]["index"].as_u64().unwrap();
    let reference = format!("payload:{operation}:{action_index}");

    let restored = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "recovery", "restore", &reference],
    );
    assert!(
        restored.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&restored.stdout),
        String::from_utf8_lossy(&restored.stderr)
    );
    assert_eq!(
        grip::observation::fingerprint::inspect_complete(
            &destination_directory,
            NodeKind::Directory,
        )
        .unwrap()
        .state,
        prior
    );
    assert_eq!(
        std::fs::read_to_string(destination_directory.join("file")).unwrap(),
        "accepted"
    );
}

#[test]
fn recovery_v2_retains_empty_and_large_xattr_values_on_private_payloads() {
    for (name, prior_value) in [
        ("com.apple.TextEncoding", Vec::new()),
        ("com.apple.ResourceFork", vec![0x5a; 2 * 1024 * 1024]),
    ] {
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let grip_home = support::minimal_home(root.path());
        let source = root.path().join("source");
        let destination = root.path().join("destination");
        std::fs::write(&source, b"accepted").unwrap();
        support::set_fixture_xattr(&source, name, &prior_value);
        support::write_registry(&grip_home, &[("file", &source, &destination)]);
        assert!(
            support::command_with_grip_home(root.path(), &grip_home, &["push"])
                .status
                .success()
        );
        let prior = grip::observation::fingerprint::inspect_complete(&destination, NodeKind::File)
            .unwrap()
            .state;
        let prior_bytes = support::fixture_xattr(&destination, name);

        support::remove_fixture_xattr(&source, name);
        support::set_fixture_mode(&source, 0o600);
        let pushed =
            support::command_with_grip_home(root.path(), &grip_home, &["--output=json", "push"]);
        assert!(pushed.status.success());
        let operation = support::json(&pushed)["details"]["operation_record"]["id"]
            .as_str()
            .unwrap()
            .to_owned();
        let reference = format!("payload:{operation}:0");
        let restored = support::command_with_grip_home(
            root.path(),
            &grip_home,
            &["recovery", "restore", &reference],
        );
        assert!(
            restored.status.success(),
            "{}",
            String::from_utf8_lossy(&restored.stdout)
        );
        assert_eq!(
            grip::observation::fingerprint::inspect_complete(&destination, NodeKind::File)
                .unwrap()
                .state,
            prior
        );
        assert_eq!(support::fixture_xattr(&destination, name), prior_bytes);
    }
}

#[test]
fn recovery_v2_restores_ordered_acl_and_supported_bsd_flags() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    std::fs::write(&source, b"accepted").unwrap();
    support::set_fixture_acl(
        &source,
        &["everyone allow read", "staff allow readsecurity"],
    );
    support::set_fixture_bsd_flags(&source, libc::UF_NODUMP | libc::UF_HIDDEN);
    support::write_registry(&grip_home, &[("file", &source, &destination)]);
    assert!(
        support::command_with_grip_home(root.path(), &grip_home, &["push"])
            .status
            .success()
    );
    let prior = grip::observation::fingerprint::inspect_complete(&destination, NodeKind::File)
        .unwrap()
        .state;

    support::set_fixture_bsd_flags(&source, 0);
    support::clear_fixture_acl(&source);
    support::set_fixture_mode(&source, 0o600);
    let pushed =
        support::command_with_grip_home(root.path(), &grip_home, &["--output=json", "push"]);
    assert!(
        pushed.status.success(),
        "{}",
        String::from_utf8_lossy(&pushed.stdout)
    );
    let operation = support::json(&pushed)["details"]["operation_record"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let restored = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["recovery", "restore", &format!("payload:{operation}:0")],
    );
    assert!(
        restored.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&restored.stdout),
        String::from_utf8_lossy(&restored.stderr)
    );
    assert_eq!(
        grip::observation::fingerprint::inspect_complete(&destination, NodeKind::File)
            .unwrap()
            .state,
        prior
    );
    support::set_fixture_bsd_flags(&destination, 0);
    support::clear_fixture_acl(&destination);
}
