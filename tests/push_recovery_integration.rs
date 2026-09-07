mod support;

use grip::classification::model::ClassificationScope;
use grip::discovery::model::SafePath;
use grip::mapping::MappingKind;
use grip::observation::model::PathSpace;
use grip::observation::model::{EntryIdentity, MappingSnapshot};
use grip::operation::model::{
    ActionCheckpointEvidenceV1, ActionCheckpointPayloadV1, OperationPlanPayloadV1,
    OperationSummaryPayloadV1, decode,
};
use grip::push::model::{
    ActionEvidence, ActionKind, ActionStatus, PushAction, PushCounts, PushPlan,
};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

fn plan(action_count: usize) -> PushPlan {
    let actions = (0..action_count)
        .map(|index| PushAction {
            index,
            kind: ActionKind::AddFile,
            identity: None,
            dependent_identities: Vec::new(),
            source_path: Some(SafePath::from_path(Path::new("/source"))),
            destination: Path::new("/destination").to_path_buf(),
            destination_path: SafePath::from_path(Path::new("/destination")),
            expected_source: None,
            expected_destination: None,
            dependencies: Vec::new(),
            status: ActionStatus::Unattempted,
            milestones: ActionEvidence::default(),
            failure: None,
        })
        .collect::<Vec<_>>();
    PushPlan {
        direction: grip::mutation::model::MutationDirection::Push,
        plan_id: "a".repeat(64),
        scope: ClassificationScope {
            kind: "all".into(),
            path_space: PathSpace::Source,
            selector: None,
            mapping_source: None,
        },
        entries: Vec::new(),
        blockers: Vec::new(),
        counts: PushCounts {
            actionable: action_count,
            unattempted: action_count,
            ..PushCounts::default()
        },
        actions,
    }
}

fn evidence() -> ActionCheckpointEvidenceV1 {
    ActionCheckpointEvidenceV1 {
        revalidation: "passed".into(),
        recovery: "not_required".into(),
        recovery_ref: None,
        staging: "verified".into(),
        publication: "visible".into(),
        verification: "verified".into(),
        durability_confirmed: true,
    }
}

#[test]
fn operation_initialization_is_private_strict_and_sparse() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    let receipt = grip::operation::publication::initialize(&home, &plan(100)).unwrap();

    assert_eq!(
        fs::symlink_metadata(receipt.directory())
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        0o700
    );
    for name in ["plan.json", "operation.json"] {
        let metadata = fs::symlink_metadata(receipt.directory().join(name)).unwrap();
        assert!(metadata.is_file());
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o600);
    }
    assert_eq!(
        fs::read_dir(receipt.directory().join("actions"))
            .unwrap()
            .count(),
        0
    );
    decode::<OperationPlanPayloadV1>(&fs::read(receipt.directory().join("plan.json")).unwrap())
        .unwrap();
    decode::<OperationSummaryPayloadV1>(
        &fs::read(receipt.directory().join("operation.json")).unwrap(),
    )
    .unwrap();
}

#[test]
fn action_checkpoint_is_atomic_and_bounded_by_one_action() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    let receipt = grip::operation::publication::initialize(&home, &plan(1_000)).unwrap();
    receipt
        .checkpoint_action(999, "in_progress", evidence(), None)
        .unwrap();
    receipt
        .checkpoint_action(999, "completed", evidence(), None)
        .unwrap();
    let checkpoint = receipt.directory().join("actions/00000999.json");
    let bytes = fs::read(&checkpoint).unwrap();
    let decoded = decode::<ActionCheckpointPayloadV1>(&bytes).unwrap();
    assert_eq!(decoded.payload.action_index, 999);
    assert!(bytes.len() < 2_000);
    assert!(
        fs::read_dir(receipt.directory().join("actions"))
            .unwrap()
            .all(|entry| !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".record.tmp-"))
    );
}

#[test]
fn checkpoint_rejects_an_index_outside_the_immutable_plan() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    let receipt = grip::operation::publication::initialize(&home, &plan(1)).unwrap();
    assert!(
        receipt
            .checkpoint_action(1, "in_progress", evidence(), None)
            .is_err()
    );
}

