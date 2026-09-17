mod support;

use std::fs;

const LARGE_FILE_BYTES: usize = 19 * 1024 * 1024;

fn json(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

fn write_large_file(path: &std::path::Path, byte: u8) {
    let chunk = vec![byte; 64 * 1024];
    let mut file = fs::File::create(path).unwrap();
    for _ in 0..(LARGE_FILE_BYTES / chunk.len()) {
        std::io::Write::write_all(&mut file, &chunk).unwrap();
    }
}

#[test]
fn add_list_and_remove_use_flat_commands_without_copying_payloads() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    fs::write(&source, "payload").unwrap();

    let add = support::project_command(
        root.path(),
        &metadata_dir,
        &["-o", "json", "add", "source", "~/destination"],
    );
    assert_eq!(add.status.code(), Some(0));
    assert_eq!(json(&add)["details"]["operation"], "add");
    assert!(!root.path().join("destination").exists());

    let list = support::project_command(
        root.path(),
        &metadata_dir,
        &["-o", "json", "list", "source"],
    );
    assert_eq!(list.status.code(), Some(0));
    assert_eq!(
        json(&list)["details"]["mappings"].as_array().unwrap().len(),
        1
    );

    let remove = support::project_command(
        root.path(),
        &metadata_dir,
        &["-o", "json", "remove", "source"],
    );
    assert_eq!(remove.status.code(), Some(0));
    assert_eq!(json(&remove)["details"]["operation"], "remove");
    assert_eq!(fs::read_to_string(source).unwrap(), "payload");
}

#[test]
fn human_mapping_commands_render_declared_rows_only() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    fs::write(&source, "payload").unwrap();

    let add = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "source", "~/destination"],
    );
    assert_eq!(
        String::from_utf8(add.stdout).unwrap(),
        "Mapped:\n source -> ~/destination\n"
    );

    let list = support::project_command(root.path(), &metadata_dir, &["list"]);
    assert_eq!(
        String::from_utf8(list.stdout).unwrap(),
        "1 mapping(s):\n source -> ~/destination\n"
    );

    let selected = support::project_command(root.path(), &metadata_dir, &["list", "source"]);
    assert_eq!(
        String::from_utf8(selected.stdout).unwrap(),
        "1 mapping(s):\n file source -> ~/destination\n"
    );

    let removed = support::project_command(root.path(), &metadata_dir, &["remove", "source"]);
    assert_eq!(
        String::from_utf8(removed.stdout).unwrap(),
        "Mapping removed:\n file source -> ~/destination\n"
    );
}

#[test]
fn force_add_replaces_one_equal_destination_file_mapping_without_mutating_payloads() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let old = root.path().join("target/debug/grip");
    let new = root.path().join("target/release/grip");
    let destination = root.path().join(".local/bin/grip");
    fs::create_dir_all(old.parent().unwrap()).unwrap();
    fs::create_dir_all(new.parent().unwrap()).unwrap();
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    fs::write(&old, "debug payload").unwrap();
    fs::write(&new, "release payload").unwrap();
    fs::write(&destination, "installed payload").unwrap();
    let old_before = fs::read(&old).unwrap();
    let new_before = fs::read(&new).unwrap();
    let destination_before = fs::read(&destination).unwrap();

    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "target/debug/grip", "~/.local/bin/./grip"],
        )
        .status
        .success()
    );

    let replaced = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "-o",
            "json",
            "add",
            "--force",
            "target/release/grip",
            "~/.local/bin/grip",
        ],
    );
    assert!(replaced.status.success(), "{replaced:?}");
    let replaced_json = json(&replaced);
    assert_eq!(replaced_json["message"], "Mapping replaced");
    assert_eq!(
        replaced_json["details"]["mapping"]["declared"]["source"],
        "target/release/grip"
    );
    assert_eq!(
        replaced_json["details"]["replaced_mapping"]["declared"]["source"],
        "target/debug/grip"
    );
    assert_eq!(fs::read(&old).unwrap(), old_before);
    assert_eq!(fs::read(&new).unwrap(), new_before);
    assert_eq!(fs::read(&destination).unwrap(), destination_before);
    let descriptor = fs::read_to_string(metadata_dir.join("config.toml")).unwrap();
    assert!(!descriptor.contains("target/debug/grip"));
    assert!(descriptor.contains("target/release/grip"));
    let status = json(&support::project_command(
        root.path(),
        &metadata_dir,
        &["-o", "json", "status"],
    ));
    assert_eq!(
        status["details"]["records"][0]["classification"],
        "source_only_change"
    );

    let human = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "--force", "target/debug/grip", "~/.local/bin/grip"],
    );
    assert!(human.status.success(), "{human:?}");
    assert_eq!(
        String::from_utf8(human.stdout).unwrap(),
        "Mapping replaced:\n  old: target/release/grip -> ~/.local/bin/grip\n  new: target/debug/grip -> ~/.local/bin/grip\n"
    );
}

