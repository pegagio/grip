//! Atomic publication of partitioned operation records.

use crate::error::GripError;
use crate::home::GripHome;
use crate::mutation::model::MutationPlan;
use crate::operation::model::{
    ActionCheckpointEnvelopeV1, ActionCheckpointEvidenceV1, ActionCheckpointPayloadV1,
    OperationPlanEnvelopeV1, OperationPlanPayloadV1, OperationSummaryEnvelopeV1,
    OperationSummaryPayloadV1, encode,
};
use rustix::fs::{AtFlags, Mode, OFlags, RenameFlags, fsync, openat, renameat_with, unlinkat};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static OPERATION_COUNTER: AtomicU64 = AtomicU64::new(0);
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A live operation record owned by the current invocation.
#[derive(Debug)]
pub struct OperationReceipt {
    operation_id: String,
    directory: PathBuf,
    summary: OperationSummaryPayloadV1,
    action_count: usize,
}

impl OperationReceipt {
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// Atomically publish the latest bounded checkpoint for one started action.
    pub fn checkpoint_action(
        &self,
        action_index: usize,
        status: &str,
        evidence: ActionCheckpointEvidenceV1,
        failure: Option<String>,
    ) -> Result<(), GripError> {
        let payload = ActionCheckpointPayloadV1 {
            operation_id: self.operation_id.clone(),
            plan_id: self.summary.plan_id.clone(),
            action_index,
            status: status.into(),
            milestones: evidence,
            failure,
        };
        crate::operation::model::validate_checkpoint_binding(
            &payload,
            &self.operation_id,
            &self.summary.plan_id,
            self.action_count,
        )?;
        let actions = open_private_directory(&self.directory.join("actions"))?;
        let checkpoint_name = format!("{action_index:08}.json");
        if let Some(bytes) = read_private_component(&actions, checkpoint_name.as_bytes())? {
            let previous = crate::operation::model::decode::<ActionCheckpointPayloadV1>(&bytes)?;
            crate::operation::model::validate_action_transition(&previous.payload, &payload)?;
        } else if status != "in_progress" {
            return Err(GripError::CorruptState(
                "first action checkpoint must record in_progress".into(),
            ));
        }
        let envelope = ActionCheckpointEnvelopeV1::new(payload)?;
        publish_replace(
            &self.directory.join("actions"),
            &format!("{action_index:08}.json"),
            &encode(&envelope)?,
        )
    }

    /// Atomically publish a bounded operation-level outcome.
    pub fn checkpoint_summary(
        &mut self,
        state: &str,
        baseline: serde_json::Value,
        result_delivery: &str,
        failure: Option<String>,
    ) -> Result<(), GripError> {
        let mut next = self.summary.clone();
        next.state = state.into();
        next.baseline = baseline;
        next.result_delivery = result_delivery.into();
        next.failure = failure;
        crate::operation::model::validate_summary_transition(&self.summary, &next)?;
        self.summary = next;
        let envelope = OperationSummaryEnvelopeV1::new(self.summary.clone())?;
        publish_replace(&self.directory, "operation.json", &encode(&envelope)?)
    }
}

/// Allocate and initialize a new partitioned record immediately before mutation.
pub fn initialize(home: &GripHome, plan: &MutationPlan) -> Result<OperationReceipt, GripError> {
    initialize_typed(
        home,
        plan.operation.as_str(),
        &plan.plan_id,
        plan,
        plan.actions.len(),
    )
}

/// Allocate and initialize a partitioned record for any validated typed plan.
pub fn initialize_typed<T: serde::Serialize>(
    home: &GripHome,
    operation: &str,
    plan_id: &str,
    plan: &T,
    action_count: usize,
) -> Result<OperationReceipt, GripError> {
    let state = crate::state::publication::prepare_directory(home)?;
    let operations = state.join("operations");
    ensure_private_directory(&operations)?;
    let (operation_id, directory) = allocate_directory(&operations, operation)?;
    ensure_private_directory(&directory.join("actions"))?;
    ensure_private_directory(&directory.join("recovery"))?;

    let plan_payload = OperationPlanPayloadV1 {
        operation_id: operation_id.clone(),
        operation: operation.into(),
        plan_id: plan_id.into(),
        plan: serde_json::to_value(plan).map_err(|error| {
            GripError::Internal(format!("could not encode operation plan: {error}"))
        })?,
    };
    let plan_envelope = OperationPlanEnvelopeV1::new(plan_payload)?;
    publish_new(&directory, "plan.json", &encode(&plan_envelope)?)?;
    let summary = OperationSummaryPayloadV1 {
        operation_id: operation_id.clone(),
        operation: operation.into(),
        state: "executing".into(),
        plan_id: plan_id.into(),
        plan_ref: "plan.json".into(),
        baseline: serde_json::json!({"outcome":"not_attempted"}),
        result_delivery: "not_attempted".into(),
        failure: None,
    };
    let envelope = OperationSummaryEnvelopeV1::new(summary.clone())?;
    publish_new(&directory, "operation.json", &encode(&envelope)?)?;
    Ok(OperationReceipt {
        operation_id,
        directory,
        summary,
        action_count,
    })
}

