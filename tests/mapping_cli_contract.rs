mod support;

use std::fs;

fn json(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn add_list_show_and_remove_support_human_and_json_contracts() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();

    let add = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output=json",
            "mapping",
            "add",
            "tree",
            source.to_str().unwrap(),
            destination.to_str().unwrap(),
        ],
    );
    assert_eq!(add.status.code(), Some(0));
    assert_eq!(json(&add)["details"]["operation"], "mapping_add");
    assert_eq!(json(&add)["details"]["mapping"]["kind"], "tree");

    let list = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    assert_eq!(
        json(&list)["details"]["mappings"].as_array().unwrap().len(),
        1
    );

    let show = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["mapping", "show", source.to_str().unwrap()],
    );
    assert!(show.status.success());
    assert!(String::from_utf8_lossy(&show.stdout).contains("Mapping found"));

    let remove = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output=json",
            "mapping",
            "remove",
            "--",
            source.to_str().unwrap(),
        ],
    );
    assert_eq!(json(&remove)["details"]["operation"], "mapping_remove");
}

#[test]
fn mapping_grammar_rejects_missing_extra_and_unknown_arguments() {
    let root = tempfile::tempdir().unwrap();
    for arguments in [
        vec!["mapping", "add", "file", "/only-source"],
        vec!["mapping", "list", "extra"],
        vec!["mapping", "show"],
        vec!["mapping", "remove"],
        vec!["mapping", "add", "unknown", "/a", "/b"],
    ] {
        let output = support::command(root.path(), &arguments);
        assert_eq!(output.status.code(), Some(2), "arguments: {arguments:?}");
    }
}

#[test]
fn option_termination_allows_dash_prefixed_path_components() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("-source");
    let destination = root.path().join("-destination");
    fs::write(&source, "payload").unwrap();
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "mapping",
            "add",
            "file",
            "--",
            source.to_str().unwrap(),
            destination.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn human_conflict_reports_canonical_paths_and_relation() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let child = source.join("child");
    fs::create_dir_all(&child).unwrap();
    let destination = root.path().join("destination");
    let first = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "mapping",
            "add",
            "tree",
            source.to_str().unwrap(),
            destination.to_str().unwrap(),
        ],
    );
    assert!(first.status.success());
    let conflict = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "mapping",
            "add",
            "tree",
            child.to_str().unwrap(),
            root.path().join("other").to_str().unwrap(),
        ],
    );
    let human = String::from_utf8(conflict.stdout).unwrap();
    assert!(human.contains(fs::canonicalize(&source).unwrap().to_str().unwrap()));
    assert!(human.contains(fs::canonicalize(&child).unwrap().to_str().unwrap()));
    assert!(human.contains("ancestor"));
}

#[test]
fn both_mapping_kinds_have_exact_tuples_in_human_and_json_lifecycle_output() {
    for kind in ["file", "tree"] {
        let root = tempfile::tempdir().unwrap();
        let grip_home = support::minimal_home(root.path());
        let source = root.path().join(format!("{kind}-source"));
        if kind == "file" {
            fs::write(&source, "payload").unwrap();
        } else {
            fs::create_dir(&source).unwrap();
        }
        let destination = root.path().join(format!("{kind}-destination"));
        let canonical_source = fs::canonicalize(&source).unwrap().display().to_string();
        let canonical_destination = fs::canonicalize(root.path())
            .unwrap()
            .join(format!("{kind}-destination"))
            .display()
            .to_string();

        let add = support::command_with_grip_home(
            root.path(),
            &grip_home,
            &[
                "--output=json",
                "mapping",
                "add",
                kind,
                source.to_str().unwrap(),
                destination.to_str().unwrap(),
            ],
        );
        let added = &json(&add)["details"]["mapping"];
        assert_eq!(added["kind"], kind);
        assert_eq!(added["source"], canonical_source);
        assert_eq!(added["destination"], canonical_destination);

        for arguments in [
            vec!["mapping", "list"],
            vec!["mapping", "show", source.to_str().unwrap()],
        ] {
            let output = support::command_with_grip_home(root.path(), &grip_home, &arguments);
            let human = String::from_utf8(output.stdout).unwrap();
            assert!(human.contains(kind));
            assert!(human.contains(&canonical_source));
            assert!(human.contains(&canonical_destination));
        }

        let remove = support::command_with_grip_home(
            root.path(),
            &grip_home,
            &[
                "--output=json",
                "mapping",
                "remove",
                source.to_str().unwrap(),
            ],
        );
        let removed = &json(&remove)["details"]["mapping"];
        assert_eq!(removed["kind"], kind);
        assert_eq!(removed["source"], canonical_source);
        assert_eq!(removed["destination"], canonical_destination);
    }
}
