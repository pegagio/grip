#![allow(clippy::result_large_err)]

mod support;

use grip::classification;
use grip::classification::model::ClassificationScope;
use grip::observation::model::{EntryIdentity, PathSpace, ResolvedMapping, Selection};
use std::fs;
use support::project::ContainedHomeFixture;

#[test]
fn unsafe_sibling_introduced_before_action_revalidation_blocks_without_mutation() {
    let fixture = ContainedHomeFixture::initialized();
    fs::write(fixture.source_root.join(".bashrc"), "safe\n").unwrap();
    assert!(fixture.command(&["add", "home/", "~/"]).status.success());

    let home = grip::project::ProjectPaths::project_metadata(
        fixture.project_root.join(".grip"),
        fixture.home_root.clone(),
    );
    let registry = grip::registry::publication::load(&home, false).unwrap();
    let state = grip::state::publication::load(&home).unwrap();
    let mapping = registry.registry.mappings().first().unwrap();
    let identity = EntryIdentity::new(ResolvedMapping::from(mapping), b".bashrc".to_vec()).unwrap();
    let selection = Selection::Entry(identity);
    let observed =
        grip::observation::inspect(&home, &registry, &state.accepted, &selection).unwrap();
    let records = observed
        .values()
        .map(|entry| classification::classify_accepted(entry, &state.accepted))
        .collect();
    let plan = grip::mutation::plan::build_with_parent_requirements(
        ClassificationScope {
            kind: "entry".into(),
            path_space: PathSpace::Source,
            selector: Some(grip::discovery::model::SafePath::from_path(
                &fixture.source_root.join(".bashrc"),
            )),
            mapping_source: Some(mapping.source.display().to_string()),
        },
        records,
        registry.missing_destination_parents(),
    )
    .unwrap();
    assert_eq!(plan.counts.actionable, 1);

    let unsafe_root = fixture.source_root.join("grip-project/home");
    let error = grip::mutation::execution::execute_with_fault_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        |phase| {
            if phase == grip::mutation::FaultPhase::BeforeActionRevalidation(0) {
                fs::create_dir_all(&unsafe_root).unwrap();
                fs::write(unsafe_root.join("unsafe"), "unsafe\n").unwrap();
            }
            Ok(())
        },
    )
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("recursive member topology appeared")
    );
    assert!(!fixture.home_root.join(".bashrc").exists());
}

#[test]
fn unsafe_sibling_introduced_before_deletion_revalidation_preserves_the_target() {
    let fixture = ContainedHomeFixture::initialized();
    let source = fixture.source_root.join(".bashrc");
    fs::write(&source, "accepted\n").unwrap();
    assert!(fixture.command(&["add", "home/", "~/"]).status.success());
    assert!(fixture.command(&["push"]).status.success());
    fs::remove_file(&source).unwrap();

    let home = grip::project::ProjectPaths::project_metadata(
        fixture.project_root.join(".grip"),
        fixture.home_root.clone(),
    );
    let registry = grip::registry::publication::load(&home, false).unwrap();
    let state = grip::state::publication::load(&home).unwrap();
    let mapping = registry.registry.mappings().first().unwrap();
    let identity = EntryIdentity::new(ResolvedMapping::from(mapping), b".bashrc".to_vec()).unwrap();
    let selection = Selection::Entry(identity);
    let observed =
        grip::observation::inspect(&home, &registry, &state.accepted, &selection).unwrap();
    let mut records = observed
        .values()
        .map(|entry| classification::classify_accepted(entry, &state.accepted))
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 1);
    records[0].blocking = false;
    let plan = grip::delete::plan::build(
        grip::delete::model::DeletionAuthority::Source,
        ClassificationScope {
            kind: "entry".into(),
            path_space: PathSpace::Source,
            selector: Some(grip::discovery::model::SafePath::from_path(&source)),
            mapping_source: Some(mapping.source.display().to_string()),
        },
        records,
    )
    .unwrap();

    let unsafe_root = fixture.source_root.join("grip-project/home");
    let mut hook = |_index: usize| {
        fs::create_dir_all(&unsafe_root).unwrap();
        fs::write(unsafe_root.join("unsafe"), "unsafe\n").unwrap();
    };
    let error = grip::delete::execution::execute_with_hook(
        &home,
        &registry,
        &state,
        &selection,
        &plan,
        None,
        Some(&mut hook),
    )
    .unwrap_err();

    assert!(error.to_string().contains("deletion revalidation"));
    assert_eq!(
        fs::read(fixture.home_root.join(".bashrc")).unwrap(),
        b"accepted\n"
    );
}