#[test]
fn retrying_a_visible_forced_replacement_completes_state_and_preserves_replacement_output() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let old = root.path().join("old");
    let new = root.path().join("new");
    let destination = root.path().join("destination");
    fs::write(&old, "old").unwrap();
    fs::write(&new, "new").unwrap();
    fs::write(&destination, "destination").unwrap();
    assert!(
        support::project_command(root.path(), &metadata_dir, &["add", "old", "~/destination"],)
            .status
            .success()
    );

    let home =
        grip::project::ProjectPaths::project_metadata(metadata_dir.clone(), root.path().into());
    let prior = grip::registry::publication::load(&home, true).unwrap();
    let state_snapshot = grip::state::publication::load(&home).unwrap();
    let portable = grip::mapping::PortableMapping::parse_cli(
        grip::mapping::MappingKind::File,
        std::ffi::OsStr::new("new"),
        std::ffi::OsStr::new("~/destination"),
    )
    .unwrap();
    let displaced_portable = grip::mapping::PortableMapping::parse_cli(
        grip::mapping::MappingKind::File,
        std::ffi::OsStr::new("old"),
        std::ffi::OsStr::new("~/destination"),
    )
    .unwrap();
    let added = grip::mapping::Mapping::new(
        grip::mapping::MappingKind::File,
        fs::canonicalize(&new).unwrap(),
        fs::canonicalize(&destination).unwrap(),
    );
    let displaced = grip::mapping::Mapping::new(
        grip::mapping::MappingKind::File,
        fs::canonicalize(&old).unwrap(),
        fs::canonicalize(&destination).unwrap(),
    );
    let candidate_descriptor =
        grip::registry::ProjectDescriptorV2::new(vec![portable.clone()]).unwrap();
    let descriptor_bytes = grip::registry::encode_descriptor(&candidate_descriptor).unwrap();
    let candidate = grip::registry::ResolvedRegistry::new(vec![added.clone()]).unwrap();
    let mut accepted = state_snapshot.accepted.clone();
    let displaced_identity = grip::observation::model::ResolvedMapping::from(&displaced);
    accepted
        .complete_baselines
        .retain(|identity, _| identity.mapping != displaced_identity);
    let selection = grip::observation::model::Selection::Mapping(
        grip::observation::model::ResolvedMapping::from(&added),
    );
    let observed = grip::observation::inspect_candidate(
        &home,
        &candidate,
        &descriptor_bytes,
        &prior,
        &accepted,
        &selection,
    )
    .unwrap();
    let records = observed
        .values()
        .map(|entry| grip::classification::classify_accepted(entry, &accepted))
        .collect::<Vec<_>>();
    let next = grip::baseline::build_for_add(&accepted, &records).unwrap();
    let state_bytes = grip::state::publication::prepare_candidate_for_descriptor(
        &home,
        &state_snapshot,
        &next.next.complete_baselines,
        &descriptor_bytes,
    )
    .unwrap();
    let fence = grip::state::add_fence::AddPublicationFenceV1::with_context(
        &added,
        grip::state::add_fence::FenceContext::replacement(
            &portable,
            &displaced,
            &displaced_portable,
        ),
        prior.bytes.clone(),
        descriptor_bytes.clone(),
        state_snapshot.bytes.clone(),
        state_bytes.clone(),
    );
    grip::state::add_fence::create_verified(&home, &fence).unwrap();
    fs::write(metadata_dir.join("config.toml"), &descriptor_bytes).unwrap();

    let retried = support::project_command(
        root.path(),
        &metadata_dir,
        &["-o", "json", "add", "--force", "new", "~/./destination"],
    );
    assert!(retried.status.success(), "{retried:?}");
    let output = json(&retried);
    assert_eq!(output["message"], "Mapping replaced");
    assert_eq!(
        output["details"]["replaced_mapping"]["declared"]["source"],
        "old"
    );
    assert_eq!(
        output["details"]["mapping"]["declared"]["destination"],
        "~/destination"
    );
    assert_eq!(
        fs::read(metadata_dir.join("state/state.json")).unwrap(),
        state_bytes
    );
    assert!(!metadata_dir.join("state/add-fence.json").exists());
}

