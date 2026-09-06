mod support;

use std::fs;

#[test]
fn active_writer_reports_exit_13_owner_and_keeps_diagnostics_separate() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    fs::write(&source, "payload").unwrap();
    support::write_registry(&grip_home, &[("file", &source, &destination)]);
    let home = grip::home::select(Some(grip_home.clone().into_os_string()), None).unwrap();
    let guard = grip::state::mutation_lock::MutationLock::acquire(&home, "mapping_add").unwrap();
    let blocked =
        support::command_with_grip_home(root.path(), &grip_home, &["--output=json", "push"]);
    assert_eq!(blocked.status.code(), Some(13));
    assert!(blocked.stderr.is_empty());
    let value = support::json(&blocked);
    assert_eq!(value["code"], "state_contention");
    assert_eq!(value["details"]["reason"], "state_contention");
    assert_eq!(value["details"]["operation"], "mapping_add");
    assert_eq!(value["details"]["owner"]["operation"], "mapping_add");
    drop(guard);

    let applied =
        support::command_with_grip_home(root.path(), &grip_home, &["--output=json", "-v", "push"]);
    assert!(applied.status.success());
    assert!(String::from_utf8_lossy(&applied.stderr).contains("command completed"));
    assert_eq!(support::json(&applied)["details"]["result"], "applied");
}
