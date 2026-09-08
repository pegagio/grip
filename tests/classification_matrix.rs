use grip::classification;
use grip::classification::model::{Classification, Direction};
use grip::discovery::model::NodeKind;
use grip::mapping::MappingKind;
use grip::observation::model::{
    ContentFingerprint, EntryIdentity, MappingSnapshot, Membership, ObservedEntry, SupportedState,
};
use std::path::PathBuf;

fn complete(seed: char, mode: &str) -> grip::metadata::model::SupportedEntryStateV3 {
    grip::metadata::model::SupportedEntryStateV3 {
        node_kind: NodeKind::File,
        content: Some(ContentFingerprint {
            algorithm: "sha256".into(),
            digest: seed.to_string().repeat(64),
            length: 1,
        }),
        metadata: grip::metadata::model::MetadataState {
            permission_mode: mode.into(),
            uid: 501,
            gid: 20,
            modified_time: grip::metadata::model::ModificationTime {
                seconds: 10,
                nanoseconds: 20,
            },
            extended_attributes: Vec::new(),
            acl: grip::metadata::model::AclState::Absent,
            bsd_flags: Default::default(),
        },
    }
}

fn complete_entry(
    source: grip::metadata::model::SupportedEntryStateV3,
    destination: grip::metadata::model::SupportedEntryStateV3,
) -> ObservedEntry {
    let mut value = entry(Some(state('a')), Some(state('a')), Membership::Active);
    value.source_complete = Some(grip::observation::model::CompleteObservedState {
        state: source,
        excluded_xattrs: Vec::new(),
        unknown_xattrs: Vec::new(),
        unsupported_bsd_flags: Vec::new(),
    });
    value.destination_complete = Some(grip::observation::model::CompleteObservedState {
        state: destination,
        excluded_xattrs: Vec::new(),
        unknown_xattrs: Vec::new(),
        unsupported_bsd_flags: Vec::new(),
    });
    value
}

#[test]
fn complete_state_three_way_classifies_metadata_and_content_changes_indivisibly() {
    let baseline = complete('a', "0644");
    let metadata_change = complete('a', "0600");
    let content_and_metadata = complete('b', "0600");
    let identity = complete_entry(baseline.clone(), baseline.clone()).identity;
    let accepted = grip::state::AcceptedState {
        generation: Some(1),
        baselines: Default::default(),
        complete_baselines: std::collections::BTreeMap::from([(identity, baseline.clone())]),
        schema_version: Some(3),
        accepted_bytes: None,
    };
    for (source, destination, expected) in [
        (
            metadata_change.clone(),
            baseline.clone(),
            Classification::SourceOnlyChange,
        ),
        (
            baseline.clone(),
            metadata_change.clone(),
            Classification::DestinationOnlyChange,
        ),
        (
            metadata_change.clone(),
            metadata_change.clone(),
            Classification::ConvergedTwoSidedChange,
        ),
        (
            metadata_change.clone(),
            content_and_metadata.clone(),
            Classification::DivergentConflict,
        ),
    ] {
        let record =
            classification::classify_accepted(&complete_entry(source, destination), &accepted);
        assert_eq!(record.classification, expected);
    }
    let record =
        classification::classify_accepted(&complete_entry(metadata_change, baseline), &accepted);
    assert_eq!(
        record.changed_dimensions.source_to_baseline,
        Some(vec![
            grip::classification::model::ChangedDimension::PermissionMode
        ])
    );
}

