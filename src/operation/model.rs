//! Strict schemas for portable Operation Record V2 authority and lifecycle evidence.

use crate::error::GripError;
use crate::state::IntegrityV1;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortableActionV2 {
    pub index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<crate::state::EntryIdentityV4>,
    pub endpoint_role: crate::state::EndpointRoleV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic_path: Option<DiagnosticPathV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticPathV1 {
    pub display: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_hex: Option<String>,
}

impl From<crate::discovery::model::SafePath> for DiagnosticPathV1 {
    fn from(value: crate::discovery::model::SafePath) -> Self {
        Self {
            display: value.display,
            raw_hex: value.raw_hex,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationRecordPayloadV2 {
    pub operation_id: String,
    pub operation: String,
    pub plan_id: String,
    pub actions: Vec<PortableActionV2>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationRecordV2 {
    pub schema_version: u8,
    pub payload: OperationRecordPayloadV2,
    pub integrity: IntegrityV1,
}

impl OperationRecordV2 {
    pub fn new(payload: OperationRecordPayloadV2) -> Result<Self, GripError> {
        validate_operation_v2(&payload)?;
        let digest = digest_v2(&payload)?;
        Ok(Self {
            schema_version: 2,
            payload,
            integrity: IntegrityV1 {
                algorithm: "sha256".into(),
                digest,
            },
        })
    }

    pub fn validate(&self) -> Result<(), GripError> {
        if self.schema_version != 2 {
            return Err(GripError::UnsupportedSchema(format!(
                "unsupported operation record schema version {}",
                self.schema_version
            )));
        }
        if self.integrity.algorithm != "sha256"
            || self.integrity.digest != digest_v2(&self.payload)?
        {
            return Err(corrupt("operation record integrity verification failed"));
        }
        validate_operation_v2(&self.payload)
    }
}

pub fn encode_operation_v2(value: &OperationRecordV2) -> Result<Vec<u8>, GripError> {
    serde_json::to_vec(value).map_err(|error| {
        GripError::Internal(format!("could not encode Operation Record V2: {error}"))
    })
}

pub fn decode_operation_v2(bytes: &[u8]) -> Result<OperationRecordV2, GripError> {
    let raw: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| GripError::CorruptState(format!("invalid operation record: {error}")))?;
    let version = raw
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| corrupt("operation record schema_version is required"))?;
    if version != 2 {
        return Err(GripError::UnsupportedSchema(format!(
            "unsupported operation record schema version {version}"
        )));
    }
    let value: OperationRecordV2 = serde_json::from_value(raw).map_err(|error| {
        GripError::CorruptState(format!("invalid Operation Record V2: {error}"))
    })?;
    value.validate()?;
    Ok(value)
}

fn validate_operation_v2(payload: &OperationRecordPayloadV2) -> Result<(), GripError> {
    validate_common(
        &payload.operation_id,
        Some(&payload.operation),
        &payload.plan_id,
    )?;
    for (index, action) in payload.actions.iter().enumerate() {
        if action.index != index {
            return Err(corrupt(
                "Operation Record V2 actions must be dense and ordered",
            ));
        }
        if let Some(identity) = &action.identity {
            crate::registry::ProjectDescriptorV2::new(vec![identity.mapping.clone()])
                .map_err(|_| corrupt("operation action portable identity is invalid"))?;
            crate::state::decode_v4_identity_path(&identity.relative_path_hex)?;
        }
    }
    Ok(())
}

fn digest_v2(payload: &OperationRecordPayloadV2) -> Result<String, GripError> {
    #[derive(Serialize)]
    struct Input<'a> {
        schema_version: u8,
        payload: &'a OperationRecordPayloadV2,
    }
    let bytes = serde_json::to_vec(&Input {
        schema_version: 2,
        payload,
    })
    .map_err(|error| {
        GripError::Internal(format!("could not encode operation integrity: {error}"))
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

/// Bounded operation-level state changed at terminal milestones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationSummaryPayloadV2 {
    pub operation_id: String,
    pub operation: String,
    pub state: String,
    pub plan_id: String,
    pub plan_ref: String,
    pub baseline: serde_json::Value,
    pub result_delivery: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
}

/// Latest bounded evidence for one started action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionCheckpointPayloadV2 {
    pub operation_id: String,
    pub plan_id: String,
    pub action_index: usize,
    pub status: String,
    pub milestones: ActionCheckpointEvidenceV2,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
}

/// Strict last-known evidence for one started action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionCheckpointEvidenceV2 {
    pub revalidation: String,
    pub staging: String,
    pub publication: String,
    pub verification: String,
    pub durability_confirmed: bool,
}

/// Integrity-protected bounded summary envelope.
pub type OperationSummaryEnvelopeV2 = LifecycleEnvelopeV2<OperationSummaryPayloadV2>;
/// Integrity-protected bounded action checkpoint envelope.
pub type ActionCheckpointEnvelopeV2 = LifecycleEnvelopeV2<ActionCheckpointPayloadV2>;

/// Common strict envelope used by each partitioned component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleEnvelopeV2<T> {
    pub schema_version: u8,
    pub payload: T,
    pub integrity: IntegrityV1,
}

#[derive(Serialize)]
struct IntegrityInput<'a, T> {
    schema_version: u8,
    payload: &'a T,
}

