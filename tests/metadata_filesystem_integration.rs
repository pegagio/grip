mod support;

use grip::discovery::model::NodeKind;
use sha2::{Digest, Sha256};

#[test]
fn descriptor_observation_captures_complete_file_metadata_and_xattr_policy() {
    let fixture = support::MetadataFixture::file(b"payload");
    support::set_fixture_mode(&fixture.source, 0o764);
    support::set_fixture_modified_time(&fixture.source, 1_700_000_000, 987_654_321);
    support::set_fixture_xattr(&fixture.source, "com.apple.TextEncoding", b"utf-8");
    support::set_fixture_xattr(&fixture.source, "com.apple.quarantine", b"excluded-value");
    support::set_fixture_xattr(&fixture.source, "com.example.unknown", b"unknown-value");

    let observed =
        grip::observation::fingerprint::inspect_complete(&fixture.source, NodeKind::File).unwrap();
    assert_eq!(observed.state.metadata.permission_mode, "0764");
    assert_eq!(observed.state.metadata.modified_time.seconds, 1_700_000_000);
    assert_eq!(
        observed.state.metadata.modified_time.nanoseconds,
        987_654_321
    );
    assert_eq!(observed.state.metadata.extended_attributes.len(), 1);
    let fingerprint = &observed.state.metadata.extended_attributes[0];
    assert_eq!(fingerprint.name, b"com.apple.TextEncoding");
    assert_eq!(fingerprint.length, 5);
    assert_eq!(
        fingerprint.digest,
        format!("{:x}", Sha256::digest(b"utf-8"))
    );
    assert!(
        observed
            .excluded_xattrs
            .contains(&b"com.apple.quarantine".to_vec())
    );
    assert_eq!(
        observed.unknown_xattrs,
        vec![b"com.example.unknown".to_vec()]
    );
}

#[test]
fn descriptor_observation_captures_directory_metadata_independently_of_children() {
    let fixture = support::MetadataFixture::tree();
    support::set_fixture_mode(&fixture.source, 0o750);
    support::set_fixture_modified_time(&fixture.source, 1_600_000_000, 123);
    let observed =
        grip::observation::fingerprint::inspect_complete(&fixture.source, NodeKind::Directory)
            .unwrap();
    assert!(observed.state.content.is_none());
    assert_eq!(observed.state.metadata.permission_mode, "0750");
    assert_eq!(observed.state.metadata.modified_time.nanoseconds, 123);
}

#[test]
fn ordered_acl_inheritance_and_supported_bsd_flags_round_trip_exactly() {
    let file = support::MetadataFixture::file(b"same");
    support::set_fixture_acl(
        &file.source,
        &["everyone allow read", "staff allow readsecurity"],
    );
    support::set_fixture_bsd_flags(&file.source, libc::UF_NODUMP | libc::UF_HIDDEN);
    let expected = grip::observation::fingerprint::inspect_complete(&file.source, NodeKind::File)
        .unwrap()
        .state;
    let grip::metadata::model::AclState::Present { entries } = &expected.metadata.acl else {
        panic!("fixture ACL must be present");
    };
    assert_eq!(entries.len(), 2);
    assert_eq!(
        expected.metadata.bsd_flags,
        std::collections::BTreeSet::from([
            grip::metadata::model::BsdFlag::Nodump,
            grip::metadata::model::BsdFlag::Hidden,
        ])
    );
    grip::metadata::macos::apply_metadata_paths(
        &file.source,
        &file.destination,
        NodeKind::File,
        &expected.metadata,
    )
    .unwrap();
    assert_eq!(
        grip::observation::fingerprint::inspect_complete(&file.destination, NodeKind::File)
            .unwrap()
            .state,
        expected
    );
    support::set_fixture_bsd_flags(&file.source, 0);
    support::set_fixture_bsd_flags(&file.destination, 0);
    support::clear_fixture_acl(&file.source);
    support::clear_fixture_acl(&file.destination);

    let directory = support::MetadataFixture::tree();
    support::set_fixture_acl(
        &directory.source,
        &["everyone allow list,search,file_inherit,directory_inherit"],
    );
    support::set_fixture_bsd_flags(&directory.source, libc::UF_OPAQUE);
    let observed =
        grip::observation::fingerprint::inspect_complete(&directory.source, NodeKind::Directory)
            .unwrap()
            .state;
    let grip::metadata::model::AclState::Present { entries } = &observed.metadata.acl else {
        panic!("directory ACL must be present");
    };
    assert!(
        entries[0]
            .flags
            .contains(&grip::metadata::model::AclEntryFlag::FileInherit)
    );
    assert!(
        entries[0]
            .flags
            .contains(&grip::metadata::model::AclEntryFlag::DirectoryInherit)
    );
    assert!(
        observed
            .metadata
            .bsd_flags
            .contains(&grip::metadata::model::BsdFlag::Opaque)
    );
    support::set_fixture_bsd_flags(&directory.source, 0);
    support::clear_fixture_acl(&directory.source);
}

