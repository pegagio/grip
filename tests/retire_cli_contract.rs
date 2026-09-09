use clap::Parser;
use grip::cli::{Cli, Command};
use std::fs;

mod support;

#[test]
fn retire_requires_path_or_all_and_rejects_ambiguous_scope() {
    assert!(Cli::try_parse_from(["grip", "retire"]).is_err());
    assert!(Cli::try_parse_from(["grip", "retire", "--all", "/tmp/item"]).is_err());
    assert!(Cli::try_parse_from(["grip", "retire", "--all", "--destination"]).is_err());
}

#[test]
fn retirement_preview_is_deterministic_and_non_mutating_for_100_runs_per_output() {
    let (root, metadata_dir, source, destination) = support::accepted_file_fixture();
    fs::remove_file(source).unwrap();
    fs::remove_file(destination).unwrap();
    let before = support::snapshot(root.path());
    for output in ["human", "json"] {
        let expected = support::project_command(
            root.path(),
            &metadata_dir,
            &["--output", output, "retire", "-n", "--all"],
        );
        for _ in 0..100 {
            let actual = support::project_command(
                root.path(),
                &metadata_dir,
                &["--output", output, "retire", "-n", "--all"],
            );
            assert_eq!(actual.status.code(), expected.status.code());
            assert_eq!(actual.stdout, expected.stdout);
            assert_eq!(actual.stderr, expected.stderr);
        }
    }
    support::assert_snapshot_unchanged(&before, root.path());
}

#[test]
fn retirement_noop_force_and_output_parity_are_explicit() {
    let (root, metadata_dir, _, _) = support::accepted_file_fixture();
    let json = support::project_command(
        root.path(),
        &metadata_dir,
        &["--output=json", "retire", "-n", "--all"],
    );
    let human = support::project_command(
        root.path(),
        &metadata_dir,
        &["retire", "-n", "--all", "--force"],
    );
    assert_eq!(support::json(&json)["details"]["result"], "blocked");
    assert!(String::from_utf8_lossy(&human.stdout).contains("Force authorized yes"));
}

#[test]
fn retire_parses_all_path_force_destination_and_preview() {
    let all = Cli::try_parse_from(["grip", "retire", "--all", "--force", "-n"]).unwrap();
    assert!(matches!(all.command, Command::Retire(ref args)
        if args.all && args.force && args.dry_run && args.path.is_none()));
    let path = Cli::try_parse_from(["grip", "retire", "--destination", "--", "-item"]).unwrap();
    assert!(matches!(path.command, Command::Retire(ref args)
        if !args.all && args.destination && args.path.as_deref().and_then(std::ffi::OsStr::to_str) == Some("-item")));
}
