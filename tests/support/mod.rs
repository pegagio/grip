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

pub fn test_push_plan(action_count: usize) -> grip::push::model::PushPlan {
    use grip::classification::model::ClassificationScope;
    use grip::discovery::model::SafePath;
    use grip::observation::model::PathSpace;
    use grip::push::model::{ActionEvidence, ActionKind, ActionStatus, PushAction, PushCounts};
    let actions = (0..action_count)
        .map(|index| PushAction {
            index,
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
        plan_id: "a".repeat(64),
        scope: ClassificationScope {
            kind: "all".into(),
            path_space: PathSpace::Source,
            selector: None,
            mapping_source: None,
        },
        entries: Vec::new(),
        blockers: Vec::new(),
        counts: PushCounts {
            actionable: action_count,
            unattempted: action_count,
            ..PushCounts::default()
        },
        actions,
    }
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
