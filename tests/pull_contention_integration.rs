mod support;

use std::fs;

#[test]
fn pull_uses_shared_writer_lock_and_recovers_stale_owner_metadata() {
    for owner_operation in ["push", "pull"] {
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let grip_home = support::minimal_home(root.path());
        let source = root.path().join("source");
        let destination = root.path().join("destination");
        fs::write(&source, "accepted").unwrap();
        support::write_registry(&grip_home, &[("file", &source, &destination)]);
        assert!(
            support::command_with_grip_home(root.path(), &grip_home, &["push"])
                .status
                .success()
        );
        fs::write(&destination, "changed").unwrap();
        let home = grip::home::select(Some(grip_home.clone().into_os_string()), None).unwrap();
        let guard =
            grip::state::mutation_lock::MutationLock::acquire(&home, owner_operation).unwrap();
        let before = support::snapshot(root.path());
        let blocked =
            support::command_with_grip_home(root.path(), &grip_home, &["--output=json", "pull"]);
        assert_eq!(blocked.status.code(), Some(13));
        let value = support::json(&blocked);
        assert_eq!(value["code"], "state_contention");
        assert_eq!(value["details"]["requested_operation"], "pull");
        assert_eq!(value["details"]["direction"], "pull");
        assert_eq!(value["details"]["owner"]["operation"], owner_operation);
        assert_eq!(support::snapshot(root.path()), before);
        drop(guard);

        fs::write(home.path().join(".mutation.lock"), "stale-owner").unwrap();
        let applied =
            support::command_with_grip_home(root.path(), &grip_home, &["--output=json", "pull"]);
        assert!(applied.status.success());
        assert_eq!(support::json(&applied)["details"]["result"], "applied");
        assert_eq!(fs::read_to_string(source).unwrap(), "changed");
    }
}
