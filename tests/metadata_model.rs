use grip::discovery::model::NodeKind;
use grip::metadata::model::{
    AccessControlEntry, AclEntryFlag, AclEntryKind, AclPermission, AclState, BsdFlag, Evidence,
    MetadataState, ModificationTime, SupportedEntryStateV3, XattrFingerprint,
};
use std::collections::BTreeSet;

fn metadata() -> MetadataState {
    MetadataState {
        permission_mode: "0644".into(),
        uid: 501,
        gid: 20,
        modified_time: ModificationTime {
            seconds: 1_700_000_000,
            nanoseconds: 123_456_789,
        },
        extended_attributes: vec![XattrFingerprint {
            name: b"com.apple.TextEncoding".to_vec(),
            length: 5,
            algorithm: "sha256".into(),
            digest: "a".repeat(64),
        }],
        acl: AclState::Absent,
        bsd_flags: BTreeSet::from([BsdFlag::Hidden]),
    }
}

#[test]
fn evidence_states_remain_distinct_during_serialization() {
    let values = [
        Evidence::Observed { value: 7_u32 },
        Evidence::Absent,
        Evidence::Unavailable {
            reason: "not returned".into(),
        },
        Evidence::Unsupported {
            reason: "not APFS".into(),
        },
        Evidence::Unreadable {
            reason: "permission denied".into(),
        },
        Evidence::Unauthorized {
            reason: "group unavailable".into(),
        },
    ];
    let encoded = values
        .iter()
        .map(|value| serde_json::to_value(value).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(encoded[0]["state"], "observed");
    assert_eq!(encoded[1]["state"], "absent");
    assert_eq!(encoded[2]["state"], "unavailable");
    assert_eq!(encoded[3]["state"], "unsupported");
    assert_eq!(encoded[4]["state"], "unreadable");
    assert_eq!(encoded[5]["state"], "unauthorized");
}

#[test]
fn complete_state_validates_file_and_directory_contracts() {
    let file = SupportedEntryStateV3 {
        node_kind: NodeKind::File,
        content: Some(grip::observation::model::ContentFingerprint {
            algorithm: "sha256".into(),
            digest: "b".repeat(64),
            length: 8,
        }),
        metadata: metadata(),
    };
    assert!(file.validate().is_ok());

    let mut directory = file.clone();
    directory.node_kind = NodeKind::Directory;
    directory.content = None;
    assert!(directory.validate().is_ok());

    directory.metadata.bsd_flags.insert(BsdFlag::Opaque);
    assert!(directory.validate().is_ok());
    let mut invalid_file = file;
    invalid_file.metadata.bsd_flags.insert(BsdFlag::Opaque);
    assert!(invalid_file.validate().is_err());
}

#[test]
fn metadata_rejects_invalid_time_mode_and_xattr_fingerprint() {
    let mut state = metadata();
    state.modified_time.nanoseconds = 1_000_000_000;
    assert!(state.validate(NodeKind::File).is_err());
    state.modified_time.nanoseconds = 0;
    state.permission_mode = "644".into();
    assert!(state.validate(NodeKind::File).is_err());
    state.permission_mode = "0644".into();
    state.extended_attributes[0].digest = "A".repeat(64);
    assert!(state.validate(NodeKind::File).is_err());
}

#[test]
fn acl_preserves_entry_sequence_but_canonicalizes_sets() {
    let first = AccessControlEntry {
        principal_uuid: [1; 16],
        kind: AclEntryKind::Deny,
        permissions: BTreeSet::from([AclPermission::WriteData, AclPermission::ReadData]),
        flags: BTreeSet::from([AclEntryFlag::Inherited, AclEntryFlag::FileInherit]),
    };
    let second = AccessControlEntry {
        principal_uuid: [2; 16],
        kind: AclEntryKind::Allow,
        permissions: BTreeSet::from([AclPermission::ReadData]),
        flags: BTreeSet::new(),
    };
    let acl = AclState::Present {
        entries: vec![first.clone(), second.clone()],
    };
    let reversed = AclState::Present {
        entries: vec![second, first],
    };
    assert_ne!(acl, reversed);
    let encoded = serde_json::to_string(&acl).unwrap();
    assert!(encoded.find("read_data").unwrap() < encoded.find("write_data").unwrap());
}

#[test]
fn metadata_values_and_xattr_policy_are_exact_and_canonical() {
    let state = metadata();
    assert_eq!(state.permission_mode, "0644");
    assert_eq!(state.uid, 501);
    assert_eq!(state.gid, 20);
    assert_eq!(state.modified_time.nanoseconds, 123_456_789);
    assert!(matches!(
        grip::metadata::xattr_policy(b"com.apple.ResourceFork"),
        grip::metadata::XattrPolicy::Synchronized
    ));
    assert!(matches!(
        grip::metadata::xattr_policy(b"com.apple.quarantine"),
        grip::metadata::XattrPolicy::Excluded
    ));
    assert!(matches!(
        grip::metadata::xattr_policy(b"com.example.unknown"),
        grip::metadata::XattrPolicy::Unknown
    ));
    assert_ne!(
        grip::metadata::xattr_policy(b"Com.Apple.ResourceFork"),
        grip::metadata::XattrPolicy::Synchronized
    );
    assert_eq!(
        BTreeSet::from([
            BsdFlag::Nodump,
            BsdFlag::Immutable,
            BsdFlag::Append,
            BsdFlag::Hidden,
            BsdFlag::Opaque,
        ])
        .len(),
        5
    );
}
