//! Versioned mapping-registry serialization and validation.

use crate::error::GripError;
use crate::mapping::{
    Mapping, MappingKind, OwnershipConflict, PortableMapping, validate_ownership,
};
use serde::{Deserialize, Serialize};

pub mod publication;

/// The portable mapping descriptor stored at `.grip/config.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDescriptorV2 {
    mappings: Vec<PortableMapping>,
}

impl ProjectDescriptorV2 {
    pub fn new(mut mappings: Vec<PortableMapping>) -> Result<Self, GripError> {
        if mappings
            .iter()
            .any(|mapping| mapping.kind == MappingKind::File && mapping.source.as_str() == ".")
        {
            return Err(GripError::mapping(
                "descriptor_validate",
                "invalid_project_relative_path",
                vec![".".into()],
                "the project root is valid only for a tree mapping",
            ));
        }
        mappings.sort();
        for pair in mappings.windows(2) {
            if pair[0].source == pair[1].source {
                return Err(GripError::mapping(
                    "descriptor_validate",
                    if pair[0] == pair[1] {
                        "duplicate_tuple"
                    } else {
                        "duplicate_source"
                    },
                    vec![pair[0].source.as_str().into()],
                    "descriptor mapping sources must be unique",
                ));
            }
        }
        Ok(Self { mappings })
    }

    pub fn empty() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }

    pub fn mappings(&self) -> &[PortableMapping] {
        &self.mappings
    }

    pub fn resolve(
        &self,
        project_root: &std::path::Path,
        home_root: &std::path::Path,
        operation: &str,
    ) -> Result<Vec<crate::mapping::ResolvedMapping>, GripError> {
        let resolved = self
            .mappings
            .iter()
            .map(|mapping| mapping.resolve(project_root, home_root, operation))
            .collect::<Result<Vec<_>, _>>()?;
        let ownership = resolved
            .iter()
            .map(crate::mapping::ResolvedMapping::ownership_mapping)
            .collect::<Vec<_>>();
        let conflicts = validate_ownership(&ownership);
        if conflicts.is_empty() {
            Ok(resolved)
        } else {
            Err(GripError::ownership_conflicts(conflicts))
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DescriptorV2Wire {
    schema_version: u64,
    mappings: Vec<PortableMapping>,
}

pub fn decode_descriptor(input: &str) -> Result<ProjectDescriptorV2, GripError> {
    let raw: toml::Value = toml::from_str(input).map_err(|error| {
        GripError::InvalidConfiguration(format!("invalid descriptor TOML: {error}"))
    })?;
    let version = raw
        .get("schema_version")
        .and_then(toml::Value::as_integer)
        .ok_or_else(|| {
            GripError::InvalidConfiguration("descriptor schema_version must be an integer".into())
        })?;
    if version != 2 {
        return Err(GripError::UnsupportedSchema(format!(
            "unsupported descriptor schema version {version}"
        )));
    }
    let wire: DescriptorV2Wire = raw.try_into().map_err(|error| {
        GripError::InvalidConfiguration(format!("invalid descriptor schema: {error}"))
    })?;
    ProjectDescriptorV2::new(wire.mappings)
}

pub fn encode_descriptor(descriptor: &ProjectDescriptorV2) -> Result<Vec<u8>, GripError> {
    let wire = DescriptorV2Wire {
        schema_version: 2,
        mappings: descriptor.mappings.clone(),
    };
    toml::to_string(&wire)
        .map(String::into_bytes)
        .map_err(|error| GripError::Internal(format!("could not encode descriptor: {error}")))
}

/// The complete accepted mapping registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRegistry {
    mappings: Vec<Mapping>,
}

impl ResolvedRegistry {
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

/// Validate ownership and expose conflicts for focused tests.
pub fn conflicts(mappings: &[Mapping]) -> Vec<OwnershipConflict> {
    validate_ownership(mappings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_registry_is_canonical_across_input_order() {
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
        let forward = ResolvedRegistry::new(vec![first.clone(), second.clone()]).unwrap();
        let reverse = ResolvedRegistry::new(vec![second, first]).unwrap();
        assert_eq!(forward, reverse);
    }
}
