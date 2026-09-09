mod support;

use grip::state::lock::PublicationLock;
use std::fs;
use std::os::unix::fs::PermissionsExt;

fn file_fixture() -> (
    tempfile::TempDir,
    std::path::PathBuf,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "same").unwrap();
    fs::write(&destination, "same").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
    (root, metadata_dir, source, destination)
}

#[test]
fn initial_refresh_noop_and_empty_scope_have_expected_generation_behavior() {
    let (root, metadata_dir, source, destination) = file_fixture();
    let initial = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(initial.status.code(), Some(0));
    assert_eq!(support::json(&initial)["details"]["generation"], 0);

    fs::write(&source, "changed").unwrap();
    fs::write(&destination, "changed").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    let prior = fs::read(metadata_dir.join("state/state.json")).unwrap();
    let refreshed = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(refreshed.status.code(), Some(0));
    assert_eq!(support::json(&refreshed)["details"]["generation"], 1);
    assert_eq!(
        fs::read(metadata_dir.join("state/recovery/generation-0/state.json")).unwrap(),
        prior
    );

    let current = fs::read(metadata_dir.join("state/state.json")).unwrap();
    let noop = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(support::json(&noop)["details"]["result"], "already_current");
    assert_eq!(
        fs::read(metadata_dir.join("state/state.json")).unwrap(),
        current
    );

    let empty_root = tempfile::tempdir().unwrap();
    let empty_home = support::initialize_project_metadata(empty_root.path());
    let empty = support::project_command(
        empty_root.path(),
        &empty_home,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(empty.status.code(), Some(0));
    assert_eq!(support::json(&empty)["details"]["selected_count"], 0);
    assert!(!empty_home.join("state").exists());
}

#[test]
fn scoped_acceptance_preserves_out_of_scope_and_pending_retirement_baselines() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&destination).unwrap();
    fs::write(source.join("a"), "a0").unwrap();
    fs::write(destination.join("a"), "a0").unwrap();
    fs::write(source.join("b"), "b0").unwrap();
    fs::write(destination.join("b"), "b0").unwrap();
    support::copy_tree_entry_metadata(&source, &destination);
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);
    assert!(
        support::project_command(root.path(), &metadata_dir, &["baseline", "accept"])
            .status
            .success()
    );

    fs::write(source.join("a"), "a1").unwrap();
    fs::write(destination.join("a"), "a1").unwrap();
    fs::write(source.join("b"), "b1").unwrap();
    fs::write(destination.join("b"), "b1").unwrap();
    support::copy_complete_metadata(
        &source.join("a"),
        &destination.join("a"),
        grip::discovery::model::NodeKind::File,
    );
    support::copy_complete_metadata(
        &source.join("b"),
        &destination.join("b"),
        grip::discovery::model::NodeKind::File,
    );
    let scoped = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "baseline", "accept", "source/a"],
    );
    assert_eq!(scoped.status.code(), Some(0));
    assert_eq!(support::json(&scoped)["details"]["selected_count"], 1);
    let status =
        support::project_command(root.path(), &metadata_dir, &["--output", "json", "status"]);
    let records = support::json(&status)["details"]["records"]
        .as_array()
        .unwrap()
        .clone();
    assert!(
        records
            .iter()
            .any(|record| record["classification"] == "synchronized")
    );
    assert!(
        records
            .iter()
            .any(|record| record["classification"] == "converged_two_sided_change")
    );

    support::write_descriptor(&metadata_dir, &[]);
    let before = fs::read(metadata_dir.join("state/state.json")).unwrap();
    let no_current_scope = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(no_current_scope.status.code(), Some(10));
    assert_eq!(
        fs::read(metadata_dir.join("state/state.json")).unwrap(),
        before
    );
}

#[test]
fn ineligible_request_reports_every_record_and_changes_nothing() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&destination).unwrap();
    fs::write(source.join("a"), "source-a").unwrap();
    fs::write(destination.join("a"), "destination-a").unwrap();
    fs::write(source.join("b"), "source-b").unwrap();
    fs::write(destination.join("b"), "destination-b").unwrap();
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);
    let before = support::snapshot(root.path());
    let rejected = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(rejected.status.code(), Some(10));
    let result = support::json(&rejected);
    assert_eq!(result["details"]["reason"], "baseline_not_acceptable");
    assert_eq!(result["details"]["records"].as_array().unwrap().len(), 2);
    assert_eq!(support::snapshot(root.path()), before);
}

