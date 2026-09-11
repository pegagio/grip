---

description: "Actionable implementation tasks for Feature 012 destination path forms"
---

# Tasks: Destination Path Forms

**Input**: Design documents from `specs/012-destination-path-forms/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), and [destination-paths.md](contracts/destination-paths.md)

**Tests**: Required. The specification and constitution require isolated filesystem coverage for all path-form and mapping-publication behavior.

**Organization**: Tasks are grouped by user story so each completed phase yields an independently testable increment.

## Format

Each task uses `- [ ] T### [P?] [US#?] Description with file path`. `[P]` marks work that can proceed in parallel after its stated prerequisites are complete.

## Phase 1: Setup

This phase establishes reusable isolated-fixture support and the test matrix before production behavior changes.

- [X] T001 [P] Add reusable absolute and home-relative destination fixture helpers in `tests/support/project.rs`
- [X] T002 [P] Add focused accepted and rejected destination declaration cases in `tests/portable_mapping_model.rs`

## Phase 2: Foundational Destination Declaration Support

This phase is blocking. It introduces a preserved declaration type and carries it through resolution, registry loading, publication, and state identity without weakening existing safety checks.

- [X] T003 Implement the destination declaration type, lexical operational-path normalization, and precise invalid-form diagnostics in `src/path_policy.rs`
- [X] T004 Replace `HomeRelativePath` usage with the destination declaration type and preserve declaration text during mapping resolution in `src/mapping.rs`
- [X] T005 Update descriptor loading and resolved-registry construction to retain original destination declarations in `src/registry/mod.rs`
- [X] T006 Update state identity and rebinding to carry original declarations rather than recreate home-relative paths in `src/state/mod.rs`
- [X] T007 Add descriptor round-trip coverage for absolute and lexically non-normalized home-relative declarations in `tests/registry_integration.rs`

**Checkpoint**: The domain can parse, persist, reload, and safely resolve every authorized destination form; existing source-path semantics remain unchanged.

## Phase 3: User Story 1 - Add a User-Relative Destination (Priority: P1) 🎯 MVP

**Goal**: An operator can add `~` and any `~/`-prefixed destination, including lexical `.`, `..`, and repeated separators, without pre-normalizing the input.

**Independent Test**: In an isolated project/home fixture, add a source to a non-normalized `~/` destination, then reload with `list`, inspect with `status`, and verify the descriptor retains the original spelling.

- [X] T008 [P] [US1] Add non-normalized `~/` add, list, status, destination-selector, and exact-descriptor-persistence tests in `tests/portable_mapping_integration.rs`
- [X] T009 [P] [US1] Add flat `grip add` acceptance coverage for `~` and non-normalized `~/` destinations without payload copying in `tests/mapping_cli_contract.rs`
- [X] T010 [US1] Publish preserved home and home-relative declarations directly instead of reconstructing them from runtime paths in `src/registry/publication.rs`
- [X] T011 [US1] Route `add` and existing destination-space selectors through the generalized declaration resolver in `src/lib.rs`
- [X] T012 [US1] Add equivalent-resolved-destination topology conflict coverage for `~/a/../b` and `~/b` in `tests/mapping_topology_integration.rs`

**Checkpoint**: `~` and non-normalized `~/` mappings are accepted, stored verbatim, reloadable, selectable, and still subject to endpoint and ownership safety checks.

## Phase 4: User Story 2 - Add an Absolute Destination (Priority: P1)

**Goal**: An operator can add and later use an absolute destination without the legacy home-only validation or publication failure.

**Independent Test**: In an isolated fixture, add a source to an absolute temporary-root destination and verify exact descriptor persistence plus successful reload and destination-space selection.

- [X] T013 [P] [US2] Add absolute-destination add, reload, list, status, and destination-selector coverage in `tests/portable_mapping_integration.rs`
- [X] T014 [P] [US2] Add flat `grip add` absolute-destination acceptance and no-copy coverage in `tests/mapping_cli_contract.rs`
- [X] T015 [US2] Extend declaration-aware publication and state binding for absolute destinations without home-prefix reconstruction in `src/registry/publication.rs` and `src/state/mod.rs`
- [X] T016 [US2] Add absolute-versus-home-relative resolved-overlap coverage in `tests/mapping_topology_integration.rs`

**Checkpoint**: Absolute mappings persist exactly, reload safely, remain usable by public destination-aware commands, and conflict correctly with equivalent resolved destinations.

## Phase 5: User Story 3 - Reject an Ambiguous Destination (Priority: P2)

**Goal**: An operator receives a precise failure for a relative or malformed destination, while project-relative source behavior remains intact.

**Independent Test**: Attempt `grip add README.md destination/README.md` in an isolated project and verify failure, an allowed-forms diagnostic, and unchanged descriptor bytes.

- [X] T017 [P] [US3] Add relative, current-directory, parent-directory, other-user-tilde, environment-expansion, and no-descriptor-mutation CLI cases in `tests/mapping_cli_contract.rs`
- [X] T018 [US3] Finalize destination-form diagnostics and keep project-relative source validation unchanged in `src/path_policy.rs` and `src/lib.rs`
- [X] T019 [US3] Extend source-relative persistence regression coverage alongside rejected destination cases in `tests/portable_mapping_integration.rs`

**Checkpoint**: Invalid destination forms fail before publication with actionable diagnostics, and source paths continue to be project-relative.

## Phase 6: Polish and Cross-Cutting Validation

This phase brings user-facing contracts, focused tests, and the repository validation gate into agreement.

- [X] T020 [P] Update destination forms, portability tradeoffs, and `add` examples in `docs/product-definition.md`
- [X] T021 [P] Update CLI examples and remove home-only or absolute-rejected guidance in `README.md`
- [X] T022 Run the focused destination declaration, registry, mapping CLI, mapping integration, and topology suites from `specs/012-destination-path-forms/quickstart.md`
- [X] T023 Run the full formatting, lint, test, and release-build gate with `mise.toml`

## Dependencies and Execution Order

- **Phase 1 → Phase 2**: Fixture and test matrix work informs the shared declaration design.
- **Phase 2 → US1, US2, US3**: The new declaration type, resolution boundary, and state/descriptor handling are prerequisites for every story.
- **US1 → US2**: US2 builds on declaration-aware publication proven by US1 and adds the absolute branch without a second persistence design.
- **US1 and US2 → US3**: Rejection tests validate the final parser and diagnostic boundary after accepted forms are complete.
- **All user stories → Phase 6**: Documentation and full validation occur only after the final public behavior is present.

## Parallel Opportunities

- **Phase 1**: T001 and T002 can proceed in parallel.
- **US1**: T008 and T009 can be written in parallel; T012 can proceed after T011.
- **US2**: T013 and T014 can be written in parallel; T016 can proceed after T015.
- **US3**: T017 can be written while T018 is implemented; T019 follows the final parser behavior.
- **Polish**: T020 and T021 can proceed in parallel; T022 and T023 are sequential validation gates.

## Implementation Strategy

### MVP First

1. Complete Phases 1 and 2.
2. Complete User Story 1 through T012.
3. Run the User Story 1 focused tests and confirm preserved `~` and non-normalized `~/` declarations work without copying payloads.

### Incremental Delivery

1. Add home-relative declaration support and verify it independently.
2. Add absolute destination support through the same declaration, publication, and state model.
3. Complete rejection diagnostics and source-regression coverage.
4. Update docs and run the full validation gate.

## Phase 7: Convergence

- [X] T024 Add isolated CLI acceptance and persistence coverage for a standalone `~` destination per FR-002 and quickstart scenario 3 (partial)
