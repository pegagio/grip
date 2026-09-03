#![allow(dead_code)]
use std::collections::BTreeMap;
use std::fs;
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
pub fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(base: &Path, at: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        if let Ok(entries) = fs::read_dir(at) {
            for entry in entries.flatten() {
                let p = entry.path();
                let relative = p.strip_prefix(base).unwrap().to_owned();
                if p.is_dir() {
                    result.insert(relative.clone(), Vec::new());
                    walk(base, &p, result);
                } else {
                    result.insert(relative, fs::read(&p).unwrap_or_default());
                }
            }
        }
    }
    let mut value = BTreeMap::new();
    walk(root, root, &mut value);
    value
}
