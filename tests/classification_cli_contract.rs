mod support;

use std::fs;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use support::project::ProjectFixture;

const LARGE_FILE_BYTES: usize = 19 * 1024 * 1024;

fn write_large_file(path: &std::path::Path, byte: u8) {
    let chunk = vec![byte; 64 * 1024];
    let mut file = fs::File::create(path).unwrap();
    for _ in 0..(LARGE_FILE_BYTES / chunk.len()) {
        std::io::Write::write_all(&mut file, &chunk).unwrap();
    }
}

fn fixture() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
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
    let add = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "source", "~/destination"],
    );
    assert!(add.status.success());
    (root, metadata_dir, source)
}

#[test]
fn status_is_success_by_default_and_exit_code_reports_attention() {
    let (root, metadata_dir, source) = fixture();
    fs::write(&source, "changed").unwrap();

    let status = support::project_command(root.path(), &metadata_dir, &["-o", "json", "status"]);
    assert_eq!(status.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(
        value["details"]["records"][0]["classification"],
        "source_only_change"
    );

    let exit = support::project_command(root.path(), &metadata_dir, &["status", "-e"]);
    assert_eq!(exit.status.code(), Some(1));
}

#[test]
fn status_json_classifies_an_established_unequal_large_file_mapping() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("large-source.bin");
    let destination = root.path().join("large-destination.bin");
    write_large_file(&source, b'S');
    write_large_file(&destination, b'D');
    let added = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "large-source.bin", "~/large-destination.bin"],
    );
    assert!(added.status.success(), "{added:?}");

    let status = support::project_command(root.path(), &metadata_dir, &["-o", "json", "status"]);
    assert_eq!(status.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(
        value["details"]["records"][0]["classification"],
        "source_only_change"
    );
    assert_eq!(value["details"]["records"].as_array().unwrap().len(), 1);
    assert_eq!(fs::read(&source).unwrap()[0], b'S');
    assert_eq!(fs::read(&destination).unwrap()[0], b'D');
}

#[test]
fn status_rejects_a_large_file_changed_during_observation() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("large-source.bin");
    let destination = root.path().join("large-destination.bin");
    write_large_file(&source, b'S');
    write_large_file(&destination, b'D');
    let added = support::project_command(
        root.path(),
        &metadata_dir,
        &["add", "large-source.bin", "~/large-destination.bin"],
    );
    assert!(added.status.success(), "{added:?}");

    let keep_writing = Arc::new(AtomicBool::new(true));
    let writer_flag = Arc::clone(&keep_writing);
    let writer_path = source.clone();
    let (ready, started) = mpsc::channel();
    let writer = std::thread::spawn(move || {
        let mut byte = b'A';
        while writer_flag.load(Ordering::Relaxed) {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .open(&writer_path)
                .unwrap();
            std::io::Write::write_all(&mut file, &[byte]).unwrap();
            file.sync_data().unwrap();
            byte = if byte == b'A' { b'B' } else { b'A' };
            let _ = ready.send(());
        }
    });
    started.recv().unwrap();
    let status = support::project_command(root.path(), &metadata_dir, &["-o", "json", "status"]);
    keep_writing.store(false, Ordering::Relaxed);
    writer.join().unwrap();

    assert!(!status.status.success(), "{status:?}");
    let output = String::from_utf8_lossy(&status.stdout);
    assert!(
        output.contains("entry changed while complete metadata was observed"),
        "unexpected status failure: {output}"
    );
}

#[test]
fn human_status_renders_a_compact_push_section() {
    let (root, metadata_dir, source) = fixture();
    fs::write(&source, "changed").unwrap();

    let status = support::project_command(root.path(), &metadata_dir, &["status"]);
    assert_eq!(status.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(status.stdout).unwrap(),
        format!(
            "Status: 1 entry checked; 0 current; 1 to push.\n\nChanges to push:\n  source -> {}\n",
            fs::canonicalize(root.path().join("destination"))
                .unwrap()
                .display(),
        )
    );
}

#[test]
fn human_status_renders_initial_matches_as_needing_a_baseline() {
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
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);

    let status = support::project_command(root.path(), &metadata_dir, &["status"]);
    assert_eq!(
        String::from_utf8(status.stdout).unwrap(),
        format!(
            "Status: 1 entry checked; 0 current; 1 needs baseline.\n\nNeeds baseline:\n  source >-< {}\n",
            fs::canonicalize(&destination).unwrap().display(),
        )
    );
}

