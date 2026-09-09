mod support;

use grip::discovery::model::{
    DiscoveryInventory, DiscoveryRecord, DiscoveryScope, NodeKind, RecordCategory, SafePath,
};
use grip::mapping::MappingKind;
use grip::result::{CommandOutcome, OutputMode};
use std::path::{Path, PathBuf};

fn accepted_fixture() -> support::MetadataFixture {
    let fixture = support::MetadataFixture::file(b"same");
    let source =
        grip::observation::fingerprint::inspect_complete(&fixture.source, NodeKind::File).unwrap();
    grip::metadata::macos::apply_metadata_paths(
        &fixture.source,
        &fixture.destination,
        NodeKind::File,
        &source.state.metadata,
    )
    .unwrap();
    let accepted = support::project_command(
        fixture.root.path(),
        &fixture.metadata_dir,
        &["baseline", "accept"],
    );
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stdout)
    );
    fixture
}

#[test]
fn status_diff_dry_run_and_push_report_and_apply_metadata_only_changes() {
    let fixture = accepted_fixture();
    support::set_fixture_mode(&fixture.source, 0o600);
    let status = support::project_command(
        fixture.root.path(),
        &fixture.metadata_dir,
        &["--output", "json", "status"],
    );
    let status_json = support::json(&status);
    assert_eq!(
        status_json["details"]["records"][0]["classification"],
        "source_only_change"
    );
    assert_eq!(
        status_json["details"]["records"][0]["changed_dimensions"]["source_to_baseline"],
        serde_json::json!(["permission_mode"])
    );

    let before = support::snapshot(fixture.root.path());
    let preview = support::project_command(
        fixture.root.path(),
        &fixture.metadata_dir,
        &["--output", "json", "push", "--dry-run"],
    );
    assert!(preview.status.success());
    assert_eq!(
        support::json(&preview)["details"]["actions"][0]["kind"],
        "apply_metadata"
    );
    assert_eq!(support::snapshot(fixture.root.path()), before);

    let push = support::project_command(
        fixture.root.path(),
        &fixture.metadata_dir,
        &["--output", "json", "push"],
    );
    assert!(
        push.status.success(),
        "{}",
        String::from_utf8_lossy(&push.stdout)
    );
    let source =
        grip::observation::fingerprint::inspect_complete(&fixture.source, NodeKind::File).unwrap();
    let destination =
        grip::observation::fingerprint::inspect_complete(&fixture.destination, NodeKind::File)
            .unwrap();
    assert_eq!(source.state, destination.state);
    assert_eq!(
        support::json(&push)["details"]["baseline"]["outcome"],
        "published"
    );
}

