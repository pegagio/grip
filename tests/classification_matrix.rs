use grip::classification;
use grip::classification::model::{Classification, Direction};
use grip::discovery::model::NodeKind;
use grip::mapping::MappingKind;
use grip::observation::model::{
    ContentFingerprint, EntryIdentity, MappingSnapshot, Membership, ObservedEntry, SupportedState,
};
use std::path::PathBuf;

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
