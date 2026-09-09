mod support;

use grip::GripError;
use grip::mapping::{Mapping, MappingKind};
use grip::path_policy;
use grip::registry;
use grip::registry::publication::{self, PublicationFault};
use grip::state::lock::PublicationLock;
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};

#[test]
fn add_canonicalizes_paths_allows_absent_destination_and_does_not_touch_payloads() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("absent/child");
    fs::create_dir(&source).unwrap();
    let payload_before = support::snapshot(&source);
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output=json",
            "mapping",
            "add",
            "tree",
            "source",
            "~/absent/child",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(payload_before, support::snapshot(&source));
    assert!(!destination.exists());
    assert!(!metadata_dir.join("state/state.json").exists());
}

#[test]
fn rejects_relative_traversal_symlink_and_wrong_kind_paths() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let file = root.path().join("file");
    let directory = root.path().join("directory");
    fs::write(&file, "x").unwrap();
    fs::create_dir(&directory).unwrap();
    let link = root.path().join("link");
    symlink(&file, &link).unwrap();
    for (kind, source, reason) in [
        ("file", "/absolute", "invalid_project_relative_path"),
        ("file", "link", "symlink_endpoint"),
        ("tree", "file", "wrong_node_kind"),
    ] {
        let output = support::project_command(
            root.path(),
            &metadata_dir,
            &[
                "--output=json",
                "mapping",
                "add",
                kind,
                source,
                "~/destination",
            ],
        );
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["details"]["reason"], reason);
        assert_eq!(value["details"]["kind"], kind);
    }
    let traversal = "directory/../file";
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output=json",
            "mapping",
            "add",
            "file",
            traversal,
            "~/target",
        ],
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()["details"]["reason"],
        "invalid_project_relative_path"
    );
}

#[cfg(unix)]
#[test]
fn rejects_non_utf8_input_without_lossy_identity() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::process::Command;
    let root = tempfile::tempdir().unwrap();
    support::initialize_project_metadata(root.path());
    let source = OsString::from_vec(vec![0xff]);
    let output = Command::new(env!("CARGO_BIN_EXE_grip"))
        .env_clear()
        .env("HOME", root.path())
        .arg("--project")
        .arg(root.path())
        .args([
            OsString::from("--output=json"),
            OsString::from("mapping"),
            OsString::from("add"),
            OsString::from("file"),
            source,
            OsString::from("~/target"),
        ])
        .output()
        .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["details"]["reason"], "non_utf8_path");
}

#[test]
fn list_is_sorted_show_is_exact_and_readers_do_not_mutate() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source_a = root.path().join("source-a");
    let source_z = root.path().join("source-z");
    fs::write(&source_a, "a").unwrap();
    fs::write(&source_z, "z").unwrap();
    support::write_descriptor(
        &metadata_dir,
        &[
            ("file", &source_z, &root.path().join("destination-z")),
            ("file", &source_a, &root.path().join("destination-a")),
        ],
    );
    let before = support::snapshot(root.path());
    let list = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "list"],
    );
    let value: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(
        value["details"]["mappings"][0]["resolved"]["source"]["display"],
        fs::canonicalize(&source_a).unwrap().display().to_string()
    );
    let show = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "show", "source-z"],
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&show.stdout).unwrap()["details"]["mapping"]["resolved"]
            ["source"]["display"],
        fs::canonicalize(&source_z).unwrap().display().to_string()
    );
    assert_eq!(before, support::snapshot(root.path()));
}

#[test]
fn remove_changes_only_registry_intent_and_retains_exact_prior_bytes() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "source payload").unwrap();
    fs::write(&destination, "destination payload").unwrap();
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
    let prior = fs::read(metadata_dir.join("config.toml")).unwrap();
    let output =
        support::project_command(root.path(), &metadata_dir, &["mapping", "remove", "source"]);
    assert!(output.status.success());
    assert_eq!(fs::read_to_string(&source).unwrap(), "source payload");
    assert_eq!(
        fs::read_to_string(&destination).unwrap(),
        "destination payload"
    );
    let digest = format!("{:x}", Sha256::digest(&prior));
    assert_eq!(
        fs::read(metadata_dir.join(format!(
            "state/recovery/registry/sha256-{digest}/config.toml"
        )))
        .unwrap(),
        prior
    );
    assert!(
        registry::decode_descriptor(&fs::read_to_string(metadata_dir.join("config.toml")).unwrap())
            .unwrap()
            .mappings()
            .is_empty()
    );
}

