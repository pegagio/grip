#![allow(dead_code, clippy::result_large_err)]
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
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

fn qualification_command(program: &str, arguments: &[&str]) -> String {
    Command::new(program)
        .args(arguments)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unavailable".into())
}

pub fn qualification_record(
    endpoint: &Path,
    build_profile: &str,
) -> grip::metadata::model::PlatformQualificationRecord {
    let file = fs::File::open(endpoint).unwrap();
    let profile = grip::metadata::macos::endpoint_capability_profile(
        &file,
        grip::metadata::model::EndpointRole::Source,
        endpoint.display().to_string(),
    )
    .unwrap();
    let observed_string = |evidence: &grip::metadata::model::Evidence<String>| match evidence {
        grip::metadata::model::Evidence::Observed { value } => value.clone(),
        _ => "unavailable".into(),
    };
    let observed_u64 = |evidence: &grip::metadata::model::Evidence<u64>| match evidence {
        grip::metadata::model::Evidence::Observed { value } => *value,
        _ => 0,
    };
    let masks = match &profile.volume_capability_masks {
        grip::metadata::model::Evidence::Observed { value } => value.clone(),
        _ => Vec::new(),
    };
    grip::metadata::model::PlatformQualificationRecord {
        macos_version: qualification_command("sw_vers", &["-productVersion"]),
        macos_build: qualification_command("sw_vers", &["-buildVersion"]),
        darwin_kernel: qualification_command("uname", &["-srv"]),
        apfs_bundle_version: qualification_command(
            "plutil",
            &[
                "-extract",
                "CFBundleShortVersionString",
                "raw",
                "-o",
                "-",
                "/System/Library/Filesystems/apfs.fs/Contents/Info.plist",
            ],
        ),
        filesystem_type: observed_string(&profile.filesystem_type),
        mount_flags: observed_u64(&profile.mount_flags),
        volume_capability_masks: masks,
        binary_build_profile: build_profile.into(),
        binary_revision: qualification_command("git", &["rev-parse", "HEAD"]),
        test_matrix_version: "feature-009-v1".into(),
        performance_host_description: qualification_command("uname", &["-m"]),
    }
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

#[derive(Debug)]
pub struct MetadataFixture {
    pub root: tempfile::TempDir,
    pub grip_home: PathBuf,
    pub source: PathBuf,
    pub destination: PathBuf,
}

impl MetadataFixture {
    pub fn file(contents: &[u8]) -> Self {
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let grip_home = minimal_home(root.path());
        let source = root.path().join("source");
        let destination = root.path().join("destination");
        fs::write(&source, contents).unwrap();
        fs::write(&destination, contents).unwrap();
        write_registry(&grip_home, &[("file", &source, &destination)]);
        Self {
            root,
            grip_home,
            source,
            destination,
        }
    }

    pub fn tree() -> Self {
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let grip_home = minimal_home(root.path());
        let source = root.path().join("source");
        let destination = root.path().join("destination");
        for tree in [&source, &destination] {
            fs::create_dir(tree).unwrap();
            fs::create_dir(tree.join("nested")).unwrap();
            fs::write(tree.join("nested/file"), b"accepted").unwrap();
            fs::create_dir(tree.join("empty")).unwrap();
        }
        write_registry(&grip_home, &[("tree", &source, &destination)]);
        Self {
            root,
            grip_home,
            source,
            destination,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AclFixtureEntry {
    pub principal_uuid: [u8; 16],
    pub kind: &'static str,
    pub permissions: Vec<&'static str>,
    pub flags: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BsdFlagFixture {
    Nodump,
    Immutable,
    Append,
    Hidden,
    Opaque,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApfsCapabilityFixture {
    pub filesystem_type: &'static str,
    pub case_sensitive: bool,
    pub case_preserving: bool,
    pub mtime_precision_nanoseconds: u32,
}

impl ApfsCapabilityFixture {
    pub fn case_insensitive() -> Self {
        Self {
            filesystem_type: "apfs",
            case_sensitive: false,
            case_preserving: true,
            mtime_precision_nanoseconds: 1,
        }
    }

    pub fn case_sensitive() -> Self {
        Self {
            case_sensitive: true,
            ..Self::case_insensitive()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataFaultPoint {
    Observe,
    Preflight,
    Revalidate,
    Preserve,
    ApplyOwnership,
    ApplyAcl,
    ApplyXattr,
    ApplyMode,
    ApplyModifiedTime,
    ApplyBsdFlags,
    Verify,
    PublishState,
    DeliverResult,
}

pub fn set_fixture_mode(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

pub fn set_fixture_modified_time(path: &Path, seconds: i64, nanoseconds: u32) {
    let file = fs::File::open(path).unwrap();
    rustix::fs::futimens(
        &file,
        &rustix::fs::Timestamps {
            last_access: rustix::fs::Timespec {
                tv_sec: 0,
                tv_nsec: rustix::fs::UTIME_OMIT,
            },
            last_modification: rustix::fs::Timespec {
                tv_sec: seconds,
                tv_nsec: i64::from(nanoseconds),
            },
        },
    )
    .unwrap();
}

pub fn set_fixture_xattr(path: &Path, name: &str, value: &[u8]) {
    rustix::fs::setxattr(path, name, value, rustix::fs::XattrFlags::empty()).unwrap();
}

pub fn remove_fixture_xattr(path: &Path, name: &str) {
    rustix::fs::removexattr(path, name).unwrap();
}

pub fn set_fixture_acl(path: &Path, entries: &[&str]) {
    for entry in entries {
        let output = Command::new("chmod")
            .arg("+a")
            .arg(entry)
            .arg(path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

pub fn clear_fixture_acl(path: &Path) {
    let output = Command::new("chmod").arg("-N").arg(path).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn set_fixture_bsd_flags(path: &Path, flags: u32) {
    let path = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
    // SAFETY: path is an owned NUL-terminated fixture path and flags contains caller-selected user flags.
    assert_eq!(unsafe { libc::chflags(path.as_ptr(), flags) }, 0);
}

pub fn copy_complete_metadata(
    source: &Path,
    destination: &Path,
    kind: grip::discovery::model::NodeKind,
) {
    let expected = grip::observation::fingerprint::inspect_complete(source, kind)
        .unwrap()
        .state;
    grip::metadata::macos::apply_metadata_paths(source, destination, kind, &expected.metadata)
        .unwrap();
}

pub fn copy_tree_entry_metadata(source: &Path, destination: &Path) {
    fn copy_children(source: &Path, destination: &Path) {
        let mut entries = fs::read_dir(source)
            .unwrap()
            .map(|entry| entry.unwrap())
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let source_path = entry.path();
            let destination_path = destination.join(entry.file_name());
            let kind = if source_path.is_dir() {
                copy_children(&source_path, &destination_path);
                grip::discovery::model::NodeKind::Directory
            } else {
                grip::discovery::model::NodeKind::File
            };
            copy_complete_metadata(&source_path, &destination_path, kind);
        }
    }

    copy_children(source, destination);
}

pub fn fixture_xattr(path: &Path, name: &str) -> Vec<u8> {
    let mut value = vec![0; 1024 * 1024];
    let length = rustix::fs::getxattr(path, name, &mut value).unwrap();
    value.truncate(length);
    value
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
    pub extended_attributes: BTreeMap<Vec<u8>, Vec<u8>>,
    pub acl: Option<Vec<u8>>,
    pub bsd_flags: u32,
}

fn snapshot_xattrs(path: &Path) -> BTreeMap<Vec<u8>, Vec<u8>> {
    let mut names = vec![0; 64 * 1024];
    let length = rustix::fs::llistxattr(path, &mut names).unwrap();
    names.truncate(length);
    names
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .map(|name| {
            let mut value = vec![0; 1024 * 1024];
            let length = rustix::fs::lgetxattr(path, name, &mut value).unwrap();
            value.truncate(length);
            (name.to_vec(), value)
        })
        .collect()
}

pub fn snapshot(root: &Path) -> BTreeMap<PathBuf, EntrySnapshot> {
    fn walk(base: &Path, at: &Path, result: &mut BTreeMap<PathBuf, EntrySnapshot>) {
        if let Ok(entries) = fs::read_dir(at) {
            for entry in entries.flatten() {
                let p = entry.path();
                let relative = p.strip_prefix(base).unwrap().to_owned();
                let metadata = fs::symlink_metadata(&p).unwrap();
                let native_metadata = if metadata.is_file() || metadata.is_dir() {
                    let file = fs::File::open(&p).unwrap();
                    Some((
                        grip::metadata::macos::raw_acl(&file).unwrap(),
                        grip::metadata::macos::raw_bsd_flags(&file).unwrap(),
                    ))
                } else {
                    None
                };
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
                    extended_attributes: snapshot_xattrs(&p),
                    acl: native_metadata.as_ref().and_then(|value| value.0.clone()),
                    bsd_flags: native_metadata.map_or(0, |value| value.1),
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
        complete_baselines: BTreeMap::new(),
        schema_version: Some(2),
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
            metadata: None,
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
        .map(|entry| grip::classification::classify_accepted(entry, &state.accepted))
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
        .map(|entry| grip::classification::classify_accepted(entry, &state.accepted))
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
        .map(|entry| grip::classification::classify_accepted(entry, &state.accepted))
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
    let identity = state
        .accepted
        .complete_baselines
        .keys()
        .next()
        .or_else(|| state.accepted.baselines.keys().next())
        .unwrap()
        .clone();
    let selection = grip::observation::model::Selection::Entry(identity);
    let observed =
        grip::observation::inspect(&home, &registry, &state.accepted, &selection).unwrap();
    let records = observed
        .values()
        .map(|entry| grip::classification::classify_accepted(entry, &state.accepted))
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
        .map(|entry| grip::classification::classify_accepted(entry, &state.accepted))
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
