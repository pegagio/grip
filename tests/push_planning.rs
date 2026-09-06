use grip::classification::model::{
    ChangedDimensions, Classification, ClassificationRecord, ClassificationScope, Direction,
};
use grip::discovery::model::{NodeKind, SafePath};
use grip::mapping::MappingKind;
use grip::observation::model::{
    ContentFingerprint, EntryIdentity, MappingSnapshot, PathSpace, SupportedState,
};
use grip::push::model::Disposition;
use std::path::PathBuf;

fn file_state(byte: char) -> SupportedState {
    SupportedState {
        node_kind: NodeKind::File,
        content: Some(ContentFingerprint {
            algorithm: "sha256".into(),
            digest: byte.to_string().repeat(64),
            length: 1,
        }),
        permission_mode: Some("0644".into()),
    }
}

fn record(index: usize, classification: Classification, blocking: bool) -> ClassificationRecord {
    let source = PathBuf::from(format!("/source-{index}"));
    let destination = PathBuf::from(format!("/destination-{index}"));
    let identity = EntryIdentity::new(
        MappingSnapshot {
            kind: MappingKind::File,
            source: source.clone(),
            destination: destination.clone(),
        },
        Vec::new(),
    )
    .unwrap();
    ClassificationRecord {
        identity,
        classification,
        mapping_kind: MappingKind::File,
        mapping_source: source.clone(),
        relative_path: None,
        source_path: SafePath::from_path(&source),
        destination_path: SafePath::from_path(&destination),
        source: Some(file_state('a')),
        destination: (classification != Classification::SourceAddition).then(|| file_state('b')),
        baseline: None,
        prospective_direction: Direction::None,
        changed_dimensions: ChangedDimensions {
            source_to_baseline: None,
            destination_to_baseline: None,
            source_to_destination: None,
        },
        attention: classification != Classification::Synchronized,
        blocking,
        reasons: blocking.then(|| "blocked".into()).into_iter().collect(),
    }
}

fn scope() -> ClassificationScope {
    ClassificationScope {
        kind: "all".into(),
        path_space: PathSpace::Source,
        selector: None,
        mapping_source: None,
    }
}

#[test]
fn build_covers_all_classifications_and_aggregates_every_blocker() {
    use Classification::*;
    let classifications = [
        SourceAddition,
        InitialMatch,
        InitialCollision,
        DestinationOnlyUnmanaged,
        Synchronized,
        SourceOnlyChange,
        DestinationOnlyChange,
        ConvergedTwoSidedChange,
        DivergentConflict,
        SourceSideDeletion,
        DestinationSideDeletion,
        DeleteChangeConflict,
        ChangeDeleteConflict,
        ConvergedDeletion,
        NewlyIgnoredPendingRetirement,
        UntrackedPendingRetirement,
        UnsupportedManaged,
        UnsafeCollision,
    ];
    let blocking = [2usize, 8, 11, 12, 16, 17];
    let records = classifications
        .into_iter()
        .enumerate()
        .map(|(index, classification)| record(index, classification, blocking.contains(&index)))
        .collect();
    let plan = grip::push::plan::build(scope(), records).unwrap();
    assert_eq!(plan.entries.len(), 18);
    assert_eq!(plan.blockers.len(), blocking.len());
    assert_eq!(plan.actions.len(), 2);
    for entry in &plan.entries {
        let expected = if matches!(
            entry.classification,
            Classification::SourceAddition | Classification::SourceOnlyChange
        ) {
            Disposition::Action
        } else if matches!(
            entry.classification,
            InitialCollision
                | DivergentConflict
                | DeleteChangeConflict
                | ChangeDeleteConflict
                | UnsupportedManaged
                | UnsafeCollision
        ) {
            Disposition::Blocked
        } else {
            Disposition::NoAction
        };
        assert_eq!(entry.disposition, expected, "{:?}", entry.classification);
    }
}

#[test]
fn build_is_deterministic_when_input_order_changes() {
    let mut records = vec![
        record(2, Classification::Synchronized, false),
        record(1, Classification::SourceOnlyChange, false),
        record(0, Classification::SourceAddition, false),
    ];
    let first = grip::push::plan::build(scope(), records.clone()).unwrap();
    records.reverse();
    let second = grip::push::plan::build(scope(), records).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.plan_id.len(), 64);
}