#[test]
fn publication_rejects_stale_bytes_preserves_mode_and_cleans_staging_on_failure() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir_path = support::initialize_project_metadata(root.path());
    fs::set_permissions(
        metadata_dir_path.join("config.toml"),
        fs::Permissions::from_mode(0o640),
    )
    .unwrap();
    let home = support::project_home(&metadata_dir_path);
    let snapshot = publication::load(&home, true).unwrap();
    let candidate = registry::ResolvedRegistry::new(Vec::new()).unwrap();
    publication::publish_with_fault(
        &home,
        &snapshot,
        &candidate,
        Some(PublicationFault::BeforeRegistryRename),
    )
    .unwrap_err();
    assert_eq!(
        fs::read(metadata_dir_path.join("config.toml")).unwrap(),
        snapshot.bytes
    );
    assert!(fs::read_dir(&metadata_dir_path).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".config.tmp-")
    }));
    publication::publish(&home, &snapshot, &candidate).unwrap();
    assert_eq!(
        fs::metadata(metadata_dir_path.join("config.toml"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o640
    );
    let stale = publication::load(&home, true).unwrap();
    fs::write(
        metadata_dir_path.join("config.toml"),
        "schema_version = 1\nmappings = []\n# changed\n",
    )
    .unwrap();
    let error = publication::publish(&home, &stale, &candidate).unwrap_err();
    assert!(error.to_string().contains("changed before publication"));
    assert_eq!(
        fs::read_to_string(metadata_dir_path.join("config.toml")).unwrap(),
        "schema_version = 1\nmappings = []\n# changed\n"
    );
}

#[test]
fn publication_rejects_retargeted_submitted_ancestry_without_mutation() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir_path = support::initialize_project_metadata(root.path());
    let home = support::project_home(&metadata_dir_path);
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
    let candidate = registry::ResolvedRegistry::new(vec![Mapping::new(
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
        fs::read(metadata_dir_path.join("config.toml")).unwrap(),
        snapshot.bytes
    );
    assert_eq!(support::snapshot(&first), first_before);
    assert_eq!(support::snapshot(&second), second_before);
    assert!(!first.join("absent").exists());
    assert!(!second.join("absent").exists());
    assert!(!metadata_dir_path.join("state/recovery").exists());
}

#[test]
fn rejects_unsafe_registry_mode_and_missing_mapping() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::set_permissions(
        metadata_dir.join("config.toml"),
        fs::Permissions::from_mode(0o666),
    )
    .unwrap();
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "list"],
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()["details"]["reason"],
        "invalid_project_metadata"
    );
    fs::set_permissions(
        metadata_dir.join("config.toml"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    let missing = root.path().join("missing");
    fs::write(&missing, "x").unwrap();
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "show", "missing"],
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()["details"]["reason"],
        "mapping_not_found"
    );
}

