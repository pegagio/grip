use grip::{ResultCategory, registry};
mod support;

#[test]
fn durable_registry_load_allows_a_deleted_accepted_source() {
    let root = tempfile::tempdir().unwrap();
    let grip_home = support::minimal_home(root.path());
    let source = root.path().join("source");
    let destination = root.path().join("destination");
    std::fs::write(&source, "payload").unwrap();
    support::write_registry(&grip_home, &[("file", &source, &destination)]);
    std::fs::remove_file(&source).unwrap();
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    assert_eq!(
        grip::registry::publication::load(&home, false)
            .unwrap()
            .registry
            .mappings()
            .len(),
        1
    );
}

#[test]
fn accepts_only_empty_strict_v1_registry() {
    assert!(registry::decode("schema_version = 1\nmappings = []\n").is_ok());
    for bad in [
        "",
        "schema_version = 1\n",
        "schema_version = 1\nmappings = []\nextra = true\n",
        "schema_version = 1\n[[mappings]]\nsource = 'x'\n",
    ] {
        assert_eq!(
            registry::decode(bad).unwrap_err().category(),
            ResultCategory::InvalidConfiguration
        );
    }
}

#[test]
fn dispatches_unsupported_version_before_typed_decode() {
    assert_eq!(
        registry::decode("schema_version = 42\nunknown = true\n")
            .unwrap_err()
            .category(),
        ResultCategory::UnsupportedSchema
    );
}
