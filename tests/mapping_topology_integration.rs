use grip::mapping::{Mapping, MappingKind, validate_ownership};
use std::path::PathBuf;

fn mapping(kind: MappingKind, source: &str, destination: &str) -> Mapping {
    Mapping::new(kind, PathBuf::from(source), PathBuf::from(destination))
}

#[test]
fn rejects_equal_nested_overlapping_and_cross_side_ownership() {
    let cases = [
        vec![mapping(MappingKind::File, "/source/a", "/source/a")],
        vec![mapping(MappingKind::Tree, "/source/a", "/source/a/child")],
        vec![
            mapping(MappingKind::Tree, "/source/a", "/destination/a"),
            mapping(MappingKind::File, "/source/a/child", "/destination/b"),
        ],
        vec![
            mapping(MappingKind::Tree, "/source/a", "/destination/a"),
            mapping(MappingKind::Tree, "/destination", "/backup"),
        ],
    ];
    for mappings in cases {
        assert!(!validate_ownership(&mappings).is_empty(), "{mappings:?}");
    }
}

#[test]
fn permits_disjoint_component_boundary_namespaces() {
    let mappings = vec![
        mapping(MappingKind::Tree, "/source/a", "/destination/a"),
        mapping(MappingKind::Tree, "/source/ab", "/destination/ab"),
    ];
    assert!(validate_ownership(&mappings).is_empty());
}

#[test]
fn conflict_order_is_deterministic() {
    let mappings = vec![
        mapping(MappingKind::Tree, "/source/a", "/destination/a"),
        mapping(MappingKind::File, "/source/a/y", "/destination/a/y"),
        mapping(MappingKind::File, "/source/a/x", "/destination/a/x"),
    ];
    let first = validate_ownership(&mappings);
    let second = validate_ownership(&mappings);
    assert_eq!(first, second);
    assert!(first.windows(2).all(|pair| pair[0] <= pair[1]));
}

#[test]
fn rejection_matrix_covers_duplicate_source_destination_and_cross_relations() {
    let cases = [
        (
            vec![
                mapping(MappingKind::File, "/source/a", "/destination/a"),
                mapping(MappingKind::File, "/source/a", "/destination/a"),
            ],
            "duplicate_tuple",
        ),
        (
            vec![
                mapping(MappingKind::File, "/source/a", "/destination/a"),
                mapping(MappingKind::File, "/source/a", "/destination/b"),
            ],
            "duplicate_source",
        ),
        (
            vec![
                mapping(MappingKind::Tree, "/source/a", "/destination/a"),
                mapping(MappingKind::File, "/source/b", "/destination/a/child"),
            ],
            "destination_overlap",
        ),
        (
            vec![
                mapping(MappingKind::File, "/source/a", "/destination/a"),
                mapping(MappingKind::Tree, "/destination", "/backup"),
            ],
            "cross_mapping_recursion",
        ),
    ];
    for (mappings, expected) in cases {
        assert!(
            validate_ownership(&mappings)
                .iter()
                .any(|conflict| conflict.reason == expected),
            "missing {expected} in {mappings:?}"
        );
    }
}

#[test]
fn containment_matrix_rejects_both_operand_orders_and_mapping_kind_combinations() {
    for nested_kind in [MappingKind::File, MappingKind::Tree] {
        let source_tree = mapping(MappingKind::Tree, "/source/root", "/destination/a");
        let nested_source = mapping(nested_kind, "/source/root/child", "/destination/b");
        let destination_tree = mapping(MappingKind::Tree, "/source/a", "/destination/root");
        let nested_destination = mapping(nested_kind, "/source/b", "/destination/root/child");
        for mappings in [
            vec![source_tree.clone(), nested_source.clone()],
            vec![nested_source.clone(), source_tree.clone()],
        ] {
            assert!(
                validate_ownership(&mappings)
                    .iter()
                    .any(|conflict| conflict.reason == "source_overlap")
            );
        }
        for mappings in [
            vec![destination_tree.clone(), nested_destination.clone()],
            vec![nested_destination.clone(), destination_tree.clone()],
        ] {
            assert!(
                validate_ownership(&mappings)
                    .iter()
                    .any(|conflict| conflict.reason == "destination_overlap")
            );
        }
    }
}

#[test]
fn cross_mapping_recursion_is_rejected_in_both_operand_orders() {
    let first = mapping(MappingKind::Tree, "/source/root", "/destination/root");
    let second = mapping(MappingKind::Tree, "/destination/root/child", "/backup/root");
    for mappings in [vec![first.clone(), second.clone()], vec![second, first]] {
        assert!(
            validate_ownership(&mappings)
                .iter()
                .any(|conflict| conflict.reason == "cross_mapping_recursion")
        );
    }
}
