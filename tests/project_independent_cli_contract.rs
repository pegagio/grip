mod support;

use support::project::ProjectFixture;

#[test]
fn help_and_versions_are_independent_of_projects_home_and_legacy_traps() {
    let fixture = ProjectFixture::new();
    for arguments in [
        vec!["--help"],
        vec!["--version"],
        vec!["version"],
        vec!["mapping", "--help"],
    ] {
        let before = fixture.snapshot_all();
        let output = fixture.command(&arguments);
        assert!(output.status.success(), "{arguments:?}");
        assert_eq!(fixture.snapshot_all(), before);
    }
}