#[test]
fn existing_read_and_mutation_commands_share_the_complete_metadata_contract() {
    let fixture = accepted_fixture();

    support::set_fixture_mode(&fixture.destination, 0o600);
    let before_pull = support::snapshot(fixture.root.path());
    let pull_preview = support::project_command(
        fixture.root.path(),
        &fixture.metadata_dir,
        &["--output=json", "pull", "--dry-run"],
    );
    assert!(pull_preview.status.success());
    assert_eq!(
        support::json(&pull_preview)["details"]["actions"][0]["kind"],
        "apply_metadata"
    );
    assert_eq!(support::snapshot(fixture.root.path()), before_pull);
    assert!(
        support::project_command(fixture.root.path(), &fixture.metadata_dir, &["pull"])
            .status
            .success()
    );

    support::set_fixture_mode(&fixture.source, 0o640);
    let sync_preview = support::project_command(
        fixture.root.path(),
        &fixture.metadata_dir,
        &["--output=json", "sync", "--dry-run"],
    );
    assert!(sync_preview.status.success());
    assert_eq!(
        support::json(&sync_preview)["details"]["actions"][0]["kind"],
        "apply_metadata"
    );
    assert!(
        support::project_command(fixture.root.path(), &fixture.metadata_dir, &["sync"])
            .status
            .success()
    );

    std::fs::write(&fixture.source, b"source conflict").unwrap();
    std::fs::write(&fixture.destination, b"destination conflict").unwrap();
    for command in ["status", "check", "diff"] {
        let output = support::project_command(
            fixture.root.path(),
            &fixture.metadata_dir,
            &["--output=json", command],
        );
        assert_eq!(
            support::json(&output)["details"]["records"][0]["classification"],
            "divergent_conflict"
        );
    }
    let resolve_preview = support::project_command(
        fixture.root.path(),
        &fixture.metadata_dir,
        &[
            "--output=json",
            "resolve",
            "source",
            "--destination",
            "--dry-run",
        ],
    );
    assert!(resolve_preview.status.success());
    assert_ne!(
        std::fs::read(&fixture.source).unwrap(),
        std::fs::read(&fixture.destination).unwrap()
    );
    assert!(
        support::project_command(
            fixture.root.path(),
            &fixture.metadata_dir,
            &["resolve", "source", "--destination"],
        )
        .status
        .success()
    );
    assert_eq!(
        grip::observation::fingerprint::inspect_complete(&fixture.source, NodeKind::File)
            .unwrap()
            .state,
        grip::observation::fingerprint::inspect_complete(&fixture.destination, NodeKind::File)
            .unwrap()
            .state
    );
}

#[test]
fn excluded_and_unknown_xattrs_have_safe_human_json_parity_and_unknown_blocks_preview() {
    let fixture = accepted_fixture();
    support::set_fixture_xattr(
        &fixture.source,
        "com.apple.quarantine",
        b"do-not-print-excluded",
    );
    support::set_fixture_xattr(
        &fixture.source,
        "com.example.unknown",
        b"do-not-print-unknown",
    );
    let before = support::snapshot(fixture.root.path());
    let json = support::project_command(
        fixture.root.path(),
        &fixture.metadata_dir,
        &["--output", "json", "status"],
    );
    let json_text = String::from_utf8_lossy(&json.stdout);
    assert!(json_text.contains("excluded_xattr"));
    assert!(json_text.contains("unknown_xattr"));
    assert!(json_text.contains("corrective_choice"));
    assert!(json_text.contains("remove the unknown attribute explicitly"));
    assert!(!json_text.contains("do-not-print"));
    let human = support::project_command(fixture.root.path(), &fixture.metadata_dir, &["status"]);
    let human_text = String::from_utf8_lossy(&human.stdout);
    assert!(human_text.contains("excluded_xattr"));
    assert!(human_text.contains("unknown_xattr"));
    assert!(human_text.contains("corrective_choice="));
    assert!(human_text.contains("remove the unknown attribute explicitly"));
    assert!(!human_text.contains("do-not-print"));
    let preview = support::project_command(
        fixture.root.path(),
        &fixture.metadata_dir,
        &["--output", "json", "push", "--dry-run"],
    );
    assert_eq!(support::json(&preview)["details"]["counts"]["blockers"], 1);
    assert_eq!(support::snapshot(fixture.root.path()), before);
}

#[test]
fn metadata_capability_results_are_deterministic_and_read_only_for_100_runs() {
    let fixture = accepted_fixture();
    support::set_fixture_mode(&fixture.source, 0o600);
    support::set_fixture_xattr(&fixture.source, "com.apple.quarantine", b"excluded-secret");
    support::set_fixture_xattr(&fixture.source, "com.example.unknown", b"unknown-secret");
    let before = support::snapshot(fixture.root.path());
    for arguments in [
        &["--output=json", "status"][..],
        &["--output=json", "diff"][..],
        &["--output=json", "push", "--dry-run"][..],
    ] {
        let expected =
            support::project_command(fixture.root.path(), &fixture.metadata_dir, arguments);
        for _ in 0..100 {
            let observed =
                support::project_command(fixture.root.path(), &fixture.metadata_dir, arguments);
            assert_eq!(observed.status.code(), expected.status.code());
            assert_eq!(observed.stdout, expected.stdout);
            assert_eq!(observed.stderr, expected.stderr);
        }
        assert!(!String::from_utf8_lossy(&expected.stdout).contains("secret"));
    }
    assert_eq!(support::snapshot(fixture.root.path()), before);
}

