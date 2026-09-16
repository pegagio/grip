//! Isolated filesystem fixtures for project-scoped Grip integration tests.

use super::{EntrySnapshot, snapshot};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub const EMPTY_DESCRIPTOR_V2: &str = "schema_version = 2\nmappings = []\n";
pub const PROJECT_GITIGNORE: &str = "/state/\n";

#[derive(Debug, Clone, Copy)]
pub struct PortableFixtureMapping<'a> {
    pub kind: &'a str,
    pub source: &'a str,
    pub destination: &'a str,
}

#[derive(Debug)]
pub struct ProjectFixture {
    pub root: tempfile::TempDir,
    pub project_root: PathBuf,
    pub home_root: PathBuf,
}

/// A no-follow snapshot of a disposable destination symlink and its referent payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestinationLinkSnapshot {
    pub link_target: PathBuf,
    pub link_device: u64,
    pub link_inode: u64,
    pub target_contents: Vec<u8>,
}

impl ProjectFixture {
    pub fn new() -> Self {
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let project_root = root.path().join("project");
        let home_root = root.path().join("home");
        fs::create_dir(&project_root).unwrap();
        fs::create_dir(&home_root).unwrap();
        Self {
            root,
            project_root,
            home_root,
        }
    }

    pub fn initialized() -> Self {
        let fixture = Self::new();
        fixture.write_descriptor(&[]);
        fixture
    }

    pub fn metadata_dir(&self) -> PathBuf {
        self.project_root.join(".grip")
    }

    pub fn descriptor_path(&self) -> PathBuf {
        self.metadata_dir().join("config.toml")
    }

    /// Return an absolute destination that is outside the fixture home directory.
    pub fn absolute_destination(&self, name: &str) -> PathBuf {
        self.root.path().join("absolute-destinations").join(name)
    }

    /// Return a destination path rooted in the fixture's configured home.
    pub fn home_destination(&self, name: &str) -> PathBuf {
        self.home_root.join(name)
    }

    /// Write an executable test helper in the fixture root.
    pub fn write_executable(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.root.path().join(name);
        fs::write(&path, contents).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        path
    }

    /// Create an exact destination leaf link without resolving or modifying its target.
    pub fn create_destination_leaf_link(&self, name: &str, target: &Path) -> PathBuf {
        let destination = self.home_destination(name);
        symlink(target, &destination).unwrap();
        destination
    }

    /// Snapshot the link object and target payload for non-interference assertions.
    pub fn snapshot_destination_leaf_link(&self, destination: &Path) -> DestinationLinkSnapshot {
        let metadata = fs::symlink_metadata(destination).unwrap();
        assert!(metadata.file_type().is_symlink());
        let target = fs::read_link(destination).unwrap();
        DestinationLinkSnapshot {
            link_target: target.clone(),
            link_device: metadata.dev(),
            link_inode: metadata.ino(),
            target_contents: fs::read(target).unwrap(),
        }
    }

    /// Replace a disposable link immediately before the selected action publishes payload.
    pub fn substitute_destination_leaf_link(&self, destination: &Path, target: &Path) {
        fs::remove_file(destination).unwrap();
        symlink(target, destination).unwrap();
    }

    /// Exercise the production private-sibling directory publication path in isolation.
    pub fn publish_empty_directory_over_destination_link(&self, destination: &Path) {
        grip::mutation::filesystem::replace_link_with_empty_directory(destination).unwrap();
    }

    pub fn state_dir(&self) -> PathBuf {
        self.metadata_dir().join("state")
    }

    /// Return the accepted-state generation for aggregate publication assertions.
    pub fn accepted_generation(&self) -> Option<u64> {
        let home = grip::project::ProjectPaths::project_metadata(
            self.metadata_dir(),
            self.home_root.clone(),
        );
        grip::state::publication::load(&home)
            .unwrap()
            .accepted
            .generation
    }

    pub fn write_descriptor(&self, mappings: &[PortableFixtureMapping<'_>]) {
        let metadata = self.metadata_dir();
        fs::create_dir_all(&metadata).unwrap();
        let descriptor = if mappings.is_empty() {
            EMPTY_DESCRIPTOR_V2.to_owned()
        } else {
            let mut value = String::from("schema_version = 2\n");
            for mapping in mappings {
                value.push_str("\n[[mappings]]\nkind = \"");
                value.push_str(mapping.kind);
                value.push_str("\"\nsource = \"");
                value.push_str(mapping.source);
                value.push_str("\"\ndestination = \"");
                value.push_str(mapping.destination);
                value.push_str("\"\n");
            }
            value
        };
        fs::write(self.descriptor_path(), descriptor).unwrap();
        fs::write(metadata.join(".gitignore"), PROJECT_GITIGNORE).unwrap();
    }

    pub fn command(&self, arguments: &[&str]) -> Output {
        self.command_from(&self.project_root, arguments)
    }

    pub fn command_from(&self, cwd: &Path, arguments: &[&str]) -> Output {
        self.command_builder(cwd).args(arguments).output().unwrap()
    }

    pub fn command_os<I, S>(&self, cwd: &Path, arguments: I) -> Output
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.command_builder(cwd).args(arguments).output().unwrap()
    }