#[test]
fn human_status_shows_force_choices_for_an_ordinary_divergent_conflict() {
    let (root, metadata_dir, source) = fixture();
    let destination = root.path().join("destination");
    fs::write(&source, "source change").unwrap();
    fs::write(&destination, "destination change").unwrap();

    let status = support::project_command(root.path(), &metadata_dir, &["status"]);
    assert_eq!(status.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(status.stdout).unwrap(),
        format!(
            "Status: 1 entry checked; 0 current; 1 conflict.\n\nConflicts:\n  source <-> {}\n    Keep source: grip push --force source\n    Keep destination: grip pull --force --destination {}\n",
            fs::canonicalize(&destination).unwrap().display(),
            fs::canonicalize(&destination).unwrap().display(),
        )
    );

    let force_push = support::project_command(
        root.path(),
        &metadata_dir,
        &["push", "--force", "--dry-run", "source"],
    );
    assert!(force_push.status.success());
    let force_pull = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "pull",
            "--force",
            "--dry-run",
            "--destination",
            destination.to_str().unwrap(),
        ],
    );
    assert!(force_pull.status.success());
}

#[test]
fn human_status_uses_diff_for_an_aggregate_tree_conflict() {
    let (root, metadata_dir, _source, destination) = support::aggregate_tree_collision_fixture();

    let status = support::project_command(root.path(), &metadata_dir, &["status"]);
    assert_eq!(status.status.code(), Some(0));
    let text = String::from_utf8(status.stdout).unwrap();
    assert!(
        text.contains(&format!(
            "  source/nested/file <-> {}/nested/file\n    Run: grip diff source/nested/file\n",
            destination.display(),
        )),
        "unexpected status output: {text}"
    );
    assert!(!text.contains("grip push --force source/nested/file"));
    assert!(!text.contains("grip pull --force --destination"));

    let force_push = support::project_command(
        root.path(),
        &metadata_dir,
        &["push", "--force", "--dry-run", "source/nested/file"],
    );
    assert_eq!(force_push.status.code(), Some(10));
    assert!(
        String::from_utf8(force_push.stdout)
            .unwrap()
            .contains("requires one exact established managed entry")
    );
}

#[test]
fn human_status_shows_force_choices_for_an_initial_collision() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "source version").unwrap();
    fs::write(&destination, "destination version").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);

    let status = support::project_command(root.path(), &metadata_dir, &["status"]);
    assert_eq!(status.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(status.stdout).unwrap(),
        format!(
            "Status: 1 entry checked; 0 current; 1 conflict.\n\nConflicts:\n  source <-> {}\n    Keep source: grip push --force source\n    Keep destination: grip pull --force --destination {}\n",
            fs::canonicalize(&destination).unwrap().display(),
            fs::canonicalize(&destination).unwrap().display(),
        )
    );

    let force_push = support::project_command(
        root.path(),
        &metadata_dir,
        &["push", "--force", "--dry-run", "source"],
    );
    assert!(force_push.status.success());
    let force_pull = support::project_command(
        root.path(),
        &metadata_dir,
        &[
            "pull",
            "--force",
            "--dry-run",
            "--destination",
            destination.to_str().unwrap(),
        ],
    );
    assert!(force_pull.status.success());
}

#[test]
fn human_status_orders_push_before_pull() {
    let (root, home, _registry, _state, _selection, _plan) =
        support::mixed_sync_execution_fixture();
    let status = support::project_command(root.path(), home.path(), &["status"]);
    assert!(status.status.success());
    let text = String::from_utf8(status.stdout).unwrap();
    assert!(
        text.find("Changes to push:").unwrap() < text.find("Changes to pull:").unwrap(),
        "status sections were not ordered for action: {text}"
    );
}

