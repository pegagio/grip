//! Read-only recovery inventory adapters.

use crate::error::GripError;
use crate::home::GripHome;
use crate::recovery::model::{
    RecoveryAvailability, RecoveryEntry, RecoveryIntegrity, RecoveryKind,
    RecoveryManifestPayloadV1, RecoveryRef,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub(crate) struct LocatedRecovery {
    pub directory: std::path::PathBuf,
    pub payload: RecoveryManifestPayloadV1,
    pub payload_path: std::path::PathBuf,
}

pub(crate) fn locate_manifest(
    home: &GripHome,
    reference: &RecoveryRef,
) -> Result<LocatedRecovery, GripError> {
    let directory = match reference {
        RecoveryRef::Payload {
            operation_id,
            action_index,
        } => home
            .path()
            .join("state/operations")
            .join(operation_id)
            .join("recovery")
            .join(format!("{action_index:08}")),
        RecoveryRef::Registry { digest } => home
            .path()
            .join("state/recovery/registry")
            .join(format!("sha256-{digest}")),
        RecoveryRef::AcceptedState { generation, .. } => home
            .path()
            .join("state/recovery")
            .join(format!("generation-{generation}")),
        RecoveryRef::Operation { .. } => {
            return Err(GripError::InvalidConfiguration(
                "operation recovery references do not identify recoverable bytes".into(),
            ));
        }
    };
    let bytes = crate::mutation::filesystem::read_private_file(&directory.join("manifest.json"))?;
    let envelope = crate::recovery::model::decode::<RecoveryManifestPayloadV1>(&bytes)?;
    if &envelope.payload.reference != reference {
        return Err(GripError::CorruptState(
            "recovery manifest does not match its storage identity".into(),
        ));
    }
    let payload_path = directory.join(&envelope.payload.payload_ref);
    Ok(LocatedRecovery {
        directory,
        payload: envelope.payload,
        payload_path,
    })
}

/// Enumerate the three recovery byte stores plus durable operation evidence.
pub fn list(home: &GripHome) -> Result<Vec<RecoveryEntry>, GripError> {
    let mut entries = Vec::new();
    operation_entries(&home.path().join("state/operations"), &mut entries)?;
    registry_entries(&home.path().join("state/recovery/registry"), &mut entries)?;
    state_entries(&home.path().join("state/recovery"), &mut entries)?;
    entries.sort_by(|first, second| first.reference.cmp(&second.reference));
    Ok(entries)
}

/// Resolve exactly one public recovery identity.
pub fn show(home: &GripHome, reference: &RecoveryRef) -> Result<RecoveryEntry, GripError> {
    let entry = match reference {
        RecoveryRef::Payload {
            operation_id,
            action_index,
        } => {
            let directory = home
                .path()
                .join("state/operations")
                .join(operation_id)
                .join("recovery")
                .join(format!("{action_index:08}"));
            if !directory.exists() {
                None
            } else {
                Some(read_payload_entry(&directory, operation_id, *action_index)?)
            }
        }
        RecoveryRef::Registry { digest } => {
            let directory = home
                .path()
                .join("state/recovery/registry")
                .join(format!("sha256-{digest}"));
            if !directory.exists() {
                None
            } else if directory.join("manifest.json").exists() {
                Some(manifest_entry(&directory, reference.clone())?)
            } else {
                Some(legacy_authority_entry(
                    &directory.join("config.toml"),
                    reference.clone(),
                    RecoveryKind::Registry,
                )?)
            }
        }
        RecoveryRef::AcceptedState { generation, .. } => {
            let directory = home
                .path()
                .join("state/recovery")
                .join(format!("generation-{generation}"));
            if !directory.exists() {
                None
            } else if directory.join("manifest.json").exists() {
                Some(manifest_entry(&directory, reference.clone())?)
            } else {
                let path = directory.join("state.json");
                let bytes = crate::mutation::filesystem::read_private_file(&path)?;
                let digest = format!("{:x}", Sha256::digest(&bytes));
                let actual = RecoveryRef::AcceptedState {
                    generation: *generation,
                    digest,
                };
                if &actual != reference {
                    return Err(GripError::CorruptState(
                        "legacy state recovery does not match the selected reference".into(),
                    ));
                }
                Some(legacy_authority_entry(
                    &path,
                    actual,
                    RecoveryKind::AcceptedState,
                )?)
            }
        }
        RecoveryRef::Operation { operation_id } => {
            let directory = home.path().join("state/operations").join(operation_id);
            if !directory.exists() {
                None
            } else {
                Some(operation_entry(&directory, operation_id))
            }
        }
    };
    entry.ok_or_else(|| {
        GripError::lifecycle(
            "recovery_show",
            "recovery_not_found",
            crate::error::ResultCategory::InvalidConfiguration,
            "recovery reference was not found",
        )
    })
}

fn operation_entries(root: &Path, result: &mut Vec<RecoveryEntry>) -> Result<(), GripError> {
    for operation in read_dirs(root)? {
        let operation_id = file_name(&operation)?;
        if !operation_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err(GripError::CorruptState(
                "unsafe operation recovery identity".into(),
            ));
        }
        result.push(operation_entry(&operation, &operation_id));
        for action in read_dirs(&operation.join("recovery"))? {
            let action_name = file_name(&action)?;
            let action_index = action_name.parse::<usize>().map_err(|_| {
                GripError::CorruptState("invalid payload recovery action identity".into())
            })?;
            result.push(read_payload_entry(&action, &operation_id, action_index)?);
        }
    }
    Ok(())
}

