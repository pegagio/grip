use std::fs;
use std::path::{Path, PathBuf};

fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn current_wiki_files(root: &Path) -> Vec<PathBuf> {
    let index = fs::read_to_string(root.join("wiki/INDEX.md")).unwrap();
    let mut files = index
        .lines()
        .filter_map(|line| {
            let marker = "(./pages/";
            let start = line.find(marker)? + marker.len();
            let end = line[start..].find(')')? + start;
            Some(root.join("wiki/pages").join(&line[start..end]))
        })
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    files
}

fn assert_absent(files: &[PathBuf], forbidden: &[&str]) {
    let mut violations = Vec::new();
    for path in files {
        let text = fs::read_to_string(path).unwrap();
        for needle in forbidden {
            for (index, line) in text.lines().enumerate() {
                if line.contains(needle) {
                    violations.push(format!(
                        "{}:{} contains {needle:?}",
                        path.display(),
                        index + 1
                    ));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "legacy interface dependencies:\n{}",
        violations.join("\n")
    );
}

#[test]
fn production_and_active_tests_do_not_depend_on_legacy_global_authority() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut files = rust_files(&root.join("src"));
    files.extend(rust_files(&root.join("tests")));
    files.retain(|path| {
        path.file_name()
            .is_none_or(|name| name != "legacy_interface_contract.rs")
    });
    assert_absent(
        &files,
        &[
            "GRIP_HOME",
            "grip_home",
            "GripHome",
            "StateEnvelopeV1",
            "StateEnvelopeV2",
            "StateEnvelopeV3",
            "RegistryEnvelopeV1",
            "project_instance_key",
            "project-instance-key",
        ],
    );
}

#[test]
fn current_user_documentation_does_not_present_legacy_interfaces_as_supported() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut files = vec![
        root.join("README.md"),
        root.join("docs/product-definition.md"),
    ];
    files.extend(current_wiki_files(&root));
    let forbidden_supported_forms = [
        "export GRIP_HOME",
        "--grip-home",
        "project-instance-key",
        "project_instance_key",
        "source = \"/",
        "destination = \"/",
    ];
    assert_absent(&files, &forbidden_supported_forms);
}

#[test]
fn historical_feature_specs_are_outside_the_current_interface_scan() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for feature in 1..=9 {
        let prefix = format!("{feature:03}-");
        assert!(
            fs::read_dir(root.join("specs")).unwrap().any(|entry| entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(&prefix)),
            "expected historical feature {prefix} to remain preserved"
        );
    }
}