    pub fn explicit_project_command(&self, arguments: &[&str]) -> Output {
        let mut complete = vec!["--project", self.project_root.to_str().unwrap()];
        complete.extend_from_slice(arguments);
        self.command_from(self.root.path(), &complete)
    }

    pub fn command_builder(&self, cwd: &Path) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_grip"));
        command
            .env_clear()
            .env("HOME", &self.home_root)
            .current_dir(cwd);
        command
    }

    pub fn snapshot_all(&self) -> BTreeMap<PathBuf, EntrySnapshot> {
        snapshot(self.root.path())
    }

    pub fn set_mode(&self, path: &Path, mode: u32) {
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
    }

    pub fn nested_project(&self, relative: &str) -> PathBuf {
        let nested = self.project_root.join(relative);
        fs::create_dir_all(&nested).unwrap();
        let metadata = nested.join(".grip");
        fs::create_dir(&metadata).unwrap();
        fs::write(metadata.join("config.toml"), EMPTY_DESCRIPTOR_V2).unwrap();
        fs::write(metadata.join(".gitignore"), PROJECT_GITIGNORE).unwrap();
        nested
    }

    pub fn deep_descendant(&self, depth: usize) -> PathBuf {
        let mut current = self.project_root.clone();
        for index in 0..depth {
            current.push(format!("d{index:02}"));
        }
        fs::create_dir_all(&current).unwrap();
        current
    }

    pub fn copy_project(&self, name: &str) -> PathBuf {
        let destination = self.root.path().join(name);
        copy_tree(&self.project_root, &destination);
        destination
    }

    pub fn fault_command(&self, fault: &str, arguments: &[&str]) -> Output {
        self.command_builder(&self.project_root)
            .env("GRIP_TEST_FAULT", fault)
            .args(arguments)
            .output()
            .unwrap()
    }

    pub fn hold_lock(&self, relative: &str) -> FixtureLock {
        let path = self.state_dir().join("locks").join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        FixtureLock { path, _file: file }
    }
}

impl Default for ProjectFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// Disposable layout for a project nested beneath the destination home with a `home/` source.
#[derive(Debug)]
pub struct ContainedHomeFixture {
    pub root: tempfile::TempDir,
    pub home_root: PathBuf,
    pub project_root: PathBuf,
    pub source_root: PathBuf,
}

impl ContainedHomeFixture {
    pub fn initialized() -> Self {
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let home_root = root.path().join("home");
        let project_root = home_root.join("grip-project");
        let source_root = project_root.join("home");
        fs::create_dir_all(&source_root).unwrap();
        let metadata = project_root.join(".grip");
        fs::create_dir(&metadata).unwrap();
        fs::write(metadata.join("config.toml"), EMPTY_DESCRIPTOR_V2).unwrap();
        fs::write(metadata.join(".gitignore"), PROJECT_GITIGNORE).unwrap();
        Self {
            root,
            home_root,
            project_root,
            source_root,
        }
    }

    pub fn command(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grip"))
            .env_clear()
            .env("HOME", &self.home_root)
            .current_dir(&self.project_root)
            .args(arguments)
            .output()
            .unwrap()
    }

    pub fn descriptor_path(&self) -> PathBuf {
        self.project_root.join(".grip/config.toml")
    }

    pub fn state_path(&self) -> PathBuf {
        self.project_root.join(".grip/state/state.json")
    }

    pub fn snapshot_payloads(&self) -> BTreeMap<PathBuf, EntrySnapshot> {
        let metadata_relative = self
            .project_root
            .join(".grip")
            .strip_prefix(&self.home_root)
            .unwrap()
            .to_path_buf();
        snapshot(&self.home_root)
            .into_iter()
            .filter(|(path, _)| !path.starts_with(&metadata_relative))
            .collect()
    }
}

#[derive(Debug)]
pub struct FixtureLock {
    path: PathBuf,
    _file: fs::File,
}

impl Drop for FixtureLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir(destination).unwrap();
    let mut entries = fs::read_dir(source)
        .unwrap()
        .map(|entry| entry.unwrap())
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path).unwrap();
        if metadata.is_dir() {
            copy_tree(&source_path, &destination_path);
        } else if metadata.is_file() {
            fs::copy(&source_path, &destination_path).unwrap();
        } else if metadata.file_type().is_symlink() {
            std::os::unix::fs::symlink(fs::read_link(&source_path).unwrap(), destination_path)
                .unwrap();
        }
    }
    fs::set_permissions(destination, fs::metadata(source).unwrap().permissions()).unwrap();
}