#[test]
fn state_and_registry_contention_are_nonblocking_and_retryable() {
    let (root, metadata_dir, _, _) = file_fixture();
    let state_dir = metadata_dir.join("state");
    fs::create_dir(&state_dir).unwrap();
    fs::set_permissions(
        &state_dir,
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .unwrap();
    let home = support::project_home(&metadata_dir);
    let state_lock = PublicationLock::acquire(
        &grip::state::lock::project_lock_path(&home, "state.lock").unwrap(),
    )
    .unwrap();
    let blocked = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(blocked.status.code(), Some(13));
    assert_eq!(
        support::json(&blocked)["details"]["reason"],
        "state_contention"
    );
    drop(state_lock);

    let registry_lock = PublicationLock::acquire(
        &grip::state::lock::project_lock_path(&home, "registry.lock").unwrap(),
    )
    .unwrap();
    let registry_blocked = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(registry_blocked.status.code(), Some(20));
    assert_eq!(
        support::json(&registry_blocked)["details"]["reason"],
        "registry_contention"
    );
    drop(registry_lock);
    assert!(
        support::project_command(root.path(), &metadata_dir, &["baseline", "accept"])
            .status
            .success()
    );
}

#[test]
fn changed_acceptance_uses_outer_mutation_lock_but_no_op_stays_lock_free() {
    let (root, metadata_dir, _, _) = file_fixture();
    let selected = support::project_home(&metadata_dir);
    let held = grip::state::mutation_lock::MutationLock::acquire(&selected, "push").unwrap();
    let blocked = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(blocked.status.code(), Some(13));
    assert_eq!(
        support::json(&blocked)["details"]["reason"],
        "state_contention"
    );
    assert_eq!(
        support::json(&blocked)["details"]["owner"]["operation"],
        "push"
    );
    drop(held);
    assert!(
        support::project_command(root.path(), &metadata_dir, &["baseline", "accept"])
            .status
            .success()
    );

    let held = grip::state::mutation_lock::MutationLock::acquire(&selected, "push").unwrap();
    let no_op = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output", "json", "baseline", "accept"],
    );
    assert!(no_op.status.success());
    assert_eq!(
        support::json(&no_op)["details"]["result"],
        "already_current"
    );
    drop(held);
}

fn direct_accept_with_hook<F>(
    metadata_dir: &std::path::Path,
    after_initial: F,
) -> Result<grip::result::CommandOutcome, Box<grip::GripError>>
where
    F: FnOnce(),
{
    let home = support::project_home(metadata_dir);
    grip::execute_baseline_accept_with_hook(
        &home,
        &grip::cli::InspectionArgs {
            destination: false,
            path: None,
        },
        after_initial,
    )
    .map_err(Box::new)
}

#[test]
fn acceptance_rejects_content_mode_and_membership_drift_before_publication() {
    for mutation in ["content", "mode"] {
        let (root, metadata_dir, source, destination) = file_fixture();
        let before_registry = fs::read(metadata_dir.join("config.toml")).unwrap();
        let error = direct_accept_with_hook(&metadata_dir, || match mutation {
            "content" => {
                fs::write(&source, "new equivalent").unwrap();
                fs::write(&destination, "new equivalent").unwrap();
            }
            "mode" => {
                fs::set_permissions(&source, fs::Permissions::from_mode(0o600)).unwrap();
                fs::set_permissions(&destination, fs::Permissions::from_mode(0o600)).unwrap();
            }
            _ => unreachable!(),
        })
        .unwrap_err();
        let outcome = grip::result::CommandOutcome::failure(&error);
        assert_eq!(outcome.category.exit_code(), 20);
        assert_eq!(outcome.details["reason"], "stale_baseline_evidence");
        assert!(!metadata_dir.join("state/state.json").exists());
        assert_eq!(
            fs::read(metadata_dir.join("config.toml")).unwrap(),
            before_registry
        );
        drop(root);
    }

    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&destination).unwrap();
    fs::write(source.join("entry"), "same").unwrap();
    fs::write(destination.join("entry"), "same").unwrap();
    support::copy_tree_entry_metadata(&source, &destination);
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);
    let error = direct_accept_with_hook(&metadata_dir, || {
        fs::write(source.join(".gripignore"), "entry\n").unwrap();
    })
    .unwrap_err();
    let outcome = grip::result::CommandOutcome::failure(&error);
    assert_eq!(outcome.details["reason"], "stale_baseline_evidence");
    assert!(!metadata_dir.join("state/state.json").exists());
}

