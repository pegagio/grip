mod support;

use grip::discovery::model::NodeKind;
use grip::mapping::MappingKind;
use grip::metadata::model::{AclState, MetadataState, ModificationTime, SupportedEntryStateV3};
use grip::observation::model::{ContentFingerprint, EntryIdentity, ResolvedMapping};
use grip::state;
use grip::state::lock::PublicationLock;
use grip::state::publication::PublicationFault;
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};

fn portable_identity(relative_path_hex: &str) -> state::EntryIdentityV4 {
    state::EntryIdentityV4 {
        mapping: grip::mapping::PortableMapping::parse(
            MappingKind::Tree,
            std::ffi::OsStr::new("payload"),
            std::ffi::OsStr::new("~/payload"),
        )
        .unwrap(),
        relative_path_hex: relative_path_hex.into(),
    }
}

fn project_binding() -> state::ProjectBindingV1 {
    state::ProjectBindingV1 {
        project_root: "/project".into(),
        user_home: "/home".into(),
        descriptor_digest: "a".repeat(64),
        resolved_mapping_digest: "b".repeat(64),
    }
}

#[test]
fn state_v4_round_trips_portable_identity_binding_and_generation() {
    let accepted = state::AcceptedStateV4 {
        generation: 4,
        binding: project_binding(),
        baselines: BTreeMap::from([
            (portable_identity("61"), complete_state()),
            (portable_identity("7a"), complete_state()),
        ]),
        pending_retirements: Vec::new(),
        accepted_bytes: None,
    };
    let bytes = state::encode_v4(&accepted).unwrap();
    let decoded = state::decode_v4(&bytes).unwrap();
    assert_eq!(decoded.generation, 4);
    assert_eq!(decoded.binding, accepted.binding);
    assert_eq!(decoded.baselines, accepted.baselines);
}

#[test]
fn state_v4_rejects_old_versions_unknown_fields_integrity_and_ordering() {
    for version in 1..=3 {
        let bytes = format!("{{\"schema_version\":{version}}}");
        assert_eq!(
            state::decode_v4(bytes.as_bytes()).unwrap_err().category(),
            grip::ResultCategory::UnsupportedSchema
        );
    }
    let accepted = state::AcceptedStateV4 {
        generation: 1,
        binding: project_binding(),
        baselines: BTreeMap::from([
            (portable_identity("61"), complete_state()),
            (portable_identity("62"), complete_state()),
        ]),
        pending_retirements: Vec::new(),
        accepted_bytes: None,
    };
    let bytes = state::encode_v4(&accepted).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["extra"] = serde_json::json!(true);
    assert!(state::decode_v4(&serde_json::to_vec(&value).unwrap()).is_err());

    let mut envelope: state::StateEnvelopeV4 = serde_json::from_slice(&bytes).unwrap();
    envelope.payload.baselines.swap(0, 1);
    envelope.integrity.digest = state::state_v4_digest(&envelope.payload).unwrap();
    assert!(envelope.validate().is_err());
    envelope.integrity.digest.replace_range(..1, "0");
    assert!(envelope.validate().is_err());
}

fn complete_state() -> SupportedEntryStateV3 {
    SupportedEntryStateV3 {
        node_kind: NodeKind::File,
        content: Some(ContentFingerprint {
            algorithm: "sha256".into(),
            digest: "c".repeat(64),
            length: 3,
        }),
        metadata: MetadataState {
            permission_mode: "0640".into(),
            uid: 501,
            gid: 20,
            modified_time: ModificationTime {
                seconds: 1_700_000_000,
                nanoseconds: 42,
            },
            extended_attributes: Vec::new(),
            acl: AclState::Absent,
            bsd_flags: Default::default(),
        },
    }
}

fn publication_fixture() -> (tempfile::TempDir, grip::project::ProjectPaths) {
    let root = tempfile::tempdir().unwrap();
    let metadata = root.path().join(".grip");
    fs::create_dir(&metadata).unwrap();
    fs::write(
        metadata.join("config.toml"),
        "schema_version = 2\n\n[[mappings]]\nkind = \"tree\"\nsource = \"payload\"\ndestination = \"~/destination\"\n",
    )
    .unwrap();
    let home = project_home(&metadata);
    (root, home)
}

fn project_home(metadata: &std::path::Path) -> grip::project::ProjectPaths {
    grip::project::ProjectPaths::project_metadata(
        metadata.to_path_buf(),
        metadata.parent().unwrap().to_path_buf(),
    )
}

fn publication_identity(root: &std::path::Path, relative: &[u8]) -> EntryIdentity {
    EntryIdentity::new(
        ResolvedMapping {
            kind: MappingKind::Tree,
            source: root.join("payload"),
            destination: root.join("destination"),
        },
        relative.to_vec(),
    )
    .unwrap()
}

