mod support;

use grip::classification::model::ClassificationScope;
use grip::delete::model::DeletionAuthority;
use grip::observation::model::{PathSpace, Selection};
use std::fs;

fn plans() -> (
    grip::delete::model::DeletionPlan,
    grip::delete::model::DeletionPlan,
) {
    let (root, grip_home, source, _) = support::accepted_file_fixture();
    fs::remove_file(&source).unwrap();
    let home = grip::home::select(Some(grip_home.into_os_string()), None).unwrap();
    let registry = grip::registry::publication::load(&home, false).unwrap();
    let state = grip::state::publication::load(&home).unwrap();
    let observed =
        grip::observation::inspect(&home, &registry, &state.accepted, &Selection::All).unwrap();
    let records: Vec<_> = observed
        .values()
        .map(|entry| {
            grip::classification::classify(entry, state.accepted.baselines.get(&entry.identity))
        })
        .collect();
    let scope = ClassificationScope {
        kind: "all".into(),
        path_space: PathSpace::Source,
        selector: None,
        mapping_source: None,
    };
    let first =
        grip::delete::plan::build(DeletionAuthority::Source, scope.clone(), records.clone())
            .unwrap();
    let second = grip::delete::plan::build(DeletionAuthority::Source, scope, records).unwrap();
    drop(root);
    (first, second)
}

#[test]
fn deletion_authority_is_direction_bound_and_deterministic() {
    let (first, second) = plans();
    assert_eq!(first.plan_id, second.plan_id);
    assert_eq!(first.actions.len(), 1);
    assert_eq!(first.actions[0].target_side, "destination");
}
