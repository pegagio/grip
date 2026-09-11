pub mod lock;
pub mod mutation_lock;
pub mod publication;
pub mod rebinding;

use crate::error::GripError;
use crate::metadata::model::SupportedEntryStateV3;
use crate::observation::model::{EntryIdentity, ResolvedMapping};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointRoleV1 {
    Source,
    Destination,
    Descriptor,
    AcceptedState,
}

/// Portable identity for one accepted entry in State V4.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntryIdentityV4 {
    pub mapping: crate::mapping::PortableMapping,
    pub relative_path_hex: String,
}

/// Publication-time local binding used to detect project moves and copies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectBindingV1 {
    pub project_root: String,
    pub user_home: String,
    pub descriptor_digest: String,
    pub resolved_mapping_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineRecordV4 {
    pub identity: EntryIdentityV4,
    pub state: SupportedEntryStateV3,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatePayloadV4 {
    pub generation: u64,
    pub binding: ProjectBindingV1,
    pub baselines: Vec<BaselineRecordV4>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateEnvelopeV4 {
    pub schema_version: u64,
    pub payload: StatePayloadV4,
    pub integrity: IntegrityV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedStateV4 {
    pub generation: u64,
    pub binding: ProjectBindingV1,
    pub baselines: BTreeMap<EntryIdentityV4, SupportedEntryStateV3>,
    pub accepted_bytes: Option<Vec<u8>>,
}

#[derive(Serialize)]
struct IntegrityInputV4<'a> {
    schema_version: u64,
    payload: &'a StatePayloadV4,
}

impl StateEnvelopeV4 {
    pub fn new(state: &AcceptedStateV4) -> Self {
        let payload = StatePayloadV4 {
            generation: state.generation,
            binding: state.binding.clone(),
            baselines: state
                .baselines
                .iter()
                .map(|(identity, state)| BaselineRecordV4 {
                    identity: identity.clone(),
                    state: state.clone(),
                })
                .collect(),
        };
        let digest = state_v4_digest(&payload).expect("typed State V4 serialization cannot fail");
        Self {
            schema_version: 4,
            payload,
            integrity: IntegrityV1 {
                algorithm: "sha256".into(),
                digest,
            },
        }
    }

    pub fn validate(&self) -> Result<(), GripError> {
        if self.schema_version != 4 {
            return Err(GripError::UnsupportedSchema(format!(
                "unsupported state schema version {}",
                self.schema_version
            )));
        }
        validate_binding(&self.payload.binding)?;
        if self.integrity.algorithm != "sha256"
            || !is_sha256(&self.integrity.digest)
            || self.integrity.digest != state_v4_digest(&self.payload)?
        {
            return Err(GripError::CorruptState(
                "state integrity verification failed".into(),
            ));
        }
        let mut previous: Option<&EntryIdentityV4> = None;
        for record in &self.payload.baselines {
            validate_v4_identity(&record.identity)?;
            record
                .state
                .validate()
                .map_err(|message| GripError::CorruptState(message.into()))?;
            if previous.is_some_and(|value| value >= &record.identity) {
                return Err(GripError::CorruptState(
                    "State V4 baselines must be unique and canonically ordered".into(),
                ));
            }
            previous = Some(&record.identity);
        }
        Ok(())
    }
}

fn validate_binding(binding: &ProjectBindingV1) -> Result<(), GripError> {
    if !std::path::Path::new(&binding.project_root).is_absolute()
        || !std::path::Path::new(&binding.user_home).is_absolute()
        || !is_sha256(&binding.descriptor_digest)
        || !is_sha256(&binding.resolved_mapping_digest)
    {
        return Err(GripError::CorruptState(
            "State V4 project binding is invalid".into(),
        ));
    }
    Ok(())
}

fn validate_v4_identity(identity: &EntryIdentityV4) -> Result<(), GripError> {
    decode_hex(&identity.relative_path_hex)?;
    crate::registry::ProjectDescriptorV2::new(vec![identity.mapping.clone()])?;
    Ok(())
}

pub fn state_v4_digest(payload: &StatePayloadV4) -> Result<String, GripError> {
    let bytes = serde_json::to_vec(&IntegrityInputV4 {
        schema_version: 4,
        payload,
    })
    .map_err(|error| {
        GripError::Internal(format!(
            "could not encode State V4 integrity input: {error}"
        ))
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

pub fn encode_v4(state: &AcceptedStateV4) -> Result<Vec<u8>, GripError> {
    serde_json::to_vec(&StateEnvelopeV4::new(state))
        .map_err(|error| GripError::Internal(format!("could not encode State V4: {error}")))
}

pub fn decode_v4(input: &[u8]) -> Result<AcceptedStateV4, GripError> {
    let raw: serde_json::Value = serde_json::from_slice(input)
        .map_err(|error| GripError::CorruptState(format!("invalid state JSON: {error}")))?;
    let version = raw
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| GripError::CorruptState("state schema_version is required".into()))?;
    if version != 4 {
        return Err(GripError::UnsupportedSchema(format!(
            "unsupported state schema version {version}"
        )));
    }
    let envelope: StateEnvelopeV4 = serde_json::from_value(raw)
        .map_err(|error| GripError::CorruptState(format!("invalid State V4 schema: {error}")))?;
    envelope.validate()?;
    Ok(AcceptedStateV4 {
        generation: envelope.payload.generation,
        binding: envelope.payload.binding,
        baselines: envelope
            .payload
            .baselines
            .into_iter()
            .map(|record| (record.identity, record.state))
            .collect(),
        accepted_bytes: Some(input.to_vec()),
    })
}

pub(crate) fn current_binding(
    home: &crate::project::ProjectPaths,
    descriptor_bytes: &[u8],
    descriptor: &crate::registry::ProjectDescriptorV2,
) -> Result<ProjectBindingV1, GripError> {
    let project_root = home.project_root()?;
    let user_home = home.destination_home().ok_or_else(|| {
        GripError::InvalidConfiguration("project destination home is unavailable".into())
    })?;
    let resolved = descriptor.resolve(project_root, user_home, "state_binding")?;
    let tuples = resolved
        .iter()
        .map(|mapping| {
            serde_json::json!({
                "identity": mapping.declaration.identity(),
                "source": mapping.source,
                "destination": mapping.destination,
            })
        })
        .collect::<Vec<_>>();
    let resolved_bytes = serde_json::to_vec(&tuples).map_err(|error| {
        GripError::Internal(format!(
            "could not encode resolved mapping binding: {error}"
        ))
    })?;
    Ok(ProjectBindingV1 {
        project_root: project_root.to_string_lossy().into_owned(),
        user_home: user_home.to_string_lossy().into_owned(),
        descriptor_digest: format!("{:x}", Sha256::digest(descriptor_bytes)),
        resolved_mapping_digest: format!("{:x}", Sha256::digest(resolved_bytes)),
    })
}

pub(crate) fn accepted_v4_from_runtime(
    home: &crate::project::ProjectPaths,
    generation: u64,
    baselines: &BTreeMap<EntryIdentity, SupportedEntryStateV3>,
) -> Result<AcceptedStateV4, GripError> {
    let descriptor_bytes = std::fs::read(home.path().join("config.toml")).map_err(|error| {
        GripError::from_io("could not read project descriptor for state", error)
    })?;
    let descriptor_text = std::str::from_utf8(&descriptor_bytes)
        .map_err(|_| GripError::InvalidConfiguration("project descriptor must be UTF-8".into()))?;
    let descriptor = crate::registry::decode_descriptor(descriptor_text)?;
    let project_root = home.project_root()?;
    let user_home = home.destination_home().ok_or_else(|| {
        GripError::InvalidConfiguration("project destination home is unavailable".into())
    })?;
    let resolved = descriptor.resolve(project_root, user_home, "state_publication")?;
    let mut portable_baselines = BTreeMap::new();
    for (identity, state) in baselines {
        let declaration = resolved
            .iter()
            .find(|mapping| {
                mapping.declaration.kind == identity.mapping.kind
                    && mapping.source == identity.mapping.source
                    && mapping.destination == identity.mapping.destination
            })
            .map(|mapping| mapping.declaration.clone())
            .ok_or_else(|| {
                GripError::CorruptState(
                    "accepted baseline has no active mapping declaration".into(),
                )
            })?;
        let portable = EntryIdentityV4 {
            mapping: declaration.clone(),
            relative_path_hex: encode_hex(&identity.relative_path),
        };
        portable_baselines.insert(portable, state.clone());
    }
    Ok(AcceptedStateV4 {
        generation,
        binding: current_binding(home, &descriptor_bytes, &descriptor)?,
        baselines: portable_baselines,
        accepted_bytes: None,
    })
}

pub(crate) fn runtime_from_accepted_v4(
    home: &crate::project::ProjectPaths,
    state: &AcceptedStateV4,
) -> Result<AcceptedStateV3, GripError> {
    let project_root = home.project_root()?;
    let user_home = home.destination_home().ok_or_else(|| {
        GripError::InvalidConfiguration("project destination home is unavailable".into())
    })?;
    let mut baselines = BTreeMap::new();
    for (identity, value) in &state.baselines {
        let resolved = identity
            .mapping
            .resolve(project_root, user_home, "state_runtime")?;
        let mapping = ResolvedMapping::from(&resolved.ownership_mapping());
        let runtime = EntryIdentity::new(mapping, decode_hex(&identity.relative_path_hex)?)
            .map_err(|message| GripError::CorruptState(message.into()))?;
        if baselines.insert(runtime, value.clone()).is_some() {
            return Err(GripError::CorruptState(
                "portable state identities resolve ambiguously".into(),
            ));
        }
    }
    Ok(AcceptedStateV3 {
        generation: state.generation,
        baselines,
        accepted_bytes: state.accepted_bytes.clone(),
    })
}

pub(crate) fn portable_identity_from_runtime(
    home: &crate::project::ProjectPaths,
    identity: &EntryIdentity,
) -> Result<EntryIdentityV4, GripError> {
    let descriptor_bytes = std::fs::read(home.path().join("config.toml")).map_err(|error| {
        GripError::from_io("could not read project descriptor for state", error)
    })?;
    let descriptor_text = std::str::from_utf8(&descriptor_bytes)
        .map_err(|_| GripError::InvalidConfiguration("project descriptor must be UTF-8".into()))?;
    let descriptor = crate::registry::decode_descriptor(descriptor_text)?;
    let resolved = descriptor.resolve(
        home.project_root()?,
        home.destination_home().ok_or_else(|| {
            GripError::InvalidConfiguration("project destination home is unavailable".into())
        })?,
        "state_identity",
    )?;
    let mapping = resolved
        .iter()
        .find(|mapping| {
            mapping.declaration.kind == identity.mapping.kind
                && mapping.source == identity.mapping.source
                && mapping.destination == identity.mapping.destination
        })
        .map(|mapping| mapping.declaration.clone())
        .ok_or_else(|| {
            GripError::CorruptState("runtime identity has no active mapping declaration".into())
        })?;
    Ok(EntryIdentityV4 {
        mapping,
        relative_path_hex: encode_hex(&identity.relative_path),
    })
}

#[allow(dead_code)]
pub(crate) fn runtime_identity_from_portable(
    home: &crate::project::ProjectPaths,
    identity: &EntryIdentityV4,
) -> Result<EntryIdentity, GripError> {
    EntryIdentity::new(
        ResolvedMapping {
            kind: identity.mapping.kind,
            source: identity.mapping.source.resolve(home.project_root()?),
            destination: identity
                .mapping
                .destination
                .resolve(home.destination_home().ok_or_else(|| {
                    GripError::InvalidConfiguration(
                        "project destination home is unavailable".into(),
                    )
                })?),
        },
        decode_hex(&identity.relative_path_hex)?,
    )
    .map_err(|message| GripError::CorruptState(message.into()))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntegrityV1 {
    pub algorithm: String,
    pub digest: String,
}

/// Version-neutral accepted state used by classification and publication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedState {
    pub generation: Option<u64>,
    pub complete_baselines: BTreeMap<EntryIdentity, SupportedEntryStateV3>,
    pub accepted_bytes: Option<Vec<u8>>,
}

/// Complete accepted state represented by State Envelope V3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedStateV3 {
    pub generation: u64,
    pub baselines: BTreeMap<EntryIdentity, SupportedEntryStateV3>,
    pub accepted_bytes: Option<Vec<u8>>,
}

impl AcceptedState {
    pub fn uninitialized() -> Self {
        Self {
            generation: None,
            complete_baselines: BTreeMap::new(),
            accepted_bytes: None,
        }
    }
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

pub(crate) fn decode_v4_identity_path(value: &str) -> Result<Vec<u8>, GripError> {
    decode_hex(value)
}
