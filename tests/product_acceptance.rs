mod support;

use grip::discovery::model::NodeKind;
use grip::mapping::MappingKind;
use grip::metadata::model::{EndpointRole, Evidence};
use grip::observation::model::{EntryIdentity, MappingSnapshot};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

fn configured_root(variable: &str) -> PathBuf {
    let root = std::env::var_os(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("{variable} must name a disposable APFS qualification root"));
    assert!(root.is_absolute() && root.is_dir());
    root
}

fn assert_case_behavior(root: &Path, expected_case_sensitive: bool) {
    let file = fs::File::open(root).unwrap();
    let profile = grip::metadata::macos::endpoint_capability_profile(
        &file,
        EndpointRole::Source,
        root.display().to_string(),
    )
    .unwrap();
    assert!(matches!(profile.filesystem_type, Evidence::Observed { ref value } if value == "apfs"));
    assert_eq!(
        profile.case_sensitive,
        Evidence::Observed {
            value: expected_case_sensitive
        }
    );
}

fn run_product_lifecycle(root: &Path, expected_case_sensitive: bool) {
    assert_case_behavior(root, expected_case_sensitive);
    let fixture = tempfile::tempdir_in(root).unwrap();
    let grip_home = support::minimal_home(fixture.path());
    let source = fixture.path().join("source");
    let destination = fixture.path().join("destination");
    fs::write(&source, b"accepted").unwrap();
    support::write_registry(&grip_home, &[("file", &source, &destination)]);

    let initial = support::command_with_grip_home(fixture.path(), &grip_home, &["push"]);
    assert!(initial.status.success());
    support::set_fixture_mode(&source, 0o740);
    let source_only =
        support::command_with_grip_home(fixture.path(), &grip_home, &["--output=json", "status"]);
    assert_eq!(
        support::json(&source_only)["details"]["records"][0]["classification"],
        "source_only_change"
    );
    assert!(
        support::command_with_grip_home(fixture.path(), &grip_home, &["push"])
            .status
            .success()
    );
    support::set_fixture_xattr(&destination, "com.apple.TextEncoding", b"utf-8");
    let destination_only =
        support::command_with_grip_home(fixture.path(), &grip_home, &["--output=json", "status"]);
    assert_eq!(
        support::json(&destination_only)["details"]["records"][0]["classification"],
        "destination_only_change"
    );
    assert!(
        support::command_with_grip_home(fixture.path(), &grip_home, &["pull"])
            .status
            .success()
    );

    fs::write(&source, b"converged change").unwrap();
    fs::write(&destination, b"converged change").unwrap();
    support::copy_complete_metadata(&source, &destination, NodeKind::File);
    let converged =
        support::command_with_grip_home(fixture.path(), &grip_home, &["--output=json", "status"]);
    assert_eq!(
        support::json(&converged)["details"]["records"][0]["classification"],
        "converged_two_sided_change"
    );
    assert!(
        support::command_with_grip_home(fixture.path(), &grip_home, &["sync"])
            .status
            .success()
    );

    fs::write(&source, b"source winner").unwrap();
    fs::write(&destination, b"destination loser").unwrap();
    let conflict =
        support::command_with_grip_home(fixture.path(), &grip_home, &["--output=json", "status"]);
    assert_eq!(
        support::json(&conflict)["details"]["records"][0]["classification"],
        "divergent_conflict"
    );
    let resolved = support::command_with_grip_home(
        fixture.path(),
        &grip_home,
        &["resolve", source.to_str().unwrap(), "--source"],
    );
    assert!(resolved.status.success());
    assert_eq!(
        grip::observation::fingerprint::inspect_complete(&source, NodeKind::File)
            .unwrap()
            .state,
        grip::observation::fingerprint::inspect_complete(&destination, NodeKind::File)
            .unwrap()
            .state
    );

    fs::remove_file(&source).unwrap();
    let deleted = support::command_with_grip_home(
        fixture.path(),
        &grip_home,
        &[
            "--output=json",
            "delete",
            "--source",
            source.to_str().unwrap(),
        ],
    );
    assert!(deleted.status.success());
    assert!(!destination.exists());
    let operation = support::json(&deleted)["details"]["operation_record"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let reference = format!("payload:{operation}:0");
    assert!(
        support::command_with_grip_home(
            fixture.path(),
            &grip_home,
            &["recovery", "restore", &reference],
        )
        .status
        .success()
    );
    assert_eq!(fs::read(destination).unwrap(), b"source winner");
}

fn run_operational_boundaries(root: &Path) {
    let fixture = tempfile::tempdir_in(root).unwrap();
    let grip_home = support::minimal_home(fixture.path());
    let source = fixture.path().join("source");
    let destination = fixture.path().join("destination");
    fs::write(&source, b"accepted").unwrap();
    support::write_registry(&grip_home, &[("file", &source, &destination)]);
    assert!(
        support::command_with_grip_home(fixture.path(), &grip_home, &["push"])
            .status
            .success()
    );

    fs::write(&source, b"contended change").unwrap();
    let destination_before = fs::read(&destination).unwrap();
    let home = grip::home::select(Some(grip_home.clone().into_os_string()), None).unwrap();
    let guard = grip::state::mutation_lock::MutationLock::acquire(&home, "qualification").unwrap();
    let contended =
        support::command_with_grip_home(fixture.path(), &grip_home, &["--output=json", "push"]);
    assert_eq!(contended.status.code(), Some(13));
    assert_eq!(fs::read(&destination).unwrap(), destination_before);
    drop(guard);
    assert!(
        support::command_with_grip_home(fixture.path(), &grip_home, &["push"])
            .status
            .success()
    );

    fs::remove_file(&source).unwrap();
    fs::remove_file(&destination).unwrap();
    let retired = support::command_with_grip_home(
        fixture.path(),
        &grip_home,
        &["--output=json", "retire", "--all"],
    );
    assert!(retired.status.success());
    assert_eq!(support::json(&retired)["details"]["result"], "applied");
}

fn run_migration_and_blocker_flows(root: &Path) {
    let migration = tempfile::tempdir_in(root).unwrap();
    let migration_home = support::minimal_home(migration.path());
    let source = migration.path().join("source");
    let destination = migration.path().join("destination");
    fs::write(&source, b"legacy").unwrap();
    fs::write(&destination, b"legacy").unwrap();
    support::copy_complete_metadata(&source, &destination, NodeKind::File);
    support::write_registry(&migration_home, &[("file", &source, &destination)]);
    let identity = EntryIdentity::new(
        MappingSnapshot {
            kind: MappingKind::File,
            source: source.clone(),
            destination: destination.clone(),
        },
        Vec::new(),
    )
    .unwrap();
    support::write_v2_state(
        &migration_home,
        4,
        BTreeMap::from([(identity, support::supported_file_state(&source))]),
    );
    let status = support::command_with_grip_home(
        migration.path(),
        &migration_home,
        &["--output=json", "status"],
    );
    assert_eq!(
        support::json(&status)["details"]["records"][0]["classification"],
        "metadata_migration_ready"
    );
    assert!(
        support::command_with_grip_home(
            migration.path(),
            &migration_home,
            &["baseline", "accept"],
        )
        .status
        .success()
    );
    let state = fs::read(migration_home.join("state/state.json")).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&state).unwrap()["schema_version"],
        3
    );

    let blocked = tempfile::tempdir_in(root).unwrap();
    let blocked_home = support::minimal_home(blocked.path());
    let blocked_source = blocked.path().join("source");
    let blocked_destination = blocked.path().join("destination");
    fs::create_dir(&blocked_source).unwrap();
    let ordinary = blocked_source.join("ordinary");
    fs::write(&ordinary, b"private xattr value").unwrap();
    let sentinel = blocked.path().join("sentinel");
    fs::write(&sentinel, b"must remain untouched").unwrap();
    std::os::unix::fs::symlink(&sentinel, blocked_source.join("unsupported-link")).unwrap();
    support::write_registry(
        &blocked_home,
        &[("tree", &blocked_source, &blocked_destination)],
    );
    let inspection = support::command_with_grip_home(
        blocked.path(),
        &blocked_home,
        &["--output=json", "mapping", "inspect"],
    );
    assert!(
        support::json(&inspection)["details"]["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|record| record["reason"] == "symlink")
    );
    fs::remove_file(blocked_source.join("unsupported-link")).unwrap();
    assert!(
        support::command_with_grip_home(blocked.path(), &blocked_home, &["push"])
            .status
            .success()
    );
    assert!(!blocked_destination.exists() || blocked_destination.is_dir());
    assert_eq!(fs::read(sentinel).unwrap(), b"must remain untouched");

    let capability = tempfile::tempdir_in(root).unwrap();
    let capability_home = support::minimal_home(capability.path());
    let capability_source = capability.path().join("source");
    let capability_destination = capability.path().join("destination");
    fs::write(&capability_source, b"accepted").unwrap();
    fs::write(&capability_destination, b"accepted").unwrap();
    support::copy_complete_metadata(&capability_source, &capability_destination, NodeKind::File);
    support::write_registry(
        &capability_home,
        &[("file", &capability_source, &capability_destination)],
    );
    assert!(
        support::command_with_grip_home(
            capability.path(),
            &capability_home,
            &["baseline", "accept"],
        )
        .status
        .success()
    );
    support::set_fixture_xattr(
        &capability_source,
        "com.example.unknown",
        b"must-not-render",
    );
    let status = support::command_with_grip_home(
        capability.path(),
        &capability_home,
        &["--output=json", "status"],
    );
    let rendered = String::from_utf8_lossy(&status.stdout);
    assert!(rendered.contains("unknown_xattr"));
    assert!(!rendered.contains("must-not-render"));
    let preview = support::command_with_grip_home(
        capability.path(),
        &capability_home,
        &["--output=json", "push", "--dry-run"],
    );
    assert_eq!(support::json(&preview)["details"]["counts"]["blockers"], 1);
}

