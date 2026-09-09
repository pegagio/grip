//! Read-only recovery inventory adapters.

use crate::error::GripError;
use crate::project::ProjectPaths;
use crate::recovery::model::{
    RecoveryAvailability, RecoveryEntry, RecoveryIntegrity, RecoveryKind,
    RecoveryManifestPayloadV2, RecoveryRef,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub(crate) struct LocatedRecovery {
    pub directory: std::path::PathBuf,
    pub payload: RecoveryManifestPayloadV2,
    pub payload_path: std::path::PathBuf,
}

pub(crate) fn locate_manifest(
    home: &ProjectPaths,
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
    let envelope = crate::recovery::model::decode_manifest_v2(&bytes)?;
    if &envelope.payload.reference != reference {
        return Err(GripError::CorruptState(
            "recovery manifest does not match its storage identity".into(),
        ));
    }
    let payload_path = directory.join(&envelope.payload.private_ref);
    Ok(LocatedRecovery {
        directory,
        payload: envelope.payload,
        payload_path,
    })
}

/// Enumerate the three recovery byte stores plus durable operation evidence.
pub fn list(home: &ProjectPaths) -> Result<Vec<RecoveryEntry>, GripError> {
    let mut entries = Vec::new();
    operation_entries(home, &home.path().join("state/operations"), &mut entries)?;
    registry_entries(
        home,
        &home.path().join("state/recovery/registry"),
        &mut entries,
    )?;
    state_entries(home, &home.path().join("state/recovery"), &mut entries)?;
    entries.sort_by(|first, second| first.reference.cmp(&second.reference));
    Ok(entries)
}

/// Resolve exactly one public recovery identity.
pub fn show(home: &ProjectPaths, reference: &RecoveryRef) -> Result<RecoveryEntry, GripError> {
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
                Some(read_payload_entry(
                    home,
                    &directory,
                    operation_id,
                    *action_index,
                )?)
            }
        }
        RecoveryRef::Registry { digest } => {
            let directory = home
                .path()
                .join("state/recovery/registry")
                .join(format!("sha256-{digest}"));
            if !directory.exists() {
                None
            } else {
                Some(manifest_entry(home, &directory, reference.clone())?)
            }
        }
        RecoveryRef::AcceptedState { generation, .. } => {
            let directory = home
                .path()
                .join("state/recovery")
                .join(format!("generation-{generation}"));
            if !directory.exists() {
                None
            } else {
                Some(manifest_entry(home, &directory, reference.clone())?)
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

fn operation_entries(
    home: &ProjectPaths,
    root: &Path,
    result: &mut Vec<RecoveryEntry>,
) -> Result<(), GripError> {
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
            result.push(read_payload_entry(
                home,
                &action,
                &operation_id,
                action_index,
            )?);
        }
    }
    Ok(())
}

