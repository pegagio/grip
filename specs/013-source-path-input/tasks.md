# Tasks: Source Path Input Normalization

**Input**: Design documents from [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), and [contracts/source-paths.md](contracts/source-paths.md)

**Tests**: Required by the specification’s automated acceptance criteria. Add focused tests before the associated implementation, then run the complete validation gate.

**Organization**: Tasks are grouped by user story so P1 mapping creation can be completed and verified before P2 source-space selection work begins.

## Phase 1: Setup

**Purpose**: No project setup is required. The feature uses the existing Rust CLI, dependency set, and isolated `ProjectFixture` test support.

## Phase 2: Foundational

**Purpose**: Establish the one shared command-input normalization boundary before either user story consumes it.

- [X] T001 Implement a command-input source parser that lexically normalizes accepted relative forms, rejects root escapes and normalized `.grip` results, and retains strict stored-declaration parsing in `src/path_policy.rs`

**Checkpoint**: Source-space commands have one safe normalization boundary while descriptor decoding remains strict.

## Phase 3: User Story 1 - Add a Mapping with an Ordinary Relative Source Path (Priority: P1) 🎯 MVP

**Goal**: Operators can create file or tree mappings with `./app/` and Grip persists only the normalized source declaration.

**Independent Test**: In an initialized fixture with an `app` directory, `add ./app/` succeeds with a valid destination, records `app`, and rejected paths leave descriptor bytes unchanged.

- [X] T002 [US1] Add failing model coverage for accepted CLI source normalization, root-only tree behavior, and continued rejection of noncanonical stored declarations in `tests/portable_mapping_model.rs`
- [X] T003 [US1] Add a CLI-specific portable-mapping construction path that uses normalized source input while preserving strict declaration construction in `src/mapping.rs`
- [X] T004 [US1] Add failing integration coverage for `grip add ./app/`, normalized descriptor persistence, and rejected escaping or `.grip` source inputs in `tests/portable_mapping_integration.rs`
- [X] T005 [US1] Route `grip add` source resolution and declaration publication through the CLI source parser in `src/lib.rs`

**Checkpoint**: `grip add ./app/ DESTINATION` works as an independently testable MVP and persists `app`.

## Phase 4: User Story 2 - Reuse Relative Source Spellings in Source-Space Commands (Priority: P2)

**Goal**: Operators can select the same mapping or source-side scope using accepted relative spellings after it is declared.

**Independent Test**: After declaring `app`, source-space `list`, `remove`, `status`, `diff`, dry-run `push`, dry-run `pull`, and dry-run `sync` with `./app/` resolve the same mapping or scope as `app`.

- [X] T006 [US2] Add failing source-space selector coverage for `./app/` across mapping lookup and operation commands in `tests/portable_mapping_integration.rs`
- [X] T007 [US2] Route source-space selector resolution plus `list` and `remove` through the shared CLI source parser without changing destination-space parsing in `src/lib.rs`
- [X] T008 [US2] Extend reserved-metadata regression coverage to prove normalized `.grip` results are rejected without descriptor mutation in `tests/project_reserved_metadata_integration.rs`

**Checkpoint**: Source-space commands consistently normalize accepted spelling and retain all project-boundary protections.

## Phase 5: Polish and Cross-Cutting Concerns

**Purpose**: Align user documentation and validate the complete feature boundary.

- [X] T009 [P] Update source-input and persisted-declaration guidance in `README.md` and `docs/product-definition.md`
- [X] T010 Verify every task against [quickstart.md](quickstart.md), run focused source-path tests, and run `mise run validate` from `specs/013-source-path-input/quickstart.md`

## Dependencies and Execution Order

- **T001** blocks both user stories because it establishes the shared normalization boundary.
- **US1**: T002 → T003 → T004 → T005. It is the MVP and must be complete before selector behavior is extended.
- **US2**: T006 → T007 → T008. It depends on the normalized mapping-creation path from US1 but is independently verifiable after that.
- **Polish**: T009 can proceed after the public contract is stable; T010 runs after all implementation and documentation tasks.

## Parallel Opportunities

- T009 can run in parallel with the final regression work because it modifies only documentation.
- Once T001 is complete, test preparation for US1 can begin while the strict-to-CLI mapping construction boundary is being finalized, but T005 must wait for T003 and T004.
- US2 test preparation can begin after US1’s persisted declaration contract is established; its production change remains sequenced after T005 because both changes touch `src/lib.rs`.

## Implementation Strategy

### MVP First

1. Complete T001 through T005.
2. Verify `grip add ./app/ DESTINATION` persists `app` and preserves rejection boundaries.
3. Stop for review before extending selector behavior.

### Incremental Delivery

1. Deliver P1 mapping creation with strict descriptor compatibility intact.
2. Deliver P2 consistent source-space selection.
3. Update documentation and validate the complete repository gate.

## Format Validation

All actionable tasks use the required checklist format with sequential IDs, user-story labels where applicable, and exact file paths.

## Phase 6: Convergence

- [X] T011 Add direct CLI-parser coverage for repeated separators and non-UTF-8 source input per FR-005 and the Edge Cases in `tests/portable_mapping_model.rs` (partial)
