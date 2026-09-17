//! Versioned mapping-registry serialization and validation.

use crate::error::GripError;
use crate::mapping::{
    Mapping, MappingKind, OwnershipConflict, PortableMapping, validate_contained_tree_reservations,
    validate_ownership,
};
use rustix::fs::{Mode, OFlags, open};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

pub mod publication;

/// The portable mapping descriptor stored at `.grip/config.toml`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectDescriptorV2 {
    mappings: Vec<PortableMapping>,
    // Keep optional diff settings opaque while loading the registry. They are relevant only to a
    // selected human diff, so unrelated commands and machine-readable output remain usable when
    // a profile is malformed.
    diff: Option<toml::Value>,
    difftool: Option<toml::Value>,
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
        Ok(Self {
            mappings,
            diff: None,
            difftool: None,
        })
    }

    pub fn empty() -> Self {
        Self {
            mappings: Vec::new(),
            diff: None,
            difftool: None,
        }
    }

    pub fn mappings(&self) -> &[PortableMapping] {
        &self.mappings
    }

    pub fn with_mappings(&self, mappings: Vec<PortableMapping>) -> Result<Self, GripError> {
        let mut next = Self::new(mappings)?;
        next.diff = self.diff.clone();
        next.difftool = self.difftool.clone();
        Ok(next)
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
        let mut conflicts = validate_ownership(&ownership);
        conflicts.extend(validate_contained_tree_reservations(&ownership));
        conflicts.sort();
        conflicts.dedup();
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
    #[serde(default)]
    diff: Option<toml::Value>,
    #[serde(default)]
    difftool: Option<toml::Value>,
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
    let mut descriptor = ProjectDescriptorV2::new(wire.mappings)?;
    descriptor.diff = wire.diff;
    descriptor.difftool = wire.difftool;
    Ok(descriptor)
}

pub fn encode_descriptor(descriptor: &ProjectDescriptorV2) -> Result<Vec<u8>, GripError> {
    let wire = DescriptorV2Wire {
        schema_version: 2,
        mappings: descriptor.mappings.clone(),
        diff: descriptor.diff.clone(),
        difftool: descriptor.difftool.clone(),
    };
    toml::to_string(&wire)
        .map(String::into_bytes)
        .map_err(|error| GripError::Internal(format!("could not encode descriptor: {error}")))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalDiff {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Default, Clone)]
struct DiffLayer {
    tool: Option<String>,
    tools: BTreeMap<String, DiffToolLayer>,
}

#[derive(Debug, Default, Clone)]
struct DiffToolLayer {
    program: Option<String>,
    args: Option<Vec<String>>,
}

impl DiffLayer {
    fn merge(mut self, project: Self) -> Self {
        if project.tool.is_some() {
            self.tool = project.tool;
        }
        for (name, project_tool) in project.tools {
            let tool = self.tools.entry(name).or_default();
            if project_tool.program.is_some() {
                tool.program = project_tool.program;
            }
            if project_tool.args.is_some() {
                tool.args = project_tool.args;
            }
        }
        self
    }
}

fn parse_layer(
    diff: Option<&toml::Table>,
    difftool: Option<&toml::Table>,
) -> Result<DiffLayer, GripError> {
    let mut layer = DiffLayer::default();
    if let Some(diff) = diff {
        if let Some(value) = diff.get("tool") {
            layer.tool = Some(
                value
                    .as_str()
                    .ok_or_else(|| {
                        GripError::InvalidConfiguration("diff.tool must be a string".into())
                    })?
                    .to_owned(),
            );
        }
        if diff.keys().any(|key| key != "tool") {
            return Err(GripError::InvalidConfiguration(
                "diff contains an unknown setting".into(),
            ));
        }
    }
    if let Some(tools) = difftool {
        for (name, value) in tools {
            let table = value.as_table().ok_or_else(|| {
                GripError::InvalidConfiguration(format!("difftool.{name} must be a table"))
            })?;
            if table.keys().any(|key| key != "program" && key != "args") {
                return Err(GripError::InvalidConfiguration(format!(
                    "difftool.{name} contains an unknown setting"
                )));
            }
            let program = table
                .get("program")
                .map(|value| {
                    value.as_str().map(str::to_owned).ok_or_else(|| {
                        GripError::InvalidConfiguration(format!(
                            "difftool.{name}.program must be a string"
                        ))
                    })
                })
                .transpose()?;
            let args = table
                .get("args")
                .map(|value| {
                    value
                        .as_array()
                        .ok_or_else(|| {
                            GripError::InvalidConfiguration(format!(
                                "difftool.{name}.args must be an array"
                            ))
                        })?
                        .iter()
                        .map(|value| {
                            value.as_str().map(str::to_owned).ok_or_else(|| {
                                GripError::InvalidConfiguration(format!(
                                    "difftool.{name}.args must contain only strings"
                                ))
                            })
                        })
                        .collect()
                })
                .transpose()?;
            layer
                .tools
                .insert(name.clone(), DiffToolLayer { program, args });
        }
    }
    Ok(layer)
}

