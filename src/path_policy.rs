//! Canonical path resolution for mapping endpoints and selectors.

use crate::error::GripError;
use crate::mapping::MappingKind;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

/// A canonical path declaration resolved beneath a Grip project root.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct ProjectRelativePath(String);

impl ProjectRelativePath {
    /// Parse a portable project-relative source declaration.
    pub fn parse(value: &std::ffi::OsStr, allow_tree_root: bool) -> Result<Self, GripError> {
        let value = value.to_str().ok_or_else(|| {
            portable_path_error("non_utf8_path", value, "portable paths must be valid UTF-8")
        })?;
        if value == "." && allow_tree_root {
            return Ok(Self(value.to_owned()));
        }
        if !valid_relative_components(value)
            || value.starts_with('~')
            || value.contains('$')
            || value.split('/').next() == Some(".grip")
        {
            return Err(portable_path_error(
                "invalid_project_relative_path",
                std::ffi::OsStr::new(value),
                "source must be a normalized project-relative path outside .grip",
            ));
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn resolve(&self, root: &Path) -> PathBuf {
        if self.0 == "." {
            root.to_path_buf()
        } else {
            root.join(&self.0)
        }
    }
}

impl<'de> serde::Deserialize<'de> for ProjectRelativePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(std::ffi::OsStr::new(&value), true).map_err(serde::de::Error::custom)
    }
}

/// A portable destination declaration, retained exactly as supplied by the user.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct DestinationPath(String);

impl DestinationPath {
    /// Parse an absolute path, exact `~`, or any `~/` path spelling.
    pub fn parse(value: &std::ffi::OsStr) -> Result<Self, GripError> {
        let value = value.to_str().ok_or_else(|| {
            portable_path_error("non_utf8_path", value, "portable paths must be valid UTF-8")
        })?;
        if value == "~" || value.starts_with("~/") || Path::new(value).is_absolute() {
            return Ok(Self(value.to_owned()));
        }
        Err(portable_path_error(
            "invalid_destination_path",
            std::ffi::OsStr::new(value),
            "destination must be an absolute path, ~, or a ~/ path",
        ))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn resolve(&self, home: &Path) -> PathBuf {
        let path = match self.0.strip_prefix("~/") {
            Some(suffix) => home.join(suffix),
            None if self.0 == "~" => home.to_path_buf(),
            None => PathBuf::from(&self.0),
        };
        lexical_normalize_absolute(&path)
    }
}

impl<'de> serde::Deserialize<'de> for DestinationPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(std::ffi::OsStr::new(&value)).map_err(serde::de::Error::custom)
    }
}

/// Normalize lexical dot components without resolving symbolic links.
fn lexical_normalize_absolute(path: &Path) -> PathBuf {
    debug_assert!(path.is_absolute());
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(value) => normalized.push(value),
            Component::Prefix(_) => unreachable!("Unix paths have no prefix components"),
        }
    }
    normalized
}

fn valid_relative_components(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.ends_with('/')
        && value
            .split('/')
            .all(|component| !component.is_empty() && component != "." && component != "..")
}

fn portable_path_error(reason: &str, value: &std::ffi::OsStr, message: &str) -> GripError {
    GripError::mapping(
        "portable_path_parse",
        reason,
        vec![value.to_string_lossy().into_owned()],
        message,
    )
}

#[cfg(test)]
thread_local! {
    static METADATA_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Revalidatable evidence for one canonical mapping endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathEvidence {
    submitted: PathBuf,
    pub canonical: PathBuf,
    pub kind: MappingKind,
    pub source: bool,
    allow_absent: bool,
    pub exists: bool,
    anchor: PathBuf,
    device: u64,
    inode: u64,
    node_mode: u32,
}

impl PathEvidence {
    /// Return absent destination parent components from the validated anchor outward.
    pub(crate) fn missing_destination_parents(&self) -> Vec<PathBuf> {
        if self.source || self.exists {
            return Vec::new();
        }
        let mut parents = Vec::new();
        let mut current = if self.kind == MappingKind::Tree {
            Some(self.canonical.as_path())
        } else {
            self.canonical.parent()
        };
        while let Some(path) = current {
            if path == self.anchor {
                break;
            }
            if !path.starts_with(&self.anchor) {
                break;
            }
            parents.push(path.to_path_buf());
            current = path.parent();
        }
        parents.reverse();
        parents
    }
}

struct ResolvedPath {
    canonical: PathBuf,
    exists: bool,
    anchor: PathBuf,
    metadata: fs::Metadata,
}

fn nonfollowing_metadata(path: &Path) -> std::io::Result<fs::Metadata> {
    #[cfg(test)]
    METADATA_CALLS.with(|calls| calls.set(calls.get() + 1));
    fs::symlink_metadata(path)
}

fn invalid(operation: &str, reason: &str, path: &Path, message: &str) -> GripError {
    GripError::mapping(
        operation,
        reason,
        vec![path.to_string_lossy().into_owned()],
        message,
    )
}

fn validate_input(path: &Path, operation: &str) -> Result<(), GripError> {
    if !path.is_absolute() {
        return Err(invalid(
            operation,
            "relative_path",
            path,
            "mapping paths must be absolute",
        ));
    }
    if path.to_str().is_none() {
        return Err(invalid(
            operation,
            "non_utf8_path",
            path,
            "mapping paths must be valid UTF-8",
        ));
    }
    if path
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err(invalid(
            operation,
            "parent_traversal",
            path,
            "mapping paths must not contain parent traversal",
        ));
    }
    Ok(())
}

