mod support;

use grip::GripError;
use grip::home;
use grip::mapping::{Mapping, MappingKind};
use grip::path_policy;
use grip::registry;
use grip::registry::publication::{self, PublicationFault};
use grip::state::lock::PublicationLock;
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};

#[test]
fn registry_round_trip_is_canonical_and_preserves_mapping_values() {
    let registry = registry::Registry::new(vec![
        Mapping::new(
            MappingKind::Tree,
            "/source/z".into(),
            "/destination/z".into(),
        ),
        Mapping::new(
            MappingKind::File,
            "/source/a".into(),
            "/destination/a".into(),
        ),
    ])
    .unwrap();
    let bytes = registry::encode(&registry).unwrap();
    let decoded = registry::decode(std::str::from_utf8(&bytes).unwrap()).unwrap();
    assert_eq!(decoded, registry);
    assert_eq!(decoded.mappings()[0].source.to_str(), Some("/source/a"));
    assert_eq!(bytes, registry::encode(&decoded).unwrap());
}

#[test]
fn add_canonicalizes_paths_allows_absent_destination_and_does_not_touch_payloads() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("absent/child");
    fs::create_dir(&source).unwrap();
    let payload_before = support::snapshot(&source);
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output=json",
            "mapping",
            "add",
            "tree",
            source.to_str().unwrap(),
            destination.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(payload_before, support::snapshot(&source));
    assert!(!destination.exists());
    assert!(!grip_home.join("state/state.json").exists());
}

#[test]
fn rejects_relative_traversal_symlink_and_wrong_kind_paths() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let file = root.path().join("file");
    let directory = root.path().join("directory");
    fs::write(&file, "x").unwrap();
    fs::create_dir(&directory).unwrap();
    let link = root.path().join("link");
    symlink(&file, &link).unwrap();
    for (kind, source, reason) in [
        ("file", "relative", "relative_path"),
        ("file", link.to_str().unwrap(), "symlink_endpoint"),
        ("tree", file.to_str().unwrap(), "wrong_node_kind"),
    ] {
        let destination = root.path().join(format!("destination-{reason}"));
        let output = support::command_with_grip_home(
            root.path(),
            &grip_home,
            &[
                "--output=json",
                "mapping",
                "add",
                kind,
                source,
                destination.to_str().unwrap(),
            ],
        );
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["details"]["reason"], reason);
        assert_eq!(value["details"]["kind"], kind);
    }
    let traversal = format!("{}/directory/../file", root.path().display());
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output=json",
            "mapping",
            "add",
            "file",
            &traversal,
            root.path().join("target").to_str().unwrap(),
        ],
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()["details"]["reason"],
        "parent_traversal"
    );
}

#[cfg(unix)]
#[test]
fn rejects_non_utf8_input_without_lossy_identity() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::process::Command;
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = OsString::from_vec(vec![b'/', b't', b'm', b'p', b'/', 0xff]);
    let output = Command::new(env!("CARGO_BIN_EXE_grip"))
        .env_clear()
        .env("HOME", root.path())
        .env("GRIP_HOME", &grip_home)
        .args([
            OsString::from("--output=json"),
            OsString::from("mapping"),
            OsString::from("add"),
            OsString::from("file"),
            source,
            root.path().join("target").into_os_string(),
        ])
        .output()
        .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["details"]["reason"], "non_utf8_path");
}

#[test]
fn list_is_sorted_show_is_exact_and_readers_do_not_mutate() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let source_a = root.path().join("source-a");
    let source_z = root.path().join("source-z");
    fs::write(&source_a, "a").unwrap();
    fs::write(&source_z, "z").unwrap();
    support::write_registry(
        &grip_home,
        &[
            ("file", &source_z, &root.path().join("destination-z")),
            ("file", &source_a, &root.path().join("destination-a")),
        ],
    );
    let before = support::snapshot(root.path());
    let list = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    let value: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(
        value["details"]["mappings"][0]["source"],
        fs::canonicalize(&source_a).unwrap().display().to_string()
    );
    let show = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output=json",
            "mapping",
            "show",
            source_z.to_str().unwrap(),
        ],
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&show.stdout).unwrap()["details"]["mapping"]["source"],
        fs::canonicalize(&source_z).unwrap().display().to_string()
    );
    assert_eq!(before, support::snapshot(root.path()));
}