fn operation_entry(directory: &Path, operation_id: &str) -> RecoveryEntry {
    let reference = RecoveryRef::Operation {
        operation_id: operation_id.into(),
    };
    let summary = crate::mutation::filesystem::read_private_file(&directory.join("operation.json"));
    let plan = crate::mutation::filesystem::read_private_file(&directory.join("plan.json"));
    let (integrity, availability, origin, byte_count, provenance) = match (summary, plan) {
        (Ok(summary_bytes), Ok(plan_bytes)) => {
            let decoded_summary = crate::operation::model::decode::<
                crate::operation::model::OperationSummaryPayloadV1,
            >(&summary_bytes);
            let decoded_plan = crate::operation::model::decode::<
                crate::operation::model::OperationPlanPayloadV1,
            >(&plan_bytes);
            match (decoded_summary, decoded_plan) {
                (Ok(summary), Ok(plan))
                    if summary.payload.operation_id == operation_id
                        && plan.payload.operation_id == operation_id
                        && summary.payload.operation == plan.payload.operation
                        && summary.payload.plan_id == plan.payload.plan_id
                        && summary.payload.plan_ref == "plan.json" =>
                {
                    (
                        RecoveryIntegrity::Verified,
                        RecoveryAvailability::Available,
                        plan.payload.operation,
                        Some((summary_bytes.len() + plan_bytes.len()) as u64),
                        "complete",
                    )
                }
                _ => (
                    RecoveryIntegrity::Failed,
                    RecoveryAvailability::Available,
                    "unverified_operation".into(),
                    Some((summary_bytes.len() + plan_bytes.len()) as u64),
                    "incomplete",
                ),
            }
        }
        (Ok(bytes), Err(_)) | (Err(_), Ok(bytes)) => (
            RecoveryIntegrity::Failed,
            RecoveryAvailability::Missing,
            "incomplete_operation".into(),
            Some(bytes.len() as u64),
            "incomplete",
        ),
        (Err(_), Err(_)) => (
            RecoveryIntegrity::Unavailable,
            RecoveryAvailability::Missing,
            "missing_operation".into(),
            None,
            "incomplete",
        ),
    };
    RecoveryEntry {
        reference,
        kind: RecoveryKind::Operation,
        origin,
        managed_identity: None,
        bound_side: None,
        bound_path: None,
        created_at: if integrity == RecoveryIntegrity::Verified {
            operation_created_at(operation_id)
        } else {
            None
        },
        integrity,
        availability,
        byte_count,
        provenance: provenance.into(),
        restore_eligibility: "not_restorable".into(),
    }
}

fn operation_created_at(operation_id: &str) -> Option<String> {
    operation_id
        .rsplit('-')
        .nth(2)
        .and_then(|seconds| seconds.parse::<u64>().ok())
        .map(|seconds| format!("{seconds}Z"))
}

