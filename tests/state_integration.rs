use grip::home;
use grip::state::lock::PublicationLock;
use grip::state::publication::PublicationFault;
use grip::state::{self, StateEnvelopeV1};
use std::fs;
use std::os::unix::fs::PermissionsExt;

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
