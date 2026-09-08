mod support;

use grip::metadata::model::Evidence;
use std::path::PathBuf;

fn key(name: &[u8], case_sensitive: bool) -> Vec<u8> {
    let Evidence::Observed { value } =
        grip::metadata::macos::name_comparison_key(name, case_sensitive)
    else {
        panic!("qualified UTF-8 name must produce a comparison key");
    };
    value
}

#[test]
fn comparison_keys_preserve_case_mode_and_detect_canonical_unicode_equivalence() {
    assert_ne!(key(b"Readme", true), key(b"README", true));
    assert_eq!(key(b"Readme", false), key(b"README", false));
    assert_eq!(
        key("caf\u{e9}".as_bytes(), true),
        key("cafe\u{301}".as_bytes(), true)
    );
}

#[test]
fn invalid_utf8_remains_exact_identity_but_comparison_is_inconclusive() {
    assert!(matches!(
        grip::metadata::macos::name_comparison_key(&[0xff, b'a'], false),
        Evidence::Unavailable { .. }
    ));
}

#[test]
fn collision_model_requires_every_distinct_raw_identity() {
    let collision = grip::metadata::model::FilesystemIdentityCollision {
        endpoint: grip::metadata::model::EndpointRole::Destination,
        comparison_evidence: Evidence::Observed {
            value: "nfd-case-fold".into(),
        },
        identity_raw_paths: vec![b"Readme".to_vec(), b"README".to_vec()],
    };
    assert!(collision.validate().is_ok());
    let incomplete = grip::metadata::model::FilesystemIdentityCollision {
        identity_raw_paths: vec![b"same".to_vec(), b"same".to_vec()],
        ..collision
    };
    assert!(incomplete.validate().is_err());
}

#[test]
#[ignore = "requires configured case-sensitive and case-insensitive APFS roots"]
fn case_only_aliases_report_every_identity_before_destination_mutation() {
    let sensitive = PathBuf::from(
        std::env::var_os("GRIP_APFS_CASE_SENSITIVE_ROOT")
            .expect("GRIP_APFS_CASE_SENSITIVE_ROOT is required"),
    );
    let insensitive = PathBuf::from(
        std::env::var_os("GRIP_APFS_CASE_INSENSITIVE_ROOT")
            .expect("GRIP_APFS_CASE_INSENSITIVE_ROOT is required"),
    );
    let source_fixture = tempfile::tempdir_in(sensitive).unwrap();
    let destination_fixture = tempfile::tempdir_in(insensitive).unwrap();
    let source = source_fixture.path().join("source");
    let destination = destination_fixture.path().join("destination");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(source.join("Readme"), b"one").unwrap();
    std::fs::write(source.join("README"), b"two").unwrap();
    let grip_home = support::minimal_home(source_fixture.path());
    support::write_registry(&grip_home, &[("tree", &source, &destination)]);
    let before = support::snapshot(source_fixture.path());
    let inspected = support::command_with_grip_home(
        source_fixture.path(),
        &grip_home,
        &["--output=json", "mapping", "inspect"],
    );
    assert!(inspected.status.success());
    let value = support::json(&inspected);
    let collisions = value["details"]["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|record| record["reason"] == "apfs_name_collision")
        .collect::<Vec<_>>();
    assert_eq!(collisions.len(), 2);
    assert!(
        collisions
            .iter()
            .any(|record| record["relative_path"]["display"] == "Readme")
    );
    assert!(
        collisions
            .iter()
            .any(|record| record["relative_path"]["display"] == "README")
    );
    assert!(!destination.exists());
    assert_eq!(support::snapshot(source_fixture.path()), before);
}