fn operation_entry(directory: &Path, operation_id: &str) -> RecoveryEntry {
    let reference = RecoveryRef::Operation {
        operation_id: operation_id.into(),
    };
    let summary = crate::mutation::filesystem::read_private_file(&directory.join("operation.json"));
    let record = crate::mutation::filesystem::read_private_file(&directory.join("record.json"));
    let (integrity, availability, origin, byte_count, provenance) = match (summary, record) {
        (Ok(summary_bytes), Ok(record_bytes)) => {
            let decoded_summary = crate::operation::model::decode::<
                crate::operation::model::OperationSummaryPayloadV2,
            >(&summary_bytes);
            let decoded_record = crate::operation::model::decode_operation_v2(&record_bytes);
            match (decoded_summary, decoded_record) {
                (Ok(summary), Ok(record))
                    if summary.payload.operation_id == operation_id
                        && record.payload.operation_id == operation_id
                        && summary.payload.operation == record.payload.operation
                        && summary.payload.plan_id == record.payload.plan_id
                        && summary.payload.plan_ref == "record.json" =>
                {
                    (
                        RecoveryIntegrity::Verified,
                        RecoveryAvailability::Available,
                        record.payload.operation,
                        Some((summary_bytes.len() + record_bytes.len()) as u64),
                        "complete",
                    )
                }
                _ => (
                    RecoveryIntegrity::Failed,
                    RecoveryAvailability::Available,
                    "unverified_operation".into(),
                    Some((summary_bytes.len() + record_bytes.len()) as u64),
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
    home: &ProjectPaths,
    directory: &Path,
    operation_id: &str,
    action_index: usize,
) -> Result<RecoveryEntry, GripError> {
    let reference = RecoveryRef::Payload {
        operation_id: operation_id.into(),
        action_index,
    };
    manifest_entry(home, directory, reference)
}

fn registry_entries(
    home: &ProjectPaths,
    root: &Path,
    result: &mut Vec<RecoveryEntry>,
) -> Result<(), GripError> {
    for directory in read_dirs(root)? {
        let name = file_name(&directory)?;
        let digest = name
            .strip_prefix("sha256-")
            .ok_or_else(|| GripError::CorruptState("invalid registry recovery identity".into()))?;
        let reference: RecoveryRef = format!("registry:sha256:{digest}")
            .parse()
            .map_err(GripError::CorruptState)?;
        result.push(manifest_entry(home, &directory, reference)?);
    }
    Ok(())
}

fn state_entries(
    home: &ProjectPaths,
    root: &Path,
    result: &mut Vec<RecoveryEntry>,
) -> Result<(), GripError> {
    for directory in read_dirs(root)? {
        let name = file_name(&directory)?;
        let Some(generation) = name.strip_prefix("generation-") else {
            continue;
        };
        let generation = generation
            .parse::<u64>()
            .map_err(|_| GripError::CorruptState("invalid state recovery generation".into()))?;
        let manifest_bytes =
            crate::mutation::filesystem::read_private_file(&directory.join("manifest.json"))?;
        let manifest = crate::recovery::model::decode_manifest_v2(&manifest_bytes)?;
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
            home,
            &directory,
            RecoveryRef::AcceptedState { generation, digest },
        )?);
    }
    Ok(())
}

fn manifest_entry(
    home: &ProjectPaths,
    directory: &Path,
    expected: RecoveryRef,
) -> Result<RecoveryEntry, GripError> {
    let bytes = crate::mutation::filesystem::read_private_file(&directory.join("manifest.json"))?;
    let envelope = crate::recovery::model::decode_manifest_v2(&bytes)?;
    if envelope.payload.reference != expected {
        return Err(GripError::CorruptState(
            "recovery manifest does not match its storage identity".into(),
        ));
    }
    let payload_path = directory.join(&envelope.payload.private_ref);
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
    let bound_path = match (
        envelope.payload.identity.as_ref(),
        envelope.payload.endpoint_role,
    ) {
        (Some(identity), Some(role)) => {
            let runtime = crate::state::runtime_identity_from_portable(home, identity)?;
            Some(match role {
                crate::state::EndpointRoleV1::Source => runtime.source_path(),
                crate::state::EndpointRoleV1::Destination => runtime.destination_path(),
                crate::state::EndpointRoleV1::Descriptor => home.path().join("config.toml"),
                crate::state::EndpointRoleV1::AcceptedState => home.path().join("state/state.json"),
            })
        }
        (None, Some(crate::state::EndpointRoleV1::Descriptor)) => {
            Some(home.path().join("config.toml"))
        }
        (None, Some(crate::state::EndpointRoleV1::AcceptedState)) => {
            Some(home.path().join("state/state.json"))
        }
        _ => None,
    };
    Ok(RecoveryEntry {
        reference: expected,
        kind: envelope.payload.kind,
        origin: envelope.payload.origin_transition,
        managed_identity: envelope.payload.identity.as_ref().map(|identity| {
            format!(
                "{}:{}",
                identity.mapping.source.as_str(),
                identity.relative_path_hex
            )
        }),
        bound_side: envelope.payload.endpoint_role.map(|role| match role {
            crate::state::EndpointRoleV1::Source => "source".into(),
            crate::state::EndpointRoleV1::Destination => "destination".into(),
            crate::state::EndpointRoleV1::Descriptor => "descriptor".into(),
            crate::state::EndpointRoleV1::AcceptedState => "accepted_state".into(),
        }),
        bound_path: bound_path.map(|path| path.display().to_string()),
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