#[allow(clippy::result_large_err)]
fn run_partial_failure_flow(root: &Path) {
    let fixture = tempfile::tempdir_in(root).unwrap();
    let grip_home = support::minimal_home(fixture.path());
    let source_a = fixture.path().join("source-a");
    let destination_a = fixture.path().join("destination-a");
    let source_b = fixture.path().join("source-b");
    let destination_b = fixture.path().join("destination-b");
    fs::write(&source_a, b"accepted-a").unwrap();
    fs::write(&source_b, b"accepted-b").unwrap();
    support::write_registry(
        &grip_home,
        &[
            ("file", &source_a, &destination_a),
            ("file", &source_b, &destination_b),
        ],
    );
    assert!(
        support::command_with_grip_home(fixture.path(), &grip_home, &["push"])
            .status
            .success()
    );
    fs::write(&source_a, b"changed-a").unwrap();
    fs::write(&source_b, b"changed-b").unwrap();
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
    assert_eq!(plan.actions.len(), 2);
    let error = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        support::fail_mutation_at(grip::mutation::FaultPhase::BeforeStaging(1)),
    )
    .unwrap_err();
    let grip::GripError::MutationFailed(failure) = error else {
        panic!("expected a journaled partial failure");
    };
    assert_eq!(failure.completion, "partial");
    assert_eq!(failure.plan.counts.completed, 1);
    assert_eq!(failure.plan.counts.failed, 1);
    assert_eq!(failure.baseline.outcome, "not_published");
}