#[test]
fn human_list_renders_five_declared_rows_without_resolved_endpoints() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    for index in 1..=5 {
        let source = format!("source-{index}");
        let destination = format!("~/destination-{index}");
        fs::write(root.path().join(&source), "payload").unwrap();
        let added =
            support::project_command(root.path(), &metadata_dir, &["add", &source, &destination]);
        assert!(added.status.success());
    }

    let output = support::project_command(root.path(), &metadata_dir, &["list"]);
    let text = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        text,
        "5 mapping(s):\n source-1 -> ~/destination-1\n source-2 -> ~/destination-2\n source-3 -> ~/destination-3\n source-4 -> ~/destination-4\n source-5 -> ~/destination-5\n"
    );
    assert!(!text.contains("resolved"));
}

#[test]
fn add_accepts_absolute_and_non_normalized_home_destinations_without_copying_payloads() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(root.path().join("first"), "payload").unwrap();
    fs::write(root.path().join("second"), "payload").unwrap();
    let absolute = root.path().join("outside").join("absolute");

    let home_relative = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "first", "~/a//./b/../home-destination"],
    );
    assert_eq!(home_relative.status.code(), Some(0));
    let absolute_add = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "second", absolute.to_str().unwrap()],
    );
    assert_eq!(absolute_add.status.code(), Some(0));
    let descriptor = fs::read_to_string(metadata_dir.join("config.toml")).unwrap();
    assert!(descriptor.contains("destination = \"~/a//./b/../home-destination\""));
    assert!(descriptor.contains(&format!("destination = \"{}\"", absolute.display())));
    assert!(!root.path().join("home-destination").exists());
    assert!(!absolute.exists());
}

#[test]
fn add_accepts_relative_destination_without_normalizing_its_declaration() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(root.path().join("README.md"), "payload").unwrap();
    let destination_root = root.path().join("..").join("grip-destination");
    fs::create_dir_all(&destination_root).unwrap();
    let output = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "README.md", "../grip-destination/./README.md"],
    );
    assert_eq!(output.status.code(), Some(0));
    assert!(
        fs::read_to_string(metadata_dir.join("config.toml"))
            .unwrap()
            .contains("destination = \"../grip-destination/./README.md\"")
    );
}

#[test]
fn add_rejects_invalid_destination_forms_without_changing_the_descriptor() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(root.path().join("README.md"), "payload").unwrap();
    let before = fs::read(metadata_dir.join("config.toml")).unwrap();
    for destination in ["", "~other/x", "$HOME/x"] {
        let output = support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "README.md", destination],
        );
        assert_ne!(output.status.code(), Some(0), "{destination}");
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains("destination must be an absolute path, ~, a ~/ path, or a project-relative path resolved from the selected project root")
        );
        assert_eq!(fs::read(metadata_dir.join("config.toml")).unwrap(), before);
    }
}

#[test]
fn add_requires_an_existing_compatible_endpoint() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let missing = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "missing", "~/also-missing"],
    );
    assert_ne!(missing.status.code(), Some(0));

    fs::write(root.path().join("file"), "payload").unwrap();
    fs::create_dir(root.path().join("directory")).unwrap();
    let incompatible =
        support::project_command(root.path(), &metadata_dir, &["add", "file", "~/directory"]);
    assert_ne!(incompatible.status.code(), Some(0));
}