#[test]
fn remove_changes_only_registry_intent_and_retains_exact_prior_bytes() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "source payload").unwrap();
    fs::write(&destination, "destination payload").unwrap();
    support::write_registry(&grip_home, &[("file", &source, &destination)]);
    let prior = fs::read(grip_home.join("config.toml")).unwrap();
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["mapping", "remove", source.to_str().unwrap()],
    );
    assert!(output.status.success());
    assert_eq!(fs::read_to_string(&source).unwrap(), "source payload");
    assert_eq!(
        fs::read_to_string(&destination).unwrap(),
        "destination payload"
    );
    let digest = format!("{:x}", Sha256::digest(&prior));
    assert_eq!(
        fs::read(grip_home.join(format!(
            "state/recovery/registry/sha256-{digest}/config.toml"
        )))
        .unwrap(),
        prior
    );
    assert!(
        registry::decode(&fs::read_to_string(grip_home.join("config.toml")).unwrap())
            .unwrap()
            .mappings()
            .is_empty()
    );
}

#[test]
fn publication_rejects_stale_bytes_preserves_mode_and_cleans_staging_on_failure() {
    let root = tempfile::tempdir().unwrap();
    let grip_home_path = support::minimal_home(root.path());
    fs::set_permissions(
        grip_home_path.join("config.toml"),
        fs::Permissions::from_mode(0o640),
    )
    .unwrap();
    let home = home::select(Some(grip_home_path.clone().into_os_string()), None).unwrap();
    let snapshot = publication::load(&home, true).unwrap();
    let candidate = registry::Registry::new(Vec::new()).unwrap();
    publication::publish_with_fault(
        &home,
        &snapshot,
        &candidate,
        Some(PublicationFault::BeforeRegistryRename),
    )
    .unwrap_err();
    assert_eq!(
        fs::read(grip_home_path.join("config.toml")).unwrap(),
        snapshot.bytes
    );
    assert!(fs::read_dir(&grip_home_path).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".config.tmp-")
    }));
    publication::publish(&home, &snapshot, &candidate).unwrap();
    assert_eq!(
        fs::metadata(grip_home_path.join("config.toml"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o640
    );
    let stale = publication::load(&home, true).unwrap();
    fs::write(
        grip_home_path.join("config.toml"),
        "schema_version = 1\nmappings = []\n# changed\n",
    )
    .unwrap();
    let error = publication::publish(&home, &stale, &candidate).unwrap_err();
    assert!(error.to_string().contains("changed before publication"));
    assert_eq!(
        fs::read_to_string(grip_home_path.join("config.toml")).unwrap(),
        "schema_version = 1\nmappings = []\n# changed\n"
    );
}

#[test]
fn publication_rejects_retargeted_submitted_ancestry_without_mutation() {
    let root = tempfile::tempdir().unwrap();
    let grip_home_path = support::minimal_home(root.path());
    let home = home::select(Some(grip_home_path.clone().into_os_string()), None).unwrap();
    let source = root.path().join("source");
    fs::write(&source, "payload").unwrap();
    let first = root.path().join("first");
    let second = root.path().join("second");
    fs::create_dir(&first).unwrap();
    fs::create_dir(&second).unwrap();
    let alias = root.path().join("alias");
    symlink(&first, &alias).unwrap();
    let destination = alias.join("absent");

    let snapshot = publication::load(&home, true).unwrap();
    let source_evidence =
        path_policy::inspect_endpoint(&source, MappingKind::File, true, "mapping_add").unwrap();
    let destination_evidence =
        path_policy::inspect_endpoint(&destination, MappingKind::File, false, "mapping_add")
            .unwrap();
    let candidate = registry::Registry::new(vec![Mapping::new(
        MappingKind::File,
        source_evidence.canonical.clone(),
        destination_evidence.canonical.clone(),
    )])
    .unwrap();
    let first_before = support::snapshot(&first);
    let second_before = support::snapshot(&second);

    fs::remove_file(&alias).unwrap();
    symlink(&second, &alias).unwrap();
    let error = publication::publish_with_evidence(
        &home,
        &snapshot,
        &candidate,
        &[source_evidence, destination_evidence],
    )
    .unwrap_err();

    assert!(matches!(
        error,
        GripError::Mapping { ref reason, .. } if reason == "stale_path_evidence"
    ));
    assert_eq!(
        fs::read(grip_home_path.join("config.toml")).unwrap(),
        snapshot.bytes
    );
    assert_eq!(support::snapshot(&first), first_before);
    assert_eq!(support::snapshot(&second), second_before);
    assert!(!first.join("absent").exists());
    assert!(!second.join("absent").exists());
    assert!(!grip_home_path.join("state/recovery").exists());
}

#[test]
fn rejects_unsafe_registry_mode_and_missing_mapping() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    fs::set_permissions(
        grip_home.join("config.toml"),
        fs::Permissions::from_mode(0o666),
    )
    .unwrap();
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()["details"]["reason"],
        "unsafe_registry_mode"
    );
    fs::set_permissions(
        grip_home.join("config.toml"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    let missing = root.path().join("missing");
    fs::write(&missing, "x").unwrap();
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output=json",
            "mapping",
            "show",
            missing.to_str().unwrap(),
        ],
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()["details"]["reason"],
        "mapping_not_found"
    );
}

