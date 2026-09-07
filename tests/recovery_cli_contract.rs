use clap::Parser;
use grip::cli::{Cli, Command, RecoveryCommand};

mod support;

fn digest() -> String {
    "a".repeat(64)
}

#[test]
fn recovery_mutation_previews_are_deterministic_and_non_mutating_for_100_runs() {
    let (root, grip_home, _, _, reference) = support::payload_recovery_fixture();
    let before = support::snapshot(root.path());
    for output in ["human", "json"] {
        for operation in ["restore", "remove"] {
            let mut args = vec!["--output", output, "recovery", operation, "-n"];
            if operation == "remove" {
                args.push("--confirm");
            }
            args.push(&reference);
            let expected = support::command_with_grip_home(root.path(), &grip_home, &args);
            for _ in 0..100 {
                let actual = support::command_with_grip_home(root.path(), &grip_home, &args);
                assert_eq!(actual.status.code(), expected.status.code());
                assert_eq!(actual.stdout, expected.stdout);
                assert_eq!(actual.stderr, expected.stderr);
            }
        }
    }
    support::assert_snapshot_unchanged(&before, root.path());
}

#[test]
fn recovery_list_show_human_json_parity_and_content_nondisclosure() {
    let (root, grip_home, _, _, reference) = support::payload_recovery_fixture();
    for command in [
        vec!["recovery", "list"],
        vec!["recovery", "show", &reference],
    ] {
        let human = support::command_with_grip_home(root.path(), &grip_home, &command);
        let mut json_args = vec!["--output=json"];
        json_args.extend(command);
        let json = support::command_with_grip_home(root.path(), &grip_home, &json_args);
        assert!(human.status.success() && json.status.success());
        assert!(String::from_utf8_lossy(&human.stdout).contains(&reference));
        assert!(!String::from_utf8_lossy(&json.stdout).contains("\"content\""));
    }
}

#[test]
fn cleanup_rejects_duplicate_and_operation_references_before_mutation() {
    let (root, grip_home, _, _, reference) = support::payload_recovery_fixture();
    let before = support::snapshot(root.path());
    for references in [
        vec![reference.as_str(), reference.as_str()],
        vec!["operation:example"],
    ] {
        let mut args = vec!["recovery", "remove", "--confirm"];
        args.extend(references);
        let result = support::command_with_grip_home(root.path(), &grip_home, &args);
        assert!(!result.status.success());
    }
    support::assert_snapshot_unchanged(&before, root.path());
}

#[test]
fn recovery_list_show_and_restore_parse_exact_references() {
    assert!(
        matches!(Cli::try_parse_from(["grip", "recovery", "list"]).unwrap().command,
        Command::Recovery(ref args) if matches!(args.command, RecoveryCommand::List))
    );
    let registry = format!("registry:sha256:{}", digest());
    let show = Cli::try_parse_from(["grip", "recovery", "show", &registry]).unwrap();
    assert!(matches!(show.command, Command::Recovery(ref args)
        if matches!(args.command, RecoveryCommand::Show(_))));
    let restore = Cli::try_parse_from([
        "grip",
        "recovery",
        "restore",
        "-n",
        "--",
        "payload:delete-1:0",
    ])
    .unwrap();
    assert!(matches!(restore.command, Command::Recovery(ref args)
        if matches!(args.command, RecoveryCommand::Restore(ref value) if value.dry_run)));
    assert!(Cli::try_parse_from(["grip", "recovery", "restore", "operation:delete-1"]).is_ok());
}

#[test]
fn recovery_remove_requires_confirmation_references_and_valid_grammar() {
    assert!(Cli::try_parse_from(["grip", "recovery", "remove", "payload:op:0"]).is_err());
    assert!(Cli::try_parse_from(["grip", "recovery", "remove", "--confirm"]).is_err());
    assert!(Cli::try_parse_from(["grip", "recovery", "show", "payload:../bad:0"]).is_err());
    let parsed = Cli::try_parse_from([
        "grip",
        "recovery",
        "remove",
        "-n",
        "--confirm",
        "payload:op:1",
        "payload:op:0",
    ])
    .unwrap();
    assert!(matches!(parsed.command, Command::Recovery(ref args)
        if matches!(args.command, RecoveryCommand::Remove(ref value)
            if value.dry_run && value.confirm && value.references.len() == 2)));
}