#[test]
fn empty_and_large_resource_forks_use_exact_length_and_digest() {
    let fixture = support::MetadataFixture::file(b"payload");
    support::set_fixture_xattr(&fixture.source, "com.apple.TextEncoding", b"");
    let empty =
        grip::observation::fingerprint::inspect_complete(&fixture.source, NodeKind::File).unwrap();
    assert_eq!(
        empty.state.metadata.extended_attributes[0].name,
        b"com.apple.TextEncoding"
    );
    assert_eq!(empty.state.metadata.extended_attributes[0].length, 0);

    let large = vec![0x5a; 2 * 1024 * 1024];
    support::remove_fixture_xattr(&fixture.source, "com.apple.TextEncoding");
    support::set_fixture_xattr(&fixture.source, "com.apple.ResourceFork", &large);
    let observed =
        grip::observation::fingerprint::inspect_complete(&fixture.source, NodeKind::File).unwrap();
    assert_eq!(
        observed.state.metadata.extended_attributes[0].length,
        large.len() as u64
    );
    assert_eq!(
        observed.state.metadata.extended_attributes[0].digest,
        format!("{:x}", Sha256::digest(&large))
    );
}

#[test]
fn xattr_size_race_is_retried_and_returns_one_stable_exact_value() {
    let fixture = support::MetadataFixture::file(b"payload");
    support::set_fixture_xattr(&fixture.source, "com.apple.TextEncoding", b"small");
    let file = std::fs::File::open(&fixture.source).unwrap();
    let replacement = vec![0x41; 256 * 1024];
    let mut changed = false;
    let observed = grip::metadata::macos::observe_xattrs_with_hook(&file, |attempt, name| {
        if attempt == 0 && name == b"com.apple.TextEncoding" && !changed {
            support::set_fixture_xattr(&fixture.source, "com.apple.TextEncoding", &replacement);
            changed = true;
        }
    })
    .unwrap();
    assert!(changed);
    assert_eq!(observed.synchronized.len(), 1);
    assert_eq!(observed.synchronized[0].length, replacement.len() as u64);
    assert_eq!(
        observed.synchronized[0].digest,
        format!("{:x}", Sha256::digest(&replacement))
    );
}

#[test]
fn complete_metadata_application_reproduces_exact_supported_state() {
    let fixture = support::MetadataFixture::file(b"same-content");
    support::set_fixture_mode(&fixture.source, 0o751);
    support::set_fixture_mode(&fixture.destination, 0o600);
    support::set_fixture_modified_time(&fixture.source, 1_500_000_000, 765_432_100);
    support::set_fixture_modified_time(&fixture.destination, 1_400_000_000, 1);
    support::set_fixture_xattr(
        &fixture.source,
        "com.apple.TextEncoding",
        b"utf-8;134217984",
    );

    let source =
        grip::observation::fingerprint::inspect_complete(&fixture.source, NodeKind::File).unwrap();
    grip::metadata::macos::apply_metadata_paths(
        &fixture.source,
        &fixture.destination,
        NodeKind::File,
        &source.state.metadata,
    )
    .unwrap();
    let destination =
        grip::observation::fingerprint::inspect_complete(&fixture.destination, NodeKind::File)
            .unwrap();
    assert_eq!(destination.state, source.state);
    assert_eq!(
        support::fixture_xattr(&fixture.destination, "com.apple.TextEncoding"),
        b"utf-8;134217984"
    );
}

