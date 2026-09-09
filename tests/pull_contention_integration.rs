mod support;

use std::fs;

#[test]
fn pull_uses_shared_writer_lock_and_recovers_stale_owner_metadata() {
    for owner_operation in ["push", "pull"] {
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let metadata_dir = support::initialize_project_metadata(root.path());
        let source = root.path().join("source");
        let destination = root.path().join("destination");
        fs::write(&source, "accepted").unwrap();
        support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
        assert!(
            support::project_command(root.path(), &metadata_dir, &["push"])
                .status
                .success()
        );
        fs::write(&destination, "changed").unwrap();
        let home = support::project_home(&metadata_dir);
        let guard =
            grip::state::mutation_lock::MutationLock::acquire(&home, owner_operation).unwrap();
        let before = support::snapshot(root.path());
        let blocked =
            support::project_command(root.path(), &metadata_dir, &["--output=json", "pull"]);
        assert_eq!(blocked.status.code(), Some(13));
        let value = support::json(&blocked);
        assert_eq!(value["code"], "state_contention");
        assert_eq!(value["details"]["requested_operation"], "pull");
        assert_eq!(value["details"]["direction"], "pull");
        assert_eq!(value["details"]["owner"]["operation"], owner_operation);
        assert_eq!(support::snapshot(root.path()), before);
        drop(guard);

        fs::write(
            grip::state::lock::project_lock_path(&home, "mutation.lock").unwrap(),
            "stale-owner",
        )
        .unwrap();
        let applied =
            support::project_command(root.path(), &metadata_dir, &["--output=json", "pull"]);
        assert!(applied.status.success());
        assert_eq!(support::json(&applied)["details"]["result"], "applied");
        assert_eq!(fs::read_to_string(source).unwrap(), "changed");
    }
}