/// Best-effort finalization of the result channel for one just-completed invocation.
pub fn finalize_result_delivery(
    home: &GripHome,
    operation_id: &str,
    result_delivery: &str,
) -> Result<(), GripError> {
    if operation_id.is_empty()
        || !operation_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(GripError::CorruptState(
            "operation identifier is not an opaque safe component".into(),
        ));
    }
    let directory = home.path().join("state/operations").join(operation_id);
    let opened = open_private_directory(&directory)?;
    let bytes = read_private_component(&opened, b"operation.json")?
        .ok_or_else(|| GripError::CorruptState("operation summary is missing".into()))?;
    let envelope = crate::operation::model::decode::<OperationSummaryPayloadV1>(&bytes)?;
    if envelope.payload.operation_id != operation_id {
        return Err(GripError::CorruptState(
            "operation summary identifier does not match its directory".into(),
        ));
    }
    let mut receipt = OperationReceipt {
        operation_id: operation_id.into(),
        directory,
        summary: envelope.payload,
        action_count: 0,
    };
    receipt.checkpoint_summary(
        &receipt.summary.state.clone(),
        receipt.summary.baseline.clone(),
        result_delivery,
        receipt.summary.failure.clone(),
    )
}

fn allocate_directory(parent: &Path, operation: &str) -> Result<(String, PathBuf), GripError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    for _ in 0..128 {
        let counter = OPERATION_COUNTER.fetch_add(1, Ordering::Relaxed);
        let id = format!("{operation}-{seconds}-{}-{counter}", std::process::id());
        let path = parent.join(&id);
        let mut builder = fs::DirBuilder::new();
        builder.mode(0o700);
        match builder.create(&path) {
            Ok(()) => {
                fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).map_err(|error| {
                    GripError::from_io("could not secure operation directory", error)
                })?;
                File::open(parent)
                    .and_then(|file| file.sync_all())
                    .map_err(|error| {
                        GripError::from_io("could not sync operations directory", error)
                    })?;
                return Ok((id, path));
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(GripError::from_io(
                    "could not allocate operation directory",
                    error,
                ));
            }
        }
    }
    Err(GripError::Internal(
        "could not allocate a unique operation identifier".into(),
    ))
}

