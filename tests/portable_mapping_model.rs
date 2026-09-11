use grip::mapping::{MappingKind, PortableMapping};
use grip::path_policy::{DestinationPath, ProjectRelativePath};
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
fn destination_path_accepts_and_preserves_all_authorized_forms() {
    for input in [
        "~",
        "~/",
        "~/.config/editor",
        "~/a//b",
        "~/a/./b",
        "~/../escape",
        "/absolute/destination",
    ] {
        assert_eq!(
            DestinationPath::parse(OsStr::new(input)).unwrap().as_str(),
            input
        );
    }
}

#[test]
fn destination_path_rejects_relative_and_expanding_forms() {
    for input in [
        "",
        "relative",
        "./relative",
        "../relative",
        "~other/x",
        "${HOME}/x",
        "$HOME/x",
    ] {
        assert!(
            DestinationPath::parse(OsStr::new(input)).is_err(),
            "{input}"
        );
    }
}

#[test]
fn destination_path_lexically_normalizes_only_for_operational_resolution() {
    let home = std::path::Path::new("/tmp/home");
    let path = DestinationPath::parse(OsStr::new("~/a//./b/../c")).unwrap();
    assert_eq!(path.as_str(), "~/a//./b/../c");
    assert_eq!(
        path.resolve(home),
        std::path::PathBuf::from("/tmp/home/a/c")
    );
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
