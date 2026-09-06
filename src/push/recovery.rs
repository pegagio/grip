//! Verified private preservation of replacement destinations.

use crate::error::GripError;
use crate::observation::model::{EntryIdentity, MappingSnapshot, SupportedState};
use crate::operation::publication::OperationReceipt;
use serde::{Deserialize, Serialize};
use std::path::Path;

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

/// Preserve and verify the current destination before replacement.
pub fn preserve(
    receipt: &OperationReceipt,
    action_index: usize,
    identity: &EntryIdentity,
    destination: &Path,
    expected: &SupportedState,
) -> Result<RecoveryEntry, GripError> {
    let directory = receipt
        .directory()
        .join("recovery")
        .join(format!("{action_index:08}"));
    crate::push::filesystem::create_recovery_directory(&directory).map_err(recovery_failure)?;
    let payload = directory.join("payload");
    let mut staged = crate::push::filesystem::stage_private_file(destination, &payload, expected)
        .map_err(recovery_failure)?;
    crate::push::filesystem::publish_addition(&mut staged, &payload).map_err(recovery_failure)?;
    let mut private_expected = expected.clone();
    private_expected.permission_mode = Some("0600".into());
    crate::push::filesystem::verify_destination(&payload, &private_expected)
        .map_err(recovery_failure)?;
    let relative_ref = format!("recovery/{action_index:08}/payload");
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
        payload_present: true,
        verified: true,
    };
    let bytes = serde_json::to_vec(&metadata).map_err(|error| {
        GripError::Internal(format!("could not encode recovery metadata: {error}"))
    })?;
    crate::operation::publication::publish_new_component(&directory, "metadata.json", &bytes)
        .map_err(recovery_failure)?;
    let reread = crate::push::filesystem::read_private_file(&directory.join("metadata.json"))
        .map_err(recovery_failure)?;
    let decoded: RecoveryMetadataV1 = serde_json::from_slice(&reread)
        .map_err(|error| GripError::CorruptState(format!("invalid recovery metadata: {error}")))?;
    if decoded != metadata {
        return Err(GripError::CorruptState(
            "recovery metadata verification failed".into(),
        ));
    }
    Ok(RecoveryEntry { relative_ref })
}

fn recovery_failure(error: GripError) -> GripError {
    if matches!(error, GripError::CorruptState(_)) {
        error
    } else {
        GripError::push_side_effect(
            "recovery_failure",
            false,
            "not_attempted",
            false,
            error.to_string(),
        )
    }
}
