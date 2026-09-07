mod support;

use grip::classification::model::Classification;
use grip::mutation::model::{Disposition, MutationDirection};

#[test]
fn pull_direction_table_covers_every_inherited_classification() {
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
    for classification in classifications {
        let expected = if classification == DestinationOnlyChange {
            Disposition::Action
        } else {
            Disposition::NoAction
        };
        assert_eq!(
            grip::mutation::plan::disposition_for_direction(
                MutationDirection::Pull,
                classification,
                false,
            ),
            expected,
            "{classification:?}"
        );
        assert_eq!(
            grip::mutation::plan::disposition_for_direction(
                MutationDirection::Pull,
                classification,
                true,
            ),
            Disposition::Blocked,
            "{classification:?}"
        );
    }
}

#[test]
fn pull_plan_is_direction_bound_and_deterministic() {
    let (_root, home, registry, state, selection, first) = support::pull_execution_fixture();
    let observed =
        grip::observation::inspect(&home, &registry, &state.accepted, &selection).unwrap();
    let records = observed
        .values()
        .map(|entry| {
            grip::classification::classify(entry, state.accepted.baselines.get(&entry.identity))
        })
        .collect::<Vec<_>>();
    let mut reversed = records.clone();
    reversed.reverse();
    let second =
        grip::mutation::plan::build_for(MutationDirection::Pull, first.scope.clone(), reversed)
            .unwrap();
    assert_eq!(first, second);
    assert_eq!(first.direction, Some(MutationDirection::Pull));
    assert_eq!(first.actions.len(), 1);
    assert_eq!(
        first.actions[0].destination,
        first.actions[0].identity.as_ref().unwrap().source_path()
    );
}
