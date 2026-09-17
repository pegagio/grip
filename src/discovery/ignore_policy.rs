//! Source-side `.gripignore` loading and matching.

use crate::discovery::filesystem::{Directory, evidence};
use crate::discovery::model::{EvidenceSide, NodeKind, PolicyEvidence, SafePath};
use crate::error::GripError;
use ignore::Match;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

const POLICY_NAME: &[u8] = b".gripignore";
const OPERATION: &str = "mapping_inspect";

/// Pruned directory prefixes that cover retained identities without opening their descendants.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RetainedIgnoredPrefixes(Vec<Vec<u8>>);

impl RetainedIgnoredPrefixes {
    pub fn new(mut prefixes: Vec<Vec<u8>>) -> Self {
        prefixes.sort();
        prefixes.dedup();
        Self(prefixes)
    }

    pub fn covers(&self, relative: &[u8]) -> bool {
        self.0.iter().any(|prefix| {
            relative == prefix
                || relative
                    .strip_prefix(prefix.as_slice())
                    .is_some_and(|suffix| suffix.first() == Some(&b'/'))
        })
    }
}

/// One compiled policy document scoped to its containing source directory.
#[derive(Clone, Debug)]
pub struct GripignorePolicy {
    matcher: Gitignore,
}

/// The effective ignore decision for one prospective source-side adoption path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdoptionIgnoreDecision {
    pub ignored: bool,
    pub recommendation_path: PathBuf,
    pub recommendation_rules: Vec<String>,
    pub policy_digest: String,
}

impl GripignorePolicy {
    /// Return the decision made by this policy for an absolute descendant path.
    pub fn decision(&self, path: &Path, is_directory: bool) -> Option<bool> {
        match self.matcher.matched(path, is_directory) {
            Match::Ignore(_) => Some(true),
            Match::Whitelist(_) => Some(false),
            Match::None => None,
        }
    }
}

/// Load and compile the exact policy entry in an already-open source directory.
pub fn load(
    directory: &Directory,
    mapping_source: &Path,
    directory_path: &Path,
    directory_relative: &[u8],
    root_device: u64,
) -> Result<(GripignorePolicy, Vec<u8>, PolicyEvidence), GripError> {
    let policy_path = directory_path.join(".gripignore");
    let metadata = directory
        .metadata(POLICY_NAME)
        .map_err(|error| operational(&policy_path, "unreadable_policy", error))?;
    if metadata.classify(root_device) != NodeKind::File {
        return Err(invalid(
            &policy_path,
            "invalid_policy",
            "Gripignore policy must be an ordinary regular file",
        ));
    }
    let descriptor = directory
        .open_child_file(POLICY_NAME)
        .map_err(|error| operational(&policy_path, "unreadable_policy", error))?;
    let mut file = File::from(descriptor);
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| operational(&policy_path, "unreadable_policy", error))?;
    let text = std::str::from_utf8(&bytes).map_err(|_| {
        invalid(
            &policy_path,
            "non_utf8_policy",
            "Gripignore policy must be valid UTF-8",
        )
    })?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut builder = GitignoreBuilder::new(directory_path);
    for line in text.lines() {
        builder
            .add_line(Some(policy_path.clone()), line)
            .map_err(|error| {
                invalid(
                    &policy_path,
                    "invalid_policy",
                    &format!("Gripignore policy is invalid: {error}"),
                )
            })?;
    }
    let matcher = builder.build().map_err(|error| {
        invalid(
            &policy_path,
            "invalid_policy",
            &format!("Gripignore policy is invalid: {error}"),
        )
    })?;
    let relative = if directory_relative.is_empty() {
        POLICY_NAME.to_vec()
    } else {
        let mut path = directory_relative.to_vec();
        path.push(b'/');
        path.extend_from_slice(POLICY_NAME);
        path
    };
    let node = evidence(
        &metadata,
        EvidenceSide::Source,
        mapping_source,
        &relative,
        None,
    );
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    Ok((
        GripignorePolicy { matcher },
        policy_path.as_os_str().as_bytes().to_vec(),
        PolicyEvidence {
            bytes,
            digest,
            node,
        },
    ))
}

