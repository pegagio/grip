# Tasks: Contained Tree Mapping Overrides

**Input**: [spec.md](./spec.md) and [plan.md](./plan.md)

## Phase 1: Ownership Foundation

- [X] T001 Narrow contained-tree exact-leaf ownership validation in `src/mapping.rs` and `src/registry/mod.rs`

## Phase 2: User Story 1 - Exact Override (Priority: P1)

- [X] T002 [US1] Add the valid `git/ignore -> ~/.gitignore` plus `home/ -> ~/` fixture in `tests/mapping_topology_integration.rs`

## Phase 3: User Story 2 - Reject Duplicate Destination Ownership (Priority: P2)

- [X] T003 [US2] Add candidate and later-load duplicate-leaf coverage in `tests/mapping_topology_integration.rs`

## Phase 4: User Story 3 - Honest Add Output (Priority: P3)

- [X] T004 [US3] Render concise mapping success output only for successful outcomes in `src/result.rs`
- [X] T005 [US3] Add human ownership-rejection output coverage in `tests/mapping_cli_contract.rs`

## Phase 5: Validation

- [X] T006 Document exact-leaf contained-tree reservations in `README.md` and `docs/product-definition.md`
- [X] T007 Run topology and mapping CLI tests, formatting, Clippy, and cross-artifact analysis