#[test]
#[ignore = "requires GRIP_APFS_CASE_INSENSITIVE_ROOT disposable qualification volume"]
fn case_insensitive_apfs_product_matrix() {
    let root = configured_root("GRIP_APFS_CASE_INSENSITIVE_ROOT");
    run_product_lifecycle(&root, false);
    run_operational_boundaries(&root);
    run_migration_and_blocker_flows(&root);
    run_partial_failure_flow(&root);
}

#[test]
#[ignore = "requires GRIP_APFS_CASE_SENSITIVE_ROOT disposable qualification volume"]
fn case_sensitive_apfs_product_matrix() {
    let root = configured_root("GRIP_APFS_CASE_SENSITIVE_ROOT");
    run_product_lifecycle(&root, true);
    run_operational_boundaries(&root);
    run_migration_and_blocker_flows(&root);
    run_partial_failure_flow(&root);
}

#[test]
#[ignore = "requires distinct disposable APFS qualification volumes"]
fn cross_volume_apfs_transfer_preserves_logical_complete_state() {
    let source_root = configured_root("GRIP_APFS_CASE_INSENSITIVE_ROOT");
    let destination_root = configured_root("GRIP_APFS_CASE_SENSITIVE_ROOT");
    let source_fixture = tempfile::tempdir_in(&source_root).unwrap();
    let destination_fixture = tempfile::tempdir_in(&destination_root).unwrap();
    let source = source_fixture.path().join("source");
    let destination = destination_fixture.path().join("destination");
    fs::write(&source, b"cross-volume").unwrap();
    std::os::unix::fs::chown(
        &source,
        None,
        Some(fs::metadata(destination_fixture.path()).unwrap().gid()),
    )
    .unwrap();
    assert_ne!(
        fs::metadata(source_fixture.path()).unwrap().dev(),
        fs::metadata(destination_fixture.path()).unwrap().dev()
    );
    let grip_home = support::minimal_home(source_fixture.path());
    support::write_registry(&grip_home, &[("file", &source, &destination)]);
    support::set_fixture_mode(&source, 0o741);
    support::set_fixture_xattr(&source, "com.apple.TextEncoding", b"utf-8");
    let push = support::command_with_grip_home(source_fixture.path(), &grip_home, &["push"]);
    assert!(
        push.status.success(),
        "{}",
        String::from_utf8_lossy(&push.stdout)
    );
    assert_eq!(
        grip::observation::fingerprint::inspect_complete(&source, NodeKind::File)
            .unwrap()
            .state,
        grip::observation::fingerprint::inspect_complete(&destination, NodeKind::File)
            .unwrap()
            .state
    );

    let collision_source_fixture = tempfile::tempdir_in(&destination_root).unwrap();
    let collision_destination_fixture = tempfile::tempdir_in(&source_root).unwrap();
    let collision_source = collision_source_fixture.path().join("source");
    let collision_destination = collision_destination_fixture.path().join("destination");
    fs::create_dir(&collision_source).unwrap();
    fs::write(collision_source.join("Readme"), b"one").unwrap();
    fs::write(collision_source.join("README"), b"two").unwrap();
    let collision_home = support::minimal_home(collision_source_fixture.path());
    support::write_registry(
        &collision_home,
        &[("tree", &collision_source, &collision_destination)],
    );
    let collision = support::command_with_grip_home(
        collision_source_fixture.path(),
        &collision_home,
        &["--output=json", "mapping", "inspect"],
    );
    let collision_records = support::json(&collision)["details"]["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|record| record["reason"] == "apfs_name_collision")
        .count();
    assert_eq!(collision_records, 2);
    assert!(!collision_destination.exists());
}