#[test]
fn absent_show_and_remove_selectors_are_distinct_not_found_results_without_mutation() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let missing = "absent/source";
    let canonical_missing = fs::canonicalize(root.path()).unwrap().join("absent/source");
    let before = support::snapshot(root.path());

    for operation in ["show", "remove"] {
        let human =
            support::project_command(root.path(), &metadata_dir, &["mapping", operation, missing]);
        assert_eq!(human.status.code(), Some(10));
        assert_eq!(
            String::from_utf8(human.stdout).unwrap(),
            "Mapping not found\n"
        );
        assert_eq!(support::snapshot(root.path()), before);

        let json = support::project_command(
            root.path(),
            &metadata_dir,
            &["--output=json", "mapping", operation, missing],
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
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    fs::write(&source, "payload").unwrap();
    let real = root.path().join("real");
    fs::create_dir(&real).unwrap();
    let alias = root.path().join("alias");
    symlink(&real, &alias).unwrap();
    let canonical_destination = fs::canonicalize(&real).unwrap().join("absent-destination");
    let payload_before = support::snapshot(&real);

    let add = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output=json",
            "mapping",
            "add",
            "file",
            "source",
            "~/alias/absent-destination",
        ],
    );
    assert!(add.status.success());
    let value: serde_json::Value = serde_json::from_slice(&add.stdout).unwrap();
    assert_eq!(
        value["details"]["mapping"]["resolved"]["destination"]["display"],
        canonical_destination.display().to_string()
    );
    assert!(!canonical_destination.exists());
    assert_eq!(support::snapshot(&real), payload_before);

    let missing_selector = "alias/absent-source";
    for operation in ["show", "remove"] {
        let before = support::snapshot(root.path());
        let output = support::project_command(
            root.path(),
            &metadata_dir,
            &["--output=json", "mapping", operation, missing_selector],
        );
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["details"]["reason"], "mapping_not_found");
        assert_eq!(
            value["details"]["paths"][0],
            fs::canonicalize(root.path())
                .unwrap()
                .join("alias/absent-source")
                .display()
                .to_string()
        );
        assert_eq!(support::snapshot(root.path()), before);
    }
}

#[test]
fn lock_contention_is_nonblocking_and_retry_succeeds_after_release() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    fs::write(&source, "x").unwrap();
    let home = support::project_home(&metadata_dir);
    let lock = PublicationLock::acquire(
        &grip::state::lock::project_lock_path(&home, "registry.lock").unwrap(),
    )
    .unwrap();
    let blocked = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output=json",
            "mapping",
            "add",
            "file",
            "source",
            "~/destination",
        ],
    );
    assert_eq!(blocked.status.code(), Some(20));
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&blocked.stdout).unwrap()["details"]["reason"],
        "registry_contention"
    );
    drop(lock);
    let retry = support::project_command(
        root.path(),
        &metadata_dir,
        &["mapping", "add", "file", "source", "~/destination"],
    );
    assert!(retry.status.success());
}

#[test]
fn recovery_collision_and_injected_recovery_failure_preserve_registry() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir_path = support::initialize_project_metadata(root.path());
    let home = support::project_home(&metadata_dir_path);
    let snapshot = publication::load(&home, true).unwrap();
    let candidate = registry::ResolvedRegistry::new(Vec::new()).unwrap();
    publication::publish_with_fault(
        &home,
        &snapshot,
        &candidate,
        Some(PublicationFault::BeforeRecoveryRename),
    )
    .unwrap_err();
    assert_eq!(
        fs::read(metadata_dir_path.join("config.toml")).unwrap(),
        snapshot.bytes
    );

    let digest = format!("{:x}", Sha256::digest(&snapshot.bytes));
    let generation = metadata_dir_path.join(format!("state/recovery/registry/sha256-{digest}"));
    fs::write(generation.join("config.toml"), "collision").unwrap();
    let error = publication::publish(&home, &snapshot, &candidate).unwrap_err();
    assert_eq!(error.category(), grip::ResultCategory::InternalError);
    assert!(matches!(
        error,
        GripError::Mapping { ref reason, .. } if reason == "registry_recovery_failure"
    ));
    assert_eq!(
        fs::read(metadata_dir_path.join("config.toml")).unwrap(),
        snapshot.bytes
    );
}

