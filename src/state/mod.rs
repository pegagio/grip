pub mod lock;
pub mod publication;

use crate::error::GripError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatePayloadV1 {
    pub generation: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntegrityV1 {
    pub algorithm: String,
    pub digest: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateEnvelopeV1 {
    pub schema_version: u64,
    pub payload: StatePayloadV1,
    pub integrity: IntegrityV1,
}

impl StateEnvelopeV1 {
    pub fn new(generation: u64) -> Self {
        let payload = StatePayloadV1 { generation };
        Self {
            schema_version: 1,
            integrity: IntegrityV1 {
                algorithm: "sha256".into(),
                digest: canonical_digest(&payload),
            },
            payload,
        }
    }
    pub fn validate(&self) -> Result<(), GripError> {
        if self.schema_version != 1 {
            return Err(GripError::UnsupportedSchema(format!(
                "unsupported state schema version {}",
                self.schema_version
            )));
        }
        if self.integrity.algorithm != "sha256"
            || self.integrity.digest.len() != 64
            || !self
                .integrity
                .digest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || self.integrity.digest != canonical_digest(&self.payload)
        {
            return Err(GripError::CorruptState(
                "state integrity verification failed".into(),
            ));
        }
        Ok(())
    }
}

fn canonical_digest(payload: &StatePayloadV1) -> String {
    let bytes = format!(
        "{{\"schema_version\":1,\"payload\":{{\"generation\":{}}}}}",
        payload.generation
    );
    format!("{:x}", Sha256::digest(bytes.as_bytes()))
}

pub fn decode(input: &str) -> Result<StateEnvelopeV1, GripError> {
    let raw: serde_json::Value = serde_json::from_str(input)
        .map_err(|e| GripError::CorruptState(format!("invalid state JSON: {e}")))?;
    let version = raw
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            GripError::CorruptState("state schema_version must be a nonnegative integer".into())
        })?;
    if version != 1 {
        return Err(GripError::UnsupportedSchema(format!(
            "unsupported state schema version {version}"
        )));
    }
    let state: StateEnvelopeV1 = serde_json::from_value(raw)
        .map_err(|e| GripError::CorruptState(format!("invalid state schema: {e}")))?;
    state.validate()?;
    Ok(state)
}

pub fn encode(state: &StateEnvelopeV1) -> Result<Vec<u8>, GripError> {
    serde_json::to_vec(state)
        .map_err(|e| GripError::Internal(format!("could not encode state: {e}")))
}
