mod support;

use std::fs;

#[test]
fn sync_uses_the_shared_writer_lock_and_reports_owner() {
    let (root, grip_home, source, destination) = support::accepted_file_fixture();
    fs::write(&source, "source change").unwrap();
    let home = grip::home::select(Some(grip_home.clone().into_os_string()), None).unwrap();
    let guard = grip::state::mutation_lock::MutationLock::acquire(&home, "resolve").unwrap();
    let blocked =
        support::command_with_grip_home(root.path(), &grip_home, &["--output", "json", "sync"]);
    assert_eq!(blocked.status.code(), Some(13));
    let value = support::json(&blocked);
    assert_eq!(value["details"]["requested_operation"], "sync");
    assert_eq!(value["details"]["owner"]["operation"], "resolve");
    assert_eq!(fs::read_to_string(destination).unwrap(), "accepted");
    drop(guard);
    assert!(
        support::command_with_grip_home(root.path(), &grip_home, &["sync"])
            .status
            .success()
    );
}

#[test]
#[allow(clippy::result_large_err)]
fn sync_rejects_lock_held_plan_registry_and_state_drift_before_record_creation() {
    for drift in ["plan", "registry", "state"] {
        let (root, home, registry, state, selection, plan) = support::sync_execution_fixture();
        let operations = home.path().join("state/operations");
        let operation_count = fs::read_dir(&operations).unwrap().count();
        let result = grip::mutation::execution::execute_with_fault_hook(
            &home,
            &registry,
            &state,
            &selection,
            &plan,
            |phase| {
                if phase == grip::mutation::FaultPhase::AfterMutationLock {
                    match drift {
                        "plan" => {
                            fs::write(root.path().join("source"), "later source change").unwrap()
                        }
                        "registry" => {
                            let path = home.path().join("config.toml");
                            let mut bytes = fs::read(&path).unwrap();
                            bytes.extend_from_slice(b"\n");
                            fs::write(path, bytes).unwrap();
                        }
                        "state" => {
                            let path = home.path().join("state/state.json");
                            let mut bytes = fs::read(&path).unwrap();
                            bytes.extend_from_slice(b"\n");
                            fs::write(path, bytes).unwrap();
                        }
                        _ => unreachable!(),
                    }
                }
                Ok(())
            },
        );
        assert!(result.is_err(), "{drift} drift must fail");
        assert_eq!(
            fs::read_dir(&operations).unwrap().count(),
            operation_count,
            "{drift} drift must fail before operation initialization"
        );
        assert_eq!(
            fs::read_to_string(root.path().join("destination")).unwrap(),
            "accepted"
        );
    }
}

#[test]
#[allow(clippy::result_large_err)]
fn sync_holds_the_outer_mutation_lock_before_lock_held_revalidation() {
    let (_root, home, registry, state, selection, plan) = support::sync_execution_fixture();
    let result = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        |phase| {
            if phase == grip::mutation::FaultPhase::AfterMutationLock {
                let nested = grip::state::mutation_lock::MutationLock::acquire(&home, "resolve");
                assert!(matches!(
                    nested,
                    Err(grip::GripError::MutationContention { .. })
                ));
            }
            Ok(())
        },
    );
    assert!(result.is_ok());
}

#[test]
fn sync_preview_and_semantic_noop_remain_lock_free() {
    let (preview_root, preview_home, preview_source, preview_destination) =
        support::accepted_file_fixture();
    fs::write(&preview_source, "source change").unwrap();
    let preview_selected =
        grip::home::select(Some(preview_home.clone().into_os_string()), None).unwrap();
    let preview_guard =
        grip::state::mutation_lock::MutationLock::acquire(&preview_selected, "push").unwrap();
    let preview = support::command_with_grip_home(
        preview_root.path(),
        &preview_home,
        &["--output=json", "sync", "--dry-run"],
    );
    assert!(preview.status.success());
    assert_eq!(fs::read_to_string(preview_destination).unwrap(), "accepted");
    drop(preview_guard);

    let (noop_root, noop_home, _source, _destination) = support::accepted_file_fixture();
    let noop_selected = grip::home::select(Some(noop_home.clone().into_os_string()), None).unwrap();
    let noop_guard =
        grip::state::mutation_lock::MutationLock::acquire(&noop_selected, "resolve").unwrap();
    let noop =
        support::command_with_grip_home(noop_root.path(), &noop_home, &["--output=json", "sync"]);
    assert!(noop.status.success());
    assert_eq!(support::json(&noop)["details"]["result"], "no_op");
    drop(noop_guard);
}