#[test]
fn invalid_unrelated_mapping_blocks_list_and_remove() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let other = root.path().join("other");
    fs::write(&source, "x").unwrap();
    fs::write(&other, "y").unwrap();
    let canonical_source = fs::canonicalize(&source).unwrap();
    let canonical_other = fs::canonicalize(&other).unwrap();
    support::write_descriptor(
        &metadata_dir,
        &[
            ("file", &canonical_source, &root.path().join("destination")),
            ("tree", &canonical_other, &root.path().join("wrong-kind")),
        ],
    );
    for arguments in [
        vec!["--output=json", "mapping", "list"],
        vec!["--output=json", "mapping", "remove", "source"],
    ] {
        let output = support::project_command(root.path(), &metadata_dir, &arguments);
        assert_eq!(output.status.code(), Some(10));
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()["details"]["reason"],
            "invalid_project_metadata"
        );
    }
}

#[test]
fn invalid_registry_precedes_invalid_command_paths_for_every_mapping_operation() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(
        metadata_dir.join("config.toml"),
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
        let output = support::project_command(root.path(), &metadata_dir, &arguments);
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(output.status.code(), Some(11));
        assert_eq!(value["details"]["operation"], "project_selection");
        assert_eq!(value["details"]["reason"], "invalid_project_metadata");
        assert!(value["details"].get("kind").is_none(), "{operation}");
        assert_eq!(support::snapshot(root.path()), before);
    }
}

#[test]
fn path_drift_between_inspection_and_publication_blocks_update() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir_path = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    fs::write(&source, "x").unwrap();
    support::write_descriptor(
        &metadata_dir_path,
        &[("file", &source, &root.path().join("destination"))],
    );
    let home = support::project_home(&metadata_dir_path);
    let snapshot = publication::load(&home, true).unwrap();
    let candidate = registry::ResolvedRegistry::new(snapshot.registry.mappings().to_vec()).unwrap();
    fs::remove_file(&source).unwrap();
    fs::create_dir(&source).unwrap();
    let error = publication::publish(&home, &snapshot, &candidate).unwrap_err();
    assert!(matches!(
        error,
        grip::GripError::Mapping { ref reason, .. } if reason == "stale_path_evidence"
    ));
    assert_eq!(
        fs::read(metadata_dir_path.join("config.toml")).unwrap(),
        snapshot.bytes
    );
}

#[test]
fn byte_identical_recovery_is_reused_and_unexpected_staging_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir_path = support::initialize_project_metadata(root.path());
    let home = support::project_home(&metadata_dir_path);
    let snapshot = publication::load(&home, true).unwrap();
    let candidate = registry::ResolvedRegistry::new(Vec::new()).unwrap();
    publication::publish(&home, &snapshot, &candidate).unwrap();
    let second = publication::load(&home, true).unwrap();
    publication::publish(&home, &second, &candidate).unwrap();

    fs::write(metadata_dir_path.join(".config.tmp-unowned"), "unexpected").unwrap();
    let third = publication::load(&home, true).unwrap();
    let error = publication::publish(&home, &third, &candidate).unwrap_err();
    assert!(matches!(error, grip::GripError::CorruptState(_)));
}

#[test]
fn empty_list_succeeds_without_mutating_any_filesystem_entry() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let before = support::snapshot(root.path());
    let output = support::project_command(
        root.path(),
        &metadata_dir,
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
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(
        metadata_dir.join("config.toml"),
        "schema_version = 1\n[[mappings]]\nkind = \"bogus\"\nsource = \"/source\"\ndestination = \"/destination\"\n",
    )
    .unwrap();
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "list"],
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["details"]["operation"], "project_selection");
    assert_eq!(value["details"]["reason"], "invalid_project_metadata");
    assert!(value["details"].get("paths").is_none());
}

#[test]
fn accepted_registry_rejects_noncanonical_intermediate_symlink_aliases() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let real = root.path().join("real");
    fs::create_dir(&real).unwrap();
    let canonical_source = real.join("source");
    fs::write(&canonical_source, "payload").unwrap();
    let alias = root.path().join("alias");
    symlink(&real, &alias).unwrap();
    fs::write(
        metadata_dir.join("config.toml"),
        "schema_version = 2\n\n[[mappings]]\nkind = \"file\"\nsource = \"alias/source\"\ndestination = \"~/destination\"\n",
    )
    .unwrap();
    let before = support::snapshot(root.path());

    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "list"],
    );
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        value["details"]["mappings"][0]["declared"]["source"],
        "alias/source"
    );
    assert_eq!(
        value["details"]["mappings"][0]["resolved"]["source"]["display"],
        fs::canonicalize(&canonical_source)
            .unwrap()
            .display()
            .to_string()
    );
    assert_eq!(support::snapshot(root.path()), before);
}