#[test]
fn metadata_only_pull_reproduces_the_complete_destination_state_and_converges() {
    let fixture = support::MetadataFixture::file(b"same-content");
    support::copy_complete_metadata(&fixture.source, &fixture.destination, NodeKind::File);
    let accepted = support::command_with_grip_home(
        fixture.root.path(),
        &fixture.grip_home,
        &["baseline", "accept"],
    );
    assert!(accepted.status.success());

    support::set_fixture_mode(&fixture.destination, 0o740);
    support::set_fixture_modified_time(&fixture.destination, 1_510_000_000, 456_789_123);
    support::set_fixture_xattr(
        &fixture.destination,
        "com.apple.TextEncoding",
        b"utf-8;134217984",
    );
    let destination_before =
        grip::observation::fingerprint::inspect_complete(&fixture.destination, NodeKind::File)
            .unwrap()
            .state;
    let preview = support::command_with_grip_home(
        fixture.root.path(),
        &fixture.grip_home,
        &["--output=json", "pull", "--dry-run"],
    );
    assert!(preview.status.success());
    assert_eq!(
        support::json(&preview)["details"]["actions"][0]["kind"],
        "apply_metadata"
    );
    let pull = support::command_with_grip_home(
        fixture.root.path(),
        &fixture.grip_home,
        &["--output=json", "pull"],
    );
    assert!(
        pull.status.success(),
        "{}",
        String::from_utf8_lossy(&pull.stdout)
    );
    assert_eq!(
        grip::observation::fingerprint::inspect_complete(&fixture.source, NodeKind::File)
            .unwrap()
            .state,
        destination_before
    );
    let status = support::command_with_grip_home(
        fixture.root.path(),
        &fixture.grip_home,
        &["--output=json", "status"],
    );
    assert_eq!(
        support::json(&status)["details"]["records"][0]["classification"],
        "synchronized"
    );
}

#[test]
fn directory_metadata_pull_runs_after_child_transfer_and_preserves_neighbors() {
    let fixture = support::MetadataFixture::tree();
    support::copy_tree_entry_metadata(&fixture.source, &fixture.destination);
    support::copy_complete_metadata(&fixture.source, &fixture.destination, NodeKind::Directory);
    let accepted = support::command_with_grip_home(
        fixture.root.path(),
        &fixture.grip_home,
        &["baseline", "accept"],
    );
    assert!(accepted.status.success());

    let destination_directory = fixture.destination.join("nested");
    support::set_fixture_mode(&destination_directory, 0o710);
    support::set_fixture_modified_time(&destination_directory, 1_520_000_000, 987);
    std::fs::write(destination_directory.join("file"), b"destination child").unwrap();
    let neighbor = fixture.source.join("empty");
    let neighbor_before = support::snapshot(&neighbor);
    let pull = support::command_with_grip_home(
        fixture.root.path(),
        &fixture.grip_home,
        &["--output=json", "pull"],
    );
    assert!(
        pull.status.success(),
        "{}",
        String::from_utf8_lossy(&pull.stdout)
    );
    assert_eq!(
        std::fs::read(fixture.source.join("nested/file")).unwrap(),
        b"destination child"
    );
    assert_eq!(
        grip::observation::fingerprint::inspect_complete(
            &fixture.source.join("nested"),
            NodeKind::Directory,
        )
        .unwrap()
        .state,
        grip::observation::fingerprint::inspect_complete(
            &fixture.destination.join("nested"),
            NodeKind::Directory,
        )
        .unwrap()
        .state
    );
    assert_eq!(support::snapshot(&neighbor), neighbor_before);
}

#[test]
fn directory_metadata_is_finalized_after_descendants_and_published_in_state_v3() {
    let fixture = support::MetadataFixture::tree();
    for relative in ["nested/file", "nested", "empty", ""] {
        let source = fixture.source.join(relative);
        let destination = fixture.destination.join(relative);
        let kind = if source.is_dir() {
            NodeKind::Directory
        } else {
            NodeKind::File
        };
        let observed = grip::observation::fingerprint::inspect_complete(&source, kind).unwrap();
        grip::metadata::macos::apply_metadata_paths(
            &source,
            &destination,
            kind,
            &observed.state.metadata,
        )
        .unwrap();
    }
    let accepted = support::command_with_grip_home(
        fixture.root.path(),
        &fixture.grip_home,
        &["baseline", "accept"],
    );
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stdout)
    );
    support::set_fixture_mode(&fixture.source.join("nested"), 0o711);
    support::set_fixture_modified_time(&fixture.source.join("nested"), 1_234_567_890, 999);
    let preview = support::command_with_grip_home(
        fixture.root.path(),
        &fixture.grip_home,
        &["--output", "json", "push", "--dry-run"],
    );
    assert!(preview.status.success());
    assert_eq!(
        support::json(&preview)["details"]["actions"][0]["kind"],
        "finalize_directory_metadata"
    );
    let push = support::command_with_grip_home(fixture.root.path(), &fixture.grip_home, &["push"]);
    assert!(
        push.status.success(),
        "{}",
        String::from_utf8_lossy(&push.stdout)
    );
    let source = grip::observation::fingerprint::inspect_complete(
        &fixture.source.join("nested"),
        NodeKind::Directory,
    )
    .unwrap();
    let destination = grip::observation::fingerprint::inspect_complete(
        &fixture.destination.join("nested"),
        NodeKind::Directory,
    )
    .unwrap();
    assert_eq!(source.state, destination.state);
    let state = std::fs::read(fixture.grip_home.join("state/state.json")).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&state).unwrap()["schema_version"],
        3
    );
}

