mod support;

use std::fs;

#[test]
fn pull_recovery_metadata_binds_verified_prior_source_to_action() {
    let (root, home, registry, state, selection, plan) = support::pull_execution_fixture();
    let source = root.path().join("source");
    let expected_prior = support::supported_file_state(&source);
    let success =
        grip::mutation::execution::execute(&home, &registry, &state, &selection, &plan).unwrap();
    let directory = home
        .path()
        .join("state/operations")
        .join(&success.operation_id)
        .join("recovery/00000000");
    let manifest = grip::recovery::model::decode_manifest_v2(
        &fs::read(directory.join("manifest.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        manifest.payload.origin_operation.as_deref(),
        Some(success.operation_id.as_str())
    );
    assert_eq!(
        manifest.payload.prior_evidence,
        serde_json::to_value(expected_prior).unwrap()
    );
    assert_eq!(manifest.payload.private_ref, "payload");
    assert_eq!(
        fs::read_to_string(directory.join("payload")).unwrap(),
        "accepted"
    );
}

#[test]
fn pull_recovery_failure_leaves_source_and_baseline_unchanged() {
    let (root, home, registry, state, selection, plan) = support::pull_execution_fixture();
    let source = root.path().join("source");
    let state_before = fs::read(home.path().join("state/state.json")).unwrap();
    let result = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        support::fail_mutation_at(grip::mutation::FaultPhase::BeforeRecovery(0)),
    );
    assert!(result.is_err());
    assert_eq!(fs::read_to_string(source).unwrap(), "accepted");
    assert_eq!(
        fs::read(home.path().join("state/state.json")).unwrap(),
        state_before
    );
}

#[test]
#[allow(clippy::result_large_err)]
fn pull_recovery_rejects_source_drift_before_preservation_completes() {
    let (root, home, registry, state, selection, plan) = support::pull_execution_fixture();
    let source = root.path().join("source");
    let state_before = fs::read(home.path().join("state/state.json")).unwrap();
    let error = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        |phase| {
            if phase == grip::mutation::FaultPhase::BeforeRecovery(0) {
                fs::write(&source, "concurrent source drift").unwrap();
            }
            Ok(())
        },
    )
    .unwrap_err();
    let grip::GripError::MutationFailed(failure) = error else {
        panic!("expected direction-neutral recovery failure");
    };
    assert_eq!(failure.reason, "recovery_failure");
    assert_eq!(failure.phase, "recovery_failure");
    assert_eq!(failure.plan.actions[0].milestones.recovery, "failed");
    assert_eq!(
        fs::read_to_string(source).unwrap(),
        "concurrent source drift"
    );
    assert_eq!(
        fs::read(home.path().join("state/state.json")).unwrap(),
        state_before
    );
}