#[test]
fn add_establishes_a_baseline_only_for_matching_existing_endpoints() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(root.path().join("source"), "same").unwrap();
    fs::write(root.path().join("destination"), "same").unwrap();
    support::copy_complete_metadata(
        &root.path().join("source"),
        &root.path().join("destination"),
        grip::discovery::model::NodeKind::File,
    );

    let added = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "source", "~/destination"],
    );
    assert_eq!(added.status.code(), Some(0));
    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(metadata_dir.join("state/state.json")).unwrap()).unwrap();
    assert_eq!(state["payload"]["baselines"].as_array().unwrap().len(), 1);
    assert!(state["payload"].get("pending_retirements").is_none());
}

#[test]
fn add_of_unequal_file_is_ready_for_an_ordinary_push() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(root.path().join("source"), "source wins").unwrap();
    fs::write(root.path().join("destination"), "destination loses").unwrap();

    assert_eq!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "source", "~/destination"],
        )
        .status
        .code(),
        Some(0)
    );
    let pushed = support::project_command(root.path(), &metadata_dir, &["push", "source"]);
    assert_eq!(pushed.status.code(), Some(0), "{:?}", pushed);
    assert_eq!(
        fs::read_to_string(root.path().join("destination")).unwrap(),
        "source wins"
    );
}

#[test]
fn add_of_unequal_large_file_preserves_payloads_and_initial_comparison_state() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("large-source.bin");
    let destination = root.path().join("large-destination.bin");
    write_large_file(&source, b'S');
    write_large_file(&destination, b'D');
    let source_before = fs::read(&source).unwrap();
    let destination_before = fs::read(&destination).unwrap();

    let added = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "--output=json",
            "add",
            "large-source.bin",
            "~/large-destination.bin",
        ],
    );
    assert_eq!(added.status.code(), Some(0), "{added:?}");
    assert_eq!(json(&added)["details"]["operation"], "add");
    assert_eq!(
        json(&added)["details"]["mapping"]["declared"]["source"],
        "large-source.bin"
    );
    assert_eq!(
        json(&added)["details"]["mapping"]["declared"]["destination"],
        "~/large-destination.bin"
    );
    assert_eq!(fs::read(&source).unwrap(), source_before);
    assert_eq!(fs::read(&destination).unwrap(), destination_before);
    assert_eq!(
        fs::metadata(&source).unwrap().len(),
        LARGE_FILE_BYTES as u64
    );
    assert_eq!(
        fs::metadata(&destination).unwrap().len(),
        LARGE_FILE_BYTES as u64
    );
    assert!(!metadata_dir.join("state/add-fence.json").exists());

    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(metadata_dir.join("state/state.json")).unwrap()).unwrap();
    assert_eq!(state["payload"]["baselines"].as_array().unwrap().len(), 1);
    let status = json(&support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "status"],
    ));
    assert_eq!(
        status["details"]["records"][0]["classification"],
        "source_only_change"
    );
}

#[test]
fn add_of_a_mixed_tree_uses_destination_state_only_for_managed_unequal_members() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(&destination).unwrap();
    fs::write(source.join("equal.txt"), "same").unwrap();
    fs::write(destination.join("equal.txt"), "same").unwrap();
    support::copy_complete_metadata(
        &source.join("equal.txt"),
        &destination.join("equal.txt"),
        grip::discovery::model::NodeKind::File,
    );
    fs::write(source.join("unequal.txt"), "source wins").unwrap();
    fs::write(destination.join("unequal.txt"), "destination loses").unwrap();
    fs::write(source.join("source-only.txt"), "source-only").unwrap();
    fs::write(source.join("ignored.txt"), "ignored").unwrap();
    fs::write(source.join(".gripignore"), "ignored.txt\n").unwrap();
    fs::write(destination.join("destination-only.txt"), "destination-only").unwrap();

    let before_source = support::snapshot(&source);
    let before_destination = support::snapshot(&destination);
    let added = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "source", "~/destination"],
    );
    assert!(added.status.success(), "{:?}", added);
    assert_eq!(support::snapshot(&source), before_source);
    assert_eq!(support::snapshot(&destination), before_destination);

    let status = json(&support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "status"],
    ));
    let records = status["details"]["records"].as_array().unwrap();
    let classification = |relative: &str| {
        records
            .iter()
            .find(|record| record["relative_path"]["display"] == relative)
            .unwrap()["classification"]
            .as_str()
            .unwrap()
    };
    assert_eq!(classification("equal.txt"), "synchronized");
    assert_eq!(classification("unequal.txt"), "source_only_change");
    assert_eq!(classification("source-only.txt"), "source_addition");
    assert!(
        records
            .iter()
            .all(|record| record["relative_path"]["display"] != "ignored.txt")
    );
    assert!(
        records
            .iter()
            .all(|record| record["relative_path"]["display"] != "destination-only.txt")
    );
}