#[test]
fn accepted_registry_validates_duplicate_canonical_sources_after_alias_resolution() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let real = root.path().join("real");
    fs::create_dir(&real).unwrap();
    let canonical_source = real.join("source");
    fs::write(&canonical_source, "payload").unwrap();
    let alias = root.path().join("alias");
    symlink(&real, &alias).unwrap();
    fs::write(
        metadata_dir.join("config.toml"),
        "schema_version = 2\n\n[[mappings]]\nkind = \"file\"\nsource = \"alias/source\"\ndestination = \"~/destination-a\"\n\n[[mappings]]\nkind = \"file\"\nsource = \"real/source\"\ndestination = \"~/destination-b\"\n",
    )
    .unwrap();
    let before = support::snapshot(root.path());

    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "list"],
    );
    assert_eq!(output.status.code(), Some(10));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["details"]["reason"], "invalid_project_metadata");
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
        let metadata_dir_path = support::initialize_project_metadata(root.path());
        let home = support::project_home(&metadata_dir_path);
        let snapshot = publication::load(&home, true).unwrap();
        let candidate = registry::ResolvedRegistry::new(Vec::new()).unwrap();
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
            fs::read(metadata_dir_path.join("config.toml")).unwrap(),
            snapshot.bytes
        );
    }

    let root = tempfile::tempdir().unwrap();
    let metadata_dir_path = support::initialize_project_metadata(root.path());
    let home = support::project_home(&metadata_dir_path);
    let snapshot = publication::load(&home, true).unwrap();
    let candidate = registry::ResolvedRegistry::new(Vec::new()).unwrap();
    let error = publication::publish_with_fault(
        &home,
        &snapshot,
        &candidate,
        Some(PublicationFault::DirectorySync),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        grip::GripError::Mapping {
            ref reason,
            publication_visible: true,
            ..
        } if reason == "publication_failure"
    ));
    assert_eq!(publication::load(&home, true).unwrap().registry, candidate);
}

#[test]
fn registry_staging_substitution_is_rejected_and_not_removed_as_attempt_owned() {
    for fault in [
        PublicationFault::SubstituteRegistryStagingWithSymlink,
        PublicationFault::SubstituteRegistryStagingWithDirectory,
        PublicationFault::SubstituteRegistryStagingWithFile,
    ] {
        let root = tempfile::tempdir().unwrap();
        let metadata_dir_path = support::initialize_project_metadata(root.path());
        let home = support::project_home(&metadata_dir_path);
        let snapshot = publication::load(&home, true).unwrap();
        let candidate = registry::ResolvedRegistry::new(Vec::new()).unwrap();

        let error =
            publication::publish_with_fault(&home, &snapshot, &candidate, Some(fault)).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("pathname no longer identifies the attempt-owned file")
        );
        assert_eq!(
            fs::read(metadata_dir_path.join("config.toml")).unwrap(),
            snapshot.bytes
        );

        let staging = fs::read_dir(&metadata_dir_path)
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
    let metadata_dir_path = support::initialize_project_metadata(root.path());
    let home = support::project_home(&metadata_dir_path);
    let snapshot = publication::load(&home, true).unwrap();
    let candidate = registry::ResolvedRegistry::new(Vec::new()).unwrap();

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
        fs::read(metadata_dir_path.join("config.toml")).unwrap(),
        snapshot.bytes
    );

    let digest = format!("{:x}", Sha256::digest(&snapshot.bytes));
    let generation = metadata_dir_path.join(format!("state/recovery/registry/sha256-{digest}"));
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
        fs::read(metadata_dir_path.join("config.toml")).unwrap(),
        snapshot.bytes
    );
}