fn ensure_private_directory(path: &Path) -> Result<(), GripError> {
    match fs::symlink_metadata(path) {
        Ok(metadata)
            if metadata.is_dir()
                && !metadata.file_type().is_symlink()
                && metadata.uid() == rustix::process::geteuid().as_raw()
                && metadata.permissions().mode() & 0o7777 == 0o700 =>
        {
            Ok(())
        }
        Ok(_) => Err(GripError::CorruptState(format!(
            "operation directory {} is unsafe",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut builder = fs::DirBuilder::new();
            builder.mode(0o700);
            builder.create(path).map_err(|error| {
                GripError::from_io("could not create operation directory", error)
            })?;
            fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|error| {
                GripError::from_io("could not secure operation directory", error)
            })?;
            Ok(())
        }
        Err(error) => Err(GripError::from_io(
            "could not inspect operation directory",
            error,
        )),
    }
}

fn publish_new(directory: &Path, name: &str, bytes: &[u8]) -> Result<(), GripError> {
    publish_component(directory, name, bytes, true)
}

/// Atomically publish one immutable private operation-local component.
pub(crate) fn publish_new_component(
    directory: &Path,
    name: &str,
    bytes: &[u8],
) -> Result<(), GripError> {
    publish_new(directory, name, bytes)
}

fn publish_replace(directory: &Path, name: &str, bytes: &[u8]) -> Result<(), GripError> {
    publish_component(directory, name, bytes, false)
}

fn publish_component(
    directory: &Path,
    name: &str,
    bytes: &[u8],
    no_replace: bool,
) -> Result<(), GripError> {
    let directory = open_private_directory(directory)?;
    let temp_name = format!(
        ".record.tmp-{}-{}",
        std::process::id(),
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    let descriptor = openat(
        directory.as_fd(),
        OsStr::new(&temp_name),
        OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::CREATE | OFlags::EXCL,
        Mode::from_raw_mode(0o600),
    )
    .map_err(|error| GripError::from_io("could not stage operation record", error.into()))?;
    let mut file = File::from(descriptor);
    let identity = file
        .metadata()
        .map_err(|error| GripError::from_io("could not inspect operation record", error))?;
    let mut published = false;
    let result = (|| {
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|error| GripError::from_io("could not sync operation record", error))?;
        let path_metadata = directory
            .metadata(temp_name.as_bytes())
            .map_err(|error| GripError::from_io("could not revalidate operation record", error))?;
        if !identity.is_file()
            || identity.uid() != rustix::process::geteuid().as_raw()
            || identity.permissions().mode() & 0o7777 != 0o600
            || identity.dev() != path_metadata.stat.st_dev as u64
            || identity.ino() != path_metadata.stat.st_ino
        {
            return Err(GripError::CorruptState(
                "staged operation record changed before publication".into(),
            ));
        }
        let mut actual = Vec::new();
        file.seek(SeekFrom::Start(0))
            .and_then(|_| file.read_to_end(&mut actual))
            .map_err(|error| GripError::from_io("could not verify operation record", error))?;
        if actual != bytes {
            return Err(GripError::CorruptState(
                "staged operation record byte verification failed".into(),
            ));
        }
        renameat_with(
            directory.as_fd(),
            OsStr::new(&temp_name),
            directory.as_fd(),
            OsStr::new(name),
            if no_replace {
                RenameFlags::NOREPLACE
            } else {
                RenameFlags::empty()
            },
        )
        .map_err(|error| GripError::from_io("could not publish operation record", error.into()))?;
        published = true;
        fsync(directory.as_fd())
            .map_err(|error| GripError::from_io("could not sync operation directory", error.into()))
    })();
    if result.is_err()
        && !published
        && directory
            .metadata(temp_name.as_bytes())
            .is_ok_and(|metadata| {
                metadata.stat.st_dev as u64 == identity.dev()
                    && metadata.stat.st_ino == identity.ino()
            })
    {
        let _ = unlinkat(directory.as_fd(), OsStr::new(&temp_name), AtFlags::empty());
    }
    result
}

fn open_private_directory(
    path: &Path,
) -> Result<crate::discovery::filesystem::Directory, GripError> {
    let directory = crate::discovery::filesystem::Directory::open(path)
        .map_err(|error| GripError::from_io("could not open operation directory safely", error))?;
    let metadata = directory.root_metadata();
    if metadata.st_uid != rustix::process::geteuid().as_raw() || metadata.st_mode & 0o7777 != 0o700
    {
        return Err(GripError::CorruptState(
            "operation directory ownership or mode is unsafe".into(),
        ));
    }
    Ok(directory)
}

fn read_private_component(
    directory: &crate::discovery::filesystem::Directory,
    name: &[u8],
) -> Result<Option<Vec<u8>>, GripError> {
    let path_metadata = match directory.metadata(name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(GripError::from_io(
                "could not inspect operation record",
                error,
            ));
        }
    };
    let descriptor = directory
        .open_child_file(name)
        .map_err(|error| GripError::from_io("could not open operation record safely", error))?;
    let mut file = File::from(descriptor);
    let metadata = file
        .metadata()
        .map_err(|error| GripError::from_io("could not inspect opened operation record", error))?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.permissions().mode() & 0o7777 != 0o600
        || metadata.dev() != path_metadata.stat.st_dev as u64
        || metadata.ino() != path_metadata.stat.st_ino
    {
        return Err(GripError::CorruptState(
            "operation record identity, ownership, or mode is unsafe".into(),
        ));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| GripError::from_io("could not read operation record", error))?;
    Ok(Some(bytes))
}
