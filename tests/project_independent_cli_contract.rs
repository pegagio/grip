mod support;

use support::project::ProjectFixture;

#[test]
fn help_and_versions_are_independent_of_projects_home() {
    let fixture = ProjectFixture::new();
    for arguments in [
        vec!["--help"],
        vec!["--version"],
        vec!["version"],
        vec!["add", "--help"],
        vec!["push", "--help"],
    ] {
        let before = fixture.snapshot_all();
        let output = fixture.command(&arguments);
        assert!(output.status.success(), "{arguments:?}");
        assert_eq!(fixture.snapshot_all(), before);
    }
}

#[test]
fn parser_and_domain_errors_begin_with_an_uppercase_error_prefix() {
    let fixture = ProjectFixture::initialized();
    let parser = fixture.command(&["add", "only-source"]);
    assert_eq!(parser.status.code(), Some(2));
    assert!(
        String::from_utf8(parser.stderr)
            .unwrap()
            .starts_with("Error: the following required arguments were not provided:")
    );

    let domain = fixture.command(&["push", "/absolute-source"]);
    assert_eq!(domain.status.code(), Some(10));
    assert!(
        String::from_utf8(domain.stdout)
            .unwrap()
            .starts_with("Error: source must be a relative path outside .grip")
    );
}