#[test]
fn absent_show_and_remove_selectors_are_distinct_not_found_results_without_mutation() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let missing = root.path().join("absent/source");
    let canonical_missing = fs::canonicalize(root.path()).unwrap().join("absent/source");
    let before = support::snapshot(root.path());

    for operation in ["show", "remove"] {
        let human = support::command_with_grip_home(
            root.path(),
            &grip_home,
            &["mapping", operation, missing.to_str().unwrap()],
        );
        assert_eq!(human.status.code(), Some(10));
        assert_eq!(
            String::from_utf8(human.stdout).unwrap(),
            "Mapping not found\n"
        );
        assert_eq!(support::snapshot(root.path()), before);

        let json = support::command_with_grip_home(
            root.path(),
            &grip_home,
            &[
                "--output=json",
                "mapping",
                operation,
                missing.to_str().unwrap(),
            ],
        );
        assert_eq!(json.status.code(), Some(10));
        let value: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
        assert_eq!(value["details"]["reason"], "mapping_not_found");
        assert_eq!(
            value["details"]["paths"],
            serde_json::json!([canonical_missing])
        );
        assert_eq!(support::snapshot(root.path()), before);
    }
}

#[test]
fn absent_mapping_identities_resolve_through_intermediate_directory_symlinks() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    fs::write(&source, "payload").unwrap();
    let real = root.path().join("real");
    fs::create_dir(&real).unwrap();
    let alias = root.path().join("alias");
    symlink(&real, &alias).unwrap();
    let destination = alias.join("absent-destination");
    let canonical_destination = fs::canonicalize(&real).unwrap().join("absent-destination");
    let payload_before = support::snapshot(&real);

    let add = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output=json",
            "mapping",
            "add",
            "file",
            source.to_str().unwrap(),
            destination.to_str().unwrap(),
        ],
    );
    assert!(add.status.success());
    let value: serde_json::Value = serde_json::from_slice(&add.stdout).unwrap();
    assert_eq!(
        value["details"]["mapping"]["destination"],
        canonical_destination.display().to_string()
    );
    assert!(!canonical_destination.exists());
    assert_eq!(support::snapshot(&real), payload_before);

    let missing_selector = alias.join("absent-source");
    let canonical_selector = fs::canonicalize(&real).unwrap().join("absent-source");
    for operation in ["show", "remove"] {
        let before = support::snapshot(root.path());
        let output = support::command_with_grip_home(
            root.path(),
            &grip_home,
            &[
                "--output=json",
                "mapping",
                operation,
                missing_selector.to_str().unwrap(),
            ],
        );
        assert_eq!(output.status.code(), Some(10));
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["details"]["reason"], "mapping_not_found");
        assert_eq!(
            value["details"]["paths"][0],
            canonical_selector.display().to_string()
        );
        assert_eq!(support::snapshot(root.path()), before);
    }
}

#[test]
fn lock_contention_is_nonblocking_and_retry_succeeds_after_release() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    fs::write(&source, "x").unwrap();
    let lock = PublicationLock::acquire(&grip_home.join(".registry.lock")).unwrap();
    let blocked = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output=json",
            "mapping",
            "add",
            "file",
            source.to_str().unwrap(),
            root.path().join("destination").to_str().unwrap(),
        ],
    );
    assert_eq!(blocked.status.code(), Some(20));
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&blocked.stdout).unwrap()["details"]["reason"],
        "registry_contention"
    );
    drop(lock);
    let retry = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "mapping",
            "add",
            "file",
            source.to_str().unwrap(),
            root.path().join("destination").to_str().unwrap(),
        ],
    );
    assert!(retry.status.success());
}