fn read_payload_entry(
    directory: &Path,
    operation_id: &str,
    action_index: usize,
) -> Result<RecoveryEntry, GripError> {
    let reference = RecoveryRef::Payload {
        operation_id: operation_id.into(),
        action_index,
    };
    if directory.join("manifest.json").exists() {
        return manifest_entry(directory, reference);
    }
    let bytes = crate::mutation::filesystem::read_private_file(&directory.join("metadata.json"))?;
    let metadata: crate::mutation::recovery::RecoveryMetadataV1 = serde_json::from_slice(&bytes)
        .map_err(|error| {
            GripError::CorruptState(format!("invalid legacy recovery metadata: {error}"))
        })?;
    if metadata.operation_id != operation_id || metadata.action_index != action_index {
        return Err(GripError::CorruptState(
            "legacy recovery layout does not match metadata".into(),
        ));
    }
    let available = directory.join("payload").is_file();
    Ok(RecoveryEntry {
        reference,
        kind: RecoveryKind::Payload,
        origin: operation_id.into(),
        managed_identity: Some(format!(
            "{}:{}",
            metadata.identity.mapping.source.display(),
            metadata.identity.relative_path_hex
        )),
        bound_side: None,
        bound_path: None,
        created_at: None,
        integrity: if metadata.verified {
            RecoveryIntegrity::Verified
        } else {
            RecoveryIntegrity::Failed
        },
        availability: if available {
            RecoveryAvailability::Available
        } else {
            RecoveryAvailability::Missing
        },
        byte_count: if available {
            Some(
                fs::metadata(directory.join("payload"))
                    .map_err(|error| {
                        GripError::from_io("could not inspect recovery payload", error)
                    })?
                    .len(),
            )
        } else {
            None
        },
        provenance: "legacy_incomplete".into(),
        restore_eligibility: "legacy_provenance_incomplete".into(),
    })
}

fn registry_entries(root: &Path, result: &mut Vec<RecoveryEntry>) -> Result<(), GripError> {
    for directory in read_dirs(root)? {
        let name = file_name(&directory)?;
        let digest = name
            .strip_prefix("sha256-")
            .ok_or_else(|| GripError::CorruptState("invalid registry recovery identity".into()))?;
        let reference: RecoveryRef = format!("registry:sha256:{digest}")
            .parse()
            .map_err(GripError::CorruptState)?;
        result.push(if directory.join("manifest.json").exists() {
            manifest_entry(&directory, reference)?
        } else {
            legacy_authority_entry(
                &directory.join("config.toml"),
                reference,
                RecoveryKind::Registry,
            )?
        });
    }
    Ok(())
}

fn state_entries(root: &Path, result: &mut Vec<RecoveryEntry>) -> Result<(), GripError> {
    for directory in read_dirs(root)? {
        let name = file_name(&directory)?;
        let Some(generation) = name.strip_prefix("generation-") else {
            continue;
        };
        let generation = generation
            .parse::<u64>()
            .map_err(|_| GripError::CorruptState("invalid state recovery generation".into()))?;
        let path = directory.join("state.json");
        if directory.join("manifest.json").exists() {
            let manifest_bytes =
                crate::mutation::filesystem::read_private_file(&directory.join("manifest.json"))?;
            let manifest =
                crate::recovery::model::decode::<RecoveryManifestPayloadV1>(&manifest_bytes)?;
            let RecoveryRef::AcceptedState {
                generation: bound_generation,
                digest,
            } = manifest.payload.reference
            else {
                return Err(GripError::CorruptState(
                    "state recovery manifest has the wrong kind".into(),
                ));
            };
            if bound_generation != generation {
                return Err(GripError::CorruptState(
                    "state recovery layout does not match its manifest".into(),
                ));
            }
            result.push(manifest_entry(
                &directory,
                RecoveryRef::AcceptedState { generation, digest },
            )?);
            continue;
        }
        let bytes = crate::mutation::filesystem::read_private_file(&path)?;
        let digest = format!("{:x}", Sha256::digest(&bytes));
        let reference = RecoveryRef::AcceptedState { generation, digest };
        result.push(legacy_authority_entry(
            &path,
            reference,
            RecoveryKind::AcceptedState,
        )?);
    }
    Ok(())
}

