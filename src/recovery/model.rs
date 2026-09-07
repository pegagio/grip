//! Typed public recovery references and lifecycle evidence.

use crate::error::GripError;
use crate::state::IntegrityV1;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::fmt;
use std::str::FromStr;

/// Stable public identity for retained recovery evidence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RecoveryRef {
    Payload {
        operation_id: String,
        action_index: usize,
    },
    Registry {
        digest: String,
    },
    AcceptedState {
        generation: u64,
        digest: String,
    },
    Operation {
        operation_id: String,
    },
}

impl RecoveryRef {
    /// Whether this reference identifies bytes eligible for restore or cleanup.
    pub const fn has_recoverable_bytes(&self) -> bool {
        !matches!(self, Self::Operation { .. })
    }
}

impl fmt::Display for RecoveryRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Payload {
                operation_id,
                action_index,
            } => write!(formatter, "payload:{operation_id}:{action_index}"),
            Self::Registry { digest } => write!(formatter, "registry:sha256:{digest}"),
            Self::AcceptedState { generation, digest } => {
                write!(formatter, "state:generation:{generation}:sha256:{digest}")
            }
            Self::Operation { operation_id } => write!(formatter, "operation:{operation_id}"),
        }
    }
}

impl FromStr for RecoveryRef {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parts = value.split(':').collect::<Vec<_>>();
        match parts.as_slice() {
            ["payload", operation_id, action_index] if valid_component(operation_id) => {
                Ok(Self::Payload {
                    operation_id: (*operation_id).into(),
                    action_index: action_index
                        .parse()
                        .map_err(|_| "payload action index must be an unsigned integer")?,
                })
            }
            ["registry", "sha256", digest] if valid_digest(digest) => Ok(Self::Registry {
                digest: (*digest).into(),
            }),
            ["state", "generation", generation, "sha256", digest] if valid_digest(digest) => {
                Ok(Self::AcceptedState {
                    generation: generation
                        .parse()
                        .map_err(|_| "state generation must be an unsigned integer")?,
                    digest: (*digest).into(),
                })
            }
            ["operation", operation_id] if valid_component(operation_id) => Ok(Self::Operation {
                operation_id: (*operation_id).into(),
            }),
            _ => Err("invalid recovery reference".into()),
        }
    }
}

/// Kind of recovery evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryKind {
    Payload,
    Registry,
    AcceptedState,
    Operation,
}

/// Immutable provenance for newly retained recovery bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryManifestPayloadV1 {
    pub reference: RecoveryRef,
    pub kind: RecoveryKind,
    pub created_at: String,
    pub origin_operation: Option<String>,
    pub origin_transition: String,
    pub managed_identity: Option<String>,
    pub bound_side: Option<String>,
    pub bound_target: Option<String>,
    pub prior_evidence: serde_json::Value,
    pub expected_post_evidence: serde_json::Value,
    pub byte_count: u64,
    pub payload_ref: String,
}

/// Immutable proof that recoverable bytes were explicitly removed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CleanupTombstonePayloadV1 {
    pub reference: RecoveryRef,
    pub cleaned_at: String,
    pub removed_byte_count: u64,
    pub cleanup_operation_id: String,
}

/// Strict integrity-protected recovery envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryEnvelopeV1<T> {
    pub schema_version: u8,
    pub payload: T,
    pub integrity: IntegrityV1,
}

#[derive(Serialize)]
struct IntegrityInput<'a, T> {
    schema_version: u8,
    payload: &'a T,
}

/// Semantic validation implemented by recovery envelope payloads.
pub trait ValidateRecoveryPayload {
    fn validate_payload(&self) -> Result<(), GripError>;
}

impl<T> RecoveryEnvelopeV1<T>
where
    T: Clone + Serialize + DeserializeOwned + ValidateRecoveryPayload,
{
    /// Construct a strict V1 envelope.
    pub fn new(payload: T) -> Result<Self, GripError> {
        payload.validate_payload()?;
        let digest = integrity_digest(&payload)?;
        Ok(Self {
            schema_version: 1,
            payload,
            integrity: IntegrityV1 {
                algorithm: "sha256".into(),
                digest,
            },
        })
    }

    /// Validate schema, semantic binding, and canonical integrity.
    pub fn validate(&self) -> Result<(), GripError> {
        if self.schema_version != 1 {
            return Err(GripError::UnsupportedSchema(format!(
                "unsupported recovery evidence schema version {}",
                self.schema_version
            )));
        }
        self.payload.validate_payload()?;
        if self.integrity.algorithm != "sha256"
            || self.integrity.digest != integrity_digest(&self.payload)?
        {
            return Err(corrupt("recovery evidence integrity verification failed"));
        }
        Ok(())
    }
}