#[test]
fn recovery_collision_and_injected_recovery_failure_preserve_registry() {
    let root = tempfile::tempdir().unwrap();
    let grip_home_path = support::minimal_home(root.path());
    let home = home::select(Some(grip_home_path.clone().into_os_string()), None).unwrap();
    let snapshot = publication::load(&home, true).unwrap();
    let candidate = registry::Registry::new(Vec::new()).unwrap();
    publication::publish_with_fault(
        &home,
        &snapshot,
        &candidate,
        Some(PublicationFault::BeforeRecoveryRename),
    )
    .unwrap_err();
    assert_eq!(
        fs::read(grip_home_path.join("config.toml")).unwrap(),
        snapshot.bytes
    );

    let digest = format!("{:x}", Sha256::digest(&snapshot.bytes));
    let generation = grip_home_path.join(format!("state/recovery/registry/sha256-{digest}"));
    fs::write(generation.join("config.toml"), "collision").unwrap();
    let error = publication::publish(&home, &snapshot, &candidate).unwrap_err();
    assert_eq!(error.category(), grip::ResultCategory::InternalError);
    assert!(matches!(
        error,
        GripError::Mapping { ref reason, .. } if reason == "registry_recovery_failure"
    ));
    assert_eq!(
        fs::read(grip_home_path.join("config.toml")).unwrap(),
        snapshot.bytes
    );
}

#[test]
fn invalid_unrelated_mapping_blocks_list_and_remove() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let other = root.path().join("other");
    fs::write(&source, "x").unwrap();
    fs::write(&other, "y").unwrap();
    let canonical_source = fs::canonicalize(&source).unwrap();
    let canonical_other = fs::canonicalize(&other).unwrap();
    support::write_registry(
        &grip_home,
        &[
            ("file", &canonical_source, &root.path().join("destination")),
            ("tree", &canonical_other, &root.path().join("wrong-kind")),
        ],
    );
    for arguments in [
        vec!["--output=json", "mapping", "list"],
        vec![
            "--output=json",
            "mapping",
            "remove",
            canonical_source.to_str().unwrap(),
        ],
    ] {
        let output = support::command_with_grip_home(root.path(), &grip_home, &arguments);
        assert_eq!(output.status.code(), Some(10));
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()["details"]["reason"],
            "wrong_node_kind"
        );
    }
}

#[test]
fn invalid_registry_precedes_invalid_command_paths_for_every_mapping_operation() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    fs::write(
        grip_home.join("config.toml"),
        "schema_version = 1\n[[mappings]]\nkind = \"bogus\"\nsource = \"/source\"\ndestination = \"/destination\"\n",
    )
    .unwrap();
    let before = support::snapshot(root.path());

    for (operation, arguments) in [
        (
            "mapping_add",
            vec![
                "--output=json",
                "mapping",
                "add",
                "file",
                "relative-source",
                "relative-destination",
            ],
        ),
        (
            "mapping_show",
            vec!["--output=json", "mapping", "show", "relative-source"],
        ),
        (
            "mapping_remove",
            vec!["--output=json", "mapping", "remove", "relative-source"],
        ),
    ] {
        let output = support::command_with_grip_home(root.path(), &grip_home, &arguments);
        assert_eq!(output.status.code(), Some(10));
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["details"]["operation"], operation);
        assert_eq!(value["details"]["reason"], "invalid_registry");
        assert_eq!(
            value["details"]["paths"],
            serde_json::json!([grip_home.join("config.toml")])
        );
        if operation == "mapping_add" {
            assert_eq!(value["details"]["kind"], "file");
        }
        assert_eq!(support::snapshot(root.path()), before);
    }
}

#[test]
fn path_drift_between_inspection_and_publication_blocks_update() {
    let root = tempfile::tempdir().unwrap();
    let grip_home_path = support::minimal_home(root.path());
    let source = root.path().join("source");
    fs::write(&source, "x").unwrap();
    support::write_registry(
        &grip_home_path,
        &[("file", &source, &root.path().join("destination"))],
    );
    let home = home::select(Some(grip_home_path.clone().into_os_string()), None).unwrap();
    let snapshot = publication::load(&home, true).unwrap();
    let candidate = registry::Registry::new(snapshot.registry.mappings().to_vec()).unwrap();
    fs::remove_file(&source).unwrap();
    fs::create_dir(&source).unwrap();
    let error = publication::publish(&home, &snapshot, &candidate).unwrap_err();
    assert!(matches!(
        error,
        grip::GripError::Mapping { ref reason, .. } if reason == "stale_path_evidence"
    ));
    assert_eq!(
        fs::read(grip_home_path.join("config.toml")).unwrap(),
        snapshot.bytes
    );
}

#[test]
fn byte_identical_recovery_is_reused_and_unexpected_staging_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    let grip_home_path = support::minimal_home(root.path());
    let home = home::select(Some(grip_home_path.clone().into_os_string()), None).unwrap();
    let snapshot = publication::load(&home, true).unwrap();
    let candidate = registry::Registry::new(Vec::new()).unwrap();
    publication::publish(&home, &snapshot, &candidate).unwrap();
    let second = publication::load(&home, true).unwrap();
    publication::publish(&home, &second, &candidate).unwrap();

    fs::write(grip_home_path.join(".config.tmp-unowned"), "unexpected").unwrap();
    let third = publication::load(&home, true).unwrap();
    let error = publication::publish(&home, &third, &candidate).unwrap_err();
    assert!(matches!(error, grip::GripError::CorruptState(_)));
}