#[test]
fn every_complete_metadata_dimension_uses_the_same_three_way_semantics() {
    use grip::classification::model::ChangedDimension;
    use grip::metadata::model::{
        AccessControlEntry, AclEntryKind, AclPermission, AclState, BsdFlag, XattrFingerprint,
    };
    use std::collections::BTreeSet;

    let baseline = complete('a', "0644");
    let mut variants = Vec::new();
    let mut mode = baseline.clone();
    mode.metadata.permission_mode = "0600".into();
    variants.push((ChangedDimension::PermissionMode, mode));
    let mut owner = baseline.clone();
    owner.metadata.uid += 1;
    variants.push((ChangedDimension::Owner, owner));
    let mut group = baseline.clone();
    group.metadata.gid += 1;
    variants.push((ChangedDimension::Group, group));
    let mut mtime = baseline.clone();
    mtime.metadata.modified_time.nanoseconds += 1;
    variants.push((ChangedDimension::ModificationTime, mtime));
    let mut xattr = baseline.clone();
    xattr.metadata.extended_attributes.push(XattrFingerprint {
        name: b"com.apple.TextEncoding".to_vec(),
        length: 1,
        algorithm: "sha256".into(),
        digest: "b".repeat(64),
    });
    variants.push((ChangedDimension::ExtendedAttribute, xattr));
    let mut acl = baseline.clone();
    acl.metadata.acl = AclState::Present {
        entries: vec![AccessControlEntry {
            principal_uuid: [1; 16],
            kind: AclEntryKind::Allow,
            permissions: BTreeSet::from([AclPermission::ReadData]),
            flags: BTreeSet::new(),
        }],
    };
    variants.push((ChangedDimension::AccessControlList, acl));
    let mut flags = baseline.clone();
    flags.metadata.bsd_flags.insert(BsdFlag::Hidden);
    variants.push((ChangedDimension::BsdFlags, flags));

    let identity = complete_entry(baseline.clone(), baseline.clone()).identity;
    let accepted = grip::state::AcceptedState {
        generation: Some(1),
        baselines: Default::default(),
        complete_baselines: std::collections::BTreeMap::from([(identity, baseline.clone())]),
        schema_version: Some(3),
        accepted_bytes: None,
    };
    for (dimension, changed) in variants {
        for (source, destination, expected) in [
            (
                changed.clone(),
                baseline.clone(),
                Classification::SourceOnlyChange,
            ),
            (
                baseline.clone(),
                changed.clone(),
                Classification::DestinationOnlyChange,
            ),
            (
                changed.clone(),
                changed.clone(),
                Classification::ConvergedTwoSidedChange,
            ),
        ] {
            let record =
                classification::classify_accepted(&complete_entry(source, destination), &accepted);
            assert_eq!(record.classification, expected, "{dimension:?}");
        }
        let record = classification::classify_accepted(
            &complete_entry(changed.clone(), complete('b', "0600")),
            &accepted,
        );
        assert_eq!(record.classification, Classification::DivergentConflict);
        let record = classification::classify_accepted(
            &complete_entry(changed, baseline.clone()),
            &accepted,
        );
        assert_eq!(
            record.changed_dimensions.source_to_baseline,
            Some(vec![dimension])
        );
    }
}

fn state(seed: char) -> SupportedState {
    SupportedState {
        node_kind: NodeKind::File,
        content: Some(ContentFingerprint {
            algorithm: "sha256".into(),
            digest: seed.to_string().repeat(64),
            length: 1,
        }),
        permission_mode: Some("0644".into()),
    }
}

fn entry(
    source: Option<SupportedState>,
    destination: Option<SupportedState>,
    membership: Membership,
) -> ObservedEntry {
    let identity = EntryIdentity::new(
        MappingSnapshot {
            kind: MappingKind::File,
            source: PathBuf::from("/source"),
            destination: PathBuf::from("/destination"),
        },
        Vec::new(),
    )
    .unwrap();
    ObservedEntry {
        identity,
        membership,
        source,
        destination,
        source_complete: None,
        destination_complete: None,
        metadata_findings: Vec::new(),
        endpoint_capabilities: Vec::new(),
        source_diagnostic: None,
        destination_diagnostic: None,
        unsupported: Vec::new(),
        blocking: false,
    }
}