impl ValidateRecoveryPayload for RecoveryManifestPayloadV1 {
    fn validate_payload(&self) -> Result<(), GripError> {
        let expected_kind = match self.reference {
            RecoveryRef::Payload { .. } => RecoveryKind::Payload,
            RecoveryRef::Registry { .. } => RecoveryKind::Registry,
            RecoveryRef::AcceptedState { .. } => RecoveryKind::AcceptedState,
            RecoveryRef::Operation { .. } => RecoveryKind::Operation,
        };
        if self.kind != expected_kind || self.kind == RecoveryKind::Operation {
            return Err(corrupt(
                "recovery manifest kind does not match its reference",
            ));
        }
        if self.created_at.is_empty()
            || self.origin_transition.is_empty()
            || !valid_private_component(&self.payload_ref)
        {
            return Err(corrupt("recovery manifest contains invalid provenance"));
        }
        match self.kind {
            RecoveryKind::Payload => {
                if self
                    .origin_operation
                    .as_deref()
                    .is_none_or(|id| !valid_component(id))
                    || self.managed_identity.as_deref().is_none_or(str::is_empty)
                    || !matches!(self.bound_side.as_deref(), Some("source" | "destination"))
                    || self.bound_target.as_deref().is_none_or(str::is_empty)
                {
                    return Err(corrupt("payload recovery binding is incomplete"));
                }
            }
            RecoveryKind::Registry | RecoveryKind::AcceptedState => {
                if self.managed_identity.is_some()
                    || self.bound_side.is_some()
                    || self.bound_target.is_some()
                {
                    return Err(corrupt("authority recovery contains payload-only binding"));
                }
            }
            RecoveryKind::Operation => unreachable!(),
        }
        Ok(())
    }
}

impl ValidateRecoveryPayload for CleanupTombstonePayloadV1 {
    fn validate_payload(&self) -> Result<(), GripError> {
        if !self.reference.has_recoverable_bytes()
            || self.cleaned_at.is_empty()
            || !valid_component(&self.cleanup_operation_id)
        {
            return Err(corrupt("cleanup tombstone contains invalid evidence"));
        }
        Ok(())
    }
}

/// Current byte lifecycle reported by recovery inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryAvailability {
    Available,
    Cleaned,
    CleanupIncomplete,
    Missing,
}

/// Integrity result reported by recovery inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryIntegrity {
    Verified,
    Failed,
    Unavailable,
}

/// Public metadata-only projection of one recovery entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecoveryEntry {
    pub reference: RecoveryRef,
    pub kind: RecoveryKind,
    pub origin: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bound_side: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bound_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    pub integrity: RecoveryIntegrity,
    pub availability: RecoveryAvailability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_count: Option<u64>,
    pub provenance: String,
    pub restore_eligibility: String,
}

/// One exact cleanup action derived entirely from a public recovery reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CleanupAction {
    pub index: usize,
    pub reference: RecoveryRef,
    pub byte_count: u64,
    pub status: String,
}

/// Deterministic, immutable recovery cleanup plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CleanupPlan {
    pub operation: &'static str,
    pub plan_id: String,
    pub actions: Vec<CleanupAction>,
    pub blockers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_record: Option<String>,
}

/// Deterministic plan for restoring one exact recovery identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RestorePlan {
    pub operation: &'static str,
    pub plan_id: String,
    pub reference: RecoveryRef,
    pub target: Option<String>,
    pub kind: RecoveryKind,
    pub expected_current: serde_json::Value,
    pub displacement_recovery: String,
    pub authority_evidence: serde_json::Value,
    pub actions: Vec<RestoreAction>,
    pub blockers: Vec<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_record: Option<String>,
}

/// The single action in an exact-reference restore plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RestoreAction {
    pub index: usize,
    pub reference: RecoveryRef,
    pub target: Option<String>,
    pub status: String,
    pub milestones: crate::operation::model::ActionCheckpointEvidenceV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
}

/// Encode an integrity-protected recovery envelope deterministically.
pub fn encode<T: Serialize>(value: &RecoveryEnvelopeV1<T>) -> Result<Vec<u8>, GripError> {
    serde_json::to_vec(value).map_err(|error| {
        GripError::Internal(format!("could not encode recovery evidence: {error}"))
    })
}

/// Strictly decode and validate recovery evidence.
pub fn decode<T>(bytes: &[u8]) -> Result<RecoveryEnvelopeV1<T>, GripError>
where
    T: Clone + Serialize + DeserializeOwned + ValidateRecoveryPayload,
{
    let value: RecoveryEnvelopeV1<T> = serde_json::from_slice(bytes)
        .map_err(|error| GripError::CorruptState(format!("invalid recovery evidence: {error}")))?;
    value.validate()?;
    Ok(value)
}

