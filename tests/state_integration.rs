mod support;

use grip::discovery::model::NodeKind;
use grip::home;
use grip::mapping::MappingKind;
use grip::observation::model::{
    ContentFingerprint, EntryIdentity, MappingSnapshot, SupportedState,
};
use grip::state::lock::PublicationLock;
use grip::state::publication::PublicationFault;
use grip::state::{self, StateEnvelopeV1};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;

#[test]
fn validates_digest_schema_and_generation() {
    let state = StateEnvelopeV1::new(0);
    let bytes = state::encode(&state).unwrap();
    assert_eq!(
        state::decode(std::str::from_utf8(&bytes).unwrap()).unwrap(),
        state
    );
    let mut corrupt = state.clone();
    corrupt.integrity.digest.replace_range(..1, "0");
    assert!(corrupt.validate().is_err());
    let text = String::from_utf8(bytes)
        .unwrap()
        .replace("\"schema_version\":1", "\"schema_version\":2");
    assert_eq!(state::decode(&text).unwrap_err().category().exit_code(), 11);
}

#[test]
fn publication_retains_verified_prior_generation_and_permissions() {
    let root = tempfile::tempdir().unwrap();
    let home = home::select(Some(root.path().to_owned().into_os_string()), None).unwrap();
    state::publication::publish(&home, &StateEnvelopeV1::new(0)).unwrap();
    state::publication::publish(&home, &StateEnvelopeV1::new(1)).unwrap();
    let current = fs::read_to_string(root.path().join("state/state.json")).unwrap();
    assert_eq!(state::decode(&current).unwrap().payload.generation, 1);
    let prior =
        fs::read_to_string(root.path().join("state/recovery/generation-0/state.json")).unwrap();
    assert_eq!(state::decode(&prior).unwrap().payload.generation, 0);
    assert_eq!(
        fs::metadata(root.path().join("state"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(root.path().join("state/state.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
}

#[test]
fn rejects_nonincreasing_generation_and_recovery_collision() {
    let root = tempfile::tempdir().unwrap();
    let home = home::select(Some(root.path().to_owned().into_os_string()), None).unwrap();
    state::publication::publish(&home, &StateEnvelopeV1::new(0)).unwrap();
    assert!(state::publication::publish(&home, &StateEnvelopeV1::new(0)).is_err());
    fs::create_dir_all(root.path().join("state/recovery/generation-0")).unwrap();
    fs::write(
        root.path().join("state/recovery/generation-0/state.json"),
        "different",
    )
    .unwrap();
    assert!(state::publication::publish(&home, &StateEnvelopeV1::new(1)).is_err());
}

#[test]
fn lock_contention_is_bounded_and_release_allows_retry() {
    let root = tempfile::tempdir().unwrap();
    let lock_path = root.path().join("state.lock");
    let first = PublicationLock::acquire(&lock_path).unwrap();
    assert!(matches!(
        PublicationLock::acquire(&lock_path),
        Err(grip::GripError::StateContention)
    ));
    drop(first);
    PublicationLock::acquire(&lock_path).unwrap();
}

#[test]
fn injected_pre_rename_failure_preserves_accepted_state_and_cleans_staging() {
    let root = tempfile::tempdir().unwrap();
    let home = home::select(Some(root.path().to_owned().into_os_string()), None).unwrap();
    state::publication::publish(&home, &StateEnvelopeV1::new(0)).unwrap();
    assert!(
        state::publication::publish_with_fault(
            &home,
            &StateEnvelopeV1::new(1),
            Some(PublicationFault::BeforeStateRename),
        )
        .is_err()
    );
    let accepted = fs::read_to_string(root.path().join("state/state.json")).unwrap();
    assert_eq!(state::decode(&accepted).unwrap().payload.generation, 0);
    assert!(
        fs::read_dir(root.path().join("state"))
            .unwrap()
            .all(|entry| {
                !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".state.tmp-")
            })
    );
}

#[test]
fn version_neutral_reader_accepts_absent_v1_and_v2_state() {
    let absent = state::decode_accepted(None).unwrap();
    assert_eq!(absent.generation, None);
    assert!(absent.baselines.is_empty());

    let v1 = state::encode(&StateEnvelopeV1::new(7)).unwrap();
    let predecessor = state::decode_accepted(Some(&v1)).unwrap();
    assert_eq!(predecessor.generation, Some(7));
    assert!(predecessor.baselines.is_empty());

    let identity = EntryIdentity::new(
        MappingSnapshot {
            kind: MappingKind::File,
            source: PathBuf::from("/source/file"),
            destination: PathBuf::from("/destination/file"),
        },
        Vec::new(),
    )
    .unwrap();
    let state_value = SupportedState {
        node_kind: NodeKind::File,
        content: Some(ContentFingerprint {
            algorithm: "sha256".into(),
            digest: "b".repeat(64),
            length: 9,
        }),
        permission_mode: Some("0600".into()),
    };
    let mut baselines = BTreeMap::new();
    baselines.insert(identity.clone(), state_value.clone());
    let accepted = state::AcceptedState {
        generation: Some(8),
        baselines,
        accepted_bytes: None,
    };
    let v2 = state::encode_v2(&accepted, 8).unwrap();
    let decoded = state::decode_accepted(Some(&v2)).unwrap();
    assert_eq!(decoded.generation, Some(8));
    assert_eq!(decoded.baselines.get(&identity), Some(&state_value));
}

fn accepted_with_digest(digest_byte: char) -> state::AcceptedState {
    let identity = EntryIdentity::new(
        MappingSnapshot {
            kind: MappingKind::File,
            source: PathBuf::from("/source/file"),
            destination: PathBuf::from("/destination/file"),
        },
        Vec::new(),
    )
    .unwrap();
    let mut baselines = BTreeMap::new();
    baselines.insert(
        identity,
        SupportedState {
            node_kind: NodeKind::File,
            content: Some(ContentFingerprint {
                algorithm: "sha256".into(),
                digest: digest_byte.to_string().repeat(64),
                length: 1,
            }),
            permission_mode: Some("0600".into()),
        },
    );
    state::AcceptedState {
        generation: None,
        baselines,
        accepted_bytes: None,
    }
}

#[test]
fn v2_pre_rename_and_corrupt_staging_failures_preserve_prior_state() {
    let root = tempfile::tempdir().unwrap();
    let home = home::select(Some(root.path().to_owned().into_os_string()), None).unwrap();
    let state_dir = state::publication::prepare_directory(&home).unwrap();
    let lock = PublicationLock::acquire(&state_dir.join("state.lock")).unwrap();
    let absent = state::publication::load(&home).unwrap();
    state::publication::publish_accepted_locked(&home, &absent, &accepted_with_digest('a'))
        .unwrap();
    let before = fs::read(state_dir.join("state.json")).unwrap();
    let expected = state::publication::load(&home).unwrap();

    for fault in [
        PublicationFault::BeforeV2StateRename,
        PublicationFault::CorruptV2Staging,
    ] {
        assert!(
            state::publication::publish_accepted_locked_with_fault(
                &home,
                &expected,
                &accepted_with_digest('b'),
                Some(fault),
            )
            .is_err()
        );
        assert_eq!(fs::read(state_dir.join("state.json")).unwrap(), before);
        assert!(fs::read_dir(&state_dir).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".state.tmp-")
        }));
    }
    drop(lock);
}

#[test]
fn v2_post_rename_failure_reports_visibility_and_retry_is_noop() {
    let root = tempfile::tempdir().unwrap();
    let home = home::select(Some(root.path().to_owned().into_os_string()), None).unwrap();
    let state_dir = state::publication::prepare_directory(&home).unwrap();
    let lock = PublicationLock::acquire(&state_dir.join("state.lock")).unwrap();
    let absent = state::publication::load(&home).unwrap();
    let next = accepted_with_digest('a');
    let error = state::publication::publish_accepted_locked_with_fault(
        &home,
        &absent,
        &next,
        Some(PublicationFault::AfterV2StateRename),
    )
    .unwrap_err();
    let outcome = grip::result::CommandOutcome::failure(&error);
    assert_eq!(outcome.category.exit_code(), 20);
    assert_eq!(outcome.details["publication_visible"], true);
    assert_eq!(outcome.details["durability_confirmed"], false);
    let visible = state::publication::load(&home).unwrap();
    assert_eq!(visible.accepted.baselines, next.baselines);
    assert_eq!(
        state::publication::publish_accepted_locked(&home, &visible, &next).unwrap(),
        None
    );
    drop(lock);
}

#[test]
fn v2_rejects_recovery_collisions_and_unsafe_state_artifacts() {
    let root = tempfile::tempdir().unwrap();
    let home = home::select(Some(root.path().to_owned().into_os_string()), None).unwrap();
    let state_dir = state::publication::prepare_directory(&home).unwrap();
    let lock = PublicationLock::acquire(&state_dir.join("state.lock")).unwrap();
    let absent = state::publication::load(&home).unwrap();
    state::publication::publish_accepted_locked(&home, &absent, &accepted_with_digest('a'))
        .unwrap();
    let expected = state::publication::load(&home).unwrap();
    let recovery_dir = state_dir.join("recovery/generation-0");
    fs::create_dir_all(&recovery_dir).unwrap();
    fs::set_permissions(
        state_dir.join("recovery"),
        fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    fs::set_permissions(&recovery_dir, fs::Permissions::from_mode(0o700)).unwrap();
    let recovery = recovery_dir.join("state.json");
    fs::write(&recovery, b"different").unwrap();
    fs::set_permissions(&recovery, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(
        state::publication::publish_accepted_locked(&home, &expected, &accepted_with_digest('b'))
            .is_err()
    );
    drop(lock);

    fs::set_permissions(&state_dir, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(state::publication::prepare_directory(&home).is_err());
    fs::set_permissions(&state_dir, fs::Permissions::from_mode(0o700)).unwrap();
    fs::set_permissions(
        state_dir.join("state.json"),
        fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    assert!(state::publication::load(&home).is_err());
}

#[test]
fn read_only_load_rejects_unsafe_state_directories_even_when_state_is_absent() {
    for mode in [0o755, 0o770] {
        let root = tempfile::tempdir().unwrap();
        let home = home::select(Some(root.path().to_owned().into_os_string()), None).unwrap();
        fs::create_dir(root.path().join("state")).unwrap();
        fs::set_permissions(root.path().join("state"), fs::Permissions::from_mode(mode)).unwrap();
        assert_eq!(
            state::publication::load(&home)
                .unwrap_err()
                .category()
                .exit_code(),
            12
        );
    }

    let root = tempfile::tempdir().unwrap();
    let home = home::select(Some(root.path().to_owned().into_os_string()), None).unwrap();
    fs::write(root.path().join("state"), b"not a directory").unwrap();
    assert_eq!(
        state::publication::load(&home)
            .unwrap_err()
            .category()
            .exit_code(),
        12
    );

    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let home = home::select(Some(root.path().to_owned().into_os_string()), None).unwrap();
    symlink(outside.path(), root.path().join("state")).unwrap();
    assert_eq!(
        state::publication::load(&home)
            .unwrap_err()
            .category()
            .exit_code(),
        12
    );
}

#[test]
fn v2_staging_substitution_is_rejected_preserved_and_does_not_publish() {
    for fault in [
        PublicationFault::SubstituteV2StagingFile,
        PublicationFault::SubstituteV2StagingSymlink,
    ] {
        let root = tempfile::tempdir().unwrap();
        let home = home::select(Some(root.path().to_owned().into_os_string()), None).unwrap();
        let state_dir = state::publication::prepare_directory(&home).unwrap();
        let lock = PublicationLock::acquire(&state_dir.join("state.lock")).unwrap();
        let absent = state::publication::load(&home).unwrap();
        let error = state::publication::publish_accepted_locked_with_fault(
            &home,
            &absent,
            &accepted_with_digest('a'),
            Some(fault),
        )
        .unwrap_err();
        assert_eq!(error.category().exit_code(), 12);
        assert!(!state_dir.join("state.json").exists());
        assert!(fs::read_dir(&state_dir).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".state.tmp-")
        }));
        drop(lock);
    }
}

#[test]
fn stale_state_and_exhausted_generation_have_stable_non_success_results() {
    let root = tempfile::tempdir().unwrap();
    let home = home::select(Some(root.path().to_owned().into_os_string()), None).unwrap();
    let absent = state::publication::load(&home).unwrap();
    let state_dir = state::publication::prepare_directory(&home).unwrap();
    let bytes = state::encode_v2(&accepted_with_digest('a'), 0).unwrap();
    fs::write(state_dir.join("state.json"), bytes).unwrap();
    fs::set_permissions(
        state_dir.join("state.json"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    let stale = state::publication::revalidate(&home, &absent).unwrap_err();
    let stale_outcome = grip::result::CommandOutcome::failure(&stale);
    assert_eq!(stale_outcome.category.exit_code(), 20);
    assert_eq!(stale_outcome.details["reason"], "stale_state_evidence");

    let mut maximum = accepted_with_digest('a');
    maximum.generation = Some(u64::MAX);
    support::write_v2_state(root.path(), u64::MAX, maximum.baselines.clone());
    let snapshot = state::publication::load(&home).unwrap();
    let before = fs::read(state_dir.join("state.json")).unwrap();
    let lock = PublicationLock::acquire(&state_dir.join("state.lock")).unwrap();
    let exhausted =
        state::publication::publish_accepted_locked(&home, &snapshot, &accepted_with_digest('b'))
            .unwrap_err();
    let outcome = grip::result::CommandOutcome::failure(&exhausted);
    assert_eq!(outcome.category.exit_code(), 20);
    assert_eq!(outcome.details["reason"], "generation_exhausted");
    assert_eq!(fs::read(state_dir.join("state.json")).unwrap(), before);
    assert!(
        !state_dir
            .join("recovery/generation-18446744073709551615")
            .exists()
    );
    assert!(fs::read_dir(&state_dir).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".state.tmp-")
    }));
    drop(lock);
}
