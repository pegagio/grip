//! Verified private preservation of replacement targets.

use crate::error::GripError;
use crate::observation::model::{EntryIdentity, MappingSnapshot, SupportedState};
use crate::operation::publication::OperationReceipt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Strict metadata binding one recovery payload to an operation action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryMetadataV1 {
    pub schema_version: u8,
    pub operation_id: String,
    pub action_index: usize,
    pub identity: RecoveryIdentityV1,
    pub prior_state: SupportedState,
    pub payload_ref: String,
    pub payload_present: bool,
    pub verified: bool,
}

/// Lossless durable projection of a managed entry identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryIdentityV1 {
    pub mapping: MappingSnapshot,
    pub relative_path_hex: String,
}

/// Opaque operation-local recovery result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryEntry {
    pub relative_ref: String,
}

/// Publish verified Recovery Metadata V2 after complete payload and xattr preservation.
pub fn publish_complete_metadata(
    receipt: &OperationReceipt,
    action_index: usize,
    identity: &EntryIdentity,
    prior_state: &crate::metadata::model::SupportedEntryStateV3,
    payload_ref: Option<String>,
    xattrs: Vec<crate::recovery::model::RecoveryXattrReferenceV2>,
) -> Result<RecoveryEntry, GripError> {
    let directory = receipt
        .directory()
        .join("recovery")
        .join(format!("{action_index:08}"));
    if !directory.exists() {
        crate::mutation::filesystem::create_recovery_directory(&directory)
            .map_err(|error| recovery_failure_at("create recovery directory", error))?;
    }
    let metadata = crate::recovery::model::RecoveryMetadataEnvelopeV2::new(
        crate::recovery::model::RecoveryMetadataPayloadV2 {
            operation_id: receipt.operation_id().into(),
            action_index,
            identity: crate::recovery::model::RecoveryIdentityV2 {
                mapping: identity.mapping.clone(),
                relative_path_hex: identity
                    .relative_path
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect(),
            },
            prior_state: prior_state.clone(),
            payload_ref,
            xattrs,
            private_security: crate::recovery::model::PrivateRecoverySecurityV2 {
                permission_mode: "0600".into(),
                uid: rustix::process::geteuid().as_raw(),
                gid: rustix::process::getegid().as_raw(),
            },
            preserved: true,
            verified: true,
        },
    )?;
    let bytes = serde_json::to_vec(&metadata).map_err(|error| {
        GripError::Internal(format!("could not encode Recovery Metadata V2: {error}"))
    })?;
    crate::operation::publication::publish_new_component(&directory, "metadata-v2.json", &bytes)
        .map_err(|error| recovery_failure_at("publish Recovery Metadata V2", error))?;
    let reread =
        crate::mutation::filesystem::read_private_file(&directory.join("metadata-v2.json"))
            .map_err(|error| recovery_failure_at("read Recovery Metadata V2", error))?;
    let decoded = crate::recovery::model::decode_metadata_v2(&reread)?;
    if decoded != metadata {
        return Err(GripError::CorruptState(
            "Recovery Metadata V2 verification failed".into(),
        ));
    }
    Ok(RecoveryEntry {
        relative_ref: format!("recovery/{action_index:08}/metadata-v2.json"),
    })
}

/// Preserve the losing complete state and publish action-bound Recovery Metadata V2.
pub fn preserve_complete(
    receipt: &OperationReceipt,
    action_index: usize,
    identity: &EntryIdentity,
    target: &Path,
    expected_legacy: &SupportedState,
    prior_state: &crate::metadata::model::SupportedEntryStateV3,
    expected_post: &crate::metadata::model::SupportedEntryStateV3,
) -> Result<RecoveryEntry, GripError> {
    preserve_complete_with_post(
        receipt,
        action_index,
        identity,
        target,
        expected_legacy,
        prior_state,
        Some(expected_post),
    )
}

