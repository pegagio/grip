# Tasks: Simplify Default Command Output

**Input**: Design documents from `specs/017-simplify-command-output/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [default-human-output.md](contracts/default-human-output.md), and [quickstart.md](quickstart.md)

**Tests**: Exact human-output tests are required by the feature specification and the constitution. They use the existing isolated temporary-root command fixtures.

**Organization**: Tasks are grouped by user story so each output contract can be implemented and verified independently. The shared renderer routing belongs to the first story and follows its exact-output tests.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it changes a different file and has no incomplete dependency.
- **[Story]**: Maps a task to the user story in [spec.md](spec.md).

## Phase 1: Setup

**Purpose**: No project initialization is needed. Feature 017 reuses the existing Rust CLI, typed result envelope, and temporary-root fixture support.

## Phase 2: Foundational Renderer Boundary

**Purpose**: No standalone foundational behavior is required. The output-neutral renderer scaffolding belongs to User Story 1 so its expected transcripts are recorded before implementation.

**Checkpoint**: User Story 1 supplies the shared renderer boundary after its tests define the changed output contract.

## Phase 3: User Story 1 - Act on a concise mutation result (Priority: P1) 🎯 MVP

**Goal**: Let an operator preview or complete push, pull, forced push, and mixed sync work using concise direction-correct rows instead of operation-engine evidence.

**Independent Test**: In isolated fixtures, exact default-human transcripts for action-bearing push, pull, forced push, and mixed sync results match the approved headings and arrows; JSON and mutation effects remain unchanged.

### Tests for User Story 1

- [X] T001 [P] [US1] Add exact concise preview, apply, forced-push, and unchanged-JSON assertions in tests/push_cli_contract.rs.
- [X] T002 [P] [US1] Add exact concise pull preview/apply assertions and retained no-action or blocked regression assertions in tests/pull_cli_contract.rs.
- [X] T003 [P] [US1] Add exact mixed-direction sync preview/apply assertions with one heading and per-row arrows in tests/sync_cli_contract.rs.

### Implementation for User Story 1

- [X] T004 [US1] Add action-bearing successful-result routing and renderer-local projection helpers in src/result.rs, preserving current blocked, failed, no-action, JSON, and `diff` paths.
- [X] T005 [US1] Implement concise push, pull, resolution, and sync headings plus per-action source-left directional rows in src/result.rs, omitting internal action, recovery, verification, baseline, and operation-record evidence only for action-bearing successes.
- [X] T006 [US1] Run the User Story 1 output suites recorded in tests/push_cli_contract.rs, tests/pull_cli_contract.rs, and tests/sync_cli_contract.rs and correct transcript mismatches.

**Checkpoint**: Push, pull, forced push, and mixed sync action results are independently usable from their first line and each affected row.

## Phase 4: User Story 2 - Read mapping and status output without repeated detail (Priority: P2)

**Goal**: Present declared mappings and status changes in the approved concise forms, including the amended status order and ordinary-conflict row.

**Independent Test**: Add/list/select/remove and mixed-status fixtures produce exact declared-path and section-order transcripts without resolved rows or a generic ordinary-conflict explanation.

### Tests for User Story 2

- [X] T007 [P] [US2] Add exact add, all-list, selected-list, and remove human-output assertions while retaining mapping JSON assertions in tests/mapping_cli_contract.rs.
- [X] T008 [P] [US2] Add exact status section-order, ordinary-divergent-conflict, CWD-relative-path, and retained-technical-blocker assertions in tests/classification_cli_contract.rs.

### Implementation for User Story 2

- [X] T009 [US2] Render declared mapping rows without resolved endpoints and reorder status sections while suppressing only the ordinary divergent-conflict explanation in src/result.rs.
- [X] T010 [US2] Run the User Story 2 contract suites in tests/mapping_cli_contract.rs and tests/classification_cli_contract.rs and correct transcript mismatches.

**Checkpoint**: Mapping and status output are independently readable, Feature 015 safety blockers remain visible, and Feature 016 CWD-relative source labels remain intact.

## Phase 5: User Story 3 - Receive clear errors without changing contracts (Priority: P3)

**Goal**: Prefix human parser and domain errors consistently while retaining exit codes, usage content, detailed diagnostics, and structured automation contracts.

**Independent Test**: A missing argument and invalid absolute source selector begin `Error:` but retain their expected exit categories; exact `diff` and JSON regression checks continue to pass.

### Tests for User Story 3

- [X] T011 [P] [US3] Add parser-error and domain-error `Error:` prefix assertions with unchanged usage and exit categories in tests/project_independent_cli_contract.rs.
- [X] T012 [US3] Add explicit unchanged-human-`diff` and unchanged-JSON-envelope regression assertions in tests/classification_cli_contract.rs and tests/push_cli_contract.rs.

### Implementation for User Story 3

- [X] T013 [US3] Normalize parser-error output in src/lib.rs and domain-error output in src/result.rs without changing JSON rendering, error categories, or message bodies after the prefix.
- [X] T014 [US3] Run the User Story 3 regression suites in tests/project_independent_cli_contract.rs, tests/classification_cli_contract.rs, and tests/push_cli_contract.rs and correct contract regressions.

**Checkpoint**: Human errors are visually consistent while parser behavior, JSON, `diff`, and safety outcomes retain their prior contracts.

## Phase 6: Polish and Cross-Cutting Validation

**Purpose**: Document the operator-facing output and validate the whole feature against the approved contract.

- [X] T015 [P] Update concise mapping, status, mutation, and unchanged-`diff` examples in README.md and docs/product-definition.md.
- [X] T016 Reconcile the implementation and exact scenarios against specs/017-simplify-command-output/contracts/default-human-output.md and specs/017-simplify-command-output/updated-output.md.
- [X] T017 Run the focused command-output validation documented in specs/017-simplify-command-output/quickstart.md.
- [X] T018 Run the full format, lint, test, and release-build gate from mise.toml with `mise run validate`.

## Dependencies and Execution Order

### Phase Dependencies

- **Phase 1**: No implementation work; the existing project structure is ready.
- **Phase 2**: No standalone task; User Story 1 establishes the shared renderer boundary after its contract tests.
- **User Story 1**: T001–T003 may be prepared in parallel; T004 follows those expected-output tests, T005 implements the concise output, and T006 verifies the completed slice.
- **User Story 2**: T007–T008 may be prepared in parallel after T004; T009 follows those expected-output tests; T010 verifies the completed slice.
- **User Story 3**: T011 may be prepared independently after the affected renderer paths are stable. T012 follows T001 and T008 because it edits their test files; T013 follows both tests; T014 verifies the completed slice.
- **Polish**: T015 can proceed after its associated output slices; T016–T018 follow all implementation and contract-test tasks.
- **Conflict Resolution Guidance**: T022 and T023 define the revised contract in separate test files; T024 follows T022, T025 follows T024 and T023 because both adjust the outcome renderer, T026 follows the implementation, and T027 verifies the full slice.

### User Story Dependencies

- **US1 (P1)**: Begins with its contract tests and is the MVP. T004 establishes the shared renderer boundary before T005 changes concise mutation output.
- **US2 (P2)**: Depends on T004. It shares src/result.rs with US1, so T009 follows T005 in a single working tree; its acceptance tests remain independently runnable.
- **US3 (P3)**: Depends on the stable result renderer from US1 and US2 because it changes the same human-rendering boundary; its parser path in src/lib.rs is otherwise independent.

## Parallel Opportunities

- T001, T002, and T003 edit distinct mutation contract-test files and can run in parallel.
- T007 and T008 edit distinct mapping and status contract-test files and can run in parallel.
- T011 can proceed independently once the affected renderer paths are stable. T012 follows T001 and T008 because it shares both test files.
- T015 can be drafted in parallel with late regression work because it edits documentation only.
- T022 and T023 edit distinct contract-test files and can be prepared in parallel.

## Implementation Strategy

### MVP First

1. Complete T001–T006.
2. Run the User Story 1 tests and demonstrate concise push, pull, forced-push, and sync output.
3. Confirm JSON and filesystem outcomes are unchanged before progressing.

### Incremental Delivery

1. Add mapping and status simplification through T007–T010.
2. Add the uniform error prefix and regression boundary through T011–T014.
3. Complete documentation and the full validation gate through T015–T018.
4. Add concrete conflict-resolution guidance through T022–T027, then rerun the full validation gate.

## Phase 7: Convergence

**Purpose**: Close the remaining exact-output contract-test coverage gaps found after implementation.

- [X] T019 Add explicit absence assertions for internal action, recovery, verification, durability, baseline, and operation-record evidence in concise mutation transcripts in tests/push_cli_contract.rs, tests/pull_cli_contract.rs, and tests/sync_cli_contract.rs per SC-002 (partial).
- [X] T020 Add a five-mapping unselected-list transcript with one declared row per mapping and no resolved rows in tests/mapping_cli_contract.rs per SC-003 (partial).
- [X] T021 Add an exact completed mixed-sync transcript asserting both direction-correct rows in tests/sync_cli_contract.rs per SC-001 (partial).

## Phase 8: Conflict Resolution Guidance

**Purpose**: Turn force-resolvable conflict output into concrete next actions without weakening existing safety or automation contracts.

**Independent Test**: In isolated fixtures, initial-collision and ordinary-divergent status output, plus blocked human push and pull output, contain valid source-winning and destination-winning commands. Mixed or technical blockers retain their existing safety detail without invented commands, and JSON/exit behavior remains unchanged.

### Tests for conflict resolution guidance

- [X] T022 [P] [US2] Add exact initial-collision and ordinary-divergent status guidance assertions, including retained technical-blocker behavior, in tests/classification_cli_contract.rs.
- [X] T023 [P] [US3] Add exact blocked human push and pull guidance assertions with CWD-valid source and destination selectors while retaining JSON and exit assertions in tests/push_cli_contract.rs and tests/pull_cli_contract.rs.

### Implementation for conflict resolution guidance

- [X] T024 [US2] Add a private human-only force-resolution selector projection and render status guidance only for force-resolvable conflicts in src/result.rs.
- [X] T025 [US3] Attach CWD-relative source and destination selectors to blocked push and pull outcomes without changing JSON details or planning semantics in src/lib.rs and src/result.rs.
- [X] T026 Update conflict-resolution examples and explanatory documentation in README.md, docs/product-definition.md, specs/017-simplify-command-output/contracts/default-human-output.md, and specs/017-simplify-command-output/updated-output.md.
- [X] T027 Run the focused conflict-output suites and `mise run validate`, reconciling results with specs/017-simplify-command-output/quickstart.md.

**Checkpoint**: A human can choose either safe, explicit force path for a resolvable conflict; non-force-resolvable blockers remain safely detailed.

## Phase 9: Blocked Output Evidence Boundary

**Purpose**: Remove baseline authority evidence from ordinary human blocked push and pull output while retaining the structured contract.

**Independent Test**: Exact blocked push and pull transcripts show the blocker and force choices but no `Baseline` line; their JSON baseline values remain present and unchanged.

- [X] T028 [P] [US3] Update blocked push and pull transcript assertions to exclude baseline authority while retaining JSON baseline assertions in tests/push_cli_contract.rs and tests/pull_cli_contract.rs.
- [X] T029 [US3] Suppress baseline outcome and generation rendering only for default human blocked push and pull outcomes in src/result.rs.
- [X] T030 Update the blocked-output contract, approved transcript, quickstart, and user-facing documentation in README.md, docs/product-definition.md, specs/017-simplify-command-output/contracts/default-human-output.md, specs/017-simplify-command-output/updated-output.md, and specs/017-simplify-command-output/quickstart.md.
- [X] T031 Run the focused blocked-output tests and `mise run validate`, reconciling the results with FR-018 and SC-007 in specs/017-simplify-command-output/spec.md.

## Phase 10: Terminal Mutation Output Convergence

**Purpose**: Remove the remaining internal execution detail from default human no-action, blocked, and failed mutation output found by the full command-and-error audit.

**Independent Test**: Isolated fixtures prove that no-action, conflict-only blocked, technical blocked, forced-resolution blocked, and partial-failure mutation paths give an operator a concise next step while JSON, exit categories, mutation effects, and `diff` remain unchanged.

- [X] T032 [P] [US3] Add exact default-human no-action, baseline-only, and conflict-only blocked transcripts for push, pull, sync, and forced directional resolution in tests/push_cli_contract.rs, tests/pull_cli_contract.rs, and tests/sync_cli_contract.rs; retain JSON and exit assertions.
- [X] T033 [P] [US3] Add default-human technical/mixed-blocker and partial-failure assertions that exclude raw blocker IDs, winner, milestone, recovery, baseline, and operation-record evidence in the relevant mutation contract tests.
- [X] T034 [US3] Extend src/lib.rs human-only force-resolution projections to sync and blocked forced directional plans without changing selection, planning, execution, or JSON details.
- [X] T035 [US3] Replace the generic default-human mutation fallback in src/result.rs with terminal projections for no-action, blocked conflict, blocked technical/mixed, and failed outcomes; map internal resolution labels to public push or pull directions and preserve JSON/detailed output.
- [X] T036 [US3] Reconcile README.md, docs/product-definition.md, specs/017-simplify-command-output/contracts/default-human-output.md, specs/017-simplify-command-output/updated-output.md, and specs/017-simplify-command-output/quickstart.md with the implemented terminal transcripts.
- [X] T037 [US3] Run focused terminal-output contracts, `cargo test --all --no-fail-fast`, and `mise run validate`; reconcile results with FR-019 through FR-023 and SC-008.