fn complete_baselines(
    root: &std::path::Path,
    digest_byte: char,
) -> BTreeMap<EntryIdentity, SupportedEntryStateV3> {
    let mut value = complete_state();
    value.content.as_mut().unwrap().digest = digest_byte.to_string().repeat(64);
    BTreeMap::from([(publication_identity(root, b"file"), value)])
}

#[test]
fn complete_state_publication_is_atomic_and_loads_as_v4() {
    let (root, home) = publication_fixture();
    let state_dir = state::publication::prepare_directory(&home).unwrap();
    let _lock = PublicationLock::acquire(&state_dir.join("state.lock")).unwrap();
    let expected = state::publication::load(&home).unwrap();
    let baselines = complete_baselines(root.path(), 'c');
    assert_eq!(
        state::publication::publish_complete_locked(&home, &expected, &baselines).unwrap(),
        Some(0)
    );
    let loaded = state::publication::load(&home).unwrap();
    let complete = loaded.complete.expect("published state must remain V4");
    assert_eq!(complete.generation, 0);
    assert_eq!(complete.baselines, baselines);
    assert_eq!(
        fs::metadata(state_dir.join("state.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
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
fn v4_pre_rename_and_corrupt_staging_failures_preserve_prior_state() {
    let (root, home) = publication_fixture();
    let state_dir = state::publication::prepare_directory(&home).unwrap();
    let lock = PublicationLock::acquire(&state_dir.join("state.lock")).unwrap();
    let absent = state::publication::load(&home).unwrap();
    state::publication::publish_complete_locked(
        &home,
        &absent,
        &complete_baselines(root.path(), 'a'),
    )
    .unwrap();
    let before = fs::read(state_dir.join("state.json")).unwrap();
    let expected = state::publication::load(&home).unwrap();

    for fault in [
        PublicationFault::BeforeV2StateRename,
        PublicationFault::CorruptV2Staging,
    ] {
        assert!(
            state::publication::publish_complete_locked_with_fault(
                &home,
                &expected,
                &complete_baselines(root.path(), 'b'),
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
fn v4_post_rename_failure_reports_visibility_and_retry_is_noop() {
    let (root, home) = publication_fixture();
    let state_dir = state::publication::prepare_directory(&home).unwrap();
    let lock = PublicationLock::acquire(&state_dir.join("state.lock")).unwrap();
    let absent = state::publication::load(&home).unwrap();
    let next = complete_baselines(root.path(), 'a');
    let error = state::publication::publish_complete_locked_with_fault(
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
    assert_eq!(visible.complete.as_ref().unwrap().baselines, next);
    assert_eq!(
        state::publication::publish_complete_locked(&home, &visible, &next).unwrap(),
        None
    );
    drop(lock);
}

#[test]
fn v4_rejects_recovery_collisions_and_unsafe_state_artifacts() {
    let (root, home) = publication_fixture();
    let state_dir = state::publication::prepare_directory(&home).unwrap();
    let lock = PublicationLock::acquire(&state_dir.join("state.lock")).unwrap();
    let absent = state::publication::load(&home).unwrap();
    state::publication::publish_complete_locked(
        &home,
        &absent,
        &complete_baselines(root.path(), 'a'),
    )
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
        state::publication::publish_complete_locked(
            &home,
            &expected,
            &complete_baselines(root.path(), 'b'),
        )
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
        let home = project_home(root.path());
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
    let home = project_home(root.path());
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
    let home = project_home(root.path());
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
fn stale_state_and_exhausted_generation_have_stable_non_success_results() {
    let (root, home) = publication_fixture();
    let absent = state::publication::load(&home).unwrap();
    let state_dir = state::publication::prepare_directory(&home).unwrap();
    let initial_lock = PublicationLock::acquire(&state_dir.join("state.lock")).unwrap();
    state::publication::publish_complete_locked(
        &home,
        &absent,
        &complete_baselines(root.path(), 'a'),
    )
    .unwrap();
    drop(initial_lock);
    let stale = state::publication::revalidate(&home, &absent).unwrap_err();
    let stale_outcome = grip::result::CommandOutcome::failure(&stale);
    assert_eq!(stale_outcome.category.exit_code(), 20);
    assert_eq!(stale_outcome.details["reason"], "stale_state_evidence");

    let mut maximum = state::decode_v4(&fs::read(state_dir.join("state.json")).unwrap()).unwrap();
    maximum.generation = u64::MAX;
    fs::write(
        state_dir.join("state.json"),
        state::encode_v4(&maximum).unwrap(),
    )
    .unwrap();
    let snapshot = state::publication::load(&home).unwrap();
    let before = fs::read(state_dir.join("state.json")).unwrap();
    let lock = PublicationLock::acquire(&state_dir.join("state.lock")).unwrap();
    let exhausted = state::publication::publish_complete_locked(
        &home,
        &snapshot,
        &complete_baselines(root.path(), 'b'),
    )
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