#[test]
fn retrying_the_same_add_clears_a_verified_fence_and_restores_normal_push_behavior() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "source wins").unwrap();
    fs::write(&destination, "destination loses").unwrap();
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "source", "~/destination"],
        )
        .status
        .success()
    );
    fs::write(root.path().join("other-source"), "independent").unwrap();
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "other-source", "~/other-destination"],
        )
        .status
        .success()
    );

    let descriptor = fs::read(metadata_dir.join("config.toml")).unwrap();
    let state = fs::read(metadata_dir.join("state/state.json")).unwrap();
    let home =
        grip::project::ProjectPaths::project_metadata(metadata_dir.clone(), root.path().into());
    let mapping = grip::mapping::Mapping::new(
        grip::mapping::MappingKind::File,
        fs::canonicalize(&source).unwrap(),
        fs::canonicalize(&destination).unwrap(),
    );
    let fence = grip::state::add_fence::AddPublicationFenceV1::new(
        &mapping,
        descriptor.clone(),
        descriptor,
        Some(state.clone()),
        state,
    );
    grip::state::add_fence::create_verified(&home, &fence).unwrap();

    let fenced_status = json(&support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "status"],
    ));
    assert!(
        fenced_status["details"]["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|record| {
                record["source_path"]["display"]
                    .as_str()
                    .is_some_and(|path| path.ends_with("/source"))
                    && record["classification"] == "unsafe_collision"
            })
    );
    let human_status = support::project_command(root.path(), &metadata_dir, &["status"]);
    assert!(
        String::from_utf8(human_status.stdout)
            .unwrap()
            .contains("Rerun grip add for this mapping.")
    );
    assert_ne!(
        support::project_command(root.path(), &metadata_dir, &["push", "source"])
            .status
            .code(),
        Some(0)
    );
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["push", "--dry-run", "other-source"]
        )
        .status
        .success()
    );

    let retried = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "source", "~/destination"],
    );
    assert!(retried.status.success(), "{:?}", retried);
    assert!(!metadata_dir.join("state/add-fence.json").exists());
    let status = json(&support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "status"],
    ));
    assert!(
        status["details"]["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|record| {
                record["source_path"]["display"]
                    .as_str()
                    .is_some_and(|path| path.ends_with("/source"))
                    && record["classification"] == "source_only_change"
            })
    );
}

#[test]
fn stale_fenced_candidate_restores_the_prior_pair_before_readding() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "source wins").unwrap();
    fs::write(&destination, "destination before fence").unwrap();
    let prior_descriptor = fs::read(metadata_dir.join("config.toml")).unwrap();

    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "source", "~/destination"],
        )
        .status
        .success()
    );
    let candidate_descriptor = fs::read(metadata_dir.join("config.toml")).unwrap();
    let candidate_state = fs::read(metadata_dir.join("state/state.json")).unwrap();

    fs::remove_file(metadata_dir.join("state/state.json")).unwrap();
    let home =
        grip::project::ProjectPaths::project_metadata(metadata_dir.clone(), root.path().into());
    let mapping = grip::mapping::Mapping::new(
        grip::mapping::MappingKind::File,
        fs::canonicalize(&source).unwrap(),
        fs::canonicalize(&destination).unwrap(),
    );
    let fence = grip::state::add_fence::AddPublicationFenceV1::new(
        &mapping,
        prior_descriptor,
        candidate_descriptor,
        None,
        candidate_state.clone(),
    );
    grip::state::add_fence::create_verified(&home, &fence).unwrap();
    fs::write(&destination, "destination changed after fence").unwrap();

    let retried = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "source", "~/destination"],
    );
    assert!(retried.status.success(), "{:?}", retried);
    assert!(!metadata_dir.join("state/add-fence.json").exists());
    assert_ne!(
        fs::read(metadata_dir.join("state/state.json")).unwrap(),
        candidate_state
    );
    let status = json(&support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "status"],
    ));
    assert_eq!(
        status["details"]["records"][0]["classification"],
        "source_only_change"
    );
}

