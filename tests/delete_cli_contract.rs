use clap::Parser;
use grip::cli::{Cli, Command};
use std::fs;

mod support;

#[test]
fn delete_requires_exactly_one_authority_and_one_selector() {
    for invalid in [
        vec!["grip", "delete", "/tmp/item"],
        vec!["grip", "delete", "--source"],
        vec!["grip", "delete", "--source", "--destination", "/tmp/item"],
        vec!["grip", "delete", "--source", "/tmp/a", "/tmp/b"],
    ] {
        assert!(Cli::try_parse_from(invalid).is_err());
    }
}

#[test]
fn delete_parses_authority_dry_run_and_option_terminator() {
    let source = Cli::try_parse_from(["grip", "delete", "-n", "--source", "--", "-item"]).unwrap();
    assert!(matches!(source.command, Command::Delete(ref args)
        if args.dry_run && args.source && !args.destination && args.path.to_str() == Some("-item")));
    let destination =
        Cli::try_parse_from(["grip", "delete", "--destination", "/tmp/item"]).unwrap();
    assert!(matches!(destination.command, Command::Delete(ref args)
        if !args.dry_run && !args.source && args.destination));
}

#[test]
fn ordinary_commands_retain_non_deleting_grammars() {
    assert!(matches!(
        Cli::try_parse_from(["grip", "push"]).unwrap().command,
        Command::Push(_)
    ));
    assert!(matches!(
        Cli::try_parse_from(["grip", "status"]).unwrap().command,
        Command::Status(_)
    ));
}

#[test]
fn deletion_preview_is_deterministic_and_non_mutating_for_100_runs_per_output() {
    let (root, metadata_dir, source, _) = support::accepted_file_fixture();
    fs::remove_file(&source).unwrap();
    let before = support::snapshot(root.path());
    for output in ["human", "json"] {
        let expected = support::project_command(
            root.path(),
            &metadata_dir,
            &["--output", output, "delete", "-n", "--source", "source"],
        );
        for _ in 0..100 {
            let actual = support::project_command(
                root.path(),
                &metadata_dir,
                &["--output", output, "delete", "-n", "--source", "source"],
            );
            assert_eq!(actual.status.code(), expected.status.code());
            assert_eq!(actual.stdout, expected.stdout);
            assert_eq!(actual.stderr, expected.stderr);
        }
    }
    support::assert_snapshot_unchanged(&before, root.path());
}

#[test]
fn deletion_human_json_parity_and_blocked_exit_category_are_stable() {
    let (root, metadata_dir, source, destination) = support::accepted_file_fixture();
    fs::remove_file(&source).unwrap();
    let json = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "delete", "-n", "--source", "source"],
    );
    let human = support::project_command(
        root.path(),
        &metadata_dir,
        &["delete", "-n", "--source", "source"],
    );
    assert!(String::from_utf8_lossy(&human.stdout).contains(&destination.display().to_string()));
    assert_eq!(
        support::json(&json)["details"]["actions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    fs::write(&destination, "changed").unwrap();
    let blocked = support::project_command(
        root.path(),
        &metadata_dir,
        &["delete", "--source", "source"],
    );
    assert_eq!(blocked.status.code(), Some(10));
}
