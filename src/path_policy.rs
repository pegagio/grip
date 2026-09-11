//! Canonical path resolution for mapping endpoints and selectors.

use crate::error::GripError;
use crate::mapping::MappingKind;
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
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

    /// Parse a command-line source path and normalize it for portable storage.
    ///
    /// The command line accepts ordinary relative spellings such as `./app/`;
    /// the descriptor always retains their normalized project-relative form.
    pub fn parse_cli(value: &std::ffi::OsStr, allow_tree_root: bool) -> Result<Self, GripError> {
        let value = value.to_str().ok_or_else(|| {
            portable_path_error("non_utf8_path", value, "portable paths must be valid UTF-8")
        })?;
        let normalized = normalize_project_relative_input(value).ok_or_else(|| {
            portable_path_error(
                "invalid_project_relative_path",
                std::ffi::OsStr::new(value),
                "source must resolve to a project-relative path outside .grip",
            )
        })?;
        Self::parse(std::ffi::OsStr::new(&normalized), allow_tree_root)
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

fn normalize_project_relative_input(value: &str) -> Option<String> {
    if value.is_empty() || value.starts_with('/') {
        return None;
    }
    let mut components = Vec::new();
    for component in value.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop()?;
            }
            component => components.push(component),
        }
    }
    if components.is_empty() {
        Some(".".into())
    } else {
        Some(components.join("/"))
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
    /// Parse an absolute, home-relative, or project-relative destination declaration.
    pub fn parse(value: &std::ffi::OsStr) -> Result<Self, GripError> {
        let value = value.to_str().ok_or_else(|| {
            portable_path_error("non_utf8_path", value, "portable paths must be valid UTF-8")
        })?;
        if !value.is_empty()
            && !value.starts_with('$')
            && (!value.starts_with('~') || value == "~" || value.starts_with("~/"))
        {
            return Ok(Self(value.to_owned()));
        }
        Err(portable_path_error(
            "invalid_destination_path",
            std::ffi::OsStr::new(value),
            "destination must be an absolute path, ~, a ~/ path, or a project-relative path resolved from the selected project root",
        ))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn resolve(&self, project_root: &Path, home: &Path) -> PathBuf {
        let path = match self.0.strip_prefix("~/") {
            Some(suffix) => home.join(suffix),
            None if self.0 == "~" => home.to_path_buf(),
            None if Path::new(&self.0).is_absolute() => PathBuf::from(&self.0),
            None => project_root.join(&self.0),
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

/// Render an absolute source path relative to an invocation directory for human status output.
pub fn git_relative_display(path: &Path, invocation_directory: &Path) -> String {
    let relative = relative_path(path, invocation_directory);
    git_quote_path(relative.as_os_str().as_bytes())
}

/// Resolve a relative source selector from the invocation directory without leaving the project.
pub fn resolve_cwd_relative_source_selector(
    project_root: &Path,
    invocation_directory: &Path,
    selector: &std::ffi::OsStr,
    operation: &str,
) -> Result<PathBuf, GripError> {
    let selector_text = selector.to_str().ok_or_else(|| {
        portable_path_error(
            "non_utf8_path",
            selector,
            "portable paths must be valid UTF-8",
        )
    })?;
    if selector_text.is_empty()
        || Path::new(selector_text).is_absolute()
        || selector_text.starts_with('~')
        || selector_text.contains('$')
    {
        return Err(portable_path_error(
            "invalid_project_relative_path",
            selector,
            "source must be a relative path outside .grip",
        ));
    }

    let candidate = lexical_normalize_absolute(&invocation_directory.join(selector));
    let metadata_directory = project_root.join(".grip");
    if !candidate.starts_with(project_root) || candidate.starts_with(&metadata_directory) {
        return Err(source_selector_outside_project(operation, &candidate));
    }

    let existing_ancestor = canonical_existing_ancestor(&candidate, operation)?;
    if !existing_ancestor.starts_with(project_root) {
        return Err(source_selector_outside_project(operation, &candidate));
    }

    resolve_selector(&candidate, operation)
}

fn relative_path(path: &Path, invocation_directory: &Path) -> PathBuf {
    let path_components = path.components().collect::<Vec<_>>();
    let cwd_components = invocation_directory.components().collect::<Vec<_>>();
    let shared = path_components
        .iter()
        .zip(&cwd_components)
        .take_while(|(path_component, cwd_component)| path_component == cwd_component)
        .count();
    let mut relative = PathBuf::new();
    for component in &cwd_components[shared..] {
        if matches!(component, Component::Normal(_)) {
            relative.push("..");
        }
    }
    for component in &path_components[shared..] {
        if matches!(component, Component::Normal(_)) {
            relative.push(component.as_os_str());
        }
    }
    if relative.as_os_str().is_empty() {
        relative.push(".");
    }
    relative
}

fn git_quote_path(bytes: &[u8]) -> String {
    let needs_quotes = bytes
        .iter()
        .any(|byte| !matches!(byte, b'!'..=b'~') || matches!(byte, b'"' | b'\\'));
    if !needs_quotes {
        return String::from_utf8(bytes.to_vec()).expect("printable ASCII path is UTF-8");
    }

    let mut quoted = String::from("\"");
    for byte in bytes {
        match byte {
            b'\\' => quoted.push_str("\\\\"),
            b'"' => quoted.push_str("\\\""),
            b'\n' => quoted.push_str("\\n"),
            b'\r' => quoted.push_str("\\r"),
            b'\t' => quoted.push_str("\\t"),
            b'\x08' => quoted.push_str("\\b"),
            b'\x0c' => quoted.push_str("\\f"),
            b' '..=b'~' => quoted.push(char::from(*byte)),
            _ => quoted.push_str(&format!("\\{:03o}", byte)),
        }
    }
    quoted.push('"');
    quoted
}

fn canonical_existing_ancestor(path: &Path, operation: &str) -> Result<PathBuf, GripError> {
    let mut current = path;
    loop {
        match fs::canonicalize(current) {
            Ok(canonical) => return Ok(canonical),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                current = current.parent().ok_or_else(|| {
                    invalid(
                        operation,
                        "path_unavailable",
                        path,
                        "source selector has no existing ancestor",
                    )
                })?;
            }
            Err(error) => {
                return Err(invalid(
                    operation,
                    "path_unavailable",
                    path,
                    &format!("source selector cannot be resolved: {error}"),
                ));
            }
        }
    }
}

fn source_selector_outside_project(operation: &str, path: &Path) -> GripError {
    invalid(
        operation,
        "source_selector_outside_project",
        path,
        "source selector must remain inside the selected project",
    )
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
    fn git_relative_display_uses_parent_components_and_c_style_quotes() {
        let root = tempfile::tempdir().unwrap();
        let project = root.path().join("project");
        let app = project.join("app");
        let shared = project.join("shared");
        std::fs::create_dir_all(&app).unwrap();
        std::fs::create_dir_all(&shared).unwrap();

        assert_eq!(git_relative_display(&app.join("main.py"), &app), "main.py");
        assert_eq!(
            git_relative_display(&shared.join("config.yml"), &app),
            "../shared/config.yml"
        );
        assert_eq!(
            git_relative_display(&app.join("my file.py"), &app),
            "\"my file.py\""
        );
        assert_eq!(
            git_relative_display(&app.join("line\nbreak"), &app),
            "\"line\\nbreak\""
        );
    }

    #[test]
    fn cwd_relative_source_selector_rejects_project_escapes_before_selection() {
        let root = tempfile::tempdir().unwrap();
        let project = root.path().join("project");
        let app = project.join("app");
        let shared = project.join("shared");
        let outside = root.path().join("outside");
        std::fs::create_dir_all(&app).unwrap();
        std::fs::create_dir_all(&shared).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(shared.join("config.yml"), "config").unwrap();
        std::fs::write(outside.join("secret"), "secret").unwrap();
        let canonical_project = std::fs::canonicalize(&project).unwrap();
        let canonical_app = std::fs::canonicalize(&app).unwrap();

        assert_eq!(
            resolve_cwd_relative_source_selector(
                &canonical_project,
                &canonical_app,
                std::ffi::OsStr::new("../shared/config.yml"),
                "push",
            )
            .unwrap(),
            std::fs::canonicalize(shared.join("config.yml")).unwrap()
        );
        for selector in ["../../outside/secret", "/project/app/main.py"] {
            let error = resolve_cwd_relative_source_selector(
                &canonical_project,
                &canonical_app,
                std::ffi::OsStr::new(selector),
                "push",
            )
            .unwrap_err();
            assert!(matches!(error, GripError::Mapping { .. }));
        }

        std::os::unix::fs::symlink(&outside, app.join("escape")).unwrap();
        let error = resolve_cwd_relative_source_selector(
            &canonical_project,
            &canonical_app,
            std::ffi::OsStr::new("escape/secret"),
            "push",
        )
        .unwrap_err();
        assert!(matches!(
            error,
            GripError::Mapping { ref reason, .. } if reason == "source_selector_outside_project"
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