fn longest_existing_prefix(
    path: &Path,
    operation: &str,
) -> Result<(PathBuf, Vec<OsString>, fs::Metadata), GripError> {
    let mut existing = path;
    let mut suffix = Vec::new();
    let metadata = loop {
        match nonfollowing_metadata(existing) {
            Ok(metadata) => break metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let name = existing.file_name().ok_or_else(|| {
                    invalid(
                        operation,
                        "path_unavailable",
                        path,
                        "path has no existing ancestor",
                    )
                })?;
                suffix.push(name.to_owned());
                existing = existing.parent().ok_or_else(|| {
                    invalid(
                        operation,
                        "path_unavailable",
                        path,
                        "path has no existing ancestor",
                    )
                })?;
            }
            Err(error) => {
                return Err(invalid(
                    operation,
                    "path_unavailable",
                    path,
                    &format!("mapping path is unavailable: {error}"),
                ));
            }
        }
    };
    suffix.reverse();
    Ok((existing.to_path_buf(), suffix, metadata))
}

fn inspect_path(
    path: &Path,
    allow_absent: bool,
    operation: &str,
) -> Result<ResolvedPath, GripError> {
    validate_input(path, operation)?;
    let (existing, suffix, metadata) = longest_existing_prefix(path, operation)?;
    if !allow_absent && !suffix.is_empty() {
        return Err(invalid(
            operation,
            "path_unavailable",
            path,
            "mapping source must exist",
        ));
    }
    if suffix.is_empty() && metadata.file_type().is_symlink() {
        return Err(invalid(
            operation,
            "symlink_endpoint",
            path,
            "mapping endpoint must not be a symbolic link",
        ));
    }
    let anchor = fs::canonicalize(&existing).map_err(|error| {
        invalid(
            operation,
            "path_unavailable",
            path,
            &format!("mapping path cannot be canonicalized: {error}"),
        )
    })?;
    let metadata = if !suffix.is_empty() && metadata.file_type().is_symlink() {
        nonfollowing_metadata(&anchor).map_err(|error| {
            invalid(
                operation,
                "path_unavailable",
                path,
                &format!("resolved mapping ancestor is unavailable: {error}"),
            )
        })?
    } else {
        metadata
    };
    if !suffix.is_empty() && !metadata.is_dir() {
        return Err(invalid(
            operation,
            "unsafe_ancestry",
            path,
            "nearest existing destination ancestor must be a directory",
        ));
    }
    let exists = suffix.is_empty();
    let mut canonical = anchor.clone();
    for component in &suffix {
        canonical.push(component);
    }
    Ok(ResolvedPath {
        canonical,
        exists,
        anchor,
        metadata,
    })
}

fn validate_endpoint_kind(
    inspected: &ResolvedPath,
    path: &Path,
    kind: MappingKind,
    operation: &str,
) -> Result<(), GripError> {
    if inspected.exists {
        let valid_kind = match kind {
            MappingKind::File => inspected.metadata.is_file(),
            MappingKind::Tree => inspected.metadata.is_dir(),
        };
        if !valid_kind {
            return Err(invalid(
                operation,
                "wrong_node_kind",
                path,
                "mapping endpoint does not match the requested mapping kind",
            ));
        }
    }
    Ok(())
}

/// Resolve and validate a mapping endpoint for the requested kind.
pub fn resolve_endpoint(
    path: &Path,
    kind: MappingKind,
    source: bool,
    operation: &str,
) -> Result<PathBuf, GripError> {
    let inspected = inspect_path(path, !source, operation)?;
    validate_endpoint_kind(&inspected, path, kind, operation)?;
    Ok(inspected.canonical)
}

/// Resolve an endpoint and capture the safety facts used by publication.
pub fn inspect_endpoint(
    path: &Path,
    kind: MappingKind,
    source: bool,
    operation: &str,
) -> Result<PathEvidence, GripError> {
    inspect_endpoint_with_absence(path, kind, source, !source, operation)
}

/// Inspect a durable accepted endpoint without requiring it to remain present.
pub fn inspect_durable_endpoint(
    path: &Path,
    kind: MappingKind,
    source: bool,
    operation: &str,
) -> Result<PathEvidence, GripError> {
    inspect_endpoint_with_absence(path, kind, source, true, operation)
}