fn tree_addition(relative: Vec<u8>, kind: NodeKind) -> ClassificationRecord {
    let mapping = MappingSnapshot {
        kind: MappingKind::Tree,
        source: PathBuf::from("/tree-source"),
        destination: PathBuf::from("/tree-destination"),
    };
    let identity = EntryIdentity::new(mapping, relative).unwrap();
    let state = if kind == NodeKind::Directory {
        SupportedState::directory()
    } else {
        file_state('c')
    };
    ClassificationRecord {
        mapping_kind: MappingKind::Tree,
        mapping_source: identity.mapping.source.clone(),
        relative_path: identity.relative_safe_path(),
        source_path: SafePath::from_path(&identity.source_path()),
        destination_path: SafePath::from_path(&identity.destination_path()),
        identity,
        classification: Classification::SourceAddition,
        source: Some(state),
        destination: None,
        baseline: None,
        prospective_direction: Direction::SourceToDestination,
        changed_dimensions: ChangedDimensions {
            source_to_baseline: None,
            destination_to_baseline: None,
            source_to_destination: None,
        },
        attention: true,
        blocking: false,
        reasons: vec!["baseline_uninitialized".into()],
    }
}

#[test]
fn build_orders_parent_before_shared_children_and_uses_raw_identity_order() {
    let raw_later = vec![b'd', b'i', b'r', b'/', 0xff];
    let raw_first = b"dir/a".to_vec();
    let plan = grip::push::plan::build(
        scope(),
        vec![
            tree_addition(raw_later, NodeKind::File),
            tree_addition(b"dir".to_vec(), NodeKind::Directory),
            tree_addition(raw_first, NodeKind::File),
        ],
    )
    .unwrap();
    assert_eq!(
        plan.actions[0].kind,
        grip::push::model::ActionKind::CreateDirectory
    );
    assert_eq!(plan.actions[1].dependencies, vec![0]);
    assert_eq!(plan.actions[2].dependencies, vec![0]);
    assert!(plan.actions.iter().enumerate().all(|(index, action)| {
        action
            .dependencies
            .iter()
            .all(|dependency| *dependency < index)
    }));
    assert_eq!(
        plan.actions[1].identity.as_ref().unwrap().relative_path,
        b"dir/a"
    );
}

#[test]
fn filesystem_plan_deduplicates_synthetic_parents_and_keeps_dependencies_acyclic() {
    let root = tempfile::tempdir().unwrap();
    let mapping = MappingSnapshot {
        kind: MappingKind::Tree,
        source: root.path().join("source"),
        destination: root.path().join("destination"),
    };
    let make = |relative: &[u8]| {
        let identity = EntryIdentity::new(mapping.clone(), relative.to_vec()).unwrap();
        ClassificationRecord {
            mapping_kind: MappingKind::Tree,
            mapping_source: mapping.source.clone(),
            relative_path: identity.relative_safe_path(),
            source_path: SafePath::from_path(&identity.source_path()),
            destination_path: SafePath::from_path(&identity.destination_path()),
            identity,
            classification: Classification::SourceAddition,
            source: Some(file_state('d')),
            destination: None,
            baseline: None,
            prospective_direction: Direction::SourceToDestination,
            changed_dimensions: ChangedDimensions {
                source_to_baseline: None,
                destination_to_baseline: None,
                source_to_destination: None,
            },
            attention: true,
            blocking: false,
            reasons: vec!["baseline_uninitialized".into()],
        }
    };
    let requirements = vec![
        grip::push::plan::ParentRequirement {
            mapping: mapping.clone(),
            path: mapping.destination.clone(),
        },
        grip::push::plan::ParentRequirement {
            mapping: mapping.clone(),
            path: mapping.destination.join("shared"),
        },
        grip::push::plan::ParentRequirement {
            mapping: mapping.clone(),
            path: mapping.destination.join("shared"),
        },
    ];
    let plan = grip::push::plan::build_with_parent_requirements(
        scope(),
        vec![make(b"shared/a"), make(b"shared/b")],
        requirements,
    )
    .unwrap();
    assert_eq!(
        plan.actions
            .iter()
            .filter(|action| action.kind == grip::push::model::ActionKind::CreateParentDirectory)
            .count(),
        2
    );
    assert_eq!(plan.actions[1].dependencies, vec![0]);
    assert_eq!(plan.actions[2].dependencies, vec![1]);
    assert_eq!(plan.actions[3].dependencies, vec![1]);
    assert!(plan.actions.iter().enumerate().all(|(index, action)| {
        action
            .dependencies
            .iter()
            .all(|dependency| *dependency < index)
    }));
}