#[test]
fn pre_descriptor_fence_is_visible_and_blocks_only_its_exact_mapping() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(root.path().join("other-source"), "current").unwrap();
    fs::write(root.path().join("other-destination"), "current").unwrap();
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "other-source", "~/other-destination"],
        )
        .status
        .success()
    );
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "source").unwrap();
    fs::write(&destination, "destination").unwrap();
    let home =
        grip::project::ProjectPaths::project_metadata(metadata_dir.clone(), root.path().into());
    let prior = grip::registry::publication::load(&home, false).unwrap();
    let mut declarations = prior.portable_mappings().unwrap();
    declarations.push(
        grip::mapping::PortableMapping::parse_cli(
            grip::mapping::MappingKind::File,
            std::ffi::OsStr::new("source"),
            std::ffi::OsStr::new("~/destination"),
        )
        .unwrap(),
    );
    let candidate_descriptor = grip::registry::encode_descriptor(
        &grip::registry::ProjectDescriptorV2::new(declarations).unwrap(),
    )
    .unwrap();
    let state = fs::read(metadata_dir.join("state/state.json")).unwrap();
    let mapping = grip::mapping::Mapping::new(
        grip::mapping::MappingKind::File,
        fs::canonicalize(&source).unwrap(),
        fs::canonicalize(&destination).unwrap(),
    );
    let fence = grip::state::add_fence::AddPublicationFenceV1::new(
        &mapping,
        prior.bytes,
        candidate_descriptor,
        Some(state.clone()),
        state,
    );
    grip::state::add_fence::create_verified(&home, &fence).unwrap();

    let status = support::project_command(root.path(), &metadata_dir, &["status"]);
    assert!(status.status.success());
    let status = String::from_utf8(status.stdout).unwrap();
    assert!(status.contains("Conflicts:"));
    assert!(status.contains("Rerun grip add for this mapping."));

    let selected_status =
        support::project_command(root.path(), &metadata_dir, &["status", "source"]);
    assert!(selected_status.status.success());
    assert!(
        String::from_utf8(selected_status.stdout)
            .unwrap()
            .contains("Rerun grip add for this mapping.")
    );
    let blocked_push = support::project_command(root.path(), &metadata_dir, &["push", "source"]);
    assert!(!blocked_push.status.success());
    let blocked_error = String::from_utf8(blocked_push.stdout).unwrap();
    assert!(
        blocked_error.contains("incomplete add publication"),
        "{blocked_error}"
    );
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["push", "--dry-run", "other-source"],
        )
        .status
        .success()
    );
}

#[test]
fn status_exit_code_only_escalates_attention() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(root.path().join("source"), "payload").unwrap();
    assert_eq!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "source", "~/destination"],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(
        support::project_command(root.path(), &metadata_dir, &["status"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(
        support::project_command(root.path(), &metadata_dir, &["status", "-e"])
            .status
            .code(),
        Some(1)
    );
}

#[test]
fn remove_prunes_baseline_and_readd_treats_existing_endpoints_as_new() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "same").unwrap();
    fs::write(&destination, "same").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );

    assert_eq!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "source", "~/destination"],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(
        support::project_command(root.path(), &metadata_dir, &["remove", "source"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(fs::read_to_string(&source).unwrap(), "same");
    assert_eq!(fs::read_to_string(&destination).unwrap(), "same");

    fs::write(&destination, "changed while untracked").unwrap();
    assert_eq!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "source", "~/destination"],
        )
        .status
        .code(),
        Some(0)
    );
    let status = support::project_command(root.path(), &metadata_dir, &["-o", "json", "status"]);
    assert_eq!(status.status.code(), Some(0));
    assert_eq!(
        json(&status)["details"]["records"][0]["classification"],
        "source_only_change"
    );
}