#[test]
fn file_addition_applies_complete_metadata_before_state_v3_publication() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    std::fs::write(&source, b"payload").unwrap();
    support::set_fixture_mode(&source, 0o751);
    support::set_fixture_modified_time(&source, 1_600_000_001, 456);
    support::set_fixture_xattr(&source, "com.apple.TextEncoding", b"utf-8");
    support::write_registry(&grip_home, &[("file", &source, &destination)]);
    let push = support::command_with_grip_home(root.path(), &grip_home, &["push"]);
    assert!(
        push.status.success(),
        "{}",
        String::from_utf8_lossy(&push.stdout)
    );
    let source_state =
        grip::observation::fingerprint::inspect_complete(&source, NodeKind::File).unwrap();
    let destination_state =
        grip::observation::fingerprint::inspect_complete(&destination, NodeKind::File).unwrap();
    assert_eq!(source_state.state, destination_state.state);
    let state = std::fs::read(grip_home.join("state/state.json")).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&state).unwrap()["schema_version"],
        3
    );
}

#[test]
fn full_replacement_transfers_content_and_metadata_as_one_complete_entry() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    std::fs::write(&source, b"accepted").unwrap();
    support::write_registry(&grip_home, &[("file", &source, &destination)]);
    assert!(
        support::command_with_grip_home(root.path(), &grip_home, &["push"])
            .status
            .success()
    );

    std::fs::write(&source, b"replacement").unwrap();
    support::set_fixture_mode(&source, 0o741);
    support::set_fixture_modified_time(&source, 1_530_000_000, 321);
    support::set_fixture_xattr(&source, "com.apple.TextEncoding", b"utf-8");
    let expected = grip::observation::fingerprint::inspect_complete(&source, NodeKind::File)
        .unwrap()
        .state;
    let push = support::command_with_grip_home(root.path(), &grip_home, &["push"]);
    assert!(
        push.status.success(),
        "{}",
        String::from_utf8_lossy(&push.stdout)
    );
    assert_eq!(
        grip::observation::fingerprint::inspect_complete(&destination, NodeKind::File)
            .unwrap()
            .state,
        expected
    );
}

#[test]
fn tree_addition_finalizes_directories_deepest_first_after_child_creation() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    std::fs::create_dir_all(source.join("nested/deep")).unwrap();
    std::fs::write(source.join("nested/deep/file"), b"payload").unwrap();
    support::set_fixture_mode(&source, 0o751);
    support::set_fixture_mode(&source.join("nested"), 0o711);
    support::set_fixture_mode(&source.join("nested/deep"), 0o700);
    support::write_registry(&grip_home, &[("tree", &source, &destination)]);
    let preview = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output", "json", "push", "--dry-run"],
    );
    assert!(
        preview.status.success(),
        "{}",
        String::from_utf8_lossy(&preview.stdout)
    );
    let value = support::json(&preview);
    let actions = value["details"]["actions"].as_array().unwrap();
    let finalizers = actions
        .iter()
        .filter(|action| action["kind"] == "finalize_directory_metadata")
        .collect::<Vec<_>>();
    assert_eq!(finalizers.len(), 2);
    assert!(
        finalizers[0]["destination_path"]["display"]
            .as_str()
            .unwrap()
            .ends_with("nested/deep")
    );
    let push = support::command_with_grip_home(root.path(), &grip_home, &["push"]);
    assert!(
        push.status.success(),
        "{}",
        String::from_utf8_lossy(&push.stdout)
    );
    for relative in ["nested/deep/file", "nested/deep", "nested"] {
        let source_path = source.join(relative);
        let destination_path = destination.join(relative);
        let kind = if source_path.is_dir() {
            NodeKind::Directory
        } else {
            NodeKind::File
        };
        assert_eq!(
            grip::observation::fingerprint::inspect_complete(&source_path, kind)
                .unwrap()
                .state,
            grip::observation::fingerprint::inspect_complete(&destination_path, kind)
                .unwrap()
                .state,
        );
    }
}