/// Evaluate the policy stack that would govern a source path introduced by adoption.
///
/// This deliberately walks only the existing source ancestor chain. It never discovers a
/// destination tree or treats an unselected destination sibling as a managed member.
pub fn adoption_decision(
    project_root: &Path,
    mapping_source: &Path,
    target: &Path,
) -> Result<AdoptionIgnoreDecision, GripError> {
    let relative = target.strip_prefix(mapping_source).map_err(|_| {
        GripError::InvalidConfiguration("adoption target is outside its source mapping".into())
    })?;
    let components = relative.components().collect::<Vec<_>>();
    if components.is_empty() {
        return Err(GripError::InvalidConfiguration(
            "adoption target must be below its source mapping".into(),
        ));
    }

    let mut policies = Vec::new();
    let project_policy = project_root.join(".gripignore");
    if project_policy.exists() {
        policies.push(read_policy(&project_policy, project_root, &[])?);
    }

    let mut placement = mapping_source.to_path_buf();
    let mut cursor = mapping_source.to_path_buf();
    for component in components.iter().take(components.len().saturating_sub(1)) {
        let component = component.as_os_str();
        cursor.push(component);
        match fs::symlink_metadata(&cursor) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                placement = cursor.clone();
                let policy = cursor.join(".gripignore");
                if policy.exists() {
                    policies.push(read_policy(&policy, &cursor, &[])?);
                }
            }
            Ok(_) | Err(_) => break,
        }
    }
    let root_policy = mapping_source.join(".gripignore");
    if root_policy.exists() && placement != mapping_source {
        policies.insert(
            usize::from(project_policy.exists()),
            read_policy(&root_policy, mapping_source, &[])?,
        );
    } else if root_policy.exists() {
        policies.push(read_policy(&root_policy, mapping_source, &[])?);
    }

    let ignored = policies
        .iter()
        .rev()
        .find_map(|policy| policy.decision(target, false))
        .unwrap_or(false);
    let target_from_placement = target.strip_prefix(&placement).map_err(|_| {
        GripError::InvalidConfiguration("could not determine adoption policy placement".into())
    })?;
    let mut recommendation_rules = Vec::new();
    let mut prefix = PathBuf::new();
    for (index, component) in target_from_placement.components().enumerate() {
        prefix.push(component.as_os_str());
        let mut rule = format!("!{}", prefix.to_string_lossy());
        if index + 1 < target_from_placement.components().count() {
            rule.push('/');
        }
        recommendation_rules.push(rule);
    }

    if ignored {
        let virtual_policy = read_policy(
            &placement.join(".gripignore"),
            &placement,
            &recommendation_rules,
        )?;
        policies.push(virtual_policy);
        let mut prefix = placement.clone();
        for (index, component) in target_from_placement.components().enumerate() {
            prefix.push(component.as_os_str());
            let is_directory = index + 1 < target_from_placement.components().count();
            if policies
                .iter()
                .rev()
                .find_map(|policy| policy.decision(&prefix, is_directory))
                != Some(false)
            {
                return Err(GripError::InvalidConfiguration(
                    "could not produce a verified .gripignore adoption recommendation".into(),
                ));
            }
        }
    }

    Ok(AdoptionIgnoreDecision {
        ignored,
        recommendation_path: placement.join(".gripignore"),
        recommendation_rules,
        policy_digest: adoption_policy_digest(project_root, mapping_source, target)?,
    })
}

/// Return stable content evidence for every policy document governing an adoption target.
pub fn adoption_policy_digest(
    project_root: &Path,
    mapping_source: &Path,
    target: &Path,
) -> Result<String, GripError> {
    let mut digest = Sha256::new();
    let mut directories = vec![project_root.to_path_buf(), mapping_source.to_path_buf()];
    let mut cursor = mapping_source.to_path_buf();
    if let Ok(relative) = target.strip_prefix(mapping_source) {
        let components = relative.components().collect::<Vec<_>>();
        for component in components.iter().take(components.len().saturating_sub(1)) {
            cursor.push(component.as_os_str());
            directories.push(cursor.clone());
        }
    }
    directories.sort();
    directories.dedup();
    for directory in directories {
        let policy = directory.join(".gripignore");
        digest.update(policy.as_os_str().as_bytes());
        match fs::symlink_metadata(&policy) {
            Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
                digest.update([1]);
                digest.update(
                    fs::read(&policy)
                        .map_err(|error| operational(&policy, "unreadable_policy", error))?,
                );
            }
            Ok(_) => {
                return Err(invalid(
                    &policy,
                    "invalid_policy",
                    "Gripignore policy must be an ordinary regular file",
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => digest.update([0]),
            Err(error) => return Err(operational(&policy, "unreadable_policy", error)),
        }
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn read_policy(
    policy_path: &Path,
    base: &Path,
    appended_lines: &[String],
) -> Result<GripignorePolicy, GripError> {
    let mut builder = GitignoreBuilder::new(base);
    if policy_path.exists() {
        let metadata = fs::symlink_metadata(policy_path)
            .map_err(|error| operational(policy_path, "unreadable_policy", error))?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(invalid(
                policy_path,
                "invalid_policy",
                "Gripignore policy must be an ordinary regular file",
            ));
        }
        let text = fs::read_to_string(policy_path)
            .map_err(|error| operational(policy_path, "unreadable_policy", error))?;
        for line in text.strip_prefix('\u{feff}').unwrap_or(&text).lines() {
            builder
                .add_line(Some(policy_path.to_path_buf()), line)
                .map_err(|error| {
                    invalid(
                        policy_path,
                        "invalid_policy",
                        &format!("Gripignore policy is invalid: {error}"),
                    )
                })?;
        }
    }
    for line in appended_lines {
        builder
            .add_line(Some(policy_path.to_path_buf()), line)
            .map_err(|error| {
                invalid(
                    policy_path,
                    "invalid_policy",
                    &format!("Gripignore policy is invalid: {error}"),
                )
            })?;
    }
    let matcher = builder.build().map_err(|error| {
        invalid(
            policy_path,
            "invalid_policy",
            &format!("Gripignore policy is invalid: {error}"),
        )
    })?;
    Ok(GripignorePolicy { matcher })
}

fn invalid(path: &Path, reason: &str, message: &str) -> GripError {
    GripError::discovery_invalid(
        OPERATION,
        reason,
        vec![SafePath::from_path(path).display],
        message,
    )
}

fn operational(path: &Path, reason: &str, error: std::io::Error) -> GripError {
    GripError::discovery_operational(
        OPERATION,
        reason,
        vec![SafePath::from_path(path).display],
        &format!("Could not read Gripignore policy: {error}"),
    )
}