#[test]
fn empty_list_succeeds_without_mutating_any_filesystem_entry() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let before = support::snapshot(root.path());
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["details"]["mappings"], serde_json::json!([]));
    assert_eq!(before, support::snapshot(root.path()));
}

#[test]
fn invalid_registry_has_stable_mapping_failure_details() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    fs::write(
        grip_home.join("config.toml"),
        "schema_version = 1\n[[mappings]]\nkind = \"bogus\"\nsource = \"/source\"\ndestination = \"/destination\"\n",
    )
    .unwrap();
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["details"]["operation"], "mapping_list");
    assert_eq!(value["details"]["reason"], "invalid_registry");
    assert_eq!(
        value["details"]["paths"][0],
        grip_home.join("config.toml").display().to_string()
    );
}

#[test]
fn accepted_registry_rejects_noncanonical_intermediate_symlink_aliases() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let real = root.path().join("real");
    fs::create_dir(&real).unwrap();
    let canonical_source = real.join("source");
    fs::write(&canonical_source, "payload").unwrap();
    let canonical_source = fs::canonicalize(&canonical_source).unwrap();
    let alias = root.path().join("alias");
    symlink(&real, &alias).unwrap();
    let aliased_source = alias.join("source");
    let destination = root.path().join("destination");
    fs::write(
        grip_home.join("config.toml"),
        format!(
            "schema_version = 1\n\n[[mappings]]\nkind = \"file\"\nsource = {:?}\ndestination = {:?}\n",
            aliased_source.display().to_string(),
            destination.display().to_string()
        ),
    )
    .unwrap();
    let before = support::snapshot(root.path());

    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    assert_eq!(output.status.code(), Some(10));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["details"]["reason"], "unsafe_ancestry");
    assert_eq!(
        value["details"]["paths"],
        serde_json::json!([aliased_source, canonical_source])
    );
    assert_eq!(support::snapshot(root.path()), before);
}

#[test]
fn accepted_registry_validates_duplicate_canonical_sources_after_alias_resolution() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let real = root.path().join("real");
    fs::create_dir(&real).unwrap();
    let canonical_source = real.join("source");
    fs::write(&canonical_source, "payload").unwrap();
    let alias = root.path().join("alias");
    symlink(&real, &alias).unwrap();
    let aliased_source = alias.join("source");
    let first_destination = root.path().join("destination-a");
    let second_destination = root.path().join("destination-b");
    fs::write(
        grip_home.join("config.toml"),
        format!(
            "schema_version = 1\n\n[[mappings]]\nkind = \"file\"\nsource = {:?}\ndestination = {:?}\n\n[[mappings]]\nkind = \"file\"\nsource = {:?}\ndestination = {:?}\n",
            aliased_source.display().to_string(),
            first_destination.display().to_string(),
            canonical_source.display().to_string(),
            second_destination.display().to_string()
        ),
    )
    .unwrap();
    let before = support::snapshot(root.path());

    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    assert_eq!(output.status.code(), Some(10));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["details"]["reason"], "ownership_conflicts");
    assert!(
        value["details"]["conflicts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|conflict| conflict["reason"] == "duplicate_source")
    );
    assert_eq!(support::snapshot(root.path()), before);
}

