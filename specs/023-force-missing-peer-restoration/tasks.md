# Tasks: Forced Missing-Peer Restoration

**Input**: Design documents from `specs/023-force-missing-peer-restoration/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), and [force-restoration.md](contracts/force-restoration.md)

**Tests**: Required. The feature changes endpoint payloads, accepted state, and forced-direction behavior.

**Organization**: Tasks are grouped by user story so each directional restoration path can be demonstrated independently after shared force-planning support exists.

## Phase 1: Setup

**Purpose**: Capture the reported regression and inspect the existing force matrix before changing behavior.

- [x] T001 Record the exact tree-entry deleted-destination regression and the reciprocal missing-source case in `tests/push_filesystem_integration.rs` and `tests/pull_filesystem_integration.rs`.
- [x] T002 Map every one-sided-absence classification to its permitted forced winner and action kind in `src/mutation/plan.rs` tests.

## Phase 2: Foundational Force-Planning Support

**Purpose**: Extend only the exact present-winner branches while preserving absent-winner deletion and all other blockers.

- [x] T003 Extend exact `Resolve` disposition and directional action selection for source-present/destination-absent and destination-present/source-absent states in `src/mutation/plan.rs`.
- [x] T004 Extend resolution revalidation to accept only the same direction-compatible present-winner missing-peer states in `src/mutation/execution.rs`.
- [x] T005 Preserve the absent-winner deletion split and reject direction-incompatible one-sided conflicts in `src/lib.rs` and `tests/push_planning.rs`.

**Checkpoint**: Exact forced restoration can reach the existing verified-copy pipeline, while absent-winner force remains deletion-only.

## Phase 3: User Story 1 - Restore a Missing Destination (Priority: P1) 🎯 MVP

**Goal**: Let exact forced push restore a destination entry from a present source winner.

**Independent Test**: In an accepted tree mapping, remove one destination file and verify exact forced push restores only that file; repeat after changing the surviving source and with dry-run.

### Tests for User Story 1

- [x] T006 [P] [US1] Add exact file and tree-member destination-restoration execution and unaffected-sibling coverage in `tests/push_filesystem_integration.rs`.
- [x] T007 [P] [US1] Add exact source-selected dry-run, changed-source winner, and broad-selector rejection coverage in `tests/push_planning.rs`.

### Implementation for User Story 1

- [x] T008 [US1] Plan source-winning missing-destination files as no-replace add actions and directories as safe create actions in `src/mutation/plan.rs`.
- [x] T009 [US1] Validate exact forced push restores the missing peer through existing execution and publishes fresh accepted state in `src/mutation/execution.rs` and `src/lib.rs`.

**Checkpoint**: Exact forced push restores only the selected missing destination entry and ordinary push remains blocked on one-sided absence.

## Phase 4: User Story 2 - Restore a Missing Source (Priority: P2)

**Goal**: Let exact destination-selected forced pull restore a source entry from a present destination winner.

**Independent Test**: In an accepted tree mapping, remove one source file and verify exact destination-selected forced pull restores only that file; repeat after changing the surviving destination and with dry-run.

### Tests for User Story 2

- [x] T010 [P] [US2] Add exact file and tree-member source-restoration execution and unaffected-sibling coverage in `tests/pull_filesystem_integration.rs`.
- [x] T011 [P] [US2] Add exact destination-selected dry-run, changed-destination winner, and broad-selector rejection coverage in `tests/pull_planning.rs`.

### Implementation for User Story 2

- [x] T012 [US2] Validate exact forced pull restores the missing peer through existing execution and publishes fresh accepted state in `src/mutation/plan.rs`, `src/mutation/execution.rs`, and `src/lib.rs`.

**Checkpoint**: Exact forced pull restores only the selected missing source entry and ordinary pull remains blocked on one-sided absence.

## Phase 5: User Story 3 - Retain Deliberate Force Boundaries (Priority: P3)

**Goal**: Explain exact missing-peer force accurately while retaining absent-winner deletion and every existing exact-selection guardrail.

**Independent Test**: Inspect human and JSON output for exact missing-peer cases, then verify absent-winner deletion and aggregate selection rejection remain unchanged.

### Tests for User Story 3

- [x] T013 [P] [US3] Add exact one-sided-absence human guidance and corrected missing-peer wording coverage in `tests/classification_cli_contract.rs`.
- [x] T014 [P] [US3] Add reciprocal absent-winner deletion and direction-incompatible restoration rejection coverage in `tests/pull_filesystem_integration.rs` and `tests/push_filesystem_integration.rs`.

### Implementation for User Story 3

- [x] T015 [US3] Render executable exact force guidance for resolvable one-sided absence and distinguish missing-peer states from changed-peer conflicts in `src/lib.rs` and `src/result.rs`.

**Checkpoint**: Human guidance is executable and truthful, while force remains exact-entry-only and continues to propagate a selected absence as deletion.

## Phase 6: Polish and Cross-Cutting Validation

**Purpose**: Align user documentation and execute the required quality gates.

- [x] T016 [P] Update missing-peer forced restoration and absent-winner deletion guidance in `README.md` and `docs/product-definition.md`.
- [x] T017 Run focused force, planning, filesystem, and human-output regressions from `tests/push_filesystem_integration.rs`, `tests/pull_filesystem_integration.rs`, `tests/push_planning.rs`, `tests/pull_planning.rs`, and `tests/classification_cli_contract.rs`.
- [x] T018 Run `mise run validate` and the end-to-end scenarios in `specs/023-force-missing-peer-restoration/quickstart.md`.

## Dependencies and Execution Order

- **Phase 1**: Establishes the reproduction and direction matrix.
- **Phase 2**: Blocks all restoration behavior because it supplies planner and revalidation support.
- **US1**: Delivers the reported source-to-destination restoration path after Phase 2.
- **US2**: Delivers the symmetric destination-to-source path after Phase 2 and can be implemented after US1 or in parallel if files do not overlap.
- **US3**: Depends on both restoration paths so human guidance reflects verified behavior.
- **Polish**: Depends on every intended user story.

## Parallel Opportunities

- T001 and T002 can proceed in parallel because they target different test files.
- T006 and T007 can proceed in parallel after Phase 2.
- T010 and T011 can proceed in parallel after Phase 2.
- T013 and T014 can proceed in parallel after both directional restoration paths work.
- T016 can proceed alongside the final validation preparation once observable output is settled.

## Implementation Strategy

1. Implement and validate the P1 source-winning restoration path first.
2. Extend the same narrow direction matrix to the P2 destination-winning restoration path.
3. Finish accurate human guidance and preservation tests before documentation and full validation.
4. Do not broaden the solution to aggregate force, ordinary one-sided synchronization, or new state/recovery behavior.
