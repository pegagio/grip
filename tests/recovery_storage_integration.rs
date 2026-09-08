mod support;
use grip::discovery::model::NodeKind;
use grip::mapping::MappingKind;
use grip::metadata::model::{
    AclState, MetadataState, ModificationTime, SupportedEntryStateV3, XattrFingerprint,
};
use grip::observation::model::{ContentFingerprint, MappingSnapshot};
use grip::recovery::model::{
    PrivateRecoverySecurityV2, RecoveryIdentityV2, RecoveryMetadataEnvelopeV2,
    RecoveryMetadataPayloadV2, RecoveryXattrReferenceV2,
};
use std::fs;
use std::path::PathBuf;

fn recovery_v2() -> RecoveryMetadataEnvelopeV2 {
    let xattr = XattrFingerprint {
        name: b"com.apple.ResourceFork".to_vec(),
        length: 4,
        algorithm: "sha256".into(),
        digest: "d".repeat(64),
    };
    RecoveryMetadataEnvelopeV2::new(RecoveryMetadataPayloadV2 {
        operation_id: "push-123".into(),
        action_index: 7,
        identity: RecoveryIdentityV2 {
            mapping: MappingSnapshot {
                kind: MappingKind::Tree,
                source: PathBuf::from("/source"),
                destination: PathBuf::from("/destination"),
            },
            relative_path_hex: "66696c65".into(),
        },
        prior_state: SupportedEntryStateV3 {
            node_kind: NodeKind::File,
            content: Some(ContentFingerprint {
                algorithm: "sha256".into(),
                digest: "a".repeat(64),
                length: 4,
            }),
            metadata: MetadataState {
                permission_mode: "0644".into(),
                uid: 501,
                gid: 20,
                modified_time: ModificationTime {
                    seconds: 12,
                    nanoseconds: 34,
                },
                extended_attributes: vec![xattr.clone()],
                acl: AclState::Absent,
                bsd_flags: Default::default(),
            },
        },
        payload_ref: Some("payload".into()),
        xattrs: vec![RecoveryXattrReferenceV2 {
            fingerprint: xattr,
            payload_ref: "xattr-00000000".into(),
            preserved: true,
            verified: true,
        }],
        private_security: PrivateRecoverySecurityV2 {
            permission_mode: "0600".into(),
            uid: 501,
            gid: 20,
        },
        preserved: true,
        verified: true,
    })
    .unwrap()
}

#[test]
fn recovery_metadata_v2_binds_action_complete_state_and_private_security() {
    let envelope = recovery_v2();
    let bytes = serde_json::to_vec(&envelope).unwrap();
    let decoded = grip::recovery::model::decode_metadata_v2(&bytes).unwrap();
    assert_eq!(decoded.payload.operation_id, "push-123");
    assert_eq!(decoded.payload.action_index, 7);
    assert_eq!(decoded.payload.prior_state.metadata.permission_mode, "0644");
    assert_eq!(decoded.payload.private_security.permission_mode, "0600");
    assert_ne!(
        decoded.payload.prior_state.metadata.permission_mode,
        decoded.payload.private_security.permission_mode
    );
}

#[test]
fn recovery_metadata_v2_rejects_tampering_unknown_fields_and_unverified_xattrs() {
    let envelope = recovery_v2();
    let mut value = serde_json::to_value(&envelope).unwrap();
    value["payload"]["unknown"] = serde_json::json!(true);
    assert!(
        grip::recovery::model::decode_metadata_v2(&serde_json::to_vec(&value).unwrap()).is_err()
    );

    let mut tampered = envelope.clone();
    tampered.payload.xattrs[0].verified = false;
    assert!(tampered.validate().is_err());

    let mut digest_tampered = envelope;
    digest_tampered.integrity.digest.replace_range(..1, "0");
    assert!(digest_tampered.validate().is_err());
}

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