#[test]
fn human_status_renders_source_paths_relative_to_the_invocation_directory() {
    let fixture = ProjectFixture::initialized();
    let app = fixture.project_root.join("app");
    let shared = fixture.project_root.join("shared");
    fs::create_dir_all(&app).unwrap();
    fs::create_dir_all(&shared).unwrap();
    let main = app.join("main.py");
    let config = shared.join("config.yml");
    let main_destination = fixture.home_destination("workspace/app/main.py");
    let config_destination = fixture.home_destination("workspace/shared/config.yml");
    fs::create_dir_all(main_destination.parent().unwrap()).unwrap();
    fs::create_dir_all(config_destination.parent().unwrap()).unwrap();
    fs::write(&main, "same").unwrap();
    fs::write(&config, "same").unwrap();
    fs::write(&main_destination, "same").unwrap();
    fs::write(&config_destination, "same").unwrap();
    support::copy_complete_metadata(
        &main,
        &main_destination,
        grip::discovery::model::NodeKind::File,
    );
    support::copy_complete_metadata(
        &config,
        &config_destination,
        grip::discovery::model::NodeKind::File,
    );
    assert!(
        fixture
            .command(&["add", "app/main.py", "~/workspace/app/main.py"])
            .status
            .success()
    );
    assert!(
        fixture
            .command(&["add", "shared/config.yml", "~/workspace/shared/config.yml"])
            .status
            .success()
    );
    fs::write(&main, "changed").unwrap();
    fs::write(&config, "changed").unwrap();

    let status = fixture.command_from(&app, &["status"]);
    assert!(status.status.success());
    let stdout = String::from_utf8(status.stdout).unwrap();
    assert!(stdout.contains("  main.py -> "));
    assert!(stdout.contains("  ../shared/config.yml -> "));

    let json = fixture.command_from(&app, &["--output=json", "status"]);
    assert!(json.status.success());
    let value: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(
        value["details"]["records"][0]["source_path"]["display"],
        fs::canonicalize(&main).unwrap().display().to_string()
    );
}

#[test]
fn human_status_distinguishes_clean_and_empty_scopes() {
    let (root, metadata_dir, _) = fixture();
    let clean = support::project_command(root.path(), &metadata_dir, &["status"]);
    assert_eq!(
        String::from_utf8(clean.stdout).unwrap(),
        "Status: 1 entry checked; 1 current; no action needed.\n"
    );

    let empty_root = tempfile::tempdir().unwrap();
    let empty_metadata_dir = support::initialize_project_metadata(empty_root.path());
    let empty_source = empty_root.path().join("source");
    let empty_destination = empty_root.path().join("destination");
    fs::create_dir(&empty_source).unwrap();
    fs::create_dir(&empty_destination).unwrap();
    support::write_descriptor(
        &empty_metadata_dir,
        &[("tree", &empty_source, &empty_destination)],
    );
    let empty = support::project_command(empty_root.path(), &empty_metadata_dir, &["status"]);
    assert_eq!(
        String::from_utf8(empty.stdout).unwrap(),
        "Status: no managed entries found.\n"
    );
}

#[test]
fn status_json_retains_classification_capabilities_and_exit_contracts() {
    let (root, metadata_dir, source) = fixture();
    fs::write(&source, "changed").unwrap();

    let status = support::project_command(root.path(), &metadata_dir, &["-o", "json", "status"]);
    assert_eq!(status.status.code(), Some(0));
    let value = support::json(&status);
    assert_eq!(
        value["details"]["records"][0]["classification"],
        "source_only_change"
    );
    assert_eq!(value["details"]["attention_count"], 1);
    assert_eq!(value["details"]["blocking_count"], 0);
    assert_eq!(
        value["details"]["endpoint_capabilities"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let exit =
        support::project_command(root.path(), &metadata_dir, &["-o", "json", "status", "-e"]);
    assert_eq!(exit.status.code(), Some(1));
    assert_eq!(support::json(&exit)["details"], value["details"]);
}

#[test]
fn diff_is_read_only_and_supports_destination_selection() {
    let (root, metadata_dir, source) = fixture();
    fs::write(&source, "changed").unwrap();
    let state = fs::read(metadata_dir.join("state/state.json")).unwrap();

    let diff = support::project_command(
        root.path(),
        &metadata_dir,
        &["-o", "json", "diff", "-d", "~/destination"],
    );
    assert_eq!(diff.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_slice(&diff.stdout).unwrap();
    assert_eq!(value["details"]["scope"]["path_space"], "destination");
    assert_eq!(
        fs::read(metadata_dir.join("state/state.json")).unwrap(),
        state
    );

    let human = support::project_command(root.path(), &metadata_dir, &["diff"]);
    let text = String::from_utf8(human.stdout).unwrap();
    assert!(text.starts_with("Diff complete: 1 entries; 1 attention; 0 blocking\n"));
    assert!(text.contains("source_only_change"));
    assert!(text.contains("source_to_destination"));
}