impl<T> LifecycleEnvelopeV2<T>
where
    T: Clone + Serialize + DeserializeOwned + ValidatePayload,
{
    /// Construct a V2 envelope with canonical SHA-256 integrity.
    pub fn new(payload: T) -> Result<Self, GripError> {
        payload.validate_payload()?;
        let digest = digest(&payload)?;
        Ok(Self {
            schema_version: 2,
            payload,
            integrity: IntegrityV1 {
                algorithm: "sha256".into(),
                digest,
            },
        })
    }

    /// Validate schema and canonical integrity.
    pub fn validate(&self) -> Result<(), GripError> {
        if self.schema_version != 2 {
            return Err(GripError::UnsupportedSchema(format!(
                "unsupported operation record schema version {}",
                self.schema_version
            )));
        }
        if self.integrity.algorithm != "sha256" || self.integrity.digest != digest(&self.payload)? {
            return Err(GripError::CorruptState(
                "operation record integrity verification failed".into(),
            ));
        }
        self.payload.validate_payload()?;
        Ok(())
    }
}

/// Encode one partitioned record component deterministically.
pub fn encode<T: Serialize>(value: &LifecycleEnvelopeV2<T>) -> Result<Vec<u8>, GripError> {
    serde_json::to_vec(value)
        .map_err(|error| GripError::Internal(format!("could not encode operation record: {error}")))
}

/// Strictly decode and validate one partitioned record component.
pub fn decode<T>(bytes: &[u8]) -> Result<LifecycleEnvelopeV2<T>, GripError>
where
    T: Clone + Serialize + DeserializeOwned + ValidatePayload,
{
    let value: LifecycleEnvelopeV2<T> = serde_json::from_slice(bytes)
        .map_err(|error| GripError::CorruptState(format!("invalid operation record: {error}")))?;
    value.validate()?;
    Ok(value)
}

/// Component-specific semantic validation for strict record payloads.
pub trait ValidatePayload {
    fn validate_payload(&self) -> Result<(), GripError>;
}

impl ValidatePayload for OperationSummaryPayloadV2 {
    fn validate_payload(&self) -> Result<(), GripError> {
        validate_common(&self.operation_id, Some(&self.operation), &self.plan_id)?;
        if self.plan_ref != "record.json"
            || !matches!(self.state.as_str(), "executing" | "completed" | "failed")
            || !matches!(
                self.result_delivery.as_str(),
                "not_attempted" | "prepared" | "delivered" | "failed"
            )
            || (self.state == "executing" && self.failure.is_some())
            || (self.state == "completed" && self.failure.is_some())
            || (self.state == "failed" && self.failure.is_none())
        {
            return Err(corrupt("operation summary has an invalid state"));
        }
        Ok(())
    }
}

impl ValidatePayload for ActionCheckpointPayloadV2 {
    fn validate_payload(&self) -> Result<(), GripError> {
        validate_common(&self.operation_id, None, &self.plan_id)?;
        if !matches!(self.status.as_str(), "in_progress" | "completed" | "failed")
            || (self.status == "failed" && self.failure.is_none())
            || (self.status != "failed" && self.failure.is_some())
        {
            return Err(corrupt("action checkpoint has an invalid state"));
        }
        self.milestones.validate()?;
        Ok(())
    }
}

impl ActionCheckpointEvidenceV2 {
    fn validate(&self) -> Result<(), GripError> {
        if !matches!(
            self.revalidation.as_str(),
            "not_attempted" | "passed" | "failed"
        ) || !matches!(
            self.staging.as_str(),
            "not_attempted" | "verified" | "failed"
        ) || !matches!(
            self.publication.as_str(),
            "not_attempted" | "not_visible" | "visible"
        ) || !matches!(
            self.verification.as_str(),
            "not_attempted" | "verified" | "failed"
        ) {
            return Err(corrupt("action checkpoint milestone is invalid"));
        }
        Ok(())
    }
}

/// Validate a monotonic operation-summary transition.
pub fn validate_summary_transition(
    previous: &OperationSummaryPayloadV2,
    next: &OperationSummaryPayloadV2,
) -> Result<(), GripError> {
    if previous.operation_id != next.operation_id || previous.plan_id != next.plan_id {
        return Err(corrupt(
            "operation identity changed during summary transition",
        ));
    }
    let state_ok = previous.state == "executing"
        && matches!(next.state.as_str(), "executing" | "completed" | "failed")
        || previous.state == next.state;
    if !state_ok {
        return Err(corrupt("operation summary transition is not monotonic"));
    }
    Ok(())
}