/// Preserve complete state for a transition whose post-state may be absence.
pub fn preserve_complete_with_post(
    receipt: &OperationReceipt,
    action_index: usize,
    identity: &EntryIdentity,
    target: &Path,
    expected_legacy: &SupportedState,
    prior_state: &crate::metadata::model::SupportedEntryStateV3,
    expected_post: Option<&crate::metadata::model::SupportedEntryStateV3>,
) -> Result<RecoveryEntry, GripError> {
    let directory = receipt
        .directory()
        .join("recovery")
        .join(format!("{action_index:08}"));
    let payload_ref = if prior_state.node_kind == crate::discovery::model::NodeKind::File {
        preserve_with_post(
            receipt,
            action_index,
            identity,
            target,
            expected_legacy,
            expected_post
                .map(|state| SupportedState {
                    node_kind: state.node_kind,
                    content: state.content.clone(),
                    permission_mode: Some(state.metadata.permission_mode.clone()),
                })
                .as_ref(),
        )?;
        "payload"
    } else {
        preserve_with_post(
            receipt,
            action_index,
            identity,
            target,
            expected_legacy,
            expected_post
                .map(|state| SupportedState {
                    node_kind: state.node_kind,
                    content: state.content.clone(),
                    permission_mode: None,
                })
                .as_ref(),
        )?;
        OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(directory.join("metadata-object"))
            .map_err(|error| {
                GripError::from_io("could not create private metadata object", error)
            })?;
        "metadata-object"
    };
    let origin = OpenOptions::new()
        .read(true)
        .open(target)
        .map_err(|error| GripError::from_io("could not open recovery origin", error))?;
    let private = OpenOptions::new()
        .read(true)
        .write(true)
        .open(directory.join(payload_ref))
        .map_err(|error| GripError::from_io("could not open private recovery object", error))?;
    crate::metadata::macos::copy_recovery_acl(&origin, &private)
        .map_err(|error| GripError::from_io("could not preserve recovery ACL", error))?;
    let fingerprints = crate::metadata::macos::copy_synchronized_xattrs(&origin, &private)
        .map_err(|error| GripError::from_io("could not preserve recovery xattrs", error))?;
    let xattrs = fingerprints
        .into_iter()
        .map(
            |fingerprint| crate::recovery::model::RecoveryXattrReferenceV2 {
                fingerprint,
                payload_ref: payload_ref.into(),
                preserved: true,
                verified: true,
            },
        )
        .collect();
    publish_complete_metadata(
        receipt,
        action_index,
        identity,
        prior_state,
        Some(payload_ref.into()),
        xattrs,
    )
}

/// Preserve and verify the current mutation target before replacement.
pub fn preserve(
    receipt: &OperationReceipt,
    action_index: usize,
    identity: &EntryIdentity,
    target: &Path,
    expected: &SupportedState,
) -> Result<RecoveryEntry, GripError> {
    preserve_with_post(receipt, action_index, identity, target, expected, None)
}