#[test]
fn staged_and_publication_faults_preserve_or_precisely_report_acceptance() {
    for fault in [
        PublicationFault::CorruptStagedCandidate,
        PublicationFault::BeforeFinalRevalidation,
        PublicationFault::BeforeRegistryRename,
        PublicationFault::RegistryRename,
    ] {
        let root = tempfile::tempdir().unwrap();
        let grip_home_path = support::minimal_home(root.path());
        let home = home::select(Some(grip_home_path.clone().into_os_string()), None).unwrap();
        let snapshot = publication::load(&home, true).unwrap();
        let candidate = registry::Registry::new(Vec::new()).unwrap();
        let error =
            publication::publish_with_fault(&home, &snapshot, &candidate, Some(fault)).unwrap_err();
        assert_eq!(error.category(), grip::ResultCategory::InternalError);
        assert!(matches!(
            error,
            grip::GripError::Mapping {
                ref reason,
                publication_visible: false,
                ..
            } if reason == "publication_failure"
        ));
        assert_eq!(
            fs::read(grip_home_path.join("config.toml")).unwrap(),
            snapshot.bytes
        );
    }

    let root = tempfile::tempdir().unwrap();
    let grip_home_path = support::minimal_home(root.path());
    let source = root.path().join("source");
    fs::write(&source, "payload").unwrap();
    let canonical_source = fs::canonicalize(&source).unwrap();
    let canonical_destination = fs::canonicalize(root.path()).unwrap().join("destination");
    let home = home::select(Some(grip_home_path.clone().into_os_string()), None).unwrap();
    let snapshot = publication::load(&home, true).unwrap();
    let candidate = registry::Registry::new(vec![Mapping::new(
        MappingKind::File,
        canonical_source,
        canonical_destination,
    )])
    .unwrap();
    let error = publication::publish_with_fault(
        &home,
        &snapshot,
        &candidate,
        Some(PublicationFault::DirectorySync),
    )
    .unwrap_err();
    let outcome = grip::CommandOutcome::failure(&error);
    assert_eq!(outcome.details["publication_visible"], true);
    assert_eq!(outcome.details["durability_confirmed"], false);
    assert!(matches!(
        error,
        grip::GripError::Mapping {
            ref reason,
            publication_visible: true,
            ..
        } if reason == "publication_failure"
    ));
    assert_eq!(
        registry::decode(&fs::read_to_string(grip_home_path.join("config.toml")).unwrap()).unwrap(),
        candidate
    );
}

#[test]
fn registry_staging_substitution_is_rejected_and_not_removed_as_attempt_owned() {
    for fault in [
        PublicationFault::SubstituteRegistryStagingWithSymlink,
        PublicationFault::SubstituteRegistryStagingWithDirectory,
        PublicationFault::SubstituteRegistryStagingWithFile,
    ] {
        let root = tempfile::tempdir().unwrap();
        let grip_home_path = support::minimal_home(root.path());
        let home = home::select(Some(grip_home_path.clone().into_os_string()), None).unwrap();
        let snapshot = publication::load(&home, true).unwrap();
        let candidate = registry::Registry::new(Vec::new()).unwrap();

        let error =
            publication::publish_with_fault(&home, &snapshot, &candidate, Some(fault)).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("pathname no longer identifies the attempt-owned file")
        );
        assert_eq!(
            fs::read(grip_home_path.join("config.toml")).unwrap(),
            snapshot.bytes
        );

        let staging = fs::read_dir(&grip_home_path)
            .unwrap()
            .map(Result::unwrap)
            .find(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".config.tmp-")
            })
            .expect("substituted staging node must be preserved");
        let metadata = fs::symlink_metadata(staging.path()).unwrap();
        match fault {
            PublicationFault::SubstituteRegistryStagingWithSymlink => {
                assert!(metadata.file_type().is_symlink());
            }
            PublicationFault::SubstituteRegistryStagingWithDirectory => {
                assert!(metadata.is_dir());
            }
            PublicationFault::SubstituteRegistryStagingWithFile => {
                assert!(metadata.is_file());
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn recovery_staging_symlink_substitution_is_rejected_and_preserved() {
    let root = tempfile::tempdir().unwrap();
    let grip_home_path = support::minimal_home(root.path());
    let home = home::select(Some(grip_home_path.clone().into_os_string()), None).unwrap();
    let snapshot = publication::load(&home, true).unwrap();
    let candidate = registry::Registry::new(Vec::new()).unwrap();

    let error = publication::publish_with_fault(
        &home,
        &snapshot,
        &candidate,
        Some(PublicationFault::SubstituteRecoveryStagingWithSymlink),
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("pathname no longer identifies the attempt-owned file")
    );
    assert_eq!(
        fs::read(grip_home_path.join("config.toml")).unwrap(),
        snapshot.bytes
    );

    let digest = format!("{:x}", Sha256::digest(&snapshot.bytes));
    let generation = grip_home_path.join(format!("state/recovery/registry/sha256-{digest}"));
    let staging = fs::read_dir(generation)
        .unwrap()
        .map(Result::unwrap)
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".config.tmp-")
        })
        .expect("substituted recovery staging node must be preserved");
    assert!(
        fs::symlink_metadata(staging.path())
            .unwrap()
            .file_type()
            .is_symlink()
    );
    let retry = publication::publish(&home, &snapshot, &candidate).unwrap_err();
    assert!(matches!(retry, GripError::CorruptState(_)));
    assert!(
        fs::symlink_metadata(staging.path())
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        fs::read(grip_home_path.join("config.toml")).unwrap(),
        snapshot.bytes
    );
}

