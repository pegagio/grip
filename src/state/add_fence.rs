//! Bounded coordination state for an add publication spanning descriptor and State V4 files.

use crate::error::GripError;
use crate::mapping::{Mapping, MappingKind};
use crate::project::ProjectPaths;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

const FENCE_FILE: &str = "add-fence.json";
static TEMP_ID: AtomicU64 = AtomicU64::new(0);

/// Fault points used only to verify fenced-publication recovery behavior.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FenceFault {
    BeforeClear,
}

/// A single, exact add transition which must be completed or restored by a later `add`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddPublicationFenceV1 {
    pub schema_version: u8,
    pub mapping: FenceMapping,
    pub prior_descriptor_digest: String,
    pub candidate_descriptor_digest: String,
    pub prior_state_digest: Option<String>,
    pub candidate_state_digest: String,
    pub prior_descriptor_bytes: Vec<u8>,
    pub prior_state_bytes: Option<Vec<u8>>,
    pub candidate_state_bytes: Vec<u8>,
}

/// Canonical runtime identity for the one mapping protected by a fence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FenceMapping {
    pub kind: MappingKind,
    pub source: String,
    pub destination: String,
}

impl AddPublicationFenceV1 {
    pub fn new(
        mapping: &Mapping,
        prior_descriptor_bytes: Vec<u8>,
        candidate_descriptor_bytes: Vec<u8>,
        prior_state_bytes: Option<Vec<u8>>,
        candidate_state_bytes: Vec<u8>,
    ) -> Self {
        Self {
            schema_version: 1,
            mapping: FenceMapping::from(mapping),
            prior_descriptor_digest: digest(&prior_descriptor_bytes),
            candidate_descriptor_digest: digest(&candidate_descriptor_bytes),
            prior_state_digest: prior_state_bytes.as_deref().map(digest),
            candidate_state_digest: digest(&candidate_state_bytes),
            prior_descriptor_bytes,
            prior_state_bytes,
            candidate_state_bytes,
        }
    }

    pub fn protects(&self, mapping: &Mapping) -> bool {
        self.mapping == FenceMapping::from(mapping)
    }

    pub fn protects_resolved(&self, mapping: &crate::observation::model::ResolvedMapping) -> bool {
        self.mapping.kind == mapping.kind
            && self.mapping.source == mapping.source.to_string_lossy()
            && self.mapping.destination == mapping.destination.to_string_lossy()
    }

    /// Return whether one exact resolved selector identifies the fenced mapping endpoint.
    pub fn protects_selector(
        &self,
        selector: &Path,
        path_space: crate::observation::model::PathSpace,
    ) -> bool {
        let expected = match path_space {
            crate::observation::model::PathSpace::Source => &self.mapping.source,
            crate::observation::model::PathSpace::Destination => &self.mapping.destination,
        };
        selector == Path::new(expected)
    }

    /// Recover the canonical runtime mapping identity recorded by the fence.
    pub fn resolved_mapping(&self) -> crate::observation::model::ResolvedMapping {
        crate::observation::model::ResolvedMapping {
            kind: self.mapping.kind,
            source: self.mapping.source.clone().into(),
            destination: self.mapping.destination.clone().into(),
        }
    }
}

impl From<&Mapping> for FenceMapping {
    fn from(mapping: &Mapping) -> Self {
        Self {
            kind: mapping.kind,
            source: mapping.source.to_string_lossy().into_owned(),
            destination: mapping.destination.to_string_lossy().into_owned(),
        }
    }
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn load(home: &ProjectPaths) -> Result<Option<AddPublicationFenceV1>, GripError> {
    let path = home.path().join("state").join(FENCE_FILE);
    match read_private(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map(Some).map_err(|error| {
            GripError::CorruptState(format!("invalid add publication fence: {error}"))
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(GripError::from_io(
            "could not read add publication fence",
            error,
        )),
    }
}

/// Persist and reread a fence before the descriptor/state transition begins.
pub fn create_verified(
    home: &ProjectPaths,
    fence: &AddPublicationFenceV1,
) -> Result<(), GripError> {
    let directory = crate::state::publication::prepare_directory(home)?;
    let path = directory.join(FENCE_FILE);
    if let Some(existing) = load(home)? {
        return if existing == *fence {
            Ok(())
        } else {
            Err(GripError::InvalidConfiguration(
                "a different incomplete add publication is already fenced".into(),
            ))
        };
    }
    let bytes = serde_json::to_vec(fence).map_err(|error| {
        GripError::Internal(format!("could not encode add publication fence: {error}"))
    })?;
    let temporary = directory.join(format!(
        ".add-fence.tmp-{}-{}",
        std::process::id(),
        TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)
            .map_err(|error| GripError::from_io("could not stage add publication fence", error))?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|error| {
                GripError::from_io("could not sync staged add publication fence", error)
            })?;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|error| {
                GripError::from_io("could not secure staged add publication fence", error)
            })?;
        let reread = read_private(&temporary).map_err(|error| {
            GripError::from_io("could not verify staged add publication fence", error)
        })?;
        if reread != bytes {
            return Err(GripError::CorruptState(
                "staged add publication fence changed before publication".into(),
            ));
        }
        fs::rename(&temporary, &path).map_err(|error| {
            GripError::from_io("could not publish add publication fence", error)
        })?;
        File::open(&directory)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| {
                GripError::from_io("could not sync add publication fence directory", error)
            })?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result?;
    if load(home)?.as_ref() != Some(fence) {
        return Err(GripError::CorruptState(
            "add publication fence verification failed".into(),
        ));
    }
    Ok(())
}

/// Remove a fence only after the caller has verified the exact candidate transition.
pub fn clear_verified(
    home: &ProjectPaths,
    expected: &AddPublicationFenceV1,
) -> Result<(), GripError> {
    clear_verified_with_fault(home, expected, None)
}

/// Remove a verified fence, optionally stopping immediately before removal for fault tests.
#[doc(hidden)]
pub fn clear_verified_with_fault(
    home: &ProjectPaths,
    expected: &AddPublicationFenceV1,
    fault: Option<FenceFault>,
) -> Result<(), GripError> {
    if load(home)?.as_ref() != Some(expected) {
        return Err(GripError::InvalidConfiguration(
            "add publication fence changed before completion".into(),
        ));
    }
    if fault == Some(FenceFault::BeforeClear) {
        return Err(GripError::Internal(
            "injected pre-clear add publication fence failure".into(),
        ));
    }
    let directory = home.path().join("state");
    let path = directory.join(FENCE_FILE);
    fs::remove_file(&path)
        .map_err(|error| GripError::from_io("could not clear add publication fence", error))?;
    File::open(&directory)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            GripError::from_io("could not sync cleared add publication fence", error)
        })?;
    if load(home)?.is_some() {
        return Err(GripError::CorruptState(
            "add publication fence remained after completion".into(),
        ));
    }
    Ok(())
}

fn read_private(path: &Path) -> std::io::Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.permissions().mode() & 0o7777 != 0o600
    {
        return Err(std::io::Error::other("unsafe add publication fence node"));
    }
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}