/// Preserve a target together with the originating operation's expected post-action state.
pub fn preserve_with_post(
    receipt: &OperationReceipt,
    action_index: usize,
    identity: &EntryIdentity,
    target: &Path,
    expected: &SupportedState,
    expected_post: Option<&SupportedState>,
) -> Result<RecoveryEntry, GripError> {
    let directory = receipt
        .directory()
        .join("recovery")
        .join(format!("{action_index:08}"));
    crate::mutation::filesystem::create_recovery_directory(&directory)
        .map_err(|error| recovery_failure_at("create recovery directory", error))?;
    let payload = directory.join("payload");
    let payload_present = expected.node_kind == crate::discovery::model::NodeKind::File;
    if payload_present {
        let mut staged =
            crate::mutation::filesystem::stage_private_file(target, &payload, expected)
                .map_err(|error| recovery_failure_at("stage recovery payload", error))?;
        crate::mutation::filesystem::publish_addition(&mut staged, &payload)
            .map_err(|error| recovery_failure_at("publish recovery payload", error))?;
        let mut private_expected = expected.clone();
        private_expected.permission_mode = Some("0600".into());
        crate::mutation::filesystem::verify_target(&payload, &private_expected)
            .map_err(|error| recovery_failure_at("verify recovery payload", error))?;
    }
    let relative_ref = if payload_present {
        format!("recovery/{action_index:08}/payload")
    } else {
        format!("recovery/{action_index:08}/metadata.json")
    };
    let metadata = RecoveryMetadataV1 {
        schema_version: 1,
        operation_id: receipt.operation_id().into(),
        action_index,
        identity: RecoveryIdentityV1 {
            mapping: identity.mapping.clone(),
            relative_path_hex: identity
                .relative_path
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        },
        prior_state: expected.clone(),
        payload_ref: relative_ref.clone(),
        payload_present,
        verified: true,
    };
    let bytes = serde_json::to_vec(&metadata).map_err(|error| {
        GripError::Internal(format!("could not encode recovery metadata: {error}"))
    })?;
    crate::operation::publication::publish_new_component(&directory, "metadata.json", &bytes)
        .map_err(|error| recovery_failure_at("publish recovery metadata", error))?;
    let reread = crate::mutation::filesystem::read_private_file(&directory.join("metadata.json"))
        .map_err(|error| recovery_failure_at("read recovery metadata", error))?;
    let decoded: RecoveryMetadataV1 = serde_json::from_slice(&reread)
        .map_err(|error| GripError::CorruptState(format!("invalid recovery metadata: {error}")))?;
    if decoded != metadata {
        return Err(GripError::CorruptState(
            "recovery metadata verification failed".into(),
        ));
    }
    let bound_side = if target == identity.source_path() {
        "source"
    } else if target == identity.destination_path() {
        "destination"
    } else {
        return Err(GripError::CorruptState(
            "recovery target is not bound to the managed identity".into(),
        ));
    };
    let manifest = crate::recovery::model::RecoveryEnvelopeV1::new(
        crate::recovery::model::RecoveryManifestPayloadV1 {
            reference: crate::recovery::model::RecoveryRef::Payload {
                operation_id: receipt.operation_id().into(),
                action_index,
            },
            kind: crate::recovery::model::RecoveryKind::Payload,
            created_at: unix_timestamp(),
            origin_operation: Some(receipt.operation_id().into()),
            origin_transition: if expected_post.is_some() {
                "replacement"
            } else {
                "deletion"
            }
            .into(),
            managed_identity: format!(
                "{}:{}",
                identity.mapping.source.display(),
                metadata.identity.relative_path_hex
            )
            .into(),
            bound_side: Some(bound_side.into()),
            bound_target: Some(target.display().to_string()),
            prior_evidence: serde_json::to_value(expected).map_err(|error| {
                GripError::Internal(format!("could not encode recovery prior evidence: {error}"))
            })?,
            expected_post_evidence: expected_post.map_or(serde_json::Value::Null, |state| {
                serde_json::to_value(state).expect("supported state serialization cannot fail")
            }),
            byte_count: if payload_present {
                fs::metadata(&payload)
                    .map_err(|error| GripError::from_io("could not inspect recovery bytes", error))?
                    .len()
            } else {
                0
            },
            payload_ref: if payload_present {
                "payload"
            } else {
                "metadata"
            }
            .into(),
        },
    )?;
    crate::operation::publication::publish_new_component(
        &directory,
        "manifest.json",
        &crate::recovery::model::encode(&manifest)?,
    )?;
    Ok(RecoveryEntry { relative_ref })
}

fn unix_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| format!("{}Z", duration.as_secs()))
        .unwrap_or_else(|_| "0Z".into())
}

fn recovery_failure_at(phase: &str, error: GripError) -> GripError {
    if matches!(error, GripError::CorruptState(_)) {
        error
    } else {
        GripError::mutation_side_effect(
            "recovery_failure",
            false,
            "not_attempted",
            false,
            format!("{phase}: {error}"),
        )
    }
}
