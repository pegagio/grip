mod support;
use grip::discovery::model::NodeKind;
use grip::mapping::MappingKind;
use grip::metadata::model::{AclState, MetadataState, ModificationTime, SupportedEntryStateV3};
use grip::observation::model::ContentFingerprint;
use std::fs;

fn portable_identity() -> grip::state::EntryIdentityV4 {
    grip::state::EntryIdentityV4 {
        mapping: grip::mapping::PortableMapping::parse(
            MappingKind::File,
            std::ffi::OsStr::new("source"),
            std::ffi::OsStr::new("~/destination"),
        )
        .unwrap(),
        relative_path_hex: String::new(),
    }
}

#[test]
fn portable_recovery_versions_round_trip_without_absolute_authority() {
    let identity = portable_identity();
    let mutation = grip::mutation::recovery::MutationRecoveryV2::new(
        grip::mutation::recovery::MutationRecoveryPayloadV2 {
            operation_id: "push-1".into(),
            action_index: 0,
            identity: identity.clone(),
            endpoint_role: grip::state::EndpointRoleV1::Destination,
            private_ref: "recovery/00000000/payload".into(),
        },
    )
    .unwrap();
    let mutation_bytes = serde_json::to_vec(&mutation).unwrap();
    assert_eq!(
        grip::mutation::recovery::decode_mutation_recovery_v2(&mutation_bytes).unwrap(),
        mutation
    );

    let manifest = grip::recovery::model::RecoveryManifestV2::new(
        grip::recovery::model::RecoveryManifestPayloadV2 {
            reference: grip::recovery::model::RecoveryRef::Payload {
                operation_id: "push-1".into(),
                action_index: 0,
            },
            kind: grip::recovery::model::RecoveryKind::Payload,
            identity: Some(identity.clone()),
            endpoint_role: Some(grip::state::EndpointRoleV1::Destination),
            private_ref: "payload".into(),
            diagnostic_target: None,
            created_at: "1Z".into(),
            origin_operation: Some("push-1".into()),
            origin_transition: "payload_preservation".into(),
            prior_evidence: serde_json::json!({}),
            expected_post_evidence: serde_json::json!({}),
            byte_count: 0,
        },
    )
    .unwrap();
    let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
    assert_eq!(
        grip::recovery::model::decode_manifest_v2(&manifest_bytes).unwrap(),
        manifest
    );

    let metadata = grip::recovery::model::RecoveryMetadataV3::new(
        grip::recovery::model::RecoveryMetadataPayloadV3 {
            operation_id: "push-1".into(),
            action_index: 0,
            identity,
            endpoint_role: grip::state::EndpointRoleV1::Destination,
            prior_state: complete_prior_state(),
            payload_ref: Some("payload".into()),
            preserved: true,
            verified: true,
        },
    )
    .unwrap();
    let metadata_bytes = serde_json::to_vec(&metadata).unwrap();
    assert_eq!(
        grip::recovery::model::decode_metadata_v3(&metadata_bytes).unwrap(),
        metadata
    );
}

#[test]
fn portable_recovery_versions_reject_old_schema_unknown_fields_and_tampering() {
    assert_eq!(
        grip::recovery::model::decode_manifest_v2(br#"{"schema_version":1}"#)
            .unwrap_err()
            .category(),
        grip::ResultCategory::UnsupportedSchema
    );
    let mut mutation = grip::mutation::recovery::MutationRecoveryV2::new(
        grip::mutation::recovery::MutationRecoveryPayloadV2 {
            operation_id: "push-1".into(),
            action_index: 0,
            identity: portable_identity(),
            endpoint_role: grip::state::EndpointRoleV1::Destination,
            private_ref: "payload".into(),
        },
    )
    .unwrap();
    mutation.payload.private_ref = "../escape".into();
    assert!(mutation.validate().is_err());
}

fn complete_prior_state() -> SupportedEntryStateV3 {
    SupportedEntryStateV3 {
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
            extended_attributes: Vec::new(),
            acl: AclState::Absent,
            bsd_flags: Default::default(),
        },
    }
}

#[test]
fn recovery_metadata_v3_rejects_v2_without_compatibility_decode() {
    assert_eq!(
        grip::recovery::model::decode_metadata_v3(br#"{"schema_version":2}"#)
            .unwrap_err()
            .category(),
        grip::ResultCategory::UnsupportedSchema
    );
}

#[test]
fn new_payload_recovery_has_immutable_portable_records() {
    let (root, metadata_dir, _, _, reference) = support::payload_recovery_fixture();
    let operation = reference.split(':').nth(1).unwrap();
    let directory = metadata_dir
        .join("state/operations")
        .join(operation)
        .join("recovery/00000000");
    let manifest = fs::read(directory.join("manifest.json")).unwrap();
    let recovery = fs::read(directory.join("recovery-v2.json")).unwrap();
    let metadata = fs::read(directory.join("metadata-v3.json")).unwrap();
    let show = support::project_command(
        root.path(),
        &metadata_dir,
        &["recovery", "show", &reference],
    );
    assert!(show.status.success());
    assert_eq!(fs::read(directory.join("manifest.json")).unwrap(), manifest);
    assert_eq!(
        fs::read(directory.join("recovery-v2.json")).unwrap(),
        recovery
    );
    assert_eq!(
        fs::read(directory.join("metadata-v3.json")).unwrap(),
        metadata
    );
}