#[test]
fn same_byte_registry_replacement_is_detected_by_descriptor_identity() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir_path = support::initialize_project_metadata(root.path());
    let home = support::project_home(&metadata_dir_path);
    let snapshot = publication::load(&home, true).unwrap();
    let replacement = metadata_dir_path.join("replacement.toml");
    fs::write(&replacement, &snapshot.bytes).unwrap();
    fs::set_permissions(&replacement, fs::Permissions::from_mode(snapshot.mode)).unwrap();
    fs::rename(&replacement, metadata_dir_path.join("config.toml")).unwrap();
    let candidate = registry::ResolvedRegistry::new(Vec::new()).unwrap();
    let error = publication::publish(&home, &snapshot, &candidate).unwrap_err();
    assert!(matches!(
        error,
        grip::GripError::Mapping { ref reason, .. } if reason == "stale_registry"
    ));
    assert_eq!(
        fs::read(metadata_dir_path.join("config.toml")).unwrap(),
        snapshot.bytes
    );
}

#[test]
fn payload_content_and_metadata_survive_success_and_rejection_paths() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let payload = root.path().join("payload");
    fs::create_dir(&payload).unwrap();
    let source = payload.join("source");
    let destination = payload.join("destination");
    fs::write(&source, "source").unwrap();
    fs::write(&destination, "destination").unwrap();
    let before = support::snapshot(&payload);
    let add = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "mapping",
            "add",
            "file",
            "payload/source",
            "~/payload/destination",
        ],
    );
    assert!(add.status.success());
    assert_eq!(before, support::snapshot(&payload));
    assert!(!metadata_dir.join("state/state.json").exists());

    let rejected = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "mapping",
            "add",
            "file",
            "payload/source",
            "~/payload/destination",
        ],
    );
    assert!(!rejected.status.success());
    assert_eq!(before, support::snapshot(&payload));
    assert!(!metadata_dir.join("state/state.json").exists());

    let remove = support::project_command(
        root.path(),
        &metadata_dir,
        &["mapping", "remove", "payload/source"],
    );
    assert!(remove.status.success());
    assert_eq!(before, support::snapshot(&payload));
    assert!(!metadata_dir.join("state/state.json").exists());
}

#[test]
fn reader_and_writer_registry_permission_policy_is_enforced() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let config = metadata_dir.join("config.toml");
    let canonical_config = fs::canonicalize(root.path())
        .unwrap()
        .join(".grip/config.toml");
    let assert_failure =
        |output: &std::process::Output, exit: i32, code: &str, operation: &str, reason: &str| {
            assert_eq!(output.status.code(), Some(exit));
            let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(value["code"], code);
            assert_eq!(value["details"]["operation"], operation);
            assert_eq!(value["details"]["reason"], reason);
            if let Some(path) = value["details"]["paths"].get(0) {
                assert_eq!(path.as_str(), Some(canonical_config.to_str().unwrap()));
            }
        };

    fs::set_permissions(&config, fs::Permissions::from_mode(0o400)).unwrap();
    let before = support::snapshot(root.path());
    let list = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "list"],
    );
    assert!(list.status.success());
    assert_eq!(support::snapshot(root.path()), before);

    let source = root.path().join("source");
    fs::write(&source, "payload").unwrap();
    let before = support::snapshot(root.path());
    let add = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output=json",
            "mapping",
            "add",
            "file",
            "source",
            "~/destination",
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
    let unsafe_mode = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "list"],
    );
    assert_failure(
        &unsafe_mode,
        10,
        "invalid_configuration",
        "project_selection",
        "invalid_project_metadata",
    );
    assert_eq!(support::snapshot(root.path()), before);

    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    let real_config = metadata_dir.join("real-config.toml");
    fs::rename(&config, &real_config).unwrap();
    symlink(&real_config, &config).unwrap();
    let before = support::snapshot(root.path());
    let symlinked = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "list"],
    );
    assert_failure(
        &symlinked,
        10,
        "invalid_configuration",
        "project_selection",
        "invalid_project_metadata",
    );
    assert_eq!(support::snapshot(root.path()), before);

    fs::remove_file(&config).unwrap();
    fs::create_dir(&config).unwrap();
    let before = support::snapshot(root.path());
    let directory = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "list"],
    );
    assert_failure(
        &directory,
        10,
        "invalid_configuration",
        "project_selection",
        "invalid_project_metadata",
    );
    assert_eq!(support::snapshot(root.path()), before);

    fs::remove_dir(&config).unwrap();
    let before = support::snapshot(root.path());
    let missing = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "list"],
    );
    assert_failure(
        &missing,
        10,
        "invalid_configuration",
        "project_selection",
        "invalid_project_root",
    );
    assert_eq!(support::snapshot(root.path()), before);

    fs::write(&config, "not valid TOML = [").unwrap();
    let before = support::snapshot(root.path());
    let malformed = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "list"],
    );
    assert_failure(
        &malformed,
        10,
        "invalid_configuration",
        "project_selection",
        "invalid_project_metadata",
    );
    assert_eq!(support::snapshot(root.path()), before);

    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    let before = support::snapshot(root.path());
    fs::set_permissions(&config, fs::Permissions::from_mode(0o000)).unwrap();
    let inaccessible = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "list"],
    );
    assert_failure(
        &inaccessible,
        10,
        "invalid_configuration",
        "project_selection",
        "invalid_project_metadata",
    );
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(support::snapshot(root.path()), before);
}

