//! Strict envelopes for the partitioned Operation Record V1.

use crate::error::GripError;
use crate::state::IntegrityV1;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};

/// Immutable plan payload written once before payload mutation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationPlanPayloadV1 {
    pub operation_id: String,
    pub operation: String,
    pub plan_id: String,
    pub plan: serde_json::Value,
}

/// Bounded operation-level state changed at terminal milestones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationSummaryPayloadV1 {
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
pub struct ActionCheckpointPayloadV1 {
    pub operation_id: String,
    pub plan_id: String,
    pub action_index: usize,
    pub status: String,
    pub milestones: ActionCheckpointEvidenceV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
}

/// Strict last-known evidence for one started action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionCheckpointEvidenceV1 {
    pub revalidation: String,
    pub recovery: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_ref: Option<String>,
    pub staging: String,
    pub publication: String,
    pub verification: String,
    pub durability_confirmed: bool,
}

/// Integrity-protected immutable plan envelope.
pub type OperationPlanEnvelopeV1 = EnvelopeV1<OperationPlanPayloadV1>;
/// Integrity-protected bounded summary envelope.
pub type OperationSummaryEnvelopeV1 = EnvelopeV1<OperationSummaryPayloadV1>;
/// Integrity-protected bounded action checkpoint envelope.
pub type ActionCheckpointEnvelopeV1 = EnvelopeV1<ActionCheckpointPayloadV1>;

/// Common strict envelope used by each partitioned component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvelopeV1<T> {
    pub schema_version: u8,
    pub payload: T,
    pub integrity: IntegrityV1,
}

#[derive(Serialize)]
struct IntegrityInput<'a, T> {
    schema_version: u8,
    payload: &'a T,
}