fn integrity_digest<T: Serialize>(payload: &T) -> Result<String, GripError> {
    let bytes = serde_json::to_vec(&IntegrityInput {
        schema_version: 1,
        payload,
    })
    .map_err(|error| GripError::Internal(format!("could not hash recovery evidence: {error}")))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn valid_component(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_private_component(value: &str) -> bool {
    valid_component(value) && value != "." && value != ".."
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn corrupt(message: &str) -> GripError {
    GripError::CorruptState(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest() -> String {
        "a".repeat(64)
    }

    #[test]
    fn recovery_ref_expected_round_trip_for_every_kind() {
        for reference in [
            "payload:delete-1:0".to_owned(),
            format!("registry:sha256:{}", digest()),
            format!("state:generation:7:sha256:{}", digest()),
            "operation:delete-1".to_owned(),
        ] {
            assert_eq!(
                reference.parse::<RecoveryRef>().unwrap().to_string(),
                reference
            );
        }
    }

    #[test]
    fn recovery_ref_expected_failure_for_unsafe_or_noncanonical_input() {
        for reference in [
            "payload:../escape:0",
            "payload:op:-1",
            "registry:sha256:ABC",
            "state:generation:x:sha256:abc",
            "operation:",
            "unknown:value",
        ] {
            assert!(reference.parse::<RecoveryRef>().is_err(), "{reference}");
        }
    }

    #[test]
    fn manifest_expected_round_trip_when_binding_is_complete() {
        let payload = RecoveryManifestPayloadV1 {
            reference: RecoveryRef::Payload {
                operation_id: "delete-1".into(),
                action_index: 0,
            },
            kind: RecoveryKind::Payload,
            created_at: "1".into(),
            origin_operation: Some("delete-1".into()),
            origin_transition: "deletion".into(),
            managed_identity: Some("mapping:entry".into()),
            bound_side: Some("destination".into()),
            bound_target: Some("/target".into()),
            prior_evidence: serde_json::json!({"state":"prior"}),
            expected_post_evidence: serde_json::json!({"state":"absent"}),
            byte_count: 4,
            payload_ref: "payload".into(),
        };
        let envelope = RecoveryEnvelopeV1::new(payload).unwrap();
        assert_eq!(
            decode::<RecoveryManifestPayloadV1>(&encode(&envelope).unwrap()).unwrap(),
            envelope
        );
    }

    #[test]
    fn tombstone_expected_failure_when_reference_is_operation_only() {
        let payload = CleanupTombstonePayloadV1 {
            reference: RecoveryRef::Operation {
                operation_id: "delete-1".into(),
            },
            cleaned_at: "1".into(),
            removed_byte_count: 0,
            cleanup_operation_id: "cleanup-1".into(),
        };
        assert!(RecoveryEnvelopeV1::new(payload).is_err());
    }

    #[test]
    fn manifest_expected_failure_for_layout_kind_mismatch() {
        let payload = RecoveryManifestPayloadV1 {
            reference: RecoveryRef::Registry { digest: digest() },
            kind: RecoveryKind::Payload,
            created_at: "1".into(),
            origin_operation: None,
            origin_transition: "registry_publication".into(),
            managed_identity: None,
            bound_side: None,
            bound_target: None,
            prior_evidence: serde_json::Value::Null,
            expected_post_evidence: serde_json::Value::Null,
            byte_count: 0,
            payload_ref: "config.toml".into(),
        };
        assert!(RecoveryEnvelopeV1::new(payload).is_err());
    }

    #[test]
    fn manifest_expected_failure_when_integrity_or_shape_changes() {
        let payload = CleanupTombstonePayloadV1 {
            reference: RecoveryRef::Registry { digest: digest() },
            cleaned_at: "1".into(),
            removed_byte_count: 4,
            cleanup_operation_id: "cleanup-1".into(),
        };
        let envelope = RecoveryEnvelopeV1::new(payload).unwrap();
        let mut changed = serde_json::to_value(&envelope).unwrap();
        changed["payload"]["removed_byte_count"] = 5.into();
        assert!(
            decode::<CleanupTombstonePayloadV1>(&serde_json::to_vec(&changed).unwrap()).is_err()
        );
        let mut unknown = serde_json::to_value(envelope).unwrap();
        unknown["payload"]["unknown"] = true.into();
        assert!(
            decode::<CleanupTombstonePayloadV1>(&serde_json::to_vec(&unknown).unwrap()).is_err()
        );
    }

    #[test]
    fn recovery_ref_expected_canonical_kind_and_identity_order() {
        let mut values = [
            RecoveryRef::Operation {
                operation_id: "z".into(),
            },
            RecoveryRef::AcceptedState {
                generation: 2,
                digest: digest(),
            },
            RecoveryRef::Registry { digest: digest() },
            RecoveryRef::Payload {
                operation_id: "a".into(),
                action_index: 1,
            },
            RecoveryRef::Payload {
                operation_id: "a".into(),
                action_index: 0,
            },
        ];
        values.sort();
        assert!(matches!(
            values[0],
            RecoveryRef::Payload {
                action_index: 0,
                ..
            }
        ));
        assert!(matches!(
            values[1],
            RecoveryRef::Payload {
                action_index: 1,
                ..
            }
        ));
        assert!(matches!(values[2], RecoveryRef::Registry { .. }));
        assert!(matches!(values[3], RecoveryRef::AcceptedState { .. }));
        assert!(matches!(values[4], RecoveryRef::Operation { .. }));
    }
}