#[test]
fn unsupported_filesystem_findings_have_safe_human_json_parity() {
    let mapping_source = PathBuf::from("/safe/source");
    let records = [
        (b"hard-link".as_slice(), NodeKind::HardLink, "hard_link"),
        (b"sparse".as_slice(), NodeKind::SparseFile, "sparse_file"),
        (
            b"nested-mount".as_slice(),
            NodeKind::NestedMount,
            "nested_mount",
        ),
    ]
    .into_iter()
    .map(|(relative, node_kind, reason)| DiscoveryRecord {
        category: RecordCategory::UnsupportedSource,
        mapping_kind: MappingKind::Tree,
        mapping_source: mapping_source.clone(),
        relative_path: Some(SafePath::from_bytes(relative)),
        source_path: Some(SafePath::from_path(
            &mapping_source.join(Path::new(std::str::from_utf8(relative).unwrap())),
        )),
        destination_path: SafePath::from_path(
            &PathBuf::from("/safe/destination")
                .join(Path::new(std::str::from_utf8(relative).unwrap())),
        ),
        node_kind,
        reason: Some(reason),
        blocking: true,
    })
    .collect();
    let inventory = DiscoveryInventory::new(DiscoveryScope::All, records);
    assert_eq!(inventory.blocking_count, 3);

    let mut human = Vec::new();
    grip::result::render(
        CommandOutcome::discovery(&inventory),
        OutputMode::Human,
        &mut human,
    )
    .unwrap();
    let human = String::from_utf8(human).unwrap();
    let mut json = Vec::new();
    grip::result::render(
        CommandOutcome::discovery(&inventory),
        OutputMode::Json,
        &mut json,
    )
    .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&json).unwrap();

    for (relative, node_kind, reason) in [
        ("hard-link", "hard_link", "hard_link"),
        ("sparse", "sparse_file", "sparse_file"),
        ("nested-mount", "nested_mount", "nested_mount"),
    ] {
        assert!(human.contains(relative));
        assert!(human.contains(node_kind));
        assert!(human.contains(reason));
        assert!(json["details"]["records"].as_array().unwrap().iter().any(
            |record| record["relative_path"]["display"] == relative
                && record["node_kind"] == node_kind
                && record["reason"] == reason
                && record["blocking"] == true
        ));
    }
    assert!(!human.contains("payload-secret"));
    assert!(
        !String::from_utf8(json.to_string().into_bytes())
            .unwrap()
            .contains("payload-secret")
    );
}

#[test]
fn unsupported_link_output_never_reads_or_discloses_its_referent() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    let sentinel = root.path().join("payload-secret");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(&sentinel, b"payload-secret-must-not-be-read").unwrap();
    std::os::unix::fs::symlink(&sentinel, source.join("unsafe-link")).unwrap();
    support::write_descriptor(&metadata_dir, &[("tree", &source, &destination)]);

    let json = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "mapping", "inspect"],
    );
    let human = support::project_command(root.path(), &metadata_dir, &["mapping", "inspect"]);
    let json_text = String::from_utf8(json.stdout).unwrap();
    let human_text = String::from_utf8(human.stdout).unwrap();
    for output in [&json_text, &human_text] {
        assert!(output.contains("unsafe-link"));
        assert!(output.contains("symlink"));
        assert!(!output.contains("payload-secret-must-not-be-read"));
    }
    assert_eq!(
        std::fs::read(&sentinel).unwrap(),
        b"payload-secret-must-not-be-read"
    );
    assert!(!destination.exists());
}
