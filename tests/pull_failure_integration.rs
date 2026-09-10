mod support;

use std::fs;

#[test]
fn pull_staging_failure_is_terminal_and_preserves_authority() {
    let (root, home, registry, state, selection, plan) = support::pull_execution_fixture();
    let source = root.path().join("source");
    let before_state = fs::read(home.path().join("state/state.json")).unwrap();
    let error = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        support::fail_mutation_at(grip::mutation::FaultPhase::BeforeStaging(0)),
    )
    .unwrap_err();
    let grip::GripError::MutationFailed(failure) = error else {
        panic!("expected journaled mutation failure");
    };
    assert_eq!(
        failure.plan.direction,
        Some(grip::mutation::model::MutationDirection::Pull)
    );
    assert_eq!(failure.plan.counts.failed, 1);
    assert_eq!(failure.baseline.outcome, "not_published");
    assert_eq!(fs::read_to_string(source).unwrap(), "accepted");
    assert_eq!(
        fs::read(home.path().join("state/state.json")).unwrap(),
        before_state
    );
}

#[test]
fn pull_post_publication_failure_reports_visible_source_without_accepting_it() {
    let (root, home, registry, state, selection, plan) = support::pull_execution_fixture();
    let source = root.path().join("source");
    let before_state = fs::read(home.path().join("state/state.json")).unwrap();
    let error = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        support::fail_mutation_at(grip::mutation::FaultPhase::AfterPayloadPublication(0)),
    )
    .unwrap_err();
    let grip::GripError::MutationFailed(failure) = error else {
        panic!("expected journaled mutation failure");
    };
    assert_eq!(failure.completion, "partial");
    assert_eq!(failure.plan.actions[0].milestones.publication, "visible");
    assert_eq!(fs::read_to_string(source).unwrap(), "destination change");
    assert_eq!(
        fs::read(home.path().join("state/state.json")).unwrap(),
        before_state
    );
    let status = support::project_command(root.path(), home.path(), &["--output=json", "status"]);
    assert!(status.status.success());
    assert!(
        support::json(&status)["details"]["attention_count"]
            .as_u64()
            .unwrap()
            > 0
    );
}

#[test]
fn pull_baseline_failure_retains_verified_payload_and_prior_generation() {
    let (root, home, registry, state, selection, plan) = support::pull_execution_fixture();
    let source = root.path().join("source");
    let error = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        support::fail_mutation_at(grip::mutation::FaultPhase::BeforeBaselinePublication),
    )
    .unwrap_err();
    let grip::GripError::MutationFailed(failure) = error else {
        panic!("expected journaled mutation failure");
    };
    assert_eq!(failure.reason, "baseline_publication_failure");
    assert_eq!(
        failure.baseline.authoritative_generation,
        state.accepted.generation
    );
    assert_eq!(fs::read_to_string(source).unwrap(), "destination change");
}

#[test]
#[allow(clippy::result_large_err)]
fn pull_revalidation_rejects_source_and_destination_drift_before_publication() {
    for drift_source in [true, false] {
        let (root, home, registry, state, selection, plan) = support::pull_execution_fixture();
        let source = root.path().join("source");
        let destination = root.path().join("destination");
        let result = grip::mutation::execution::execute_with_fault_hook(
            &home,
            &registry,
            &state,
            &selection,
            &plan,
            |phase| {
                if phase == grip::mutation::FaultPhase::BeforeActionRevalidation(0) {
                    if drift_source {
                        fs::write(&source, "concurrent source").unwrap();
                    } else {
                        fs::write(&destination, "concurrent destination").unwrap();
                    }
                }
                Ok(())
            },
        );
        let grip::GripError::MutationFailed(failure) = result.unwrap_err() else {
            panic!("expected journaled revalidation failure");
        };
        assert_eq!(failure.reason, "stale_pull_evidence");
        assert_eq!(failure.phase, "revalidation_failure");
        assert_eq!(failure.failed_action_index, Some(0));
        assert_eq!(failure.expected_source, plan.actions[0].expected_source);
        assert_eq!(
            failure.expected_destination,
            plan.actions[0].expected_destination
        );
        if drift_source {
            assert_ne!(failure.observed_source, failure.expected_source);
        } else {
            assert_ne!(failure.observed_destination, failure.expected_destination);
        }
        assert_eq!(
            failure.plan.actions[0].milestones.publication,
            "not_attempted"
        );
    }
}

#[test]
fn pull_rejects_registry_and_baseline_drift_before_operation_initialization() {
    for drift_registry in [true, false] {
        let (root, home, registry, state, selection, plan) = support::pull_execution_fixture();
        let operations = home.path().join("state/operations");
        let operation_count = fs::read_dir(&operations).unwrap().count();
        let path = if drift_registry {
            home.path().join("config.toml")
        } else {
            home.path().join("state/state.json")
        };
        let mut bytes = fs::read(&path).unwrap();
        bytes.push(b'\n');
        fs::write(&path, bytes).unwrap();
        let error = grip::mutation::execution::execute(&home, &registry, &state, &selection, &plan)
            .unwrap_err();
        assert!(matches!(
            error,
            grip::GripError::Mapping {
                ref operation,
                ref reason,
                ..
            } if operation == "pull" && reason == "stale_pull_evidence"
        ));
        assert_eq!(
            fs::read_to_string(root.path().join("source")).unwrap(),
            "accepted"
        );
        assert_eq!(fs::read_dir(operations).unwrap().count(), operation_count);
    }
}