#[test]
fn acceptance_rejects_registry_and_state_drift_with_stable_reasons() {
    let (_root, metadata_dir, _, _) = file_fixture();
    let error = direct_accept_with_hook(&metadata_dir, || {
        let mut registry = fs::read_to_string(metadata_dir.join("config.toml")).unwrap();
        registry.push_str("\n# concurrent rewrite\n");
        fs::write(metadata_dir.join("config.toml"), registry).unwrap();
    })
    .unwrap_err();
    let outcome = grip::result::CommandOutcome::failure(&error);
    assert_eq!(outcome.details["reason"], "stale_registry_evidence");
    assert!(!metadata_dir.join("state/state.json").exists());

    let (root, metadata_dir, _, _) = file_fixture();
    assert!(
        support::project_command(root.path(), &metadata_dir, &["baseline", "accept"])
            .status
            .success()
    );
    let concurrent_state = fs::read(metadata_dir.join("state/state.json")).unwrap();
    fs::remove_dir_all(metadata_dir.join("state")).unwrap();
    let error = direct_accept_with_hook(&metadata_dir, || {
        let state_dir = metadata_dir.join("state");
        fs::create_dir(&state_dir).unwrap();
        fs::set_permissions(&state_dir, fs::Permissions::from_mode(0o700)).unwrap();
        let path = state_dir.join("state.json");
        fs::write(&path, &concurrent_state).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    })
    .unwrap_err();
    let accepted = fs::read(metadata_dir.join("state/state.json")).unwrap();
    let outcome = grip::result::CommandOutcome::failure(&error);
    assert_eq!(outcome.details["reason"], "stale_state_evidence");
    assert_eq!(
        fs::read(metadata_dir.join("state/state.json")).unwrap(),
        accepted
    );
}

#[test]
fn push_publishes_one_scoped_generation_and_preserves_out_of_scope_baselines() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source_a = root.path().join("source-a");
    let source_b = root.path().join("source-b");
    let destination_a = root.path().join("destination-a");
    let destination_b = root.path().join("destination-b");
    for path in [&source_a, &source_b, &destination_a, &destination_b] {
        fs::write(path, "accepted").unwrap();
    }
    support::copy_complete_metadata(
        &source_a,
        &destination_a,
        grip::discovery::model::NodeKind::File,
    );
    support::copy_complete_metadata(
        &source_b,
        &destination_b,
        grip::discovery::model::NodeKind::File,
    );
    support::write_descriptor(
        &metadata_dir,
        &[
            ("file", &source_a, &destination_a),
            ("file", &source_b, &destination_b),
        ],
    );
    let accepted = support::project_command(root.path(), &metadata_dir, &["baseline", "accept"]);
    assert!(accepted.status.success());
    fs::write(&source_a, "pushed-a").unwrap();
    fs::write(&source_b, "pending-b").unwrap();
    let pushed = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "push", "source-a"],
    );
    assert!(pushed.status.success());
    assert_eq!(
        support::json(&pushed)["details"]["baseline"]["published_generation"],
        1
    );
    assert_eq!(fs::read_to_string(destination_a).unwrap(), "pushed-a");
    assert_eq!(fs::read_to_string(destination_b).unwrap(), "accepted");
    let home = support::project_home(&metadata_dir);
    let state = grip::state::publication::load(&home).unwrap();
    assert_eq!(state.accepted.generation, Some(1));
    assert_eq!(state.accepted.complete_baselines.len(), 2);
}
