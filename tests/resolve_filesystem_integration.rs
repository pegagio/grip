mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;

#[test]
fn source_winner_preserves_destination_and_publishes_complete_state() {
    let (root, metadata_dir, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "source wins").unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o640)).unwrap();
    fs::write(&destination, "destination loses").unwrap();
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "resolve", "--source", "--", "source"],
    );
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let value = support::json(&output);
    assert_eq!(value["details"]["winner"], "source");
    assert_eq!(fs::read_to_string(&destination).unwrap(), "source wins");
    assert_eq!(
        fs::metadata(&destination).unwrap().permissions().mode() & 0o7777,
        0o640
    );
    let recovery = value["details"]["actions"][0]["milestones"]["recovery_ref"]
        .as_str()
        .unwrap();
    let operation = value["details"]["operation_record"]["id"].as_str().unwrap();
    assert!(recovery.ends_with("metadata-v3.json"));
    assert_eq!(
        fs::read_to_string(
            metadata_dir
                .join("state/operations")
                .join(operation)
                .join("recovery/00000000/payload")
        )
        .unwrap(),
        "destination loses"
    );
}

#[test]
fn destination_winner_symmetrically_replaces_source() {
    let (root, metadata_dir, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "source loses").unwrap();
    fs::write(&destination, "destination wins").unwrap();
    fs::set_permissions(&destination, fs::Permissions::from_mode(0o600)).unwrap();
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output",
            "json",
            "resolve",
            "--destination",
            "--",
            "source",
        ],
    );
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let value = support::json(&output);
    assert_eq!(value["details"]["actions"][0]["direction"], "pull");
    assert_eq!(fs::read_to_string(&source).unwrap(), "destination wins");
    assert_eq!(
        fs::metadata(&source).unwrap().permissions().mode() & 0o7777,
        0o600
    );
}

#[test]
fn changed_conflict_evidence_is_rejected_before_record_creation() {
    let (root, home, registry, state, selection, plan) =
        support::resolution_execution_fixture(grip::mutation::model::ConflictWinner::Source);
    let operation_count = fs::read_dir(home.path().join("state/operations"))
        .unwrap()
        .count();
    fs::write(root.path().join("source"), "newer source change").unwrap();
    let error = grip::mutation::execution::execute(&home, &registry, &state, &selection, &plan)
        .unwrap_err();
    assert!(error.to_string().contains("plan changed"));
    assert_eq!(
        fs::read_dir(home.path().join("state/operations"))
            .unwrap()
            .count(),
        operation_count
    );
    assert_eq!(
        fs::read_to_string(root.path().join("destination")).unwrap(),
        "destination change"
    );
}

#[test]
fn changed_destination_or_accepted_baseline_rejects_the_stale_resolution() {
    for drift in ["destination", "baseline"] {
        let (root, home, registry, state, selection, plan) =
            support::resolution_execution_fixture(grip::mutation::model::ConflictWinner::Source);
        let operation_count = fs::read_dir(home.path().join("state/operations"))
            .unwrap()
            .count();
        if drift == "destination" {
            fs::write(root.path().join("destination"), "newer destination change").unwrap();
        } else {
            let mut baselines = state.accepted.complete_baselines.clone();
            let identity = baselines.keys().next().unwrap().clone();
            baselines.insert(
                identity,
                grip::observation::fingerprint::inspect_complete(
                    &root.path().join("source"),
                    grip::discovery::model::NodeKind::File,
                )
                .unwrap()
                .state,
            );
            let mut envelope =
                grip::state::decode_v4(&fs::read(home.path().join("state/state.json")).unwrap())
                    .unwrap();
            envelope.generation = state.accepted.generation.unwrap_or(0) + 1;
            let replacement = baselines.values().next().unwrap().clone();
            *envelope.baselines.values_mut().next().unwrap() = replacement;
            fs::write(
                home.path().join("state/state.json"),
                grip::state::encode_v4(&envelope).unwrap(),
            )
            .unwrap();
        }
        let result =
            grip::mutation::execution::execute(&home, &registry, &state, &selection, &plan);
        assert!(result.is_err(), "{drift} drift must fail");
        assert_eq!(
            fs::read_dir(home.path().join("state/operations"))
                .unwrap()
                .count(),
            operation_count
        );
    }
}

#[test]
fn resolution_treats_opposing_content_and_metadata_changes_as_one_winner_state() {
    let (root, metadata_dir, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "source content").unwrap();
    fs::set_permissions(&destination, fs::Permissions::from_mode(0o600)).unwrap();

    let preview = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output=json",
            "resolve",
            "--dry-run",
            "--source",
            "--",
            "source",
        ],
    );
    assert!(preview.status.success());
    let entry = &support::json(&preview)["details"]["entries"][0];
    assert_eq!(entry["classification"], "divergent_conflict");

    let applied = support::project_command(
        root.path(),
        &metadata_dir,
        &["resolve", "--source", "--", "source"],
    );
    assert!(applied.status.success());
    assert_eq!(fs::read_to_string(&destination).unwrap(), "source content");
    assert_eq!(
        fs::metadata(&destination).unwrap().permissions().mode() & 0o7777,
        fs::metadata(&source).unwrap().permissions().mode() & 0o7777
    );
}

#[test]
fn resolution_rejects_deletions_and_pending_retirement_without_mutation() {
    for deleted_side in ["source", "destination"] {
        let (root, metadata_dir, source, destination) = support::accepted_file_fixture();
        if deleted_side == "source" {
            fs::remove_file(&source).unwrap();
        } else {
            fs::remove_file(&destination).unwrap();
        }
        let before = support::snapshot(root.path());
        let output = support::project_command(
            root.path(),
            &metadata_dir,
            &["--output=json", "resolve", "--source", "--", "source"],
        );
        assert_eq!(output.status.code(), Some(10));
        assert_eq!(support::json(&output)["details"]["result"], "blocked");
        assert_eq!(support::snapshot(root.path()), before);
    }

    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source-tree");
    let destination = root.path().join("destination-tree");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("retired"), "accepted").unwrap();
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);
    assert!(
        support::project_command(root.path(), &metadata_dir, &["push"])
            .status
            .success()
    );
    fs::write(source.join(".gripignore"), "retired\n").unwrap();
    let before = support::snapshot(root.path());
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output=json",
            "resolve",
            "--source",
            "--",
            "source-tree/retired",
        ],
    );
    assert_eq!(output.status.code(), Some(10));
    assert_eq!(support::json(&output)["details"]["result"], "blocked");
    assert_eq!(support::snapshot(root.path()), before);
}