impl<T> EnvelopeV1<T>
where
    T: Clone + Serialize + DeserializeOwned + ValidatePayload,
{
    /// Construct a V1 envelope with canonical SHA-256 integrity.
    pub fn new(payload: T) -> Result<Self, GripError> {
        payload.validate_payload()?;
        let digest = digest(&payload)?;
        Ok(Self {
            schema_version: 1,
            payload,
            integrity: IntegrityV1 {
                algorithm: "sha256".into(),
                digest,
            },
        })
    }

    /// Validate schema and canonical integrity.
    pub fn validate(&self) -> Result<(), GripError> {
        if self.schema_version != 1 {
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
pub fn encode<T: Serialize>(value: &EnvelopeV1<T>) -> Result<Vec<u8>, GripError> {
    serde_json::to_vec(value)
        .map_err(|error| GripError::Internal(format!("could not encode operation record: {error}")))
}

/// Strictly decode and validate one partitioned record component.
pub fn decode<T>(bytes: &[u8]) -> Result<EnvelopeV1<T>, GripError>
where
    T: Clone + Serialize + DeserializeOwned + ValidatePayload,
{
    let value: EnvelopeV1<T> = serde_json::from_slice(bytes)
        .map_err(|error| GripError::CorruptState(format!("invalid operation record: {error}")))?;
    value.validate()?;
    Ok(value)
}

/// Component-specific semantic validation for strict record payloads.
pub trait ValidatePayload {
    fn validate_payload(&self) -> Result<(), GripError>;
}

impl ValidatePayload for OperationPlanPayloadV1 {
    fn validate_payload(&self) -> Result<(), GripError> {
        validate_common(&self.operation_id, Some(&self.operation), &self.plan_id)?;
        if !self.plan.is_object() {
            return Err(corrupt("operation plan must be an object"));
        }
        if self.plan.get("plan_id").and_then(serde_json::Value::as_str)
            != Some(self.plan_id.as_str())
        {
            return Err(corrupt(
                "operation plan identity does not match its payload",
            ));
        }
        let embedded_operation = self
            .plan
            .get("operation")
            .and_then(serde_json::Value::as_str);
        if embedded_operation.is_some_and(|value| value != self.operation)
            || embedded_operation.is_none() && !matches!(self.operation.as_str(), "push" | "pull")
        {
            return Err(corrupt("operation plan command does not match its payload"));
        }
        let direction = self
            .plan
            .get("direction")
            .and_then(serde_json::Value::as_str);
        if matches!(self.operation.as_str(), "push" | "pull")
            && direction != Some(self.operation.as_str())
            || matches!(self.operation.as_str(), "sync" | "resolve") && direction.is_some()
            || matches!(
                self.operation.as_str(),
                "delete" | "retire" | "recovery_restore" | "recovery_remove"
            ) && direction.is_some()
        {
            return Err(corrupt("operation plan direction is invalid"));
        }
        let winner = self.plan.get("winner").and_then(serde_json::Value::as_str);
        if self.operation == "resolve" && !matches!(winner, Some("source" | "destination"))
            || self.operation != "resolve" && winner.is_some()
        {
            return Err(corrupt("operation plan winner is invalid"));
        }
        let actions = self
            .plan
            .get("actions")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| corrupt("operation plan actions must be an array"))?;
        for (expected, action) in actions.iter().enumerate() {
            if action.get("index").and_then(serde_json::Value::as_u64) != Some(expected as u64) {
                return Err(corrupt(
                    "operation plan actions must be dense and canonically ordered",
                ));
            }
            if embedded_operation.is_some()
                && matches!(
                    self.operation.as_str(),
                    "push" | "pull" | "sync" | "resolve"
                )
            {
                let action_direction = action.get("direction").and_then(serde_json::Value::as_str);
                let direction_valid = match self.operation.as_str() {
                    "push" => action_direction == Some("push"),
                    "pull" => action_direction == Some("pull"),
                    "sync" => matches!(action_direction, Some("push" | "pull")),
                    "resolve" if winner == Some("source") => action_direction == Some("push"),
                    "resolve" if winner == Some("destination") => action_direction == Some("pull"),
                    _ => false,
                };
                if !direction_valid {
                    return Err(corrupt("operation action direction is invalid"));
                }
            }
        }
        Ok(())
    }
}

impl ValidatePayload for OperationSummaryPayloadV1 {
    fn validate_payload(&self) -> Result<(), GripError> {
        validate_common(&self.operation_id, Some(&self.operation), &self.plan_id)?;
        if self.plan_ref != "plan.json"
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

impl ValidatePayload for ActionCheckpointPayloadV1 {
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

impl ActionCheckpointEvidenceV1 {
    fn validate(&self) -> Result<(), GripError> {
        if !matches!(
            self.revalidation.as_str(),
            "not_attempted" | "passed" | "failed"
        ) || !matches!(
            self.recovery.as_str(),
            "not_required" | "planned" | "preserved" | "failed"
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
        if let Some(reference) = &self.recovery_ref {
            let path = std::path::Path::new(reference);
            if path.is_absolute()
                || reference.is_empty()
                || path
                    .components()
                    .any(|component| !matches!(component, std::path::Component::Normal(_)))
            {
                return Err(corrupt("recovery reference must remain operation-local"));
            }
        }
        Ok(())
    }
}

/// Validate a monotonic operation-summary transition.
pub fn validate_summary_transition(
    previous: &OperationSummaryPayloadV1,
    next: &OperationSummaryPayloadV1,
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
    previous: &ActionCheckpointPayloadV1,
    next: &ActionCheckpointPayloadV1,
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
        || operation.is_some_and(|value| {
            !matches!(
                value,
                "push"
                    | "pull"
                    | "sync"
                    | "resolve"
                    | "delete"
                    | "retire"
                    | "recovery_restore"
                    | "recovery_remove"
            )
        })
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
        schema_version: 1,
        payload,
    })
    .map_err(|error| GripError::Internal(format!("could not encode integrity input: {error}")))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

/// Validate one action checkpoint against immutable operation identity and bounds.
pub fn validate_checkpoint_binding(
    checkpoint: &ActionCheckpointPayloadV1,
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

    fn summary() -> OperationSummaryPayloadV1 {
        OperationSummaryPayloadV1 {
            operation_id: "operation-1".into(),
            operation: "push".into(),
            state: "executing".into(),
            plan_id: "a".repeat(64),
            plan_ref: "plan.json".into(),
            baseline: serde_json::json!({"outcome":"not_attempted"}),
            result_delivery: "not_attempted".into(),
            failure: None,
        }
    }

    fn evidence() -> ActionCheckpointEvidenceV1 {
        ActionCheckpointEvidenceV1 {
            revalidation: "passed".into(),
            recovery: "not_required".into(),
            recovery_ref: None,
            staging: "verified".into(),
            publication: "visible".into(),
            verification: "verified".into(),
            durability_confirmed: true,
        }
    }

    #[test]
    fn encode_expected_round_trip_when_summary_is_valid() {
        let envelope = OperationSummaryEnvelopeV1::new(summary()).unwrap();
        let bytes = encode(&envelope).unwrap();
        assert_eq!(
            decode::<OperationSummaryPayloadV1>(&bytes).unwrap(),
            envelope
        );
    }

    #[test]
    fn decode_expected_failure_when_unknown_field_is_present() {
        let envelope = OperationSummaryEnvelopeV1::new(summary()).unwrap();
        let mut value = serde_json::to_value(envelope).unwrap();
        value["payload"]["unknown"] = true.into();
        assert!(decode::<OperationSummaryPayloadV1>(&serde_json::to_vec(&value).unwrap()).is_err());
    }

    #[test]
    fn validate_expected_failure_when_payload_changes_after_digest() {
        let mut envelope = OperationSummaryEnvelopeV1::new(summary()).unwrap();
        envelope.payload.state = "completed".into();
        assert!(envelope.validate().is_err());
    }

    #[test]
    fn validate_checkpoint_binding_expected_failure_when_index_is_outside_plan() {
        let checkpoint = ActionCheckpointPayloadV1 {
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
        assert!(decode::<OperationSummaryPayloadV1>(bytes).is_err());
    }

    #[test]
    fn validate_plan_rejects_non_dense_action_order() {
        let plan_id = "a".repeat(64);
        let payload = OperationPlanPayloadV1 {
            operation_id: "operation-1".into(),
            operation: "push".into(),
            plan_id: plan_id.clone(),
            plan: serde_json::json!({"direction":"push","plan_id":plan_id,"actions":[{"index":1}]}),
        };
        assert!(OperationPlanEnvelopeV1::new(payload).is_err());
    }

    #[test]
    fn validate_plan_rejects_winner_direction_mismatch() {
        let plan_id = "a".repeat(64);
        let payload = OperationPlanPayloadV1 {
            operation_id: "resolve-1".into(),
            operation: "resolve".into(),
            plan_id: plan_id.clone(),
            plan: serde_json::json!({
                "operation":"resolve",
                "winner":"source",
                "plan_id":plan_id,
                "actions":[{"index":0,"direction":"pull"}]
            }),
        };
        assert!(OperationPlanEnvelopeV1::new(payload).is_err());
    }

    #[test]
    fn validate_plan_accepts_each_feature_eight_operation() {
        for operation in ["delete", "retire", "recovery_restore", "recovery_remove"] {
            let plan_id = "a".repeat(64);
            let payload = OperationPlanPayloadV1 {
                operation_id: format!("{operation}-1"),
                operation: operation.into(),
                plan_id: plan_id.clone(),
                plan: serde_json::json!({
                    "operation": operation,
                    "plan_id": plan_id,
                    "actions": [{"index": 0}]
                }),
            };
            assert!(OperationPlanEnvelopeV1::new(payload).is_ok(), "{operation}");
        }
    }

    #[test]
    fn validate_checkpoint_rejects_escaping_recovery_reference() {
        let payload = ActionCheckpointPayloadV1 {
            operation_id: "operation-1".into(),
            plan_id: "a".repeat(64),
            action_index: 0,
            status: "in_progress".into(),
            milestones: ActionCheckpointEvidenceV1 {
                recovery_ref: Some("../outside".into()),
                ..evidence()
            },
            failure: None,
        };
        assert!(ActionCheckpointEnvelopeV1::new(payload).is_err());
    }

    #[test]
    fn encode_round_trips_plan_and_action_envelopes() {
        let plan_id = "a".repeat(64);
        let plan = OperationPlanEnvelopeV1::new(OperationPlanPayloadV1 {
            operation_id: "operation-1".into(),
            operation: "push".into(),
            plan_id: plan_id.clone(),
            plan: serde_json::json!({"direction":"push","plan_id":plan_id,"actions":[{"index":0}]}),
        })
        .unwrap();
        let action = ActionCheckpointEnvelopeV1::new(ActionCheckpointPayloadV1 {
            operation_id: "operation-1".into(),
            plan_id: "a".repeat(64),
            action_index: 0,
            status: "completed".into(),
            milestones: evidence(),
            failure: None,
        })
        .unwrap();
        assert_eq!(
            decode::<OperationPlanPayloadV1>(&encode(&plan).unwrap()).unwrap(),
            plan
        );
        assert_eq!(
            decode::<ActionCheckpointPayloadV1>(&encode(&action).unwrap()).unwrap(),
            action
        );
    }
}
