mod support;

use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::symlink;
use support::project::ProjectFixture;

#[test]
fn portable_mapping_cli_rejects_noncanonical_and_escaping_forms() {
    let fixture = ProjectFixture::initialized();
    fs::write(fixture.project_root.join("source"), "value").unwrap();
    let before = fs::read(fixture.descriptor_path()).unwrap();
    for (source, destination) in [
        ("/source", "~/target"),
        ("../source", "~/target"),
        ("./source", "~/target"),
        ("source//child", "~/target"),
        ("source/", "~/target"),
        ("source", "/target"),
        ("source", "~/../target"),
        ("source", "~/target/"),
        ("source", "~/target//child"),
        ("source", "$HOME/target"),
        ("source", "${HOME}/target"),
        ("source", "~other/target"),
    ] {
        let output = fixture.command(&["mapping", "add", "file", source, destination]);
        assert!(
            !output.status.success(),
            "accepted {source} -> {destination}"
        );
    }
    assert_eq!(fs::read(fixture.descriptor_path()).unwrap(), before);
    for selector in ["/source", "../source", "./source", "source/"] {
        assert!(
            !fixture
                .command(&["mapping", "inspect", selector])
                .status
                .success()
        );
    }
}

#[test]
fn portable_mapping_cli_rejects_non_utf8_and_symlink_escape() {
    let fixture = ProjectFixture::initialized();
    let outside = fixture.root.path().join("outside");
    fs::write(&outside, "value").unwrap();
    symlink(&outside, fixture.project_root.join("link")).unwrap();
    let linked = fixture.command(&["mapping", "add", "file", "link", "~/target"]);
    assert!(!linked.status.success());

    let invalid = OsString::from_vec(vec![b's', 0xff]);
    for arguments in [
        vec![
            OsString::from("mapping"),
            OsString::from("add"),
            OsString::from("file"),
            invalid.clone(),
            OsString::from("~/target"),
        ],
        vec![
            OsString::from("mapping"),
            OsString::from("add"),
            OsString::from("file"),
            OsString::from("source"),
            invalid.clone(),
        ],
    ] {
        let output = fixture.command_os(&fixture.project_root, arguments);
        assert!(!output.status.success());
    }
}