#[test]
fn recovery_is_private_verified_contained_and_collision_safe() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    let outside = root.path().join("outside");
    fs::write(&source, "new").unwrap();
    fs::write(&destination, "prior payload").unwrap();
    fs::write(&outside, "untouched").unwrap();
    fs::set_permissions(&destination, fs::Permissions::from_mode(0o640)).unwrap();
    let expected = support::supported_file_state(&destination);
    let identity = EntryIdentity::new(
        MappingSnapshot {
            kind: MappingKind::File,
            source,
            destination: destination.clone(),
        },
        Vec::new(),
    )
    .unwrap();
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    let receipt = grip::operation::publication::initialize(&home, &plan(1)).unwrap();
    let entry =
        grip::push::recovery::preserve(&receipt, 0, &identity, &destination, &expected).unwrap();
    assert_eq!(entry.relative_ref, "recovery/00000000/payload");
    let recovery = receipt.directory().join("recovery/00000000");
    assert!(recovery.starts_with(receipt.directory()));
    assert_eq!(
        fs::read(recovery.join("payload")).unwrap(),
        b"prior payload"
    );
    assert_eq!(
        fs::metadata(recovery.join("payload"))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        0o600
    );
    assert_eq!(
        fs::metadata(&recovery).unwrap().permissions().mode() & 0o7777,
        0o700
    );
    let metadata: grip::push::recovery::RecoveryMetadataV1 =
        serde_json::from_slice(&fs::read(recovery.join("metadata.json")).unwrap()).unwrap();
    assert_eq!(metadata.identity.mapping, identity.mapping);
    assert_eq!(metadata.identity.relative_path_hex, "");
    assert!(metadata.payload_present && metadata.verified);
    assert!(matches!(
        grip::push::recovery::preserve(&receipt, 0, &identity, &destination, &expected,),
        Err(grip::GripError::CorruptState(_))
    ));
    assert_eq!(fs::read_to_string(outside).unwrap(), "untouched");
}

#[test]
fn interrupted_operation_is_immutable_while_a_fresh_record_is_allocated() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    let first = grip::operation::publication::initialize(&home, &plan(1)).unwrap();
    let sentinel = first.directory().join("recovery/sentinel");
    fs::write(&sentinel, "preserved").unwrap();
    let first_plan = fs::read(first.directory().join("plan.json")).unwrap();
    let first_summary = fs::read(first.directory().join("operation.json")).unwrap();
    fs::write(
        home.path()
            .join("state/operations/unrelated-retained-history"),
        "not traversed",
    )
    .unwrap();
    let second = grip::operation::publication::initialize(&home, &plan(1)).unwrap();
    assert_ne!(first.operation_id(), second.operation_id());
    assert_eq!(
        fs::read(first.directory().join("plan.json")).unwrap(),
        first_plan
    );
    assert_eq!(
        fs::read(first.directory().join("operation.json")).unwrap(),
        first_summary
    );
    assert_eq!(fs::read_to_string(sentinel).unwrap(), "preserved");
}

#[test]
fn operation_finalization_rejects_corrupt_unsupported_and_inconsistent_components() {
    for replacement in [
        b"not-json".to_vec(),
        br#"{"schema_version":99,"payload":{},"integrity":{"algorithm":"sha256","digest":"00"}}"#
            .to_vec(),
    ] {
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let grip_home = support::minimal_home(root.path());
        let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
        let receipt = grip::operation::publication::initialize(&home, &plan(0)).unwrap();
        fs::write(receipt.directory().join("operation.json"), replacement).unwrap();
        assert!(
            grip::operation::publication::finalize_result_delivery(
                &home,
                receipt.operation_id(),
                "failed",
            )
            .is_err()
        );
    }

    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    assert!(
        grip::operation::publication::finalize_result_delivery(
            &home,
            "../escaping-operation",
            "failed",
        )
        .is_err()
    );
}

#[test]
fn action_checkpoint_rejects_symlink_substitution_without_touching_the_target() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    let receipt = grip::operation::publication::initialize(&home, &plan(1)).unwrap();
    let outside = root.path().join("outside.json");
    fs::write(&outside, b"outside").unwrap();
    fs::set_permissions(&outside, fs::Permissions::from_mode(0o600)).unwrap();
    symlink(&outside, receipt.directory().join("actions/00000000.json")).unwrap();
    let before = fs::read(&outside).unwrap();
    let result = receipt.checkpoint_action(
        0,
        "in_progress",
        grip::operation::model::ActionCheckpointEvidenceV1 {
            revalidation: "not_attempted".into(),
            recovery: "not_required".into(),
            recovery_ref: None,
            staging: "not_attempted".into(),
            publication: "not_attempted".into(),
            verification: "not_attempted".into(),
            durability_confirmed: false,
        },
        None,
    );
    assert!(result.is_err());
    assert_eq!(fs::read(outside).unwrap(), before);
}
