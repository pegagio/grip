//! Source-side `.gripignore` loading and matching.

use crate::discovery::filesystem::{Directory, evidence};
use crate::discovery::model::{EvidenceSide, NodeKind, PolicyEvidence, SafePath};
use crate::error::GripError;
use ignore::Match;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

const POLICY_NAME: &[u8] = b".gripignore";
const OPERATION: &str = "mapping_inspect";

/// One compiled policy document scoped to its containing source directory.
#[derive(Clone, Debug)]
pub struct GripignorePolicy {
    matcher: Gitignore,
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