#[test]
fn pull_prebaseline_fault_matrix_keeps_partial_work_unaccepted() {
    use grip::mutation::FaultPhase;

    let cases = [
        (FaultPhase::BeforeStaging(0), false, true),
        (FaultPhase::AfterStaging(0), false, true),
        (FaultPhase::BeforePayloadPublication(0), false, true),
        (
            FaultPhase::AfterMetadataProtectedFlagsCleared(0),
            true,
            true,
        ),
        (FaultPhase::AfterMetadataOwnership(0), true, true),
        (FaultPhase::AfterMetadataAcl(0), true, true),
        (FaultPhase::AfterMetadataExtendedAttributes(0), true, true),
        (FaultPhase::AfterMetadataPermissionMode(0), true, true),
        (FaultPhase::AfterMetadataModificationTime(0), true, true),
        (FaultPhase::AfterMetadataBsdFlags(0), true, true),
        (FaultPhase::AfterMetadataDurability(0), true, true),
        (FaultPhase::BeforeMetadataVerification(0), true, true),
        (FaultPhase::AfterMetadataVerification(0), true, true),
        (FaultPhase::AfterPayloadPublication(0), true, true),
        (FaultPhase::BeforeTargetVerification(0), true, true),
        (FaultPhase::AfterTargetVerification(0), true, true),
        (FaultPhase::AfterStagingCleanup(0), true, true),
        (FaultPhase::BeforeFinalObservation, true, true),
        (FaultPhase::BeforeFinalCoordination, true, true),
        (FaultPhase::BeforeBaselinePublication, true, true),
    ];
    for (phase, source_changed, _prior_payload_unavailable) in cases {
        let (root, home, registry, state, selection, plan) = support::pull_execution_fixture();
        let error = grip::mutation::execution::execute_with_fault_hook(
            &home,
            &registry,
            &state,
            &selection,
            &plan,
            support::fail_mutation_at(phase),
        )
        .unwrap_err();
        let grip::GripError::MutationFailed(failure) = error else {
            panic!("expected direction-neutral mutation failure for {phase:?}");
        };
        assert_eq!(
            failure.plan.direction,
            Some(grip::mutation::model::MutationDirection::Pull)
        );
        assert_eq!(failure.baseline.outcome, "not_published");
        assert_eq!(
            grip::state::publication::load(&home)
                .unwrap()
                .accepted
                .generation,
            state.accepted.generation
        );
        assert_eq!(
            fs::read_to_string(root.path().join("source")).unwrap(),
            if source_changed {
                "destination change"
            } else {
                "accepted"
            }
        );
    }
}

#[test]
fn first_pull_action_failure_leaves_later_actions_unattempted() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source_a = root.path().join("source-a");
    let source_b = root.path().join("source-b");
    let destination_a = root.path().join("destination-a");
    let destination_b = root.path().join("destination-b");
    fs::write(&source_a, "accepted-a").unwrap();
    fs::write(&source_b, "accepted-b").unwrap();
    support::write_descriptor(
        &metadata_dir,
        &[
            ("file", &source_a, &destination_a),
            ("file", &source_b, &destination_b),
        ],
    );
    assert!(
        support::project_command(root.path(), &metadata_dir, &["push"])
            .status
            .success()
    );
    fs::write(&destination_a, "changed-a").unwrap();
    fs::write(&destination_b, "changed-b").unwrap();
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
    let plan = grip::mutation::plan::build_for(
        grip::mutation::model::MutationDirection::Pull,
        grip::classification::model::ClassificationScope {
            kind: "all".into(),
            path_space: grip::observation::model::PathSpace::Source,
            selector: None,
            mapping_source: None,
        },
        records,
    )
    .unwrap();
    assert_eq!(plan.actions.len(), 2);
    let error = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        support::fail_push_at(grip::mutation::FaultPhase::BeforeStaging(0)),
    )
    .unwrap_err();
    let grip::GripError::MutationFailed(failure) = error else {
        panic!("expected journaled mutation failure");
    };
    assert_eq!(failure.plan.counts.failed, 1);
    assert_eq!(failure.plan.counts.unattempted, 1);
    assert_eq!(fs::read_to_string(source_b).unwrap(), "accepted-b");
}

#[test]
#[allow(clippy::result_large_err)]
fn pull_state_publication_faults_report_the_authoritative_generation() {
    for (fault, visible) in [
        (
            grip::state::publication::PublicationFault::BeforeV2StateRename,
            false,
        ),
        (
            grip::state::publication::PublicationFault::AfterV2StateRename,
            true,
        ),
    ] {
        let (_root, home, registry, state, selection, plan) = support::pull_execution_fixture();
        let error = grip::mutation::execution::execute_with_faults(
            &home,
            &registry,
            &state,
            &selection,
            &plan,
            |_| Ok(()),
            Some(fault),
        )
        .unwrap_err();
        let grip::GripError::MutationFailed(failure) = error else {
            panic!("expected journaled state-publication failure");
        };
        assert_eq!(failure.baseline.publication_visible, visible);
        assert_eq!(failure.baseline.durability_confirmed, !visible);
        let expected = if visible {
            state.accepted.generation.map(|generation| generation + 1)
        } else {
            state.accepted.generation
        };
        assert_eq!(failure.baseline.authoritative_generation, expected);
        assert_eq!(
            grip::state::publication::load(&home)
                .unwrap()
                .accepted
                .generation,
            expected
        );
    }
}
