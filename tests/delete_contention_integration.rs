mod support;
use std::fs;
use std::os::unix::fs::PermissionsExt;

#[test]
fn deletion_preview_is_lock_free_and_execute_reports_contention() {
    let (root, grip_home, source, destination) = support::accepted_file_fixture();
    fs::remove_file(&source).unwrap();
    let home = grip::home::select(Some(grip_home.clone().into_os_string()), None).unwrap();
    let _guard = grip::state::mutation_lock::MutationLock::acquire(&home, "test-owner").unwrap();
    let preview = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["delete", "-n", "--source", source.to_str().unwrap()],
    );
    assert!(preview.status.success());
    let execute = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["delete", "--source", source.to_str().unwrap()],
    );
    assert_eq!(execute.status.code(), Some(13));
    assert!(destination.exists());
}

#[test]
fn deletion_execution_overwrites_released_stale_lock_metadata() {
    let (root, grip_home, source, destination) = support::accepted_file_fixture();
    fs::remove_file(&source).unwrap();
    let lock_path = grip_home.join(".mutation.lock");
    fs::write(&lock_path, "stale-owner").unwrap();
    std::fs::set_permissions(&lock_path, fs::Permissions::from_mode(0o600)).unwrap();

    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output=json",
            "delete",
            "--source",
            source.to_str().unwrap(),
        ],
    );

    assert!(output.status.success());
    assert!(!destination.exists());
}

#[test]
fn deletion_rejects_registry_state_and_target_drift_before_removal() {
    let (_root, home, registry, state, selection, plan) = support::deletion_execution_fixture();
    let target = plan.actions[0].target.clone();
    let mut registry_bytes = fs::read(home.path().join("config.toml")).unwrap();
    registry_bytes.extend_from_slice(b"\n");
    fs::write(home.path().join("config.toml"), registry_bytes).unwrap();
    assert!(grip::delete::execution::execute(&home, &registry, &state, &selection, &plan).is_err());
    assert!(target.exists());

    let (_root, home, registry, state, selection, plan) = support::deletion_execution_fixture();
    let target = plan.actions[0].target.clone();
    let state_bytes = fs::read(home.path().join("state/state.json")).unwrap();
    let replacement = home.path().join("state/replacement.json");
    fs::write(&replacement, state_bytes).unwrap();
    fs::rename(replacement, home.path().join("state/state.json")).unwrap();
    assert!(grip::delete::execution::execute(&home, &registry, &state, &selection, &plan).is_err());
    assert!(target.exists());

    let (_root, home, registry, state, selection, plan) = support::deletion_execution_fixture();
    let target = plan.actions[0].target.clone();
    fs::write(&target, "replacement").unwrap();
    assert!(grip::delete::execution::execute(&home, &registry, &state, &selection, &plan).is_err());
    assert_eq!(fs::read_to_string(target).unwrap(), "replacement");
}

#[test]
fn deletion_revalidates_each_action_and_stops_when_a_child_is_recreated() {
    let (_root, grip_home, source, destination) = support::accepted_tree_fixture();
    fs::remove_dir_all(&source).unwrap();
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    let registry = grip::registry::publication::load(&home, false).unwrap();
    let state = grip::state::publication::load(&home).unwrap();
    let selection = grip::observation::model::resolve_selection(
        &registry,
        &state.accepted,
        Some(&source),
        grip::observation::model::PathSpace::Source,
        "delete",
    )
    .unwrap();
    let observed =
        grip::observation::inspect(&home, &registry, &state.accepted, &selection).unwrap();
    let records = observed
        .values()
        .map(|entry| {
            grip::classification::classify(entry, state.accepted.baselines.get(&entry.identity))
        })
        .collect();
    let plan = grip::delete::plan::build(
        grip::delete::model::DeletionAuthority::Source,
        grip::classification::model::ClassificationScope {
            kind: "subtree".into(),
            path_space: grip::observation::model::PathSpace::Source,
            selector: Some(grip::discovery::model::SafePath::from_path(&source)),
            mapping_source: Some(source.display().to_string()),
        },
        records,
    )
    .unwrap();
    assert!(plan.actions.len() > 1);
    let recreated = destination.join("nested/recreated");
    let mut hook = |index| {
        if index == plan.actions.len() - 1 {
            fs::write(&recreated, "concurrent").unwrap();
        }
    };
    let error = grip::delete::execution::execute_with_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        None,
        Some(&mut hook),
    )
    .unwrap_err();
    assert!(recreated.exists());
    assert!(matches!(error, grip::GripError::OperationLifecycle { .. }));
    assert_eq!(
        grip::state::publication::load(&home)
            .unwrap()
            .accepted
            .generation,
        state.accepted.generation
    );
}