#[test]
fn unreadable_unsafe_mode_and_fifo_registry_are_rejected_without_mutation() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let config = metadata_dir.join("config.toml");

    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    let before = support::snapshot(root.path());
    fs::set_permissions(&config, fs::Permissions::from_mode(0o020)).unwrap();
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "list"],
    );
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(output.status.code(), Some(10));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["code"], "invalid_configuration");
    assert_eq!(value["details"]["operation"], "project_selection");
    assert_eq!(value["details"]["reason"], "invalid_project_metadata");
    assert!(value["details"].get("paths").is_none());
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
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "list"],
    );
    assert_eq!(output.status.code(), Some(10));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["code"], "invalid_configuration");
    assert_eq!(value["details"]["operation"], "project_selection");
    assert_eq!(value["details"]["reason"], "invalid_project_metadata");
    assert!(value["details"].get("paths").is_none());
    assert_eq!(support::snapshot(root.path()), before);
}

#[test]
fn unsupported_registry_schema_retains_mapping_details_for_every_operation() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let config = metadata_dir.join("config.toml");
    fs::write(&config, "schema_version = 3\nmappings = []\n").unwrap();
    let source = root.path().join("source");
    fs::write(&source, "payload").unwrap();
    let source_text = "source";
    let destination_text = "~/destination";
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
        let output = support::project_command(root.path(), &metadata_dir, &args);
        assert_eq!(output.status.code(), Some(11));
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["code"], "unsupported_schema");
        assert_eq!(value["details"]["operation"], "project_selection");
        assert_eq!(value["details"]["reason"], "invalid_project_metadata");
        assert!(value["details"].get("paths").is_none());
        assert!(
            value["details"].get("kind").is_none(),
            "{operation} {kind:?}"
        );
        assert_eq!(support::snapshot(root.path()), before);
    }
}

#[test]
fn mapping_writes_use_outer_mutation_lock_while_reads_remain_lock_free() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "payload").unwrap();
    let selected = support::project_home(&metadata_dir);
    let held = grip::state::mutation_lock::MutationLock::acquire(&selected, "push").unwrap();

    let read = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "list"],
    );
    assert!(read.status.success());

    let blocked = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output=json",
            "mapping",
            "add",
            "file",
            "source",
            "~/destination",
        ],
    );
    assert_eq!(blocked.status.code(), Some(13));
    let value = support::json(&blocked);
    assert_eq!(value["details"]["reason"], "state_contention");
    assert_eq!(value["details"]["owner"]["operation"], "push");
    assert!(!destination.exists());
    drop(held);
}
