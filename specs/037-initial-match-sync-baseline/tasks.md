---

description: "Task list for Feature 037 initial-match baseline synchronization"
---

# Tasks: Initial-Match Baseline Synchronization

**Input**: Design documents from `specs/037-initial-match-sync-baseline/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [sync contract](contracts/sync-baseline-acceptance.md), and [quickstart.md](quickstart.md)

**Tests**: Required by the specification and Constitution V. Add focused planning, CLI, filesystem, dry-run, scope, and safety regressions before changing behavior.

**Organization**: Tasks are grouped by user story. The P1 change is intentionally small but must preserve exact selection and existing accepted-state publication safeguards.

## Phase 1: Setup

**Purpose**: Establish the focused feature contract and executable test boundary.

- [X] T001 Reconcile Feature 037’s exact-single-entry acceptance rule with existing sync planning tests in `src/mutation/plan.rs` and `tests/sync_filesystem_integration.rs`.
- [X] T002 [P] Record the selected-sync initial-match contract in `README.md` and `docs/product-definition.md`.

## Phase 2: Foundational

**Purpose**: Define the scope predicate that prevents a selected-child repair from becoming broad tree acceptance.

- [X] T003 Add failing scope-policy coverage for an exact file mapping, exact tree child, tree root, no-selector scope, and multi-entry subtree in `src/mutation/plan.rs`.
- [X] T004 Add a validated exact-single-entry eligibility input from sync orchestration through sync plan construction in `src/lib.rs` and `src/mutation/plan.rs`.

**Checkpoint**: Sync planning can distinguish one resolved managed entry from mapping-wide or multi-entry scopes without changing payload action policy.

## Phase 3: User Story 1 - Accept an Equal Tree Member (Priority: P1)

**Goal**: Let one exact equivalent unbaselined file-mapping entry or tree child become current through ordinary sync without a payload copy.

**Independent Test**: An isolated unbaselined tree mapping with two equivalent children accepts only the selected child; its endpoints remain unchanged and its sibling remains under `Needs baseline`.

### Tests for User Story 1

- [X] T005 [P] [US1] Add dry-run and executed exact-tree-child baseline-only scenarios, including endpoint and state snapshots, in `tests/sync_filesystem_integration.rs`.
- [X] T006 [US1] Add an exact file-mapping initial-match baseline-only scenario and result-envelope assertions in `tests/sync_filesystem_integration.rs`.
- [X] T007 [P] [US1] Add a real CLI transcript assertion for planned and completed initial-match baseline establishment in `tests/sync_cli_contract.rs`.

### Implementation for User Story 1

- [X] T008 [US1] Make only an eligible exact initial match use the existing acceptance-only sync disposition in `src/mutation/plan.rs`.
- [X] T009 [US1] Preserve exact selection and lock-held plan rebuilding when executing the acceptance-only sync path in `src/lib.rs` and `src/mutation/execution.rs`.
- [X] T010 [US1] Run the focused sync planning, CLI-contract, and filesystem-integration suites and record the quickstart outcome in `specs/037-initial-match-sync-baseline/quickstart.md`.

**Checkpoint**: A selected eligible initial match publishes only its accepted baseline and subsequently reports current.

## Phase 4: User Story 2 - Preserve Non-Eligible Safeguards (Priority: P2)

**Goal**: Ensure acceptance-only sync cannot admit an unsafe, incomplete, unequal, unmanaged, or broad scope.

**Independent Test**: Each excluded selected condition preserves state and endpoint snapshots and retains its prior result.

### Tests for User Story 2

- [X] T011 [US2] Add scope-regression coverage proving no-selector, tree-root, and multi-entry subtree sync do not accept initial matches in `tests/sync_filesystem_integration.rs`.
- [X] T012 [US2] Add no-publication regressions for initial collision, absence, ignored membership, destination-only content, and unsafe or unsupported evidence in `tests/sync_filesystem_integration.rs`.

### Implementation for User Story 2

- [X] T013 [US2] Keep the exact-selection gate and all blocking dispositions unchanged for ineligible records in `src/mutation/plan.rs` and `src/lib.rs`.
- [X] T014 [US2] Run the focused safety regressions and inspect `git diff --check` for the Feature 037 scope.

**Checkpoint**: The new path does not broaden ownership, force, conflict, or broad-tree mutation authority.

## Phase 5: User Story 3 - Understand Acceptance-Only Synchronization (Priority: P3)

**Goal**: Make previews and completions clearly identify baseline establishment without implying a payload copy.

**Independent Test**: Dry-run and completion for an exact initial match show baseline-only output with zero payload actions, while existing result schemas remain unchanged.

### Tests for User Story 3

- [X] T015 [US3] Verify human and JSON parity for initial-match dry-run and completion using `tests/sync_cli_contract.rs` and `tests/sync_filesystem_integration.rs`.

### Implementation for User Story 3

- [X] T016 [US3] Confirm the existing baseline-only renderer remains the sole presentation path and adjust only necessary wording in `src/result.rs`.
- [X] T017 [US3] Reconcile `README.md`, `docs/product-definition.md`, and `specs/037-initial-match-sync-baseline/contracts/sync-baseline-acceptance.md` with the implemented exact-scope behavior.

**Checkpoint**: Operators can distinguish baseline-only acceptance from a copy, in both preview and completion.

## Phase 6: Polish and Cross-Cutting Validation

**Purpose**: Complete traceability, validation, and governed closeout preparation.

- [X] T018 [P] Map FR-001 through FR-010 and SC-001 through SC-004 to implementation and tests in `specs/037-initial-match-sync-baseline/spec.md` and `tasks.md`.
- [X] T019 Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `cargo build --release`, and `mise run validate` from the repository root.
- [X] T020 Re-run every scenario in `specs/037-initial-match-sync-baseline/quickstart.md` and record measured outcomes without real user paths.

## Dependencies and Execution Order

- **Setup (Phase 1)**: Starts immediately. T002 can run in parallel with T001.
- **Foundational (Phase 2)**: T003 establishes the failing scope boundary; T004 follows it and blocks all implementation.
- **US1 (Phase 3)**: T005–T007 can be authored in parallel after T004; T008 and T009 follow, then T010 verifies the MVP.
- **US2 (Phase 4)**: T011–T012 can be authored in parallel after T009; T013 and T014 follow.
- **US3 (Phase 5)**: T015 follows the stable result behavior; T016 and T017 follow it.
- **Polish (Phase 6)**: T018–T020 follow all user stories.

## Parallel Examples

```text
T005: Add tree-child baseline-only filesystem coverage in tests/sync_filesystem_integration.rs
T007: Add CLI transcript coverage in tests/sync_cli_contract.rs
```

```text
T011: Add broad-scope regression coverage in tests/sync_filesystem_integration.rs
T015: Verify result parity in tests/sync_cli_contract.rs and tests/sync_filesystem_integration.rs
```

## Implementation Strategy

### MVP First

1. Complete T001–T004 to establish the exact-selection gate.
2. Complete T005–T010 to accept only an exact equivalent initial match.
3. Validate the MVP with the focused suites and quickstart before proceeding.

### Incremental Delivery

1. Add the narrowly scoped acceptance-only transition.
2. Prove excluded states and broad scopes remain unchanged.
3. Confirm human and JSON result clarity, then run full validation.
