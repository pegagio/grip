# Tasks: Bidirectional Synchronization and Conflict Resolution

**Input**: Design documents from `specs/007-bidirectional-sync-conflicts/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`

**Tests**: Automated tests are required by FR-030 and the project constitution. Write each story's tests first and confirm they fail for the missing behavior before implementation.

**Organization**: Tasks are grouped by user story so preview, bidirectional execution, conflict resolution, and automation/failure behavior can be reviewed at explicit checkpoints.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it changes different files and does not depend on incomplete work
- **[Story]**: Maps the task to a user story from `spec.md`
- Every task includes exact repository-relative file paths

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish the new command adapters and reusable test fixtures without changing accepted behavior.

- [X] T001 Add thin `sync` and `resolve` module shells and expose them from `src/lib.rs`, `src/sync/mod.rs`, and `src/resolve/mod.rs`
- [X] T002 [P] Add reusable mixed-direction, converged-change, and divergent-conflict fixture builders without changing existing push/pull fixtures in `tests/support/mod.rs`

**Checkpoint**: The crate builds with empty adapters, and existing push/pull behavior remains unchanged.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add shared operation, per-action direction, winner, and persistent evidence contracts required by every story.

**Critical**: No user-story implementation begins until these model and compatibility gates pass.

- [X] T003 Add failing exhaustive serialization, plan-identity, operation/direction, winner-validity, and legacy push/pull compatibility unit tests in `src/mutation/model.rs` and `src/mutation/plan.rs`
- [X] T004 Implement `MutationOperation`, per-action `MutationDirection`, optional `ConflictWinner`, acceptance identities, and operation-aware plan/result fields while preserving push/pull compatibility aliases in `src/mutation/model.rs`
- [X] T005 [P] Add failing tests for V1 acceptance of historical push/pull plans and new sync/resolve plans plus rejection of operation, direction, winner, action-index, and integrity mismatches in `tests/operation_record_integration.rs`
- [X] T006 Extend strict Partitioned Operation Record V1 validation and initialization for `sync` and `resolve` without rewriting retained records in `src/operation/model.rs` and `src/operation/publication.rs`
- [X] T007 Make existing push and pull planners populate operation and action-direction fields while preserving their prior ordering, plan semantics, and public results in `src/mutation/plan.rs`, `src/lib.rs`, and `src/result.rs`
- [X] T008 Run existing push, pull, and operation-record suites and resolve only compatibility regressions in `src/`, `tests/push_*.rs`, `tests/pull_*.rs`, and `tests/operation_record_integration.rs`

**Checkpoint**: The shared model can represent all four operations, historical evidence remains valid, and every pre-Feature-007 regression passes.

---

## Phase 3: User Story 1 - Preview a Unified Synchronization (Priority: P1) MVP

**Goal**: Produce one deterministic, non-mutating sync plan containing both directions, complete blockers, converged acceptance evidence, and unmanaged destination non-actions.

**Independent Test**: In an isolated scope containing source-only, destination-only, synchronized, converged, divergent, unsupported, and unmanaged evidence, `grip sync --dry-run` reports every entry and blocker in canonical order and changes no payload or Grip state.

### Tests for User Story 1

- [X] T009 [P] [US1] Add failing exhaustive 18-classification sync policy, mixed-direction ordering, parent dependency, converged acceptance, complete blocker, and deterministic plan-ID tests in `tests/sync_planning.rs`
- [X] T010 [P] [US1] Add failing CLI grammar, selector path-space, `--` termination, `-n`/`--dry-run` equivalence, invalid-usage, human/JSON parity, and preview non-mutation tests in `tests/sync_cli_contract.rs`

### Implementation for User Story 1

