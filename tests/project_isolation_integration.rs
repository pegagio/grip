mod support;

use std::fs;
use support::project::ProjectFixture;

#[test]
fn same_project_writer_contention_is_bounded_but_unrelated_projects_are_independent() {
    let first = ProjectFixture::initialized();
    let second = ProjectFixture::initialized();
    fs::write(first.project_root.join("source"), "first").unwrap();
    fs::write(second.project_root.join("source"), "second").unwrap();

    let _held = first.hold_lock("mutation.lock");
    let contended = first.command(&["--output=json", "add", "source", "~/first"]);
    assert_eq!(contended.status.code(), Some(12));

    let independent = second.command(&["add", "source", "~/second"]);
    assert!(
        independent.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&independent.stdout),
        String::from_utf8_lossy(&independent.stderr)
    );
}

#[test]
fn cloned_projects_keep_operation_and_state_bytes_isolated() {
    let fixture = ProjectFixture::initialized();
    let source = fixture.project_root.join("source");
    let destination = fixture.home_root.join("destination");
    fs::write(&source, "initial").unwrap();
    fs::write(&destination, "initial").unwrap();
    support::copy_complete_metadata(
        &source,
        &destination,
        grip::discovery::model::NodeKind::File,
    );
    assert!(
        fixture
            .command(&["add", "source", "~/destination"])
            .status
            .success()
    );

    fs::write(&source, "first-project-change").unwrap();
    assert!(fixture.command(&["push"]).status.success());
    let first_state_before = support::snapshot(&fixture.state_dir());

    let clone_root = fixture.copy_project("clone");
    let clone_home = fixture.root.path().join("clone-home");
    fs::create_dir(&clone_home).unwrap();
    let clone_destination = clone_home.join("destination");
    fs::copy(&destination, &clone_destination).unwrap();
    support::copy_complete_metadata(
        &destination,
        &clone_destination,
        grip::discovery::model::NodeKind::File,
    );
    let clone_reset = fixture
        .command_builder(&clone_root)
        .env("HOME", &clone_home)
        .args(["remove", "source"])
        .output()
        .unwrap();
    assert!(clone_reset.status.success());
    let clone_add = fixture
        .command_builder(&clone_root)
        .env("HOME", &clone_home)
        .args(["add", "source", "~/destination"])
        .output()
        .unwrap();
    assert!(clone_add.status.success());
    fs::write(clone_root.join("source"), "clone-change").unwrap();
    let clone_push = fixture
        .command_builder(&clone_root)
        .env("HOME", &clone_home)
        .arg("push")
        .output()
        .unwrap();
    assert!(
        clone_push.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&clone_push.stdout),
        String::from_utf8_lossy(&clone_push.stderr)
    );

    assert_eq!(support::snapshot(&fixture.state_dir()), first_state_before);
    assert_ne!(
        support::snapshot(&fixture.state_dir()),
        support::snapshot(&clone_root.join(".grip/state"))
    );
    assert!(fixture.state_dir().join("operations").is_dir());
    assert!(clone_root.join(".grip/state/operations").is_dir());
}
