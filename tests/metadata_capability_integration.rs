mod support;

use grip::metadata::model::{EndpointRole, Evidence, MetadataDimension};
use std::fs::File;
use std::os::unix::fs::MetadataExt;

#[test]
fn concrete_apfs_profile_reports_case_mtime_and_complete_metadata_operations() {
    let fixture = support::MetadataFixture::file(b"payload");
    let file = File::open(&fixture.source).unwrap();
    let profile = grip::metadata::macos::endpoint_capability_profile(
        &file,
        EndpointRole::Source,
        fixture.source.display().to_string(),
    )
    .unwrap();
    assert!(matches!(profile.filesystem_type, Evidence::Observed { ref value } if value == "apfs"));
    assert!(matches!(
        profile.mtime_precision_nanoseconds,
        Evidence::Observed { value: 1 }
    ));
    assert!(profile.case_sensitive.is_conclusive());
    assert!(profile.case_preserving.is_conclusive());
    for dimension in [
        MetadataDimension::PermissionMode,
        MetadataDimension::Owner,
        MetadataDimension::Group,
        MetadataDimension::ModificationTime,
        MetadataDimension::ExtendedAttribute,
        MetadataDimension::AccessControlList,
        MetadataDimension::BsdFlags,
    ] {
        let capability = profile
            .capabilities
            .iter()
            .find(|value| value.dimension == dimension)
            .unwrap();
        assert!(matches!(
            capability.inspect,
            Evidence::Observed { value: true }
        ));
        assert!(matches!(
            capability.apply,
            Evidence::Observed { value: true }
        ));
        assert!(matches!(
            capability.verify,
            Evidence::Observed { value: true }
        ));
    }
}

#[test]
fn numeric_ownership_authorization_is_conservative_and_nonmutating() {
    let fixture = support::MetadataFixture::file(b"payload");
    let before = support::snapshot(fixture.root.path());
    let file = File::open(&fixture.source).unwrap();
    let metadata = file.metadata().unwrap();
    assert!(matches!(
        grip::metadata::macos::ownership_authorization(&file, metadata.uid(), metadata.gid())
            .unwrap(),
        Evidence::Observed { value: true }
    ));
    assert!(matches!(
        grip::metadata::macos::ownership_authorization(
            &file,
            metadata.uid().saturating_add(1),
            metadata.gid()
        )
        .unwrap(),
        Evidence::Unauthorized { .. }
    ));
    assert_eq!(support::snapshot(fixture.root.path()), before);
}

#[test]
fn unknown_xattr_is_blocking_while_excluded_xattr_is_diagnostic_only() {
    let fixture = support::MetadataFixture::file(b"payload");
    support::set_fixture_xattr(&fixture.source, "com.apple.quarantine", b"excluded");
    support::set_fixture_xattr(&fixture.source, "com.example.private", b"unknown");
    let observed = grip::observation::fingerprint::inspect_complete(
        &fixture.source,
        grip::discovery::model::NodeKind::File,
    )
    .unwrap();
    assert!(
        observed
            .excluded_xattrs
            .contains(&b"com.apple.quarantine".to_vec())
    );
    assert_eq!(
        observed.unknown_xattrs,
        vec![b"com.example.private".to_vec()]
    );
}

#[test]
fn unsupported_protected_and_unknown_bsd_flags_are_named_without_becoming_equality_state() {
    use grip::discovery::model::NodeKind;

    assert_eq!(
        grip::metadata::macos::unsupported_bsd_flag_names(
            0x0000_0020 | 0x0002_0000 | 0x0200_0000,
            NodeKind::File,
        ),
        ["UF_COMPRESSED", "SF_IMMUTABLE", "0x02000000"]
    );
    assert_eq!(
        grip::metadata::macos::unsupported_bsd_flag_names(0x0000_0008, NodeKind::File),
        ["UF_OPAQUE"]
    );
    assert!(
        grip::metadata::macos::unsupported_bsd_flag_names(0x0000_0008, NodeKind::Directory)
            .is_empty()
    );
}

#[test]
fn qualification_record_captures_versioned_platform_and_binary_evidence_safely() {
    let fixture = support::MetadataFixture::file(b"payload");
    let record = support::qualification_record(&fixture.source, "debug");
    record.validate().unwrap();
    assert_eq!(record.filesystem_type, "apfs");
    assert_eq!(record.binary_build_profile, "debug");
    assert_eq!(record.binary_revision.len(), 40);
    assert!(!record.volume_capability_masks.is_empty());
    let encoded = serde_json::to_string(&record).unwrap();
    assert!(!encoded.contains(fixture.root.path().to_str().unwrap()));
    assert_eq!(
        serde_json::from_str::<grip::metadata::model::PlatformQualificationRecord>(&encoded)
            .unwrap(),
        record
    );
}

#[test]
#[ignore = "requires two distinct disposable APFS qualification volumes"]
fn cross_volume_profiles_prove_logical_metadata_capability_without_layout_promises() {
    let first = std::path::PathBuf::from(
        std::env::var_os("GRIP_APFS_CASE_INSENSITIVE_ROOT")
            .expect("GRIP_APFS_CASE_INSENSITIVE_ROOT is required"),
    );
    let second = std::path::PathBuf::from(
        std::env::var_os("GRIP_APFS_CASE_SENSITIVE_ROOT")
            .expect("GRIP_APFS_CASE_SENSITIVE_ROOT is required"),
    );
    let profiles = [first, second]
        .iter()
        .map(|root| {
            let file = File::open(root).unwrap();
            grip::metadata::macos::endpoint_capability_profile(
                &file,
                EndpointRole::Source,
                root.display().to_string(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    assert_ne!(
        profiles[0].filesystem_identity,
        profiles[1].filesystem_identity
    );
    for profile in profiles {
        assert!(
            matches!(profile.filesystem_type, Evidence::Observed { ref value } if value == "apfs")
        );
        assert!(profile.capabilities.iter().all(|capability| {
            matches!(capability.inspect, Evidence::Observed { value: true })
                && matches!(capability.apply, Evidence::Observed { value: true })
                && matches!(capability.verify, Evidence::Observed { value: true })
        }));
        let serialized = serde_json::to_value(&profile).unwrap();
        for physical_nonpromise in [
            "hard_link",
            "sparse",
            "clone",
            "compression",
            "block_layout",
        ] {
            assert!(serialized.get(physical_nonpromise).is_none());
        }
    }
}