- [X] T011 [US1] Implement the total sync disposition table, per-entry action directions, canonical mixed ordering, push parent dependencies, complete blockers, and converged acceptance identities in `src/mutation/plan.rs`
- [X] T012 [US1] Add `SyncArgs` with execute-by-default, preview aliases, one optional selector, and destination-space selection in `src/cli.rs`
- [X] T013 [US1] Implement read-only sync preflight and plan construction through the thin adapter in `src/lib.rs` and `src/sync/mod.rs`
- [X] T014 [US1] Project operation, per-action direction, accept-only entries, no-actions, and complete blockers into deterministic human and Result Envelope V1 output in `src/result.rs`
- [X] T015 [US1] Complete the preview matrix and make all US1 planning, CLI, selector, parity, blocker, determinism, and non-mutation tests pass in `tests/sync_planning.rs` and `tests/sync_cli_contract.rs`

**Checkpoint**: User Story 1 is independently demonstrable as a safe preview-only MVP; no sync payload or baseline mutation is enabled.

---

## Phase 4: User Story 2 - Synchronize Unambiguous Changes in Both Directions (Priority: P2)

**Goal**: Execute mixed push/pull actions under one lock and operation record, preserve every replaced target, verify the complete result, and publish one accepted baseline including converged entries.

**Independent Test**: Run sync over isolated source-only and destination-only changes plus converged and synchronized entries; verify canonical mixed execution, target recovery, final equivalence, one baseline generation, and a later synchronized-only no-op.

### Tests for User Story 2

- [X] T016 [P] [US2] Add failing mixed-direction replacement, source-addition, parent handling, target capability, no-follow, complete verification, converged-only, and semantic no-op tests in `tests/sync_filesystem_integration.rs`
- [X] T017 [P] [US2] Add failing losing-target recovery binding, payload verification, privacy, and direction tests for mixed sync actions in `tests/sync_recovery_integration.rs`
- [X] T018 [P] [US2] Add failing lock ownership, lock-held plan drift, registry/state revalidation, global lock-order, and synchronized-preview/no-op lock-free tests in `tests/sync_contention_integration.rs`
- [X] T019 [P] [US2] Add failing tests that initialize a sync operation record before the first payload mutation or converged-only accepted-state mutation, then cover per-action checkpoints, terminal baseline evidence, and historical-record preservation in `tests/operation_record_integration.rs`

### Implementation for User Story 2

- [X] T020 [US2] Extend accepted-state candidate construction to verify and update the ordered actioned-plus-converged acceptance set while preserving out-of-scope, synchronized, and pending-retirement records in `src/baseline.rs`
- [X] T021 [US2] Generalize plan rebuilding, per-action revalidation, origin/target selection, stop-after-first-failure execution, final observation, and acceptance-set publication by operation and action direction in `src/mutation/execution.rs`
- [X] T022 [US2] Reuse descriptor-safe per-target staging, losing-side recovery, atomic publication, durability reporting, and verification for alternating action directions in `src/mutation/filesystem.rs` and `src/mutation/recovery.rs`
- [X] T023 [US2] Initialize and checkpoint each sync record immediately before its first payload mutation or, for converged-only sync, before accepted-state mutation under the existing mutation-to-registry-to-state lock order in `src/operation/publication.rs` and `src/state/mutation_lock.rs`
- [X] T024 [US2] Route execute-mode sync through lock-held plan equality, shared execution, accepted-state publication, and applied/no-op outcomes in `src/lib.rs` and `src/sync/mod.rs`
- [X] T025 [US2] Complete the US2 execution matrix and make mixed filesystem, recovery, contention, operation-record, converged-only, and semantic no-op tests pass in `tests/sync_filesystem_integration.rs`, `tests/sync_recovery_integration.rs`, `tests/sync_contention_integration.rs`, and `tests/operation_record_integration.rs`

**Checkpoint**: User Story 2 independently demonstrates complete mixed-direction synchronization and one truthful accepted baseline.

---

## Phase 5: User Story 3 - Resolve a Whole-Entry Conflict Explicitly (Priority: P3)

**Goal**: Resolve one exact established divergent entry from a source-space path using exactly one explicit complete-state winner, with preview, fresh inspection, losing-side recovery, verification, and baseline publication.

**Independent Test**: From equivalent isolated conflicts, preview and execute source-wins and destination-wins resolution; verify direction, losing-side recovery, complete-state replacement, stale-decision rejection, and accepted convergence.

### Tests for User Story 3

