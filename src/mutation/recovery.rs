//! Verified private preservation of replacement targets.

use crate::error::GripError;
use crate::observation::model::{EntryIdentity, SupportedState};
use crate::operation::publication::OperationReceipt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationRecoveryPayloadV2 {
    pub operation_id: String,
    pub action_index: usize,
    pub identity: crate::state::EntryIdentityV4,
    pub endpoint_role: crate::state::EndpointRoleV1,
    pub private_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationRecoveryV2 {
    pub schema_version: u8,
    pub payload: MutationRecoveryPayloadV2,
    pub integrity: crate::state::IntegrityV1,
}

impl MutationRecoveryV2 {
    pub fn new(payload: MutationRecoveryPayloadV2) -> Result<Self, GripError> {
        validate_mutation_recovery(&payload)?;
        Ok(Self {
            schema_version: 2,
            integrity: crate::state::IntegrityV1 {
                algorithm: "sha256".into(),
                digest: mutation_recovery_digest(&payload)?,
            },
            payload,
        })
    }

    pub fn validate(&self) -> Result<(), GripError> {
        if self.schema_version != 2 {
            return Err(GripError::UnsupportedSchema(format!(
                "unsupported mutation recovery schema version {}",
                self.schema_version
            )));
        }
        validate_mutation_recovery(&self.payload)?;
        if self.integrity.algorithm != "sha256"
            || self.integrity.digest != mutation_recovery_digest(&self.payload)?
        {
            return Err(GripError::CorruptState(
                "mutation recovery integrity verification failed".into(),
            ));
        }
        Ok(())
    }
}

pub fn decode_mutation_recovery_v2(bytes: &[u8]) -> Result<MutationRecoveryV2, GripError> {
    let raw: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| GripError::CorruptState(format!("invalid mutation recovery: {error}")))?;
    let version = raw
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            GripError::CorruptState("mutation recovery schema_version is required".into())
        })?;
    if version != 2 {
        return Err(GripError::UnsupportedSchema(format!(
            "unsupported mutation recovery schema version {version}"
        )));
    }
    let value: MutationRecoveryV2 = serde_json::from_value(raw).map_err(|error| {
        GripError::CorruptState(format!("invalid Mutation Recovery V2: {error}"))
    })?;
    value.validate()?;
    Ok(value)
}

fn validate_mutation_recovery(payload: &MutationRecoveryPayloadV2) -> Result<(), GripError> {
    if payload.operation_id.is_empty() {
        return Err(GripError::CorruptState(
            "mutation recovery operation identity is invalid".into(),
        ));
    }
    crate::registry::ProjectDescriptorV2::new(vec![payload.identity.mapping.clone()])
        .map_err(|_| GripError::CorruptState("mutation recovery identity is invalid".into()))?;
    crate::state::decode_v4_identity_path(&payload.identity.relative_path_hex)?;
    let private = Path::new(&payload.private_ref);
    if payload.private_ref.is_empty()
        || private.is_absolute()
        || private
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(GripError::CorruptState(
            "mutation recovery private reference is invalid".into(),
        ));
    }
    Ok(())
}

fn mutation_recovery_digest(payload: &MutationRecoveryPayloadV2) -> Result<String, GripError> {
    #[derive(Serialize)]
    struct Input<'a> {
        schema_version: u8,
        payload: &'a MutationRecoveryPayloadV2,
    }
    let bytes = serde_json::to_vec(&Input {
        schema_version: 2,
        payload,
    })
    .map_err(|error| GripError::Internal(format!("could not encode mutation recovery: {error}")))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

/// Opaque operation-local recovery result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryEntry {
    pub relative_ref: String,
}