#[test]
fn same_byte_registry_replacement_is_detected_by_descriptor_identity() {
    let root = tempfile::tempdir().unwrap();
    let grip_home_path = support::minimal_home(root.path());
    let home = home::select(Some(grip_home_path.clone().into_os_string()), None).unwrap();
    let snapshot = publication::load(&home, true).unwrap();
    let replacement = grip_home_path.join("replacement.toml");
    fs::write(&replacement, &snapshot.bytes).unwrap();
    fs::set_permissions(&replacement, fs::Permissions::from_mode(snapshot.mode)).unwrap();
    fs::rename(&replacement, grip_home_path.join("config.toml")).unwrap();
    let candidate = registry::Registry::new(Vec::new()).unwrap();
    let error = publication::publish(&home, &snapshot, &candidate).unwrap_err();
    assert!(matches!(
        error,
        grip::GripError::Mapping { ref reason, .. } if reason == "stale_registry"
    ));
    assert_eq!(
        fs::read(grip_home_path.join("config.toml")).unwrap(),
        snapshot.bytes
    );
}

#[test]
fn payload_content_and_metadata_survive_success_and_rejection_paths() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let payload = root.path().join("payload");
    fs::create_dir(&payload).unwrap();
    let source = payload.join("source");
    let destination = payload.join("destination");
    fs::write(&source, "source").unwrap();
    fs::write(&destination, "destination").unwrap();
    let before = support::snapshot(&payload);
    let add = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "mapping",
            "add",
            "file",
            source.to_str().unwrap(),
            destination.to_str().unwrap(),
        ],
    );
    assert!(add.status.success());
    assert_eq!(before, support::snapshot(&payload));
    assert!(!grip_home.join("state/state.json").exists());

    let rejected = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "mapping",
            "add",
            "file",
            source.to_str().unwrap(),
            destination.to_str().unwrap(),
        ],
    );
    assert!(!rejected.status.success());
    assert_eq!(before, support::snapshot(&payload));
    assert!(!grip_home.join("state/state.json").exists());

    let remove = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["mapping", "remove", source.to_str().unwrap()],
    );
    assert!(remove.status.success());
    assert_eq!(before, support::snapshot(&payload));
    assert!(!grip_home.join("state/state.json").exists());
}

#[test]
fn reader_and_writer_registry_permission_policy_is_enforced() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let config = grip_home.join("config.toml");
    let assert_failure =
        |output: &std::process::Output, exit: i32, code: &str, operation: &str, reason: &str| {
            assert_eq!(output.status.code(), Some(exit));
            let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(value["code"], code);
            assert_eq!(value["details"]["operation"], operation);
            assert_eq!(value["details"]["reason"], reason);
            assert_eq!(value["details"]["paths"][0], config.display().to_string());
        };

    fs::set_permissions(&config, fs::Permissions::from_mode(0o400)).unwrap();
    let before = support::snapshot(root.path());
    let list = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    assert!(list.status.success());
    assert_eq!(support::snapshot(root.path()), before);

    let source = root.path().join("source");
    fs::write(&source, "payload").unwrap();
    let before = support::snapshot(root.path());
    let add = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output=json",
            "mapping",
            "add",
            "file",
            source.to_str().unwrap(),
            root.path().join("destination").to_str().unwrap(),
        ],
    );
    assert_failure(
        &add,
        10,
        "invalid_configuration",
        "mapping_add",
        "unsafe_registry_mode",
    );
    assert_eq!(support::snapshot(root.path()), before);

    fs::set_permissions(&config, fs::Permissions::from_mode(0o666)).unwrap();
    let before = support::snapshot(root.path());
    let unsafe_mode = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    assert_failure(
        &unsafe_mode,
        10,
        "invalid_configuration",
        "mapping_list",
        "unsafe_registry_mode",
    );
    assert_eq!(support::snapshot(root.path()), before);

    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    let real_config = grip_home.join("real-config.toml");
    fs::rename(&config, &real_config).unwrap();
    symlink(&real_config, &config).unwrap();
    let before = support::snapshot(root.path());
    let symlinked = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    assert_failure(
        &symlinked,
        10,
        "invalid_configuration",
        "mapping_list",
        "unsafe_registry_mode",
    );
    assert_eq!(support::snapshot(root.path()), before);

    fs::remove_file(&config).unwrap();
    fs::create_dir(&config).unwrap();
    let before = support::snapshot(root.path());
    let directory = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    assert_failure(
        &directory,
        10,
        "invalid_configuration",
        "mapping_list",
        "unsafe_registry_mode",
    );
    assert_eq!(support::snapshot(root.path()), before);

    fs::remove_dir(&config).unwrap();
    let before = support::snapshot(root.path());
    let missing = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    assert_failure(
        &missing,
        10,
        "invalid_configuration",
        "mapping_list",
        "invalid_registry",
    );
    assert_eq!(support::snapshot(root.path()), before);

    fs::write(&config, "not valid TOML = [").unwrap();
    let before = support::snapshot(root.path());
    let malformed = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    assert_failure(
        &malformed,
        10,
        "invalid_configuration",
        "mapping_list",
        "invalid_registry",
    );
    assert_eq!(support::snapshot(root.path()), before);

    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    let before = support::snapshot(root.path());
    fs::set_permissions(&config, fs::Permissions::from_mode(0o000)).unwrap();
    let inaccessible = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    assert_failure(
        &inaccessible,
        10,
        "invalid_configuration",
        "mapping_list",
        "unsafe_registry_mode",
    );
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(support::snapshot(root.path()), before);
}

