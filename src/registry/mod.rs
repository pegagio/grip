//! Versioned mapping-registry serialization and validation.

use crate::error::GripError;
use crate::mapping::{Mapping, MappingKind, OwnershipConflict, validate_ownership};
use serde::{Deserialize, Serialize};
use std::path::Component;

pub mod publication;

/// The complete accepted mapping registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registry {
    mappings: Vec<Mapping>,
}

impl Registry {
    /// Construct a registry and validate its complete ownership graph.
    pub fn new(mut mappings: Vec<Mapping>) -> Result<Self, GripError> {
        mappings.sort_by(|first, second| first.source.cmp(&second.source));
        let conflicts = validate_ownership(&mappings);
        if !conflicts.is_empty() {
            return Err(GripError::ownership_conflicts(conflicts));
        }
        Ok(Self { mappings })
    }

    /// Return mappings in canonical source order.
    pub fn mappings(&self) -> &[Mapping] {
        &self.mappings
    }

    /// Consume the registry and return its mappings.
    pub fn into_mappings(self) -> Vec<Mapping> {
        self.mappings
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RegistryV1 {
    schema_version: u64,
    mappings: Vec<MappingV1>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MappingV1 {
    kind: MappingKindV1,
    source: String,
    destination: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum MappingKindV1 {
    File,
    Tree,
}

impl From<MappingKindV1> for MappingKind {
    fn from(value: MappingKindV1) -> Self {
        match value {
            MappingKindV1::File => Self::File,
            MappingKindV1::Tree => Self::Tree,
        }
    }
}

impl From<MappingKind> for MappingKindV1 {
    fn from(value: MappingKind) -> Self {
        match value {
            MappingKind::File => Self::File,
            MappingKind::Tree => Self::Tree,
        }
    }
}

/// Decode and completely validate a strict v1 registry document.
pub fn decode(input: &str) -> Result<Registry, GripError> {
    Registry::new(decode_mappings(input)?)
}

/// Decode strict v1 wire mappings without accepting their ownership graph.
pub(crate) fn decode_mappings(input: &str) -> Result<Vec<Mapping>, GripError> {
    let raw: toml::Value = toml::from_str(input).map_err(|error| {
        GripError::InvalidConfiguration(format!("invalid registry TOML: {error}"))
    })?;
    let version = raw
        .get("schema_version")
        .and_then(toml::Value::as_integer)
        .ok_or_else(|| {
            GripError::InvalidConfiguration("registry schema_version must be an integer".into())
        })?;
    if version != 1 {
        return Err(GripError::UnsupportedSchema(format!(
            "unsupported registry schema version {version}"
        )));
    }
    let wire: RegistryV1 = raw.try_into().map_err(|error| {
        GripError::InvalidConfiguration(format!("invalid registry schema: {error}"))
    })?;
    wire.mappings
        .into_iter()
        .map(|mapping| {
            let source = std::path::PathBuf::from(mapping.source);
            let destination = std::path::PathBuf::from(mapping.destination);
            if !source.is_absolute() || !destination.is_absolute() {
                return Err(GripError::mapping(
                    "registry_validate",
                    "relative_path",
                    vec![
                        source.display().to_string(),
                        destination.display().to_string(),
                    ],
                    "registry mapping paths must be absolute",
                ));
            }
            if source
                .components()
                .any(|component| component == Component::ParentDir)
                || destination
                    .components()
                    .any(|component| component == Component::ParentDir)
            {
                return Err(GripError::mapping(
                    "registry_validate",
                    "parent_traversal",
                    vec![
                        source.display().to_string(),
                        destination.display().to_string(),
                    ],
                    "registry mapping paths must not contain parent traversal",
                ));
            }
            Ok(Mapping::new(mapping.kind.into(), source, destination))
        })
        .collect()
}

/// Serialize a registry deterministically as strict v1 TOML.
pub fn encode(registry: &Registry) -> Result<Vec<u8>, GripError> {
    let wire = RegistryV1 {
        schema_version: 1,
        mappings: registry
            .mappings()
            .iter()
            .map(|mapping| {
                Ok(MappingV1 {
                    kind: mapping.kind.into(),
                    source: mapping
                        .source
                        .to_str()
                        .ok_or_else(|| {
                            GripError::InvalidConfiguration(
                                "mapping source must be valid UTF-8".into(),
                            )
                        })?
                        .to_owned(),
                    destination: mapping
                        .destination
                        .to_str()
                        .ok_or_else(|| {
                            GripError::InvalidConfiguration(
                                "mapping destination must be valid UTF-8".into(),
                            )
                        })?
                        .to_owned(),
                })
            })
            .collect::<Result<Vec<_>, GripError>>()?,
    };
    toml::to_string(&wire)
        .map(String::into_bytes)
        .map_err(|error| GripError::Internal(format!("could not encode registry: {error}")))
}

/// Validate ownership and expose conflicts for focused tests.
pub fn conflicts(mappings: &[Mapping]) -> Vec<OwnershipConflict> {
    validate_ownership(mappings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_is_canonical_across_input_order() {
        let first = Mapping::new(
            MappingKind::File,
            "/source/a".into(),
            "/destination/a".into(),
        );
        let second = Mapping::new(
            MappingKind::File,
            "/source/z".into(),
            "/destination/z".into(),
        );
        let forward = Registry::new(vec![first.clone(), second.clone()]).unwrap();
        let reverse = Registry::new(vec![second, first]).unwrap();
        assert_eq!(encode(&forward).unwrap(), encode(&reverse).unwrap());
    }

    #[test]
    fn decode_rejects_unknown_mapping_fields() {
        let input = r#"
schema_version = 1
[[mappings]]
kind = "file"
source = "/source"
destination = "/destination"
unknown = true
"#;
        assert!(decode(input).is_err());
    }

    #[test]
    fn public_decode_still_rejects_an_invalid_ownership_graph() {
        let input = r#"
schema_version = 1
[[mappings]]
kind = "file"
source = "/source"
destination = "/destination-a"
[[mappings]]
kind = "file"
source = "/source"
destination = "/destination-b"
"#;
        assert!(matches!(
            decode(input),
            Err(GripError::Mapping { ref reason, .. }) if reason == "ownership_conflicts"
        ));
    }
}