#[test]
fn exact_remove_allows_normal_readd_but_unrelated_remove_retains_destination_ownership() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    for name in [
        "old",
        "new",
        "other",
        "third",
        "destination",
        "other-destination",
    ] {
        fs::write(root.path().join(name), name).unwrap();
    }
    assert!(
        support::project_command(root.path(), &metadata_dir, &["add", "old", "~/destination"],)
            .status
            .success()
    );
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "other", "~/other-destination"],
        )
        .status
        .success()
    );

    assert!(
        support::project_command(root.path(), &metadata_dir, &["remove", "old"])
            .status
            .success()
    );
    let readded =
        support::project_command(root.path(), &metadata_dir, &["add", "new", "~/destination"]);
    assert!(readded.status.success(), "{readded:?}");
    let descriptor = fs::read_to_string(metadata_dir.join("config.toml")).unwrap();
    assert!(!descriptor.contains("source = \"old\""));
    assert!(descriptor.contains("source = \"new\""));

    assert!(
        support::project_command(root.path(), &metadata_dir, &["remove", "other"])
            .status
            .success()
    );
    let descriptor_before = fs::read(metadata_dir.join("config.toml")).unwrap();
    let state_before = fs::read(metadata_dir.join("state/state.json")).unwrap();
    let rejected = support::project_command(
        root.path(),
        &metadata_dir,
        &["-o", "json", "add", "third", "~/destination"],
    );
    assert!(!rejected.status.success());
    assert_eq!(json(&rejected)["code"], "invalid_configuration");
    assert_eq!(json(&rejected)["details"]["reason"], "ownership_conflicts");
    assert_eq!(
        fs::read(metadata_dir.join("config.toml")).unwrap(),
        descriptor_before
    );
    assert_eq!(
        fs::read(metadata_dir.join("state/state.json")).unwrap(),
        state_before
    );
}

#[test]
fn retrying_a_visible_remove_completes_the_descriptor_bound_state_transition() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "source").unwrap();
    fs::write(&destination, "destination").unwrap();
    assert!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "source", "~/destination"],
        )
        .status
        .success()
    );

    let home =
        grip::project::ProjectPaths::project_metadata(metadata_dir.clone(), root.path().into());
    let prior = grip::registry::publication::load(&home, true).unwrap();
    let state_snapshot = grip::state::publication::load(&home).unwrap();
    let removed = grip::mapping::Mapping::new(
        grip::mapping::MappingKind::File,
        fs::canonicalize(&source).unwrap(),
        fs::canonicalize(&destination).unwrap(),
    );
    let removed_declaration = grip::mapping::PortableMapping::parse_cli(
        grip::mapping::MappingKind::File,
        std::ffi::OsStr::new("source"),
        std::ffi::OsStr::new("~/destination"),
    )
    .unwrap();
    let descriptor = grip::registry::ProjectDescriptorV2::new(Vec::new()).unwrap();
    let descriptor_bytes = grip::registry::encode_descriptor(&descriptor).unwrap();
    let mut next = state_snapshot.accepted.clone();
    next.complete_baselines
        .retain(|identity, _| identity.mapping.source != removed.source);
    let state_bytes = grip::state::publication::prepare_candidate_for_descriptor(
        &home,
        &state_snapshot,
        &next.complete_baselines,
        &descriptor_bytes,
    )
    .unwrap();
    let fence = grip::state::add_fence::AddPublicationFenceV1::with_context(
        &removed,
        grip::state::add_fence::FenceContext::removal(&removed_declaration),
        prior.bytes.clone(),
        descriptor_bytes.clone(),
        state_snapshot.bytes.clone(),
        state_bytes.clone(),
    );
    grip::state::add_fence::create_verified(&home, &fence).unwrap();
    fs::write(metadata_dir.join("config.toml"), &descriptor_bytes).unwrap();

    let retried = support::project_command(root.path(), &metadata_dir, &["remove", "source"]);
    assert!(retried.status.success(), "{retried:?}");
    assert_eq!(
        String::from_utf8(retried.stdout).unwrap(),
        "Mapping removed:\n file source -> ~/destination\n"
    );
    assert_eq!(
        fs::read(metadata_dir.join("state/state.json")).unwrap(),
        state_bytes
    );
    assert!(!metadata_dir.join("state/add-fence.json").exists());
}