fn profile_table<'a>(
    value: Option<&'a toml::Value>,
    name: &str,
) -> Result<Option<&'a toml::Table>, GripError> {
    value
        .map(|value| {
            value
                .as_table()
                .ok_or_else(|| GripError::InvalidConfiguration(format!("{name} must be a table")))
        })
        .transpose()
}

fn global_layer(home: &Path) -> Result<DiffLayer, GripError> {
    let directory = home.join(".grip");
    let path = directory.join("config.toml");
    match fs::symlink_metadata(&directory) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() && metadata.uid() == rustix::process::geteuid().as_raw() && metadata.permissions().mode() & 0o022 == 0 => {}
        Ok(_) => return Err(GripError::InvalidConfiguration("global diff configuration directory must be a current-user-owned non-symlink directory with a safe permission mode".into())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(DiffLayer::default()),
        Err(error) => return Err(GripError::InvalidConfiguration(format!("could not inspect global diff configuration directory: {error}"))),
    }
    let metadata = match fs::symlink_metadata(&path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(DiffLayer::default());
        }
        Err(error) => {
            return Err(GripError::InvalidConfiguration(format!(
                "could not inspect global diff configuration: {error}"
            )));
        }
    };
    let safe = metadata.is_file()
        && !metadata.file_type().is_symlink()
        && metadata.uid() == rustix::process::geteuid().as_raw()
        && metadata.permissions().mode() & 0o022 == 0;
    if !safe {
        return Err(GripError::InvalidConfiguration("global diff configuration must be a current-user-owned non-symlink regular file with a safe permission mode".into()));
    }
    let descriptor = open(
        &path,
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
    )
    .map_err(|error| {
        GripError::InvalidConfiguration(format!(
            "could not safely open global diff configuration: {error}"
        ))
    })?;
    let mut text = String::new();
    std::fs::File::from(descriptor)
        .read_to_string(&mut text)
        .map_err(|error| {
            GripError::InvalidConfiguration(format!(
                "could not read global diff configuration: {error}"
            ))
        })?;
    let value: toml::Table = toml::from_str(&text).map_err(|error| {
        GripError::InvalidConfiguration(format!("invalid global diff configuration TOML: {error}"))
    })?;
    if value.keys().any(|key| key != "diff" && key != "difftool") {
        return Err(GripError::InvalidConfiguration(
            "global diff configuration contains an unknown setting".into(),
        ));
    }
    let diff = value
        .get("diff")
        .map(|value| {
            value
                .as_table()
                .ok_or_else(|| GripError::InvalidConfiguration("diff must be a table".into()))
        })
        .transpose()?;
    let tools = value
        .get("difftool")
        .map(|value| {
            value
                .as_table()
                .ok_or_else(|| GripError::InvalidConfiguration("difftool must be a table".into()))
        })
        .transpose()?;
    parse_layer(diff, tools)
}

pub fn resolve_external_diff(
    descriptor: &ProjectDescriptorV2,
    home: &Path,
) -> Result<ExternalDiff, GripError> {
    match std::env::var_os("GRIP_EXTERNAL_DIFF") {
        Some(value) if value.is_empty() => {
            return Err(GripError::InvalidConfiguration(
                "GRIP_EXTERNAL_DIFF must not be empty".into(),
            ));
        }
        Some(value) => {
            return Ok(ExternalDiff {
                program: value.into_string().map_err(|_| {
                    GripError::InvalidConfiguration("GRIP_EXTERNAL_DIFF must be valid UTF-8".into())
                })?,
                args: Vec::new(),
            });
        }
        None => {}
    }
    let profile = global_layer(home)?.merge(parse_layer(
        profile_table(descriptor.diff.as_ref(), "diff")?,
        profile_table(descriptor.difftool.as_ref(), "difftool")?,
    )?);
    let Some(name) = profile.tool else {
        return Ok(ExternalDiff {
            program: "diff".into(),
            args: Vec::new(),
        });
    };
    if name.is_empty() {
        return Err(GripError::InvalidConfiguration(
            "diff.tool must not be empty".into(),
        ));
    }
    let tool = profile.tools.get(&name).ok_or_else(|| {
        GripError::InvalidConfiguration(format!("diff tool {name:?} is not defined"))
    })?;
    let program = tool
        .program
        .clone()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            GripError::InvalidConfiguration(format!("difftool.{name}.program must not be empty"))
        })?;
    Ok(ExternalDiff {
        program,
        args: tool.args.clone().unwrap_or_default(),
    })
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