- [X] T026 [P] [US3] Add failing resolve grammar tests for required source-space path, mutually exclusive winner flags, both preview aliases, `--` termination, destination-path rejection, and invalid selectors in `tests/resolve_cli_contract.rs`
- [X] T027 [P] [US3] Add failing planner tests for exact divergent eligibility, source/push and destination/pull winners, complete supported-state selection, deterministic identity, and every non-conflict rejection in `tests/resolve_planning.rs`
- [X] T028 [P] [US3] Add failing source-wins, destination-wins, losing-side recovery, changed-side/baseline drift, replacement verification, and baseline-publication tests in `tests/resolve_filesystem_integration.rs`

### Implementation for User Story 3

- [X] T029 [US3] Add `ResolveArgs` and mutually exclusive `WinnerArg` flags with required source-space path and preview aliases in `src/cli.rs`
- [X] T030 [US3] Implement exact-entry divergent-conflict validation and one-action winner-bound resolution planning in `src/mutation/plan.rs` and `src/resolve/mod.rs`
- [X] T031 [US3] Route resolution preview and execute through fresh source-space selection, lock-held plan equality, shared directional execution, and one-entry baseline publication in `src/lib.rs`
- [X] T032 [US3] Project resolve operation, explicit winner, one action direction, recovery, stale evidence, verification, and baseline outcomes in `src/result.rs` and `src/error.rs`
- [X] T033 [US3] Complete the US3 matrix and make CLI, planning, both-winner, stale-decision, recovery, whole-entry, and non-mutation tests pass in `tests/resolve_cli_contract.rs`, `tests/resolve_planning.rs`, and `tests/resolve_filesystem_integration.rs`

**Checkpoint**: User Story 3 independently demonstrates explicit, fresh, whole-entry conflict resolution in both directions.

---

## Phase 6: User Story 4 - Understand Failures and Automate Safely (Priority: P4)

**Goal**: Give users and scripts stable, equivalent evidence for every sync and resolution terminal state without misrepresenting payload or baseline authority.

**Independent Test**: Exercise planned, blocked, no-op, applied, stale, contended, partial, failed, visible-but-not-durable baseline, interrupted, and result-delivery-failed outcomes in human and JSON modes; verify equivalent structured authority evidence and exit behavior.

### Tests for User Story 4

- [X] T034 [P] [US4] Add failing mixed-direction fault tests for every revalidation, recovery, staging, publication, verification, checkpoint, final-observation, baseline-publication, and no-later-action boundary in `tests/sync_failure_integration.rs`
- [X] T035 [P] [US4] Add failing result-schema, action-direction, resolve-winner, deterministic ordering, human/JSON parity, exit-category, and output-delivery tests in `tests/sync_cli_contract.rs` and `tests/resolve_cli_contract.rs`
- [X] T036 [US4] Add failing tests for immutable interrupted/failed sync and resolve records, precise terminal checkpoints, accepted-generation retention, and independent retry behavior in `tests/operation_record_integration.rs`

### Implementation for User Story 4

- [X] T037 [US4] Extend typed fault phases and mutation failures to carry invoked operation, per-action direction, winner when applicable, expected/observed evidence, recovery disposition, and safe guidance in `src/mutation/mod.rs` and `src/error.rs`
- [X] T038 [US4] Implement complete blocked, planned, no-op, applied, partial, failed, interruption, baseline-authority, and result-delivery projections for sync and resolve in `src/result.rs`
- [X] T039 [US4] Finalize only the associated sync or resolve operation record after rendering and preserve accepted payload/baseline authority on output failure in `src/lib.rs` and `src/operation/publication.rs`
- [X] T040 [US4] Complete the US4 outcome matrix and make fault, schema, parity, checkpoint, interruption, retry, baseline-authority, and delivery-failure tests pass in `tests/sync_failure_integration.rs`, `tests/sync_cli_contract.rs`, `tests/resolve_cli_contract.rs`, and `tests/operation_record_integration.rs`