#[test]
fn unreadable_unsafe_mode_and_fifo_registry_are_rejected_without_mutation() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let config = grip_home.join("config.toml");

    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    let before = support::snapshot(root.path());
    fs::set_permissions(&config, fs::Permissions::from_mode(0o020)).unwrap();
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(output.status.code(), Some(10));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["code"], "invalid_configuration");
    assert_eq!(value["details"]["operation"], "mapping_list");
    assert_eq!(value["details"]["reason"], "unsafe_registry_mode");
    assert_eq!(value["details"]["paths"][0], config.display().to_string());
    assert_eq!(support::snapshot(root.path()), before);

    fs::remove_file(&config).unwrap();
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&config)
            .status()
            .unwrap()
            .success()
    );
    let before = support::snapshot(root.path());
    let output = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    assert_eq!(output.status.code(), Some(10));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["code"], "invalid_configuration");
    assert_eq!(value["details"]["operation"], "mapping_list");
    assert_eq!(value["details"]["reason"], "unsafe_registry_mode");
    assert_eq!(value["details"]["paths"][0], config.display().to_string());
    assert_eq!(support::snapshot(root.path()), before);
}

#[test]
fn unsupported_registry_schema_retains_mapping_details_for_every_operation() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let config = grip_home.join("config.toml");
    fs::write(&config, "schema_version = 2\nmappings = []\n").unwrap();
    let source = root.path().join("source");
    fs::write(&source, "payload").unwrap();
    let destination = root.path().join("destination");
    let source_text = source.to_str().unwrap();
    let destination_text = destination.to_str().unwrap();
    let cases = [
        (
            vec![
                "--output=json",
                "mapping",
                "add",
                "file",
                source_text,
                destination_text,
            ],
            "mapping_add",
            Some("file"),
        ),
        (
            vec!["--output=json", "mapping", "list"],
            "mapping_list",
            None,
        ),
        (
            vec!["--output=json", "mapping", "show", source_text],
            "mapping_show",
            None,
        ),
        (
            vec!["--output=json", "mapping", "remove", source_text],
            "mapping_remove",
            None,
        ),
    ];

    for (args, operation, kind) in cases {
        let before = support::snapshot(root.path());
        let output = support::command_with_grip_home(root.path(), &grip_home, &args);
        assert_eq!(output.status.code(), Some(11));
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["code"], "unsupported_schema");
        assert_eq!(value["details"]["operation"], operation);
        assert_eq!(value["details"]["reason"], "invalid_registry");
        assert_eq!(value["details"]["paths"][0], config.display().to_string());
        match kind {
            Some(expected) => assert_eq!(value["details"]["kind"], expected),
            None => assert!(value["details"].get("kind").is_none()),
        }
        assert_eq!(support::snapshot(root.path()), before);
    }
}

#[test]
fn mapping_writes_use_outer_mutation_lock_while_reads_remain_lock_free() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "payload").unwrap();
    let selected = home::select(Some(grip_home.clone().into_os_string()), None).unwrap();
    let held = grip::state::mutation_lock::MutationLock::acquire(&selected, "push").unwrap();

    let read = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &["--output=json", "mapping", "list"],
    );
    assert!(read.status.success());

    let blocked = support::command_with_grip_home(
        root.path(),
        &grip_home,
        &[
            "--output=json",
            "mapping",
            "add",
            "file",
            source.to_str().unwrap(),
            destination.to_str().unwrap(),
        ],
    );
    assert_eq!(blocked.status.code(), Some(13));
    let value = support::json(&blocked);
    assert_eq!(value["details"]["reason"], "state_contention");
    assert_eq!(value["details"]["owner"]["operation"], "push");
    assert!(!destination.exists());
    drop(held);
}
