#![allow(clippy::result_large_err)]

mod support;

use grip::mapping::MappingKind;
use grip::observation::model::{EntryIdentity, ResolvedMapping};
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};

fn execution_fixture() -> (
    tempfile::TempDir,
    grip::project::ProjectPaths,
    grip::registry::publication::RegistrySnapshot,
    grip::state::publication::StateSnapshot,
    grip::observation::model::Selection,
    grip::push::model::PushPlan,
) {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "planned").unwrap();
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
    let home = support::project_home(&metadata_dir);
    let registry = grip::registry::publication::load(&home, false).unwrap();
    let state = grip::state::publication::load(&home).unwrap();
    let selection = grip::observation::model::Selection::All;
    let observed =
        grip::observation::inspect(&home, &registry, &state.accepted, &selection).unwrap();
    let records = observed
        .values()
        .map(|entry| grip::classification::classify_accepted(entry, &state.accepted))
        .collect();
    let plan = grip::push::plan::build_with_parent_requirements(
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

#[test]
fn staged_addition_preserves_content_mode_and_unmanaged_neighbor() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    let neighbor = root.path().join("neighbor");
    fs::write(&source, "new payload").unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o640)).unwrap();
    fs::write(&neighbor, "unmanaged").unwrap();
    let expected = support::supported_file_state(&source);
    let mut staged = grip::push::filesystem::stage_file(&source, &destination, &expected).unwrap();
    grip::push::filesystem::publish_addition(&mut staged, &destination).unwrap();
    grip::push::filesystem::verify_destination(&destination, &expected).unwrap();
    assert_eq!(fs::read_to_string(&destination).unwrap(), "new payload");
    assert_eq!(fs::read_to_string(&neighbor).unwrap(), "unmanaged");
    assert_eq!(
        fs::metadata(&destination).unwrap().permissions().mode() & 0o7777,
        0o640
    );
}

#[test]
fn replacement_preserves_verified_private_recovery_before_publication() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "new payload").unwrap();
    fs::write(&destination, "old payload").unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o640)).unwrap();
    fs::set_permissions(&destination, fs::Permissions::from_mode(0o600)).unwrap();
    let expected_source = support::supported_file_state(&source);
    let expected_destination = support::supported_file_state(&destination);
    let home = support::project_home(&metadata_dir);
    let receipt =
        grip::operation::publication::initialize(&home, &support::test_push_plan(1)).unwrap();
    let identity = EntryIdentity::new(
        ResolvedMapping {
            kind: MappingKind::File,
            source: source.clone(),
            destination: destination.clone(),
        },
        Vec::new(),
    )
    .unwrap();
    let recovery =
        grip::push::recovery::preserve(&receipt, 0, &identity, &destination, &expected_destination)
            .unwrap();
    assert_eq!(recovery.relative_ref, "recovery/00000000/payload");
    let payload = receipt.directory().join(&recovery.relative_ref);
    assert_eq!(fs::read_to_string(payload).unwrap(), "old payload");

    let mut staged =
        grip::push::filesystem::stage_file(&source, &destination, &expected_source).unwrap();
    grip::push::filesystem::publish_replacement(&mut staged, &destination).unwrap();
    grip::push::filesystem::verify_destination(&destination, &expected_source).unwrap();
}

#[test]
fn directory_creation_is_explicit_private_and_rejects_concurrent_appearance() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let created = root.path().join("created");
    grip::push::filesystem::create_directory(&created).unwrap();
    assert_eq!(
        fs::metadata(&created).unwrap().permissions().mode() & 0o7777,
        0o700
    );
    assert!(grip::push::filesystem::create_directory(&created).is_err());
}

#[test]
fn staging_rejects_symlink_and_hard_link_sources() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let source = root.path().join("source");
    let alias = root.path().join("alias");
    let destination = root.path().join("destination");
    fs::write(&source, "payload").unwrap();
    let expected = support::supported_file_state(&source);
    symlink(&source, &alias).unwrap();
    assert!(grip::push::filesystem::stage_file(&alias, &destination, &expected).is_err());
    fs::remove_file(&alias).unwrap();
    fs::hard_link(&source, &alias).unwrap();
    assert!(grip::push::filesystem::stage_file(&alias, &destination, &expected).is_err());
}