/// Publish verified Recovery Metadata V3 after complete payload and xattr preservation.
pub fn publish_complete_metadata(
    receipt: &OperationReceipt,
    action_index: usize,
    identity: &EntryIdentity,
    prior_state: &crate::metadata::model::SupportedEntryStateV3,
    payload_ref: Option<String>,
    endpoint_role: crate::state::EndpointRoleV1,
) -> Result<RecoveryEntry, GripError> {
    let directory = receipt
        .directory()
        .join("recovery")
        .join(format!("{action_index:08}"));
    if !directory.exists() {
        crate::mutation::filesystem::create_recovery_directory(&directory)
            .map_err(|error| recovery_failure_at("create recovery directory", error))?;
    }
    let metadata = crate::recovery::model::RecoveryMetadataV3::new(
        crate::recovery::model::RecoveryMetadataPayloadV3 {
            operation_id: receipt.operation_id().into(),
            action_index,
            identity: receipt.portable_identity(identity)?,
            endpoint_role,
            prior_state: prior_state.clone(),
            payload_ref,
            preserved: true,
            verified: true,
        },
    )?;
    let bytes = serde_json::to_vec(&metadata).map_err(|error| {
        GripError::Internal(format!("could not encode Recovery Metadata V3: {error}"))
    })?;
    crate::operation::publication::publish_new_component(&directory, "metadata-v3.json", &bytes)
        .map_err(|error| recovery_failure_at("publish Recovery Metadata V3", error))?;
    let reread =
        crate::mutation::filesystem::read_private_file(&directory.join("metadata-v3.json"))
            .map_err(|error| recovery_failure_at("read Recovery Metadata V3", error))?;
    let decoded = crate::recovery::model::decode_metadata_v3(&reread)?;
    if decoded != metadata {
        return Err(GripError::CorruptState(
            "Recovery Metadata V3 verification failed".into(),
        ));
    }
    Ok(RecoveryEntry {
        relative_ref: format!("recovery/{action_index:08}/metadata-v3.json"),
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
    crate::metadata::macos::copy_synchronized_xattrs(&origin, &private)
        .map_err(|error| GripError::from_io("could not preserve recovery xattrs", error))?;
    let endpoint_role = if target == identity.source_path() {
        crate::state::EndpointRoleV1::Source
    } else if target == identity.destination_path() {
        crate::state::EndpointRoleV1::Destination
    } else {
        return Err(GripError::CorruptState(
            "recovery target is not bound to the managed identity".into(),
        ));
    };
    publish_complete_metadata(
        receipt,
        action_index,
        identity,
        prior_state,
        Some(payload_ref.into()),
        endpoint_role,
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
    let endpoint_role = if target == identity.source_path() {
        crate::state::EndpointRoleV1::Source
    } else if target == identity.destination_path() {
        crate::state::EndpointRoleV1::Destination
    } else {
        return Err(GripError::CorruptState(
            "recovery target is not bound to the managed identity".into(),
        ));
    };
    let portable_identity = receipt.portable_identity(identity)?;
    let recovery_record = MutationRecoveryV2::new(MutationRecoveryPayloadV2 {
        operation_id: receipt.operation_id().into(),
        action_index,
        identity: portable_identity.clone(),
        endpoint_role,
        private_ref: if payload_present {
            "payload"
        } else {
            "metadata"
        }
        .into(),
    })?;
    let recovery_bytes = serde_json::to_vec(&recovery_record).map_err(|error| {
        GripError::Internal(format!("could not encode Mutation Recovery V2: {error}"))
    })?;
    crate::operation::publication::publish_new_component(
        &directory,
        "recovery-v2.json",
        &recovery_bytes,
    )
    .map_err(|error| recovery_failure_at("publish Mutation Recovery V2", error))?;
    let manifest = crate::recovery::model::RecoveryManifestV2::new(
        crate::recovery::model::RecoveryManifestPayloadV2 {
            reference: crate::recovery::model::RecoveryRef::Payload {
                operation_id: receipt.operation_id().into(),
                action_index,
            },
            kind: crate::recovery::model::RecoveryKind::Payload,
            identity: Some(portable_identity),
            endpoint_role: Some(endpoint_role),
            private_ref: if payload_present {
                "payload"
            } else {
                "metadata"
            }
            .into(),
            diagnostic_target: Some(crate::discovery::model::SafePath::from_path(target).into()),
            created_at: unix_timestamp(),
            origin_operation: Some(receipt.operation_id().into()),
            origin_transition: if expected_post.is_some() {
                "replacement"
            } else {
                "deletion"
            }
            .into(),
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
        },
    )?;
    crate::operation::publication::publish_new_component(
        &directory,
        "manifest.json",
        &serde_json::to_vec(&manifest).map_err(|error| {
            GripError::Internal(format!("could not encode Recovery Manifest V2: {error}"))
        })?,
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
