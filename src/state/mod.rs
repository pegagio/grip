pub mod lock;
pub mod mutation_lock;
pub mod publication;

use crate::error::GripError;
use crate::observation::model::{EntryIdentity, MappingSnapshot, SupportedState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

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

/// Version-neutral accepted state used by classification and publication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedState {
    pub generation: Option<u64>,
    pub baselines: BTreeMap<EntryIdentity, SupportedState>,
    pub accepted_bytes: Option<Vec<u8>>,
}

impl AcceptedState {
    pub fn uninitialized() -> Self {
        Self {
            generation: None,
            baselines: BTreeMap::new(),
            accepted_bytes: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatePayloadV2 {
    pub generation: u64,
    pub baselines: Vec<BaselineRecordV2>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineRecordV2 {
    pub mapping: MappingSnapshot,
    pub relative_path_hex: String,
    pub state: SupportedState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateEnvelopeV2 {
    pub schema_version: u64,
    pub payload: StatePayloadV2,
    pub integrity: IntegrityV1,
}

#[derive(Serialize)]
struct IntegrityInputV2<'a> {
    schema_version: u64,
    payload: &'a StatePayloadV2,
}

impl StateEnvelopeV2 {
    pub fn new(generation: u64, baselines: &BTreeMap<EntryIdentity, SupportedState>) -> Self {
        let payload = StatePayloadV2 {
            generation,
            baselines: baselines
                .iter()
                .map(|(identity, state)| BaselineRecordV2 {
                    mapping: identity.mapping.clone(),
                    relative_path_hex: encode_hex(&identity.relative_path),
                    state: state.clone(),
                })
                .collect(),
        };
        let digest = digest_v2(&payload).expect("typed State V2 serialization cannot fail");
        Self {
            schema_version: 2,
            payload,
            integrity: IntegrityV1 {
                algorithm: "sha256".into(),
                digest,
            },
        }
    }

    pub fn validate(&self) -> Result<BTreeMap<EntryIdentity, SupportedState>, GripError> {
        if self.schema_version != 2 {
            return Err(GripError::UnsupportedSchema(format!(
                "unsupported state schema version {}",
                self.schema_version
            )));
        }
        if self.integrity.algorithm != "sha256"
            || !is_sha256(&self.integrity.digest)
            || self.integrity.digest != digest_v2(&self.payload)?
        {
            return Err(GripError::CorruptState(
                "state integrity verification failed".into(),
            ));
        }
        let mut baselines = BTreeMap::new();
        let mut previous: Option<EntryIdentity> = None;
        for record in &self.payload.baselines {
            if !record.mapping.source.is_absolute()
                || !record.mapping.destination.is_absolute()
                || record.mapping.source.to_str().is_none()
                || record.mapping.destination.to_str().is_none()
            {
                return Err(GripError::CorruptState(
                    "baseline mapping paths must be absolute UTF-8 paths".into(),
                ));
            }
            let relative = decode_hex(&record.relative_path_hex)?;
            let identity = EntryIdentity::new(record.mapping.clone(), relative)
                .map_err(|message| GripError::CorruptState(message.into()))?;
            record
                .state
                .validate()
                .map_err(|message| GripError::CorruptState(message.into()))?;
            if previous.as_ref().is_some_and(|value| value >= &identity) {
                return Err(GripError::CorruptState(
                    "baseline records must be unique and canonically ordered".into(),
                ));
            }
            previous = Some(identity.clone());
            baselines.insert(identity, record.state.clone());
        }
        Ok(baselines)
    }
}

fn digest_v2(payload: &StatePayloadV2) -> Result<String, GripError> {
    let bytes = serde_json::to_vec(&IntegrityInputV2 {
        schema_version: 2,
        payload,
    })
    .map_err(|error| {
        GripError::Internal(format!("could not encode state integrity input: {error}"))
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(result, "{byte:02x}").expect("writing to string cannot fail");
    }
    result
}

fn decode_hex(value: &str) -> Result<Vec<u8>, GripError> {
    if !value.len().is_multiple_of(2)
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(GripError::CorruptState(
            "relative_path_hex must be lowercase even-length hexadecimal".into(),
        ));
    }
    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16)
                .map_err(|_| GripError::CorruptState("invalid relative_path_hex".into()))
        })
        .collect()
}

/// Decode an absent, V1, or V2 state into the version-neutral domain representation.
pub fn decode_accepted(input: Option<&[u8]>) -> Result<AcceptedState, GripError> {
    let Some(bytes) = input else {
        return Ok(AcceptedState::uninitialized());
    };
    let text = std::str::from_utf8(bytes)
        .map_err(|_| GripError::CorruptState("state must be UTF-8 JSON".into()))?;
    let raw: serde_json::Value = serde_json::from_str(text)
        .map_err(|error| GripError::CorruptState(format!("invalid state JSON: {error}")))?;
    let version = raw
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            GripError::CorruptState("state schema_version must be a nonnegative integer".into())
        })?;
    match version {
        1 => {
            let envelope = decode(text)?;
            Ok(AcceptedState {
                generation: Some(envelope.payload.generation),
                baselines: BTreeMap::new(),
                accepted_bytes: Some(bytes.to_vec()),
            })
        }
        2 => {
            let envelope: StateEnvelopeV2 = serde_json::from_value(raw).map_err(|error| {
                GripError::CorruptState(format!("invalid state schema: {error}"))
            })?;
            let baselines = envelope.validate()?;
            Ok(AcceptedState {
                generation: Some(envelope.payload.generation),
                baselines,
                accepted_bytes: Some(bytes.to_vec()),
            })
        }
        _ => Err(GripError::UnsupportedSchema(format!(
            "unsupported state schema version {version}"
        ))),
    }
}