fn manifest_entry(directory: &Path, expected: RecoveryRef) -> Result<RecoveryEntry, GripError> {
    let bytes = crate::mutation::filesystem::read_private_file(&directory.join("manifest.json"))?;
    let envelope = crate::recovery::model::decode::<RecoveryManifestPayloadV1>(&bytes)?;
    if envelope.payload.reference != expected {
        return Err(GripError::CorruptState(
            "recovery manifest does not match its storage identity".into(),
        ));
    }
    let payload_path = directory.join(&envelope.payload.payload_ref);
    let tombstone = directory.join("cleaned.json");
    let (availability, integrity) = if payload_path.exists() && tombstone.exists() {
        (
            RecoveryAvailability::CleanupIncomplete,
            RecoveryIntegrity::Failed,
        )
    } else if payload_path.exists() {
        let payload_bytes = crate::mutation::filesystem::read_private_file(&payload_path)?;
        let digest = format!("{:x}", Sha256::digest(&payload_bytes));
        let valid = match &expected {
            RecoveryRef::Payload { .. } => envelope
                .payload
                .prior_evidence
                .get("content")
                .and_then(|content| content.get("digest"))
                .and_then(serde_json::Value::as_str)
                .is_none_or(|expected| expected == digest),
            RecoveryRef::Registry { digest: expected } => expected == &digest,
            RecoveryRef::AcceptedState {
                digest: expected, ..
            } => expected == &digest,
            RecoveryRef::Operation { .. } => false,
        } && payload_bytes.len() as u64 == envelope.payload.byte_count;
        (
            RecoveryAvailability::Available,
            if valid {
                RecoveryIntegrity::Verified
            } else {
                RecoveryIntegrity::Failed
            },
        )
    } else if tombstone.exists() {
        let tombstone_bytes = crate::mutation::filesystem::read_private_file(&tombstone)?;
        let tombstone = crate::recovery::model::decode::<
            crate::recovery::model::CleanupTombstonePayloadV1,
        >(&tombstone_bytes)?;
        if tombstone.payload.reference != expected {
            return Err(GripError::CorruptState(
                "cleanup tombstone does not match recovery identity".into(),
            ));
        }
        (RecoveryAvailability::Cleaned, RecoveryIntegrity::Verified)
    } else if envelope.payload.byte_count == 0 {
        (RecoveryAvailability::Available, RecoveryIntegrity::Verified)
    } else {
        (
            RecoveryAvailability::CleanupIncomplete,
            RecoveryIntegrity::Failed,
        )
    };
    Ok(RecoveryEntry {
        reference: expected,
        kind: envelope.payload.kind,
        origin: envelope.payload.origin_transition,
        managed_identity: envelope.payload.managed_identity,
        bound_side: envelope.payload.bound_side,
        bound_path: envelope.payload.bound_target,
        created_at: Some(envelope.payload.created_at),
        integrity,
        availability,
        byte_count: Some(envelope.payload.byte_count),
        provenance: "complete".into(),
        restore_eligibility: if availability == RecoveryAvailability::Available
            && integrity == RecoveryIntegrity::Verified
        {
            "eligible"
        } else {
            "unavailable"
        }
        .into(),
    })
}

fn legacy_authority_entry(
    path: &Path,
    reference: RecoveryRef,
    kind: RecoveryKind,
) -> Result<RecoveryEntry, GripError> {
    let bytes = crate::mutation::filesystem::read_private_file(path)?;
    Ok(RecoveryEntry {
        reference,
        kind,
        origin: "legacy_authority_publication".into(),
        managed_identity: None,
        bound_side: None,
        bound_path: None,
        created_at: None,
        integrity: RecoveryIntegrity::Verified,
        availability: RecoveryAvailability::Available,
        byte_count: Some(bytes.len() as u64),
        provenance: "legacy_incomplete".into(),
        restore_eligibility: "legacy_provenance_incomplete".into(),
    })
}

fn read_dirs(root: &Path) -> Result<Vec<std::path::PathBuf>, GripError> {
    let read = match fs::read_dir(root) {
        Ok(read) => read,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(GripError::from_io(
                "could not enumerate recovery evidence",
                error,
            ));
        }
    };
    let mut entries = Vec::new();
    for entry in read {
        let entry = entry
            .map_err(|error| GripError::from_io("could not enumerate recovery evidence", error))?;
        let metadata = fs::symlink_metadata(entry.path())
            .map_err(|error| GripError::from_io("could not inspect recovery evidence", error))?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(GripError::CorruptState(
                "recovery store contains an unsafe node".into(),
            ));
        }
        entries.push(entry.path());
    }
    entries.sort();
    Ok(entries)
}

fn file_name(path: &Path) -> Result<String, GripError> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .ok_or_else(|| GripError::CorruptState("recovery identity is not valid UTF-8".into()))
}