#[test]
fn exhaustive_supported_state_matrix_has_stable_classification_and_direction() {
    let accepted = state('a');
    let changed = state('b');
    let other = state('c');
    let cases = vec![
        (
            entry(Some(accepted.clone()), None, Membership::Active),
            None,
            Classification::SourceAddition,
            Direction::SourceToDestination,
        ),
        (
            entry(
                Some(accepted.clone()),
                Some(accepted.clone()),
                Membership::Active,
            ),
            None,
            Classification::InitialMatch,
            Direction::None,
        ),
        (
            entry(
                Some(accepted.clone()),
                Some(changed.clone()),
                Membership::Active,
            ),
            None,
            Classification::InitialCollision,
            Direction::None,
        ),
        (
            entry(None, Some(accepted.clone()), Membership::DestinationOnly),
            None,
            Classification::DestinationOnlyUnmanaged,
            Direction::None,
        ),
        (
            entry(
                Some(accepted.clone()),
                Some(accepted.clone()),
                Membership::Active,
            ),
            Some(&accepted),
            Classification::Synchronized,
            Direction::None,
        ),
        (
            entry(
                Some(changed.clone()),
                Some(accepted.clone()),
                Membership::Active,
            ),
            Some(&accepted),
            Classification::SourceOnlyChange,
            Direction::SourceToDestination,
        ),
        (
            entry(
                Some(accepted.clone()),
                Some(changed.clone()),
                Membership::Active,
            ),
            Some(&accepted),
            Classification::DestinationOnlyChange,
            Direction::DestinationToSource,
        ),
        (
            entry(
                Some(changed.clone()),
                Some(changed.clone()),
                Membership::Active,
            ),
            Some(&accepted),
            Classification::ConvergedTwoSidedChange,
            Direction::None,
        ),
        (
            entry(Some(changed.clone()), Some(other), Membership::Active),
            Some(&accepted),
            Classification::DivergentConflict,
            Direction::None,
        ),
        (
            entry(None, Some(accepted.clone()), Membership::Active),
            Some(&accepted),
            Classification::SourceSideDeletion,
            Direction::SourceToDestination,
        ),
        (
            entry(Some(accepted.clone()), None, Membership::Active),
            Some(&accepted),
            Classification::DestinationSideDeletion,
            Direction::DestinationToSource,
        ),
        (
            entry(None, Some(changed.clone()), Membership::Active),
            Some(&accepted),
            Classification::DeleteChangeConflict,
            Direction::None,
        ),
        (
            entry(Some(changed), None, Membership::Active),
            Some(&accepted),
            Classification::ChangeDeleteConflict,
            Direction::None,
        ),
        (
            entry(None, None, Membership::Active),
            Some(&accepted),
            Classification::ConvergedDeletion,
            Direction::None,
        ),
    ];
    for (entry, baseline, expected, direction) in cases {
        let record = classification::classify(&entry, baseline);
        assert_eq!(record.classification, expected);
        assert_eq!(record.prospective_direction, direction);
    }
}

#[test]
fn policy_and_safety_overrides_cover_remaining_categories() {
    let accepted = state('a');
    let mut observed = entry(None, None, Membership::Ignored);
    assert_eq!(
        classification::classify(&observed, Some(&accepted)).classification,
        Classification::NewlyIgnoredPendingRetirement
    );
    observed.membership = Membership::Untracked;
    assert_eq!(
        classification::classify(&observed, Some(&accepted)).classification,
        Classification::UntrackedPendingRetirement
    );
    observed.membership = Membership::Active;
    observed.blocking = true;
    observed.unsupported = vec!["symlink".into()];
    assert_eq!(
        classification::classify(&observed, None).classification,
        Classification::UnsupportedManaged
    );
    observed.unsupported = vec!["destination:wrong_node_kind".into()];
    assert_eq!(
        classification::classify(&observed, None).classification,
        Classification::UnsafeCollision
    );
}
