mod support;

use std::fs;

fn json(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn add_list_show_and_remove_support_human_and_json_contracts() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    fs::create_dir(&source).unwrap();

    let add = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output=json",
            "mapping",
            "add",
            "tree",
            "source",
            "~/destination",
        ],
    );
    assert_eq!(add.status.code(), Some(0));
    assert_eq!(json(&add)["details"]["operation"], "mapping_add");
    assert_eq!(json(&add)["details"]["mapping"]["declared"]["kind"], "tree");

    let list = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "list"],
    );
    assert_eq!(
        json(&list)["details"]["mappings"].as_array().unwrap().len(),
        1
    );

    let show = support::project_command(root.path(), &metadata_dir, &["mapping", "show", "source"]);
    assert!(show.status.success());
    assert!(String::from_utf8_lossy(&show.stdout).contains("Mapping found"));

    let remove = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "remove", "--", "source"],
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
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("-source");
    fs::write(&source, "payload").unwrap();
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["mapping", "add", "file", "--", "-source", "~/-destination"],
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
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let child = source.join("child");
    fs::create_dir_all(&child).unwrap();
    let first = support::project_command(
        root.path(),
        &metadata_dir,
        &["mapping", "add", "tree", "source", "~/destination"],
    );
    assert!(first.status.success());
    let conflict = support::project_command(
        root.path(),
        &metadata_dir,
        &["mapping", "add", "tree", "source/child", "~/other"],
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
        let metadata_dir = support::initialize_project_metadata(root.path());
        let source = root.path().join(format!("{kind}-source"));
        if kind == "file" {
            fs::write(&source, "payload").unwrap();
        } else {
            fs::create_dir(&source).unwrap();
        }
        let declared_source = format!("{kind}-source");
        let declared_destination = format!("~/{kind}-destination");
        let canonical_source = fs::canonicalize(&source).unwrap().display().to_string();
        let canonical_destination = fs::canonicalize(root.path())
            .unwrap()
            .join(format!("{kind}-destination"))
            .display()
            .to_string();

        let add = support::project_command(
            root.path(),
            &metadata_dir,
            &[
                "--output=json",
                "mapping",
                "add",
                kind,
                &declared_source,
                &declared_destination,
            ],
        );
        let added = &json(&add)["details"]["mapping"];
        assert_eq!(added["declared"]["kind"], kind);
        assert_eq!(added["declared"]["source"], declared_source);
        assert_eq!(added["declared"]["destination"], declared_destination);
        assert_eq!(added["resolved"]["source"]["display"], canonical_source);
        assert_eq!(
            added["resolved"]["destination"]["display"],
            canonical_destination
        );

        for arguments in [
            vec!["mapping", "list"],
            vec!["mapping", "show", &declared_source],
        ] {
            let output = support::project_command(root.path(), &metadata_dir, &arguments);
            let human = String::from_utf8(output.stdout).unwrap();
            assert!(human.contains(kind));
            assert!(human.contains(&declared_source));
            assert!(human.contains(&declared_destination));
            assert!(human.contains(&canonical_source));
            assert!(human.contains(&canonical_destination));
        }

        let remove = support::project_command(
            root.path(),
            &metadata_dir,
            &["--output=json", "mapping", "remove", &declared_source],
        );
        let removed = &json(&remove)["details"]["mapping"];
        assert_eq!(removed["declared"]["kind"], kind);
        assert_eq!(removed["declared"]["source"], declared_source);
        assert_eq!(removed["declared"]["destination"], declared_destination);
        assert_eq!(removed["resolved"]["source"]["display"], canonical_source);
        assert_eq!(
            removed["resolved"]["destination"]["display"],
            canonical_destination
        );
    }
}
