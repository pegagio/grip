use grip::mapping::{MappingKind, PortableMapping};
use grip::{ResultCategory, registry};
use std::ffi::OsStr;
mod support;

#[test]
fn descriptor_v2_round_trips_in_canonical_tuple_order() {
    let descriptor = registry::ProjectDescriptorV2::new(vec![
        PortableMapping::parse(MappingKind::Tree, OsStr::new("z"), OsStr::new("~/z")).unwrap(),
        PortableMapping::parse(MappingKind::File, OsStr::new("a"), OsStr::new("~/a")).unwrap(),
    ])
    .unwrap();
    let bytes = registry::encode_descriptor(&descriptor).unwrap();
    let decoded = registry::decode_descriptor(std::str::from_utf8(&bytes).unwrap()).unwrap();
    assert_eq!(descriptor, decoded);
    assert!(
        std::str::from_utf8(&bytes)
            .unwrap()
            .find("source = \"a\"")
            .unwrap()
            < std::str::from_utf8(&bytes)
                .unwrap()
                .find("source = \"z\"")
                .unwrap()
    );
}

#[test]
fn descriptor_v2_is_strict_and_rejects_v1() {
    for invalid in [
        "schema_version = 1\nmappings = []\n",
        "schema_version = 2\nmappings = []\nunknown = true\n",
        "schema_version = 2\n[[mappings]]\nkind = \"file\"\nsource = \"/absolute\"\ndestination = \"~/x\"\n",
        "schema_version = 2\n[[mappings]]\nkind = \"file\"\nsource = \"x\"\ndestination = \"/absolute\"\n",
        "schema_version = 2\n[[mappings]]\nkind = \"file\"\nsource = \".\"\ndestination = \"~/x\"\n",
    ] {
        assert!(registry::decode_descriptor(invalid).is_err(), "{invalid}");
    }
    assert_eq!(
        registry::decode_descriptor("schema_version = 1\nmappings = []\n")
            .unwrap_err()
            .category(),
        ResultCategory::UnsupportedSchema
    );
}

#[test]
fn descriptor_v2_rejects_resolved_ownership_conflicts() {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("project");
    let home = root.path().join("home");
    std::fs::create_dir(&project).unwrap();
    std::fs::create_dir(&home).unwrap();
    std::fs::write(project.join("a"), "a").unwrap();
    std::fs::write(project.join("b"), "b").unwrap();
    let descriptor = registry::ProjectDescriptorV2::new(vec![
        PortableMapping::parse(MappingKind::File, OsStr::new("a"), OsStr::new("~/same")).unwrap(),
        PortableMapping::parse(MappingKind::File, OsStr::new("b"), OsStr::new("~/same")).unwrap(),
    ])
    .unwrap();
    assert!(matches!(
        descriptor.resolve(&project, &home, "test"),
        Err(grip::GripError::Mapping { ref reason, .. }) if reason == "ownership_conflicts"
    ));
}

#[test]
fn durable_registry_load_allows_a_deleted_accepted_source() {
    let root = tempfile::tempdir().unwrap();
    let metadata_dir = support::initialize_project_metadata(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    std::fs::write(&source, "payload").unwrap();
    support::write_descriptor(&metadata_dir, &[("file", &source, &destination)]);
    std::fs::remove_file(&source).unwrap();
    let home = support::project_home(&metadata_dir);
    assert_eq!(
        grip::registry::publication::load(&home, false)
            .unwrap()
            .registry
            .mappings()
            .len(),
        1
    );
}
