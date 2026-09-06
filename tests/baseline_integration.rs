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
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "same").unwrap();
    fs::write(&destination, "same").unwrap();
    support::write_registry(&grip_home, &[("file", &source, &destination)]);
    (root, grip_home, source, destination)
}

#[test]
fn initial_refresh_noop_and_empty_scope_have_expected_generation_behavior() {
    let (root, grip_home, source, destination) = file_fixture();
    let initial = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(initial.status.code(), Some(0));
    assert_eq!(support::json(&initial)["details"]["generation"], 0);

    fs::write(&source, "changed").unwrap();
    fs::write(&destination, "changed").unwrap();
    let prior = fs::read(grip_home.join("state/state.json")).unwrap();
    let refreshed = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(refreshed.status.code(), Some(0));
    assert_eq!(support::json(&refreshed)["details"]["generation"], 1);
    assert_eq!(
        fs::read(grip_home.join("state/recovery/generation-0/state.json")).unwrap(),
        prior
    );

    let current = fs::read(grip_home.join("state/state.json")).unwrap();
    let noop = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(support::json(&noop)["details"]["result"], "already_current");
    assert_eq!(
        fs::read(grip_home.join("state/state.json")).unwrap(),
        current
    );

    let empty_root = tempfile::tempdir().unwrap();
    let empty_home = support::minimal_home(empty_root.path());
    let empty = support::command_with_grip_home(
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
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&destination).unwrap();
    fs::write(source.join("a"), "a0").unwrap();
    fs::write(destination.join("a"), "a0").unwrap();
    fs::write(source.join("b"), "b0").unwrap();
    fs::write(destination.join("b"), "b0").unwrap();
    support::write_registry(&grip_home, &[("tree", &source, &destination)]);
    assert!(
        support::command_with_grip_home(root.path(), &grip_home, &["baseline", "accept"])
            .status
            .success()
    );

    fs::write(source.join("a"), "a1").unwrap();
    fs::write(destination.join("a"), "a1").unwrap();
    fs::write(source.join("b"), "b1").unwrap();
    fs::write(destination.join("b"), "b1").unwrap();
    let scoped = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output",
            "json",
            "baseline",
            "accept",
            source.join("a").to_str().unwrap(),
        ],
    );
    assert_eq!(scoped.status.code(), Some(0));
    assert_eq!(support::json(&scoped)["details"]["selected_count"], 1);
    let status =
        support::command_with_grip_home(root.path(), &grip_home, &["--output", "json", "status"]);
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

    support::write_registry(&grip_home, &[]);
    let before = fs::read(grip_home.join("state/state.json")).unwrap();
    let no_current_scope = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(no_current_scope.status.code(), Some(10));
    assert_eq!(
        fs::read(grip_home.join("state/state.json")).unwrap(),
        before
    );
}

#[test]
fn ineligible_request_reports_every_record_and_changes_nothing() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&destination).unwrap();
    fs::write(source.join("a"), "source-a").unwrap();
    fs::write(destination.join("a"), "destination-a").unwrap();
    fs::write(source.join("b"), "source-b").unwrap();
    fs::write(destination.join("b"), "destination-b").unwrap();
    support::write_registry(&grip_home, &[("tree", &source, &destination)]);
    let before = support::snapshot(root.path());
    let rejected = support::command_with_grip_home(
        root.path(),
        &grip_home,
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
    let (root, grip_home, _, _) = file_fixture();
    let state_dir = grip_home.join("state");
    fs::create_dir(&state_dir).unwrap();
    fs::set_permissions(
        &state_dir,
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .unwrap();
    let state_lock = PublicationLock::acquire(&state_dir.join("state.lock")).unwrap();
    let blocked = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(blocked.status.code(), Some(13));
    assert_eq!(
        support::json(&blocked)["details"]["reason"],
        "state_contention"
    );
    drop(state_lock);

    let registry_lock = PublicationLock::acquire(&grip_home.join(".registry.lock")).unwrap();
    let registry_blocked = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output", "json", "baseline", "accept"],
    );
    assert_eq!(registry_blocked.status.code(), Some(20));
    assert_eq!(
        support::json(&registry_blocked)["details"]["reason"],
        "registry_contention"
    );
    drop(registry_lock);
    assert!(
        support::command_with_grip_home(root.path(), &grip_home, &["baseline", "accept"])
            .status
            .success()
    );
}

fn direct_accept_with_hook<F>(
    grip_home: &std::path::Path,
    after_initial: F,
) -> Result<grip::result::CommandOutcome, Box<grip::GripError>>
where
    F: FnOnce(),
{
    let home = grip::home::select(Some(grip_home.to_owned().into_os_string()), None).unwrap();
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
        let (root, grip_home, source, destination) = file_fixture();
        let before_registry = fs::read(grip_home.join("config.toml")).unwrap();
        let error = direct_accept_with_hook(&grip_home, || match mutation {
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
        assert!(!grip_home.join("state/state.json").exists());
        assert_eq!(
            fs::read(grip_home.join("config.toml")).unwrap(),
            before_registry
        );
        drop(root);
    }

    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&destination).unwrap();
    fs::write(source.join("entry"), "same").unwrap();
    fs::write(destination.join("entry"), "same").unwrap();
    support::write_registry(&grip_home, &[("tree", &source, &destination)]);
    let error = direct_accept_with_hook(&grip_home, || {
        fs::write(source.join(".gripignore"), "entry\n").unwrap();
    })
    .unwrap_err();
    let outcome = grip::result::CommandOutcome::failure(&error);
    assert_eq!(outcome.details["reason"], "stale_baseline_evidence");
    assert!(!grip_home.join("state/state.json").exists());
}

#[test]
fn acceptance_rejects_registry_and_state_drift_with_stable_reasons() {
    let (_root, grip_home, _, _) = file_fixture();
    let error = direct_accept_with_hook(&grip_home, || {
        let mut registry = fs::read_to_string(grip_home.join("config.toml")).unwrap();
        registry.push_str("\n# concurrent rewrite\n");
        fs::write(grip_home.join("config.toml"), registry).unwrap();
    })
    .unwrap_err();
    let outcome = grip::result::CommandOutcome::failure(&error);
    assert_eq!(outcome.details["reason"], "stale_registry_evidence");
    assert!(!grip_home.join("state/state.json").exists());

    let (_root, grip_home, _, _) = file_fixture();
    let error = direct_accept_with_hook(&grip_home, || {
        let home = grip::home::select(Some(grip_home.clone().into_os_string()), None).unwrap();
        grip::state::publication::publish(&home, &grip::state::StateEnvelopeV1::new(0)).unwrap();
    })
    .unwrap_err();
    let accepted = fs::read(grip_home.join("state/state.json")).unwrap();
    let outcome = grip::result::CommandOutcome::failure(&error);
    assert_eq!(outcome.details["reason"], "stale_state_evidence");
    assert_eq!(
        fs::read(grip_home.join("state/state.json")).unwrap(),
        accepted
    );
}