/// Validate a monotonic checkpoint transition for one action.
pub fn validate_action_transition(
    previous: &ActionCheckpointPayloadV2,
    next: &ActionCheckpointPayloadV2,
) -> Result<(), GripError> {
    if previous.operation_id != next.operation_id
        || previous.plan_id != next.plan_id
        || previous.action_index != next.action_index
        || previous.status != "in_progress"
        || !matches!(next.status.as_str(), "in_progress" | "completed" | "failed")
    {
        return Err(corrupt("action checkpoint transition is not monotonic"));
    }
    Ok(())
}

fn validate_common(
    operation_id: &str,
    operation: Option<&str>,
    plan_id: &str,
) -> Result<(), GripError> {
    if operation_id.is_empty()
        || !operation_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        || operation
            .is_some_and(|value| !matches!(value, "push" | "pull" | "sync" | "resolve" | "delete"))
        || !is_digest(plan_id)
    {
        return Err(corrupt("operation record identity is invalid"));
    }
    Ok(())
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn corrupt(message: &str) -> GripError {
    GripError::CorruptState(message.into())
}

fn digest<T: Serialize>(payload: &T) -> Result<String, GripError> {
    let bytes = serde_json::to_vec(&IntegrityInput {
        schema_version: 2,
        payload,
    })
    .map_err(|error| GripError::Internal(format!("could not encode integrity input: {error}")))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

/// Validate one action checkpoint against immutable operation identity and bounds.
pub fn validate_checkpoint_binding(
    checkpoint: &ActionCheckpointPayloadV2,
    operation_id: &str,
    plan_id: &str,
    action_count: usize,
) -> Result<(), GripError> {
    if checkpoint.operation_id != operation_id
        || checkpoint.plan_id != plan_id
        || checkpoint.action_index >= action_count
    {
        return Err(GripError::CorruptState(
            "action checkpoint does not match its immutable operation plan".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary() -> OperationSummaryPayloadV2 {
        OperationSummaryPayloadV2 {
            operation_id: "operation-1".into(),
            operation: "push".into(),
            state: "executing".into(),
            plan_id: "a".repeat(64),
            plan_ref: "record.json".into(),
            baseline: serde_json::json!({"outcome":"not_attempted"}),
            result_delivery: "not_attempted".into(),
            failure: None,
        }
    }

    fn evidence() -> ActionCheckpointEvidenceV2 {
        ActionCheckpointEvidenceV2 {
            revalidation: "passed".into(),
            staging: "verified".into(),
            publication: "visible".into(),
            verification: "verified".into(),
            durability_confirmed: true,
        }
    }

    #[test]
    fn encode_expected_round_trip_when_summary_is_valid() {
        let envelope = OperationSummaryEnvelopeV2::new(summary()).unwrap();
        let bytes = encode(&envelope).unwrap();
        assert_eq!(
            decode::<OperationSummaryPayloadV2>(&bytes).unwrap(),
            envelope
        );
    }

    #[test]
    fn decode_expected_failure_when_unknown_field_is_present() {
        let envelope = OperationSummaryEnvelopeV2::new(summary()).unwrap();
        let mut value = serde_json::to_value(envelope).unwrap();
        value["payload"]["unknown"] = true.into();
        assert!(decode::<OperationSummaryPayloadV2>(&serde_json::to_vec(&value).unwrap()).is_err());
    }

    #[test]
    fn validate_expected_failure_when_payload_changes_after_digest() {
        let mut envelope = OperationSummaryEnvelopeV2::new(summary()).unwrap();
        envelope.payload.state = "completed".into();
        assert!(envelope.validate().is_err());
    }

    #[test]
    fn validate_checkpoint_binding_expected_failure_when_index_is_outside_plan() {
        let checkpoint = ActionCheckpointPayloadV2 {
            operation_id: "operation-1".into(),
            plan_id: "a".repeat(64),
            action_index: 2,
            status: "in_progress".into(),
            milestones: evidence(),
            failure: None,
        };
        assert!(
            validate_checkpoint_binding(&checkpoint, "operation-1", &"a".repeat(64), 2).is_err()
        );
    }

    #[test]
    fn validate_summary_transition_rejects_terminal_to_executing() {
        let mut terminal = summary();
        terminal.state = "completed".into();
        let next = summary();
        assert!(validate_summary_transition(&terminal, &next).is_err());
    }

    #[test]
    fn decode_rejects_duplicate_fields() {
        let bytes = br#"{"schema_version":1,"schema_version":1,"payload":{},"integrity":{"algorithm":"sha256","digest":"x"}}"#;
        assert!(decode::<OperationSummaryPayloadV2>(bytes).is_err());
    }

    #[test]
    fn encode_round_trips_action_envelope() {
        let action = ActionCheckpointEnvelopeV2::new(ActionCheckpointPayloadV2 {
            operation_id: "operation-1".into(),
            plan_id: "a".repeat(64),
            action_index: 0,
            status: "completed".into(),
            milestones: evidence(),
            failure: None,
        })
        .unwrap();
        assert_eq!(
            decode::<ActionCheckpointPayloadV2>(&encode(&action).unwrap()).unwrap(),
            action
        );
    }
}