pub fn encode_v2(state: &AcceptedState, generation: u64) -> Result<Vec<u8>, GripError> {
    serde_json::to_vec(&StateEnvelopeV2::new(generation, &state.baselines))
        .map_err(|error| GripError::Internal(format!("could not encode State V2: {error}")))
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

#[cfg(test)]
mod v2_tests {
    use super::*;
    use crate::discovery::model::NodeKind;
    use crate::mapping::MappingKind;
    use crate::observation::model::{ContentFingerprint, MappingSnapshot};
    use std::path::PathBuf;

    fn identity(relative: &[u8]) -> EntryIdentity {
        EntryIdentity::new(
            MappingSnapshot {
                kind: MappingKind::Tree,
                source: PathBuf::from("/source"),
                destination: PathBuf::from("/destination"),
            },
            relative.to_vec(),
        )
        .unwrap()
    }

    fn file_state() -> SupportedState {
        SupportedState {
            node_kind: NodeKind::File,
            content: Some(ContentFingerprint {
                algorithm: "sha256".into(),
                digest: "a".repeat(64),
                length: 1,
            }),
            permission_mode: Some("0644".into()),
        }
    }

    #[test]
    fn v2_rejects_noncanonical_and_duplicate_records() {
        let mut map = BTreeMap::new();
        map.insert(identity(b"a"), file_state());
        map.insert(identity(b"b"), file_state());
        let mut envelope = StateEnvelopeV2::new(2, &map);
        envelope.payload.baselines.swap(0, 1);
        envelope.integrity.digest = digest_v2(&envelope.payload).unwrap();
        assert!(envelope.validate().is_err());

        envelope.payload.baselines[1] = envelope.payload.baselines[0].clone();
        envelope.integrity.digest = digest_v2(&envelope.payload).unwrap();
        assert!(envelope.validate().is_err());
    }

    #[test]
    fn v2_rejects_fields_that_do_not_apply_to_directories() {
        let mut map = BTreeMap::new();
        map.insert(
            identity(b"directory"),
            SupportedState {
                node_kind: NodeKind::Directory,
                content: None,
                permission_mode: Some("0755".into()),
            },
        );
        assert!(StateEnvelopeV2::new(0, &map).validate().is_err());
    }
}
