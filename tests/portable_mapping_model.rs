use grip::mapping::{MappingKind, PortableMapping};
use grip::path_policy::{HomeRelativePath, ProjectRelativePath};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStringExt;

fn reason(error: grip::GripError) -> String {
    match error {
        grip::GripError::Mapping { reason, .. } => reason,
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn project_relative_path_accepts_normalized_payload_paths() {
    let path = ProjectRelativePath::parse(OsStr::new("Users/pegagio/config"), false).unwrap();
    assert_eq!(path.as_str(), "Users/pegagio/config");
}

#[test]
fn project_relative_path_accepts_dot_only_for_tree_root() {
    assert_eq!(
        ProjectRelativePath::parse(OsStr::new("."), true)
            .unwrap()
            .as_str(),
        "."
    );
    assert_eq!(
        reason(ProjectRelativePath::parse(OsStr::new("."), false).unwrap_err()),
        "invalid_project_relative_path"
    );
}

#[test]
fn project_relative_path_rejects_metadata_and_noncanonical_forms() {
    for input in [
        "",
        "/absolute",
        "../escape",
        "a/../escape",
        "a/./b",
        "a//b",
        "a/",
        ".grip",
        ".grip/state",
        "${HOME}/x",
    ] {
        assert!(
            ProjectRelativePath::parse(OsStr::new(input), true).is_err(),
            "{input}"
        );
    }
}

#[test]
fn project_relative_path_rejects_non_utf8() {
    let input = OsString::from_vec(vec![b'a', 0xff]);
    assert_eq!(
        reason(ProjectRelativePath::parse(&input, true).unwrap_err()),
        "non_utf8_path"
    );
}

#[test]
fn home_relative_path_accepts_home_and_descendants() {
    assert_eq!(
        HomeRelativePath::parse(OsStr::new("~")).unwrap().as_str(),
        "~"
    );
    assert_eq!(
        HomeRelativePath::parse(OsStr::new("~/.config/editor"))
            .unwrap()
            .as_str(),
        "~/.config/editor"
    );
}

#[test]
fn home_relative_path_rejects_noncanonical_and_expanding_forms() {
    for input in [
        "",
        "/absolute",
        "relative",
        "~/",
        "~/a/",
        "~/a//b",
        "~/a/./b",
        "~/../escape",
        "~other/x",
        "${HOME}/x",
        "$HOME/x",
    ] {
        assert!(
            HomeRelativePath::parse(OsStr::new(input)).is_err(),
            "{input}"
        );
    }
}

#[test]
fn portable_mapping_uses_the_complete_canonical_tuple() {
    let mapping = PortableMapping::parse(
        MappingKind::Tree,
        OsStr::new("Users/pegagio"),
        OsStr::new("~"),
    )
    .unwrap();
    assert_eq!(mapping.identity(), "Users/pegagio\0tree\0~");
}
