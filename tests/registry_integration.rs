use grip::{ResultCategory, registry};

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
