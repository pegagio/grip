mod support;

use grip::project::{ProjectContext, ProjectSelection};
use std::fs;
use support::project::ProjectFixture;

fn json(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

fn selected(fixture: &ProjectFixture) -> ProjectContext {
    let home = grip::home::select_user_home(Some(fixture.home_root.clone())).unwrap();
    ProjectContext::select(
        ProjectSelection::Explicit(fixture.project_root.clone()),
        home,
    )
    .unwrap()
    .unwrap()
}

#[test]
fn implicit_selection_works_from_root_deep_descendant_and_one_hundred_levels() {
    let fixture = ProjectFixture::initialized();
    for cwd in [fixture.project_root.clone(), fixture.deep_descendant(100)] {
        let output = fixture.command_from(&cwd, &["--output=json", "validate"]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            json(&output)["details"]["project"]["root"]["display"],
            fixture.project_root.display().to_string()
        );
    }
}

#[test]
fn explicit_selection_requires_the_exact_root_and_never_falls_back() {
    let fixture = ProjectFixture::initialized();
    let outside = fixture.root.path().join("outside");
    fs::create_dir(&outside).unwrap();
    let output = fixture.command_from(
        &outside,
        &[
            "validate",
            "--project",
            fixture.project_root.to_str().unwrap(),
            "--output=json",
        ],
    );
    assert!(output.status.success());

    let descendant = fixture.deep_descendant(1);
    let output = fixture.command_from(
        &fixture.project_root,
        &["--project", descendant.to_str().unwrap(), "validate"],
    );
    assert!(!output.status.success());
}

#[test]
fn implicit_selection_fails_for_zero_multiple_and_invalid_inner_candidates() {
    let absent = ProjectFixture::new();
    let output = absent.command(&["--output=json", "validate"]);
    assert_eq!(output.status.code(), Some(10));
    assert_eq!(json(&output)["details"]["reason"], "project_not_found");

    let multiple = ProjectFixture::initialized();
    let nested = multiple.nested_project("nested");
    let output = multiple.command_from(&nested, &["--output=json", "validate"]);
    assert_eq!(output.status.code(), Some(10));
    assert_eq!(json(&output)["details"]["reason"], "ambiguous_project");

    let invalid = ProjectFixture::initialized();
    let inner = invalid.project_root.join("inner");
    fs::create_dir_all(inner.join(".grip")).unwrap();
    let output = invalid.command_from(&inner, &["--output=json", "validate"]);
    assert_eq!(output.status.code(), Some(10));
    assert_ne!(
        json(&output)["details"]["project"]["root"]["display"],
        invalid.project_root.display().to_string()
    );
}

#[test]
fn retained_context_rejects_descriptor_and_ignore_substitution() {
    let fixture = ProjectFixture::initialized();
    let context = selected(&fixture);
    fs::write(fixture.descriptor_path(), "schema_version = 2\n\n[[mappings]]\nkind = \"file\"\nsource = \"missing\"\ndestination = \"~/missing\"\n").unwrap();
    assert!(context.revalidate().is_err());

    fixture.write_descriptor(&[]);
    let context = selected(&fixture);
    fs::write(fixture.metadata_dir().join(".gitignore"), "extra\n").unwrap();
    assert!(context.revalidate().is_err());
}

#[test]
fn retained_context_rejects_root_substitution() {
    let fixture = ProjectFixture::initialized();
    let context = selected(&fixture);
    fs::rename(
        &fixture.project_root,
        fixture.root.path().join("original-project"),
    )
    .unwrap();
    fs::create_dir(&fixture.project_root).unwrap();
    fs::create_dir(fixture.metadata_dir()).unwrap();
    fs::write(
        fixture.descriptor_path(),
        support::project::EMPTY_DESCRIPTOR_V2,
    )
    .unwrap();
    fs::write(
        fixture.metadata_dir().join(".gitignore"),
        support::project::PROJECT_GITIGNORE,
    )
    .unwrap();
    assert!(context.revalidate().is_err());
}

#[test]
fn retained_context_rejects_metadata_directory_substitution() {
    let fixture = ProjectFixture::initialized();
    let context = selected(&fixture);
    fs::rename(
        fixture.metadata_dir(),
        fixture.project_root.join("original-metadata"),
    )
    .unwrap();
    fixture.write_descriptor(&[]);
    assert!(context.revalidate().is_err());
}

#[test]
fn retained_context_rejects_descriptor_node_substitution_with_identical_bytes() {
    let fixture = ProjectFixture::initialized();
    let context = selected(&fixture);
    fs::rename(
        fixture.descriptor_path(),
        fixture.metadata_dir().join("original-config.toml"),
    )
    .unwrap();
    fs::write(
        fixture.descriptor_path(),
        support::project::EMPTY_DESCRIPTOR_V2,
    )
    .unwrap();
    assert!(context.revalidate().is_err());
}

#[test]
fn retained_context_rejects_ignore_node_substitution_with_identical_bytes() {
    let fixture = ProjectFixture::initialized();
    let context = selected(&fixture);
    let ignore = fixture.metadata_dir().join(".gitignore");
    fs::rename(&ignore, fixture.metadata_dir().join("original-gitignore")).unwrap();
    fs::write(&ignore, support::project::PROJECT_GITIGNORE).unwrap();
    assert!(context.revalidate().is_err());
}

#[test]
fn retained_context_rejects_destination_home_substitution() {
    let fixture = ProjectFixture::initialized();
    let context = selected(&fixture);
    fs::rename(
        &fixture.home_root,
        fixture.root.path().join("original-home"),
    )
    .unwrap();
    fs::create_dir(&fixture.home_root).unwrap();
    assert!(context.revalidate().is_err());
}

#[test]
fn retained_context_rejects_in_place_descriptor_byte_substitution() {
    let fixture = ProjectFixture::initialized();
    let context = selected(&fixture);
    fs::write(
        fixture.descriptor_path(),
        "schema_version = 2\n\n[[mappings]]\nkind = \"file\"\nsource = \"source\"\ndestination = \"~/destination\"\n",
    )
    .unwrap();
    assert!(context.revalidate().is_err());
}
