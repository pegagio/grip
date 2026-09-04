#![allow(dead_code)]
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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