#[test]
fn addition_does_not_overwrite_a_concurrently_appearing_destination() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "planned").unwrap();
    let expected = support::supported_file_state(&source);
    let mut staged = grip::push::filesystem::stage_file(&source, &destination, &expected).unwrap();
    fs::write(&destination, "concurrent").unwrap();
    assert!(grip::push::filesystem::publish_addition(&mut staged, &destination).is_err());
    assert_eq!(fs::read_to_string(destination).unwrap(), "concurrent");
}

#[test]
fn lock_held_plan_and_per_action_revalidation_reject_drift_before_mutation() {
    let (root, home, registry, state, selection, plan) = execution_fixture();
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    let first = grip::push::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        |phase| {
            if phase == grip::push::FaultPhase::AfterMutationLock {
                fs::write(&source, "changed after preflight").unwrap();
            }
            Ok(())
        },
    );
    assert!(first.is_err());
    assert!(!destination.exists());

    let (root, home, registry, state, selection, plan) = execution_fixture();
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    let second = grip::push::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        |phase| {
            if phase == grip::push::FaultPhase::BeforeActionRevalidation(0) {
                fs::write(&source, "changed before action").unwrap();
            }
            Ok(())
        },
    );
    let grip::GripError::PushFailed(failure) = second.unwrap_err() else {
        panic!("expected journaled action failure");
    };
    assert_eq!(failure.plan.counts.failed, 1);
    assert!(!destination.exists());
}

#[test]
fn cli_push_adds_then_replaces_with_recovery_and_matching_baseline() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "first").unwrap();
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);

    let first = support::project_command(root.path(), &metadata_dir, &["--output=json", "push"]);
    assert!(
        first.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );
    let first_json = support::json(&first);
    assert_eq!(first_json["details"]["result"], "applied");
    assert_eq!(first_json["details"]["baseline"]["published_generation"], 0);
    assert_eq!(fs::read_to_string(&destination).unwrap(), "first");

    fs::write(&source, "second").unwrap();
    let second = support::project_command(root.path(), &metadata_dir, &["--output=json", "push"]);
    assert!(
        second.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&second.stdout),
        String::from_utf8_lossy(&second.stderr)
    );
    let second_json = support::json(&second);
    assert_eq!(
        second_json["details"]["baseline"]["published_generation"],
        1
    );
    assert_eq!(fs::read_to_string(&destination).unwrap(), "second");
    let operation_id = second_json["details"]["operation_record"]["id"]
        .as_str()
        .unwrap();
    assert_eq!(
        fs::read_to_string(
            metadata_dir
                .join("state/operations")
                .join(operation_id)
                .join("recovery/00000000/payload")
        )
        .unwrap(),
        "first"
    );
    let status = support::project_command(root.path(), &metadata_dir, &["--output=json", "status"]);
    assert!(status.status.success());
    assert_eq!(support::json(&status)["details"]["attention_count"], 0);
}

#[test]
fn cli_push_creates_nested_tree_directories_in_dependency_order() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source-tree");
    let destination = root.path().join("destination-tree");
    fs::create_dir(&source).unwrap();
    fs::create_dir(source.join("nested")).unwrap();
    fs::write(source.join("nested/file"), "payload").unwrap();
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);
    let output = support::project_command(root.path(), &metadata_dir, &["--output=json", "push"]);
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(destination.join("nested/file")).unwrap(),
        "payload"
    );
    let actions = support::json(&output)["details"]["actions"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(actions[0]["kind"], "create_parent_directory");
    assert_eq!(actions[1]["kind"], "create_directory");
    assert_eq!(actions[2]["kind"], "add_file");
    assert_eq!(actions[1]["dependencies"], serde_json::json!([0]));
    assert_eq!(actions[2]["dependencies"], serde_json::json!([1]));
}
