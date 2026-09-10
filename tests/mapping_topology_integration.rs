mod support;

use std::fs;
use support::project::ProjectFixture;

fn add(fixture: &ProjectFixture, source: &str, destination: &str) -> bool {
    fixture
        .command(&["add", source, destination])
        .status
        .success()
}

#[test]
fn complete_registry_rejects_duplicate_nested_and_cross_recursive_ownership() {
    for (first, second) in [
        (("tree", "a", "~/x"), ("tree", "a/b", "~/y")),
        (("tree", "a", "~/x"), ("tree", "b", "~/x/y")),
        (("tree", "a", "~/x"), ("tree", "a", "~/y")),
        (("tree", "a", "~/x"), ("tree", "a", "~/x")),
    ] {
        let fixture = ProjectFixture::initialized();
        fs::create_dir_all(fixture.project_root.join("a")).unwrap();
        fs::create_dir_all(fixture.project_root.join("a/b")).unwrap();
        fs::create_dir_all(fixture.project_root.join("b")).unwrap();
        assert!(add(&fixture, first.1, first.2));
        let before = fs::read(fixture.descriptor_path()).unwrap();
        assert!(!add(&fixture, second.1, second.2));
        assert_eq!(fs::read(fixture.descriptor_path()).unwrap(), before);
    }
}

#[test]
fn cross_recursive_ownership_is_rejected_when_home_and_project_spaces_overlap() {
    let fixture = ProjectFixture::initialized();
    fs::create_dir(fixture.project_root.join("a")).unwrap();
    fs::create_dir(fixture.project_root.join("b")).unwrap();
    let first = fixture
        .command_builder(&fixture.project_root)
        .env("HOME", &fixture.project_root)
        .args(["add", "a", "~/b"])
        .output()
        .unwrap();
    assert!(first.status.success());
    let before = fs::read(fixture.descriptor_path()).unwrap();
    let second = fixture
        .command_builder(&fixture.project_root)
        .env("HOME", &fixture.project_root)
        .args(["add", "b", "~/a"])
        .output()
        .unwrap();
    assert!(!second.status.success());
    assert_eq!(fs::read(fixture.descriptor_path()).unwrap(), before);
}

#[test]
fn equal_resolved_endpoints_are_rejected() {
    let fixture = ProjectFixture::initialized();
    fs::create_dir(fixture.project_root.join("a")).unwrap();
    let output = fixture
        .command_builder(&fixture.project_root)
        .env("HOME", &fixture.project_root)
        .args(["add", "a", "~/a"])
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn component_boundary_siblings_are_disjoint() {
    let fixture = ProjectFixture::initialized();
    fs::create_dir(fixture.project_root.join("a")).unwrap();
    fs::create_dir(fixture.project_root.join("ab")).unwrap();
    assert!(add(&fixture, "a", "~/x"));
    assert!(add(&fixture, "ab", "~/xy"));
}
