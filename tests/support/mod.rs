#![allow(dead_code, clippy::result_large_err)]
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub struct FailingWriter;

impl std::io::Write for FailingWriter {
    fn write(&mut self, _buffer: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "injected output failure",
        ))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn command(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grip"))
        .env_clear()
        .env("HOME", home)
        .args(args)
        .output()
        .unwrap()
}
pub fn command_with_grip_home(home: &Path, grip_home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grip"))
        .env_clear()
        .env("HOME", home)
        .env("GRIP_HOME", grip_home)
        .args(args)
        .output()
        .unwrap()
}
pub fn minimal_home(root: &Path) -> PathBuf {
    let grip = root.join(".grip");
    fs::create_dir(&grip).unwrap();
    fs::write(
        grip.join("config.toml"),
        "schema_version = 1\nmappings = []\n",
    )
    .unwrap();
    grip
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntrySnapshot {
    pub bytes: Vec<u8>,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub device: u64,
    pub inode: u64,
    pub modified_seconds: i64,
    pub modified_nanoseconds: i64,
}

pub fn snapshot(root: &Path) -> BTreeMap<PathBuf, EntrySnapshot> {
    fn walk(base: &Path, at: &Path, result: &mut BTreeMap<PathBuf, EntrySnapshot>) {
        if let Ok(entries) = fs::read_dir(at) {
            for entry in entries.flatten() {
                let p = entry.path();
                let relative = p.strip_prefix(base).unwrap().to_owned();
                let metadata = fs::symlink_metadata(&p).unwrap();
                let snapshot = EntrySnapshot {
                    bytes: if metadata.is_file() {
                        fs::read(&p).unwrap()
                    } else {
                        Vec::new()
                    },
                    mode: metadata.mode(),
                    uid: metadata.uid(),
                    gid: metadata.gid(),
                    device: metadata.dev(),
                    inode: metadata.ino(),
                    modified_seconds: metadata.mtime(),
                    modified_nanoseconds: metadata.mtime_nsec(),
                };
                result.insert(relative.clone(), snapshot);
                if metadata.is_dir() {
                    walk(base, &p, result);
                }
            }
        }
    }
    let mut value = BTreeMap::new();
    walk(root, root, &mut value);
    value
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawMetadataSnapshot {
    pub mode: u32,
    pub device: u64,
    pub inode: u64,
    pub size: u64,
    pub modified_seconds: i64,
    pub modified_nanoseconds: i64,
}

pub fn raw_metadata_snapshot(root: &Path) -> BTreeMap<Vec<u8>, RawMetadataSnapshot> {
    fn walk(base: &Path, at: &Path, result: &mut BTreeMap<Vec<u8>, RawMetadataSnapshot>) {
        if let Ok(entries) = fs::read_dir(at) {
            for entry in entries.flatten() {
                let path = entry.path();
                let metadata = fs::symlink_metadata(&path).unwrap();
                let relative = path
                    .strip_prefix(base)
                    .unwrap()
                    .as_os_str()
                    .as_bytes()
                    .to_vec();
                result.insert(
                    relative,
                    RawMetadataSnapshot {
                        mode: metadata.mode(),
                        device: metadata.dev(),
                        inode: metadata.ino(),
                        size: metadata.size(),
                        modified_seconds: metadata.mtime(),
                        modified_nanoseconds: metadata.mtime_nsec(),
                    },
                );
                if metadata.is_dir() && !metadata.file_type().is_symlink() {
                    walk(base, &path, result);
                }
            }
        }
    }
    let mut value = BTreeMap::new();
    walk(root, root, &mut value);
    value
}

pub fn write_registry(grip_home: &Path, mappings: &[(&str, &Path, &Path)]) {
    fn canonical(path: &Path) -> PathBuf {
        if path.exists() {
            return fs::canonicalize(path).unwrap();
        }
        let mut existing = path;
        let mut suffix = Vec::new();
        while !existing.exists() {
            suffix.push(existing.file_name().unwrap().to_owned());
            existing = existing.parent().unwrap();
        }
        let mut result = fs::canonicalize(existing).unwrap();
        for component in suffix.iter().rev() {
            result.push(component);
        }
        result
    }
    let mut document = String::from("schema_version = 1\n");
    if mappings.is_empty() {
        document.push_str("mappings = []\n");
    } else {
        for (kind, source, destination) in mappings {
            let source = canonical(source);
            let destination = canonical(destination);
            document.push_str(&format!(
                "\n[[mappings]]\nkind = \"{kind}\"\nsource = {:?}\ndestination = {:?}\n",
                source.display().to_string(),
                destination.display().to_string()
            ));
        }
    }
    fs::write(grip_home.join("config.toml"), document).unwrap();
}

pub fn json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "invalid JSON output: {error}; stdout={}; stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

pub fn write_v2_state(
    grip_home: &Path,
    generation: u64,
    baselines: BTreeMap<
        grip::observation::model::EntryIdentity,
        grip::observation::model::SupportedState,
    >,
) {
    let state = grip::state::AcceptedState {
        generation: Some(generation),
        baselines,
        accepted_bytes: None,
    };
    let directory = grip_home.join("state");
    fs::create_dir_all(&directory).unwrap();
    fs::set_permissions(
        &directory,
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .unwrap();
    let path = directory.join("state.json");
    fs::write(&path, grip::state::encode_v2(&state, generation).unwrap()).unwrap();
    fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o600)).unwrap();
}

pub fn supported_file_state(path: &Path) -> grip::observation::model::SupportedState {
    grip::observation::fingerprint::inspect(path, grip::discovery::model::NodeKind::File)
        .unwrap()
        .0
}

pub fn accepted_file_fixture() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "accepted").unwrap();
    write_registry(&grip_home, &[("file", &source, &destination)]);
    let output = command_with_grip_home(root.path(), &grip_home, &["push"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    (root, grip_home, source, destination)
}

pub fn accepted_tree_fixture() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    fs::create_dir(source.join("nested")).unwrap();
    fs::write(source.join("nested/file"), "accepted").unwrap();
    write_registry(&grip_home, &[("tree", &source, &destination)]);
    let output = command_with_grip_home(root.path(), &grip_home, &["push"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    (root, grip_home, source, destination)
}

pub fn untracked_file_fixture() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
    let (root, grip_home, source, destination) = accepted_file_fixture();
    let output = command_with_grip_home(
        root.path(),
        &grip_home,
        &["mapping", "remove", source.to_str().unwrap()],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    (root, grip_home, source, destination)
}

pub fn payload_recovery_fixture() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf, String) {
    let (root, grip_home, source, destination) = accepted_file_fixture();
    fs::write(&destination, "replacement").unwrap();
    let output = command_with_grip_home(root.path(), &grip_home, &["--output=json", "pull"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value = json(&output);
    let operation = value["details"]["operation_record"]["id"].as_str().unwrap();
    let reference = format!("payload:{operation}:0");
    (root, grip_home, source, destination, reference)
}

pub fn assert_snapshot_unchanged(expected: &BTreeMap<PathBuf, EntrySnapshot>, root: &Path) {
    assert_eq!(expected, &snapshot(root), "filesystem snapshot changed");
}

pub fn fail_once_at<T>(expected: T) -> impl FnMut(T) -> Result<(), grip::GripError>
where
    T: Clone + PartialEq + std::fmt::Debug,
{
    let mut failed = false;
    move |actual| {
        if !failed && actual == expected {
            failed = true;
            Err(grip::GripError::Internal(format!(
                "injected failure at {actual:?}"
            )))
        } else {
            Ok(())
        }
    }
}

pub fn test_push_plan(action_count: usize) -> grip::push::model::PushPlan {
    use grip::classification::model::ClassificationScope;
    use grip::discovery::model::SafePath;
    use grip::observation::model::PathSpace;
    use grip::push::model::{ActionEvidence, ActionKind, ActionStatus, PushAction, PushCounts};
    let actions = (0..action_count)
        .map(|index| PushAction {
            index,
            direction: grip::mutation::model::MutationDirection::Push,
            kind: ActionKind::AddFile,
            identity: None,
            dependent_identities: Vec::new(),
            source_path: Some(SafePath::from_path(Path::new("/source"))),
            destination: Path::new("/destination").to_path_buf(),
            destination_path: SafePath::from_path(Path::new("/destination")),
            expected_source: None,
            expected_destination: None,
            dependencies: Vec::new(),
            status: ActionStatus::Unattempted,
            milestones: ActionEvidence::default(),
            failure: None,
        })
        .collect::<Vec<_>>();
    grip::push::model::PushPlan {
        operation: grip::mutation::model::MutationOperation::Push,
        direction: Some(grip::mutation::model::MutationDirection::Push),
        winner: None,
        plan_id: "a".repeat(64),
        scope: ClassificationScope {
            kind: "all".into(),
            path_space: PathSpace::Source,
            selector: None,
            mapping_source: None,
        },
        entries: Vec::new(),
        acceptance_identities: Vec::new(),
        blockers: Vec::new(),
        counts: PushCounts {
            actionable: action_count,
            unattempted: action_count,
            ..PushCounts::default()
        },
        actions,
    }
}

pub fn test_pull_plan(action_count: usize) -> grip::mutation::model::MutationPlan {
    let mut plan = test_push_plan(action_count);
    plan.operation = grip::mutation::model::MutationOperation::Pull;
    plan.direction = Some(grip::mutation::model::MutationDirection::Pull);
    for action in &mut plan.actions {
        action.direction = grip::mutation::model::MutationDirection::Pull;
        action.kind = grip::mutation::model::ActionKind::ReplaceFile;
    }
    plan
}

pub fn pull_execution_fixture() -> (
    tempfile::TempDir,
    grip::home::GripHome,
    grip::registry::publication::RegistrySnapshot,
    grip::state::publication::StateSnapshot,
    grip::observation::model::Selection,
    grip::mutation::model::MutationPlan,
) {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "accepted").unwrap();
    write_registry(&grip_home, &[("file", &source, &destination)]);
    assert!(
        command_with_grip_home(root.path(), &grip_home, &["push"])
            .status
            .success()
    );
    fs::write(&destination, "destination change").unwrap();
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    let registry = grip::registry::publication::load(&home, false).unwrap();
    let state = grip::state::publication::load(&home).unwrap();
    let selection = grip::observation::model::Selection::All;
    let observed =
        grip::observation::inspect(&home, &registry, &state.accepted, &selection).unwrap();
    let records = observed
        .values()
        .map(|entry| {
            grip::classification::classify(entry, state.accepted.baselines.get(&entry.identity))
        })
        .collect();
    let plan = grip::mutation::plan::build_for(
        grip::mutation::model::MutationDirection::Pull,
        grip::classification::model::ClassificationScope {
            kind: "all".into(),
            path_space: grip::observation::model::PathSpace::Source,
            selector: None,
            mapping_source: None,
        },
        records,
    )
    .unwrap();
    (root, home, registry, state, selection, plan)
}

pub fn deletion_execution_fixture() -> (
    tempfile::TempDir,
    grip::home::GripHome,
    grip::registry::publication::RegistrySnapshot,
    grip::state::publication::StateSnapshot,
    grip::observation::model::Selection,
    grip::delete::model::DeletionPlan,
) {
    let (root, grip_home, source, _) = accepted_file_fixture();
    fs::remove_file(source).unwrap();
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    let registry = grip::registry::publication::load(&home, false).unwrap();
    let state = grip::state::publication::load(&home).unwrap();
    let selection = grip::observation::model::Selection::All;
    let observed =
        grip::observation::inspect(&home, &registry, &state.accepted, &selection).unwrap();
    let records = observed
        .values()
        .map(|entry| {
            grip::classification::classify(entry, state.accepted.baselines.get(&entry.identity))
        })
        .collect();
    let plan = grip::delete::plan::build(
        grip::delete::model::DeletionAuthority::Source,
        grip::classification::model::ClassificationScope {
            kind: "all".into(),
            path_space: grip::observation::model::PathSpace::Source,
            selector: None,
            mapping_source: None,
        },
        records,
    )
    .unwrap();
    (root, home, registry, state, selection, plan)
}

pub fn sync_execution_fixture() -> (
    tempfile::TempDir,
    grip::home::GripHome,
    grip::registry::publication::RegistrySnapshot,
    grip::state::publication::StateSnapshot,
    grip::observation::model::Selection,
    grip::mutation::model::MutationPlan,
) {
    let (root, grip_home, source, _) = accepted_file_fixture();
    fs::write(&source, "source change").unwrap();
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    let registry = grip::registry::publication::load(&home, false).unwrap();
    let state = grip::state::publication::load(&home).unwrap();
    let selection = grip::observation::model::Selection::All;
    let observed =
        grip::observation::inspect(&home, &registry, &state.accepted, &selection).unwrap();
    let records = observed
        .values()
        .map(|entry| {
            grip::classification::classify(entry, state.accepted.baselines.get(&entry.identity))
        })
        .collect();
    let plan = grip::mutation::plan::build_sync_with_parent_requirements(
        grip::classification::model::ClassificationScope {
            kind: "all".into(),
            path_space: grip::observation::model::PathSpace::Source,
            selector: None,
            mapping_source: None,
        },
        records,
        registry.missing_destination_parents(),
    )
    .unwrap();
    (root, home, registry, state, selection, plan)
}

pub fn resolution_execution_fixture(
    winner: grip::mutation::model::ConflictWinner,
) -> (
    tempfile::TempDir,
    grip::home::GripHome,
    grip::registry::publication::RegistrySnapshot,
    grip::state::publication::StateSnapshot,
    grip::observation::model::Selection,
    grip::mutation::model::MutationPlan,
) {
    let (root, grip_home, source, destination) = accepted_file_fixture();
    fs::write(&source, "source change").unwrap();
    fs::write(&destination, "destination change").unwrap();
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    let registry = grip::registry::publication::load(&home, false).unwrap();
    let state = grip::state::publication::load(&home).unwrap();
    let identity = state.accepted.baselines.keys().next().unwrap().clone();
    let selection = grip::observation::model::Selection::Entry(identity);
    let observed =
        grip::observation::inspect(&home, &registry, &state.accepted, &selection).unwrap();
    let records = observed
        .values()
        .map(|entry| {
            grip::classification::classify(entry, state.accepted.baselines.get(&entry.identity))
        })
        .collect();
    let plan = grip::mutation::plan::build_resolution(
        grip::classification::model::ClassificationScope {
            kind: "entry".into(),
            path_space: grip::observation::model::PathSpace::Source,
            selector: Some(grip::discovery::model::SafePath::from_path(&source)),
            mapping_source: Some(source.display().to_string()),
        },
        records,
        winner,
    )
    .unwrap();
    (root, home, registry, state, selection, plan)
}

pub fn mixed_sync_execution_fixture() -> (
    tempfile::TempDir,
    grip::home::GripHome,
    grip::registry::publication::RegistrySnapshot,
    grip::state::publication::StateSnapshot,
    grip::observation::model::Selection,
    grip::mutation::model::MutationPlan,
) {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = minimal_home(root.path());
    let source_a = root.path().join("source-a");
    let destination_a = root.path().join("destination-a");
    let source_b = root.path().join("source-b");
    let destination_b = root.path().join("destination-b");
    fs::write(&source_a, "accepted-a").unwrap();
    fs::write(&source_b, "accepted-b").unwrap();
    write_registry(
        &grip_home,
        &[
            ("file", &source_a, &destination_a),
            ("file", &source_b, &destination_b),
        ],
    );
    assert!(
        command_with_grip_home(root.path(), &grip_home, &["push"])
            .status
            .success()
    );
    fs::write(&source_a, "source change").unwrap();
    fs::write(&destination_b, "destination change").unwrap();
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    let registry = grip::registry::publication::load(&home, false).unwrap();
    let state = grip::state::publication::load(&home).unwrap();
    let selection = grip::observation::model::Selection::All;
    let observed =
        grip::observation::inspect(&home, &registry, &state.accepted, &selection).unwrap();
    let records = observed
        .values()
        .map(|entry| {
            grip::classification::classify(entry, state.accepted.baselines.get(&entry.identity))
        })
        .collect();
    let plan = grip::mutation::plan::build_sync_with_parent_requirements(
        grip::classification::model::ClassificationScope {
            kind: "all".into(),
            path_space: grip::observation::model::PathSpace::Source,
            selector: None,
            mapping_source: None,
        },
        records,
        registry.missing_destination_parents(),
    )
    .unwrap();
    (root, home, registry, state, selection, plan)
}

pub fn read_operation_component<T>(path: &Path) -> grip::operation::model::EnvelopeV1<T>
where
    T: Clone
        + serde::Serialize
        + serde::de::DeserializeOwned
        + grip::operation::model::ValidatePayload,
{
    grip::operation::model::decode(&fs::read(path).unwrap()).unwrap()
}

pub fn fail_push_at(
    expected: grip::push::FaultPhase,
) -> impl FnMut(grip::push::FaultPhase) -> Result<(), grip::GripError> {
    move |actual| {
        if actual == expected {
            Err(grip::GripError::Internal(format!(
                "injected push failure at {actual:?}"
            )))
        } else {
            Ok(())
        }
    }
}

pub fn fail_mutation_at(
    expected: grip::mutation::FaultPhase,
) -> impl FnMut(grip::mutation::FaultPhase) -> Result<(), grip::GripError> {
    move |actual| {
        if actual == expected {
            Err(grip::GripError::Internal(format!(
                "injected mutation failure at {actual:?}"
            )))
        } else {
            Ok(())
        }
    }
}