fn inspect_endpoint_with_absence(
    path: &Path,
    kind: MappingKind,
    source: bool,
    allow_absent: bool,
    operation: &str,
) -> Result<PathEvidence, GripError> {
    let inspected = inspect_path(path, allow_absent, operation)?;
    validate_endpoint_kind(&inspected, path, kind, operation)?;
    Ok(PathEvidence {
        submitted: path.to_path_buf(),
        exists: inspected.exists,
        canonical: inspected.canonical,
        kind,
        source,
        allow_absent,
        anchor: inspected.anchor,
        device: inspected.metadata.dev(),
        inode: inspected.metadata.ino(),
        node_mode: inspected.metadata.mode(),
    })
}

/// Reinspect an endpoint and reject drift from the captured evidence.
pub fn revalidate(evidence: &PathEvidence, operation: &str) -> Result<(), GripError> {
    let current = inspect_endpoint_with_absence(
        &evidence.submitted,
        evidence.kind,
        evidence.source,
        evidence.allow_absent,
        operation,
    )?;
    if current != *evidence {
        return Err(GripError::mapping(
            operation,
            "stale_path_evidence",
            vec![evidence.canonical.display().to_string()],
            "mapping path evidence changed before publication",
        ));
    }
    Ok(())
}

/// Resolve an existing file or directory used as a source identity selector.
pub fn resolve_selector(path: &Path, operation: &str) -> Result<PathBuf, GripError> {
    let inspected = inspect_path(path, true, operation)?;
    if inspected.exists && !inspected.metadata.is_file() && !inspected.metadata.is_dir() {
        return Err(invalid(
            operation,
            "unsupported_node",
            path,
            "mapping source must be a regular file or directory",
        ));
    }
    Ok(inspected.canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_endpoint_rejects_parent_traversal_before_filesystem_access() {
        let error = resolve_endpoint(
            Path::new("/tmp/parent/../child"),
            MappingKind::File,
            false,
            "mapping_add",
        )
        .unwrap_err();
        assert!(matches!(
            error,
            GripError::Mapping { ref reason, .. } if reason == "parent_traversal"
        ));
    }

    #[test]
    fn revalidate_detects_node_kind_change() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("source");
        fs::write(&path, "x").unwrap();
        let evidence = inspect_endpoint(&path, MappingKind::File, true, "mapping_add").unwrap();
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(revalidate(&evidence, "mapping_add").is_err());
    }

    #[test]
    fn evidence_capture_uses_one_bounded_metadata_walk() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source");
        fs::write(&source, "payload").unwrap();

        METADATA_CALLS.with(|calls| calls.set(0));
        let source_evidence =
            inspect_endpoint(&source, MappingKind::File, true, "mapping_add").unwrap();
        assert!(source_evidence.exists);
        assert_eq!(source_evidence.anchor, source_evidence.canonical);
        assert_eq!(METADATA_CALLS.with(|calls| calls.get()), 1);

        let destination = root.path().join("absent/child");
        METADATA_CALLS.with(|calls| calls.set(0));
        let destination_evidence =
            inspect_endpoint(&destination, MappingKind::File, false, "mapping_add").unwrap();
        assert!(!destination_evidence.exists);
        assert_eq!(
            destination_evidence.anchor,
            fs::canonicalize(root.path()).unwrap()
        );
        assert_eq!(METADATA_CALLS.with(|calls| calls.get()), 3);

        let real = root.path().join("real");
        fs::create_dir(&real).unwrap();
        let alias = root.path().join("alias");
        std::os::unix::fs::symlink(&real, &alias).unwrap();
        let aliased_destination = alias.join("absent/child");
        METADATA_CALLS.with(|calls| calls.set(0));
        let aliased_evidence = inspect_endpoint(
            &aliased_destination,
            MappingKind::File,
            false,
            "mapping_add",
        )
        .unwrap();
        assert_eq!(aliased_evidence.anchor, fs::canonicalize(&real).unwrap());
        assert_eq!(METADATA_CALLS.with(|calls| calls.get()), 4);
    }

    #[test]
    fn revalidate_detects_intermediate_symlink_retargeting() {
        let root = tempfile::tempdir().unwrap();
        let first = root.path().join("first");
        let second = root.path().join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        let alias = root.path().join("alias");
        std::os::unix::fs::symlink(&first, &alias).unwrap();
        let submitted = alias.join("absent");
        let evidence =
            inspect_endpoint(&submitted, MappingKind::File, false, "mapping_add").unwrap();

        fs::remove_file(&alias).unwrap();
        std::os::unix::fs::symlink(&second, &alias).unwrap();

        let error = revalidate(&evidence, "mapping_update").unwrap_err();
        assert!(matches!(
            error,
            GripError::Mapping { ref reason, .. } if reason == "stale_path_evidence"
        ));
    }
}