#[test]
fn force_add_without_one_equal_destination_owner_does_not_publish_metadata() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(root.path().join("source"), "source").unwrap();
    fs::write(root.path().join("destination"), "destination").unwrap();
    let descriptor_before = fs::read(metadata_dir.join("config.toml")).unwrap();
    let rejected = support::project_command(
        root.path(),
        &metadata_dir,
        &["-o", "json", "add", "--force", "source", "~/destination"],
    );
    assert!(!rejected.status.success());
    assert_eq!(json(&rejected)["code"], "invalid_configuration");
    assert_eq!(
        json(&rejected)["details"]["reason"],
        "force_add_requires_exact_destination_mapping"
    );
    assert_eq!(
        fs::read(metadata_dir.join("config.toml")).unwrap(),
        descriptor_before
    );
    assert!(!metadata_dir.join("state/add-fence.json").exists());
    assert!(!metadata_dir.join("state/state.json").exists());
}

#[test]
fn rejected_human_add_does_not_render_successful_mapping_output() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    fs::write(root.path().join("source"), "source").unwrap();
    fs::write(root.path().join("destination"), "destination").unwrap();

    let rejected = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "--force", "source", "~/destination"],
    );
    assert!(!rejected.status.success());
    let human = String::from_utf8(rejected.stdout).unwrap();
    assert!(human.starts_with("Error:"), "{human}");
    assert!(!human.contains("Mapped:"), "{human}");
}

#[test]
fn add_excludes_opaque_ambient_label_xattrs() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "payload").unwrap();
    fs::write(&destination, "payload").unwrap();
    support::set_fixture_xattr(
        &destination,
        "com.apple.metadata:kMDLabel_opaque-label",
        b"ambient-value-must-not-be-managed",
    );

    let added = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "source", "destination"],
    );
    assert!(
        added.status.success(),
        "{}",
        String::from_utf8_lossy(&added.stdout)
    );
    assert_eq!(fs::read_to_string(destination).unwrap(), "payload");
}

#[test]
fn rejected_human_add_names_unknown_managed_xattr_without_its_value() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "payload").unwrap();
    fs::write(&destination, "payload").unwrap();
    support::set_fixture_xattr(&destination, "com.example.unknown", b"secret-xattr-value");

    let rejected = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "source", "destination"],
    );
    assert!(!rejected.status.success());
    let human = String::from_utf8(rejected.stdout).unwrap();
    assert!(human.contains("Blocked entry:"), "{human}");
    assert!(human.contains("/destination\n"), "{human}");
    assert!(
        human
            .contains("destination has an unknown managed extended attribute: com.example.unknown"),
        "{human}"
    );
    assert!(!human.contains("secret-xattr-value"), "{human}");
}

#[test]
fn force_add_rejects_a_malformed_registry_with_multiple_equal_destination_files() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    for name in ["first", "second", "third", "destination"] {
        fs::write(root.path().join(name), name).unwrap();
    }
    let descriptor = "schema_version = 2\n\n[[mappings]]\nkind = \"file\"\nsource = \"first\"\ndestination = \"~/destination\"\n\n[[mappings]]\nkind = \"file\"\nsource = \"second\"\ndestination = \"~/destination\"\n";
    fs::write(metadata_dir.join("config.toml"), descriptor).unwrap();
    let before = fs::read(metadata_dir.join("config.toml")).unwrap();

    let rejected = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "--force", "third", "~/destination"],
    );
    assert!(!rejected.status.success());
    assert_eq!(fs::read(metadata_dir.join("config.toml")).unwrap(), before);
    assert!(!metadata_dir.join("state/add-fence.json").exists());
}

#[test]
fn force_push_propagates_a_selected_source_absence() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "same").unwrap();
    fs::write(&destination, "same").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    assert_eq!(
        support::project_command(
            root.path(),
            &metadata_dir,
            &["add", "source", "~/destination"],
        )
        .status
        .code(),
        Some(0)
    );
    fs::remove_file(&source).unwrap();
    let forced = support::project_command(root.path(), &metadata_dir, &["push", "-f", "source"]);
    assert_eq!(forced.status.code(), Some(0), "{:?}", forced);
    assert!(!destination.exists());
    assert!(!metadata_dir.join("state/recovery").exists());
    let operations = metadata_dir.join("state/operations");
    for operation in fs::read_dir(operations).unwrap() {
        assert!(!operation.unwrap().path().join("recovery").exists());
    }
}
