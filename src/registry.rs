use crate::error::GripError;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq)]
pub struct Registry;
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryV1 {
    schema_version: u64,
    mappings: Vec<serde::de::IgnoredAny>,
}

pub fn decode(input: &str) -> Result<Registry, GripError> {
    let raw: toml::Value = toml::from_str(input)
        .map_err(|e| GripError::InvalidConfiguration(format!("invalid registry TOML: {e}")))?;
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
    let parsed: RegistryV1 = toml::from_str(input)
        .map_err(|e| GripError::InvalidConfiguration(format!("invalid registry schema: {e}")))?;
    if parsed.schema_version != 1 || !parsed.mappings.is_empty() {
        return Err(GripError::InvalidConfiguration(
            "Feature 001 requires an empty mappings array".into(),
        ));
    }
    Ok(Registry)
}