**Checkpoint**: All four stories are independently demonstrable and automation can distinguish every supported outcome and authority boundary.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Complete public documentation, compatibility review, performance acceptance, full validation, and post-implementation reconciliation.

- [X] T041 [P] Document sync and resolve grammar, preview/apply behavior, complete blocking, explicit winners, recovery, machine output, and exclusions in `README.md`
- [X] T042 [P] Add concise Rust documentation for public operation, direction, winner, plan, action, sync, and resolution contracts in `src/mutation/model.rs`, `src/sync/mod.rs`, and `src/resolve/mod.rs`
- [X] T043 [P] Extend the representative fixture and 100-run p95 harness for 10,000 mixed synchronized, source-only, destination-only, converged, and conflicting entries in `tests/performance_acceptance.rs`
- [X] T044 Execute every disposable mixed-sync, complete-blocker, both-winner, converged-only, failure, recovery, selector, and output workflow and correct any drift in `specs/007-bidirectional-sync-conflicts/quickstart.md`
- [X] T045 Run `mise run validate` and resolve only Feature 007 regressions in `Cargo.toml`, `src/`, and `tests/`
- [X] T046 Run `mise run performance`, verify the 10,000-entry sync preview and pure plan meet SC-009, and record reproducible evidence in `specs/007-bidirectional-sync-conflicts/quickstart.md`
- [X] T047 Run `$speckit-converge` after implementation and reconcile genuinely unbuilt requirements into `specs/007-bidirectional-sync-conflicts/tasks.md` against `specs/007-bidirectional-sync-conflicts/spec.md` and `specs/007-bidirectional-sync-conflicts/plan.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately.
- **Foundational (Phase 2)**: Depends on Phase 1 and blocks every user story.
- **US1 (Phase 3)**: Depends on Phase 2 and is the preview-only MVP.
- **US2 (Phase 4)**: Depends on US1's complete sync plan and adds mutation.
- **US3 (Phase 5)**: Depends on the foundational model and US2's operation-aware executor; its planner and CLI tests may begin after Phase 2.
- **US4 (Phase 6)**: Depends on US2 and US3 terminal outcomes.
- **Polish (Phase 7)**: Depends on all selected stories; convergence runs only after implementation and validation.

### Foundational Dependency Graph

1. T001 and T002 may proceed in parallel.
2. T001 -> T003 -> T004.
3. T004 unlocks T005, T006, and T007; T005 and T007 may proceed in parallel.
4. T006 + T007 -> T008.
5. T008 unlocks all story phases.

### User Story Dependencies

- **US1**: T009 and T010 run in parallel; T011-T014 implement the contract; T015 closes the story.
- **US2**: T016-T019 run in parallel after US1; T020-T024 implement accepted-state and execution behavior; T025 closes the story.
- **US3**: T026-T028 run in parallel after Phase 2; T029-T032 may proceed once US2's executor contract is stable; T033 closes the story.
- **US4**: T034 and T035 run in parallel after US2/US3; T036 follows the record shape; T037-T039 implement terminal evidence; T040 closes the story.

### Requirement Traceability

- **US1 / preview and planning**: FR-001-FR-010, FR-029-FR-031; SC-001, SC-002, SC-004, SC-008, SC-009.
- **US2 / mixed execution**: FR-011-FR-022, FR-028-FR-031; SC-003, SC-004, SC-007-SC-010.
- **US3 / conflict resolution**: FR-023-FR-030; SC-005, SC-006, SC-008, SC-010, SC-011.
- **US4 / automation and failure evidence**: FR-012-FR-022, FR-029-FR-030; SC-002, SC-007, SC-008, SC-010.
- **Cross-cutting exclusions and performance**: T041-T046 preserve FR-028-FR-031 and SC-009.

### Within Each User Story

1. Write automated tests and confirm they fail because the story behavior is absent.
2. Implement model and policy changes before orchestration and rendering.
3. Make focused tests pass without weakening push, pull, or earlier-story assertions.
4. Stop at the checkpoint and demonstrate the independent test before continuing.

---

## Parallel Execution Examples

### User Story 1

```text
Task T009: Build the exhaustive sync planning matrix in tests/sync_planning.rs
Task T010: Build the sync CLI and preview contract in tests/sync_cli_contract.rs
```

### User Story 2

```text
Task T016: Test mixed filesystem execution in tests/sync_filesystem_integration.rs
Task T017: Test losing-target recovery in tests/sync_recovery_integration.rs
Task T018: Test mutation coordination in tests/sync_contention_integration.rs
Task T019: Test sync operation evidence in tests/operation_record_integration.rs
```

### User Story 3

```text
Task T026: Test resolve grammar in tests/resolve_cli_contract.rs
Task T027: Test winner-bound planning in tests/resolve_planning.rs
Task T028: Test both filesystem winner paths in tests/resolve_filesystem_integration.rs
```

### User Story 4

```text
Task T034: Test injected mixed-direction failures in tests/sync_failure_integration.rs
Task T035: Test human/JSON and exit parity in tests/sync_cli_contract.rs and tests/resolve_cli_contract.rs
```

---

## Implementation Strategy

### MVP First

1. Complete Setup and Foundational compatibility work.
2. Complete US1 and demonstrate deterministic, side-effect-free unified preview.
3. Stop for review before enabling sync or resolution mutation.

### Incremental Delivery

1. **US1**: Unified preview and complete blocking without mutation.
2. **US2**: Mixed-direction execution plus converged accepted-state publication.
3. **US3**: Explicit whole-entry conflict resolution in both directions.
4. **US4**: Stable failure, recovery, operation-record, and automation evidence.
5. **Polish**: Documentation, representative performance, complete validation, and convergence.

### Review Discipline

- Keep each task or dependency wave small enough for focused review.
- Preserve historical push/pull operation records and every pre-existing regression.
- Do not introduce deletion, retirement, initial-collision adoption, destination import, automatic merge, rollback, state repair, cache, parallelism, or broad payload locking.
- Treat passing automation as evidence rather than intended-user acceptance.
- Run `$speckit-analyze` after task generation and before implementation, as required by the constitution.

## Notes

- `[P]` marks work that can proceed concurrently without editing the same files or depending on incomplete tasks.

## Phase 8: Convergence

**Purpose**: Close test-evidence gaps found by post-implementation convergence without changing the established Feature 007 behavior.

- [X] T048 Complete source-addition, parent creation, target capability, no-follow, complete verification, and recovery privacy/direction coverage in `tests/sync_filesystem_integration.rs` and `tests/sync_recovery_integration.rs` per FR-030 and T016-T017 (partial)
- [X] T049 Add lock-held plan, registry, and state drift plus global lock-order and preview/no-op lock-free coverage in `tests/sync_contention_integration.rs` per FR-020, FR-030, and T018 (partial)
- [X] T050 Complete exact resolution eligibility, accepted-baseline drift, changed-side drift, whole-entry content/metadata conflict, and non-conflict/deletion/retirement rejection coverage in `tests/resolve_planning.rs` and `tests/resolve_filesystem_integration.rs` per FR-024-FR-027, FR-030, and T027-T028 (partial)
- [X] T051 Exercise every shared mutation fault boundary, multi-action checkpoint and terminal baseline outcome, paired human/JSON evidence, and stable exit category for sync and resolve in `tests/sync_failure_integration.rs`, `tests/sync_cli_contract.rs`, `tests/resolve_cli_contract.rs`, and `tests/operation_record_integration.rs` per FR-015-FR-019, FR-021, FR-029-FR-030, T019, and T034-T040 (partial)

## Phase 9: Convergence

**Purpose**: Correct terminal evidence that became provably inaccurate at post-success fault boundaries.

- [X] T052 Preserve verified action evidence after the post-target-verification boundary and durable baseline evidence after successful publication, with regression assertions in `src/mutation/execution.rs` and `tests/sync_failure_integration.rs` per FR-016, FR-019, FR-021, and FR-029 (contradicts)
- `[USn]` provides direct story traceability.
- Tasks are ordered for test-first implementation and explicit review checkpoints.
- No task authorizes a commit, push, publication, deletion, deployment, or production-data mutation.
