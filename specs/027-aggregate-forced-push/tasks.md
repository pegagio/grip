---

description: "Implementation tasks for Feature 027 Aggregate Forced Push"
---

# Tasks: Aggregate Forced Push

**Input**: Design documents from `/specs/027-aggregate-forced-push/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [aggregate-forced-push.md](./contracts/aggregate-forced-push.md), and [quickstart.md](./quickstart.md)

**Tests**: Required. The specification and constitution require isolated automated coverage for every behavior that can mutate payloads or accepted state.

**Organization**: Tasks are grouped by user story. Foundational work defines the typed aggregate operation, full-scope planning, and per-entry execution primitives shared by all stories.

## Format

Every task follows `- [ ] [TaskID] [P?] [Story?] Description with file path`.

## Phase 1: Setup

**Purpose**: Make reusable isolated-test support available before aggregate tests are added.

- [X] T001 Add deterministic multi-entry project fixtures and accepted-state inspection helpers in `tests/support/project.rs` for aggregate-force integration tests.

## Phase 2: Foundational

**Purpose**: Establish the aggregate operation and execution boundary required by every user story.

**⚠️ CRITICAL**: Complete this phase before implementing user-story behavior.

- [X] T002 Add the typed `AggregateForcePush` operation and aggregate entry/publication result data without changing existing operation serialization semantics in `src/mutation/model.rs`.
- [X] T003 Extend operation-record summary and receipt accounting for per-entry accepted-state publications and failed aggregate outcomes in `src/operation/model.rs` and `src/operation/publication.rs`.
- [X] T004 Implement deterministic complete-source aggregate planning, including entry-to-action association, missing destination restoration, and aggregate-only supported source absence in `src/mutation/plan.rs` and `src/delete/plan.rs`.
- [X] T005 Refactor aggregate execution to process entries in deterministic order, verify every action belonging to an entry, publish that entry's accepted state, and reload State V4 before later revalidation in `src/mutation/execution.rs` and `src/state/publication.rs`.
- [X] T006 Rebuild and revalidate aggregate plans under the existing mutation lock without broadening destination-link replacement or ordinary-resolution policy in `src/mutation/execution.rs`.

**Checkpoint**: The typed aggregate pipeline can plan, execute, publish, and report entry-level state without exposing the command yet.

## Phase 3: User Story 1 - Force all managed source changes (Priority: P1) 🎯 MVP

**Goal**: A no-selector `grip push --force` makes the complete source state authoritative for eligible managed entries in one selected project while supplied selectors remain exact.

**Independent Test**: In an isolated project containing divergent, missing-destination, and already-converged entries, run no-selector force and verify all eligible destination states; separately verify a supplied selector continues to affect one exact entry only.

### Tests for User Story 1

- [X] T007 [P] [US1] Add no-selector force parser and exact-selector compatibility assertions in `tests/push_cli_contract.rs`.
- [X] T008 [P] [US1] Add deterministic aggregate classification and source-winner planning cases for divergent and missing-destination entries in `tests/push_planning.rs`.
- [X] T009 [US1] Add isolated successful aggregate execution coverage for multiple mappings and unchanged entries in `tests/push_filesystem_integration.rs`.
- [X] T010 [US1] Add supported source-absence aggregate coverage that distinguishes explicit aggregate authority from ordinary push behavior in `tests/push_filesystem_integration.rs`.

### Implementation for User Story 1

- [X] T011 [US1] Route omitted-selector source-side `push --force` to the aggregate operation while preserving exact-selector `push --force`, all pull behavior, and existing selector resolution in `src/lib.rs`.
- [X] T012 [US1] Update no-selector forced-push CLI help and command-result presentation to identify the aggregate source-winning scope in `src/cli.rs` and `src/result.rs`.
- [X] T013 [US1] Connect aggregate source-complete classifications to typed mutation and deletion actions without expanding ordinary `Resolve` semantics in `src/mutation/plan.rs` and `src/delete/execution.rs`.

**Checkpoint**: `grip push --force` succeeds for a safe multi-entry source-winning project, while `grip push <selector> --force` remains exact-entry only.

## Phase 4: User Story 2 - Review the aggregate force operation safely (Priority: P2)

**Goal**: Operators can preview the complete aggregate and safety blockers prevent all mutation before execution starts.

**Independent Test**: A no-selector dry run reports the same planned scope as execution with unchanged payloads and State V4; each supported preflight blocker leaves every destination and accepted record unchanged.

### Tests for User Story 2

- [X] T014 [P] [US2] Add dry-run contract assertions for omitted-selector force and machine-readable aggregate scope in `tests/push_cli_contract.rs`.
- [X] T015 [P] [US2] Add aggregate preflight planning cases for unsupported nodes, ownership, topology, and no-follow blockers in `tests/push_planning.rs`.
- [X] T016 [US2] Add filesystem proof that dry run and preflight blockers make no payload or accepted-state mutation in `tests/push_filesystem_integration.rs`.
- [X] T017 [US2] Add aggregate destination-leaf-link and unsafe-link-ancestry regression cases that preserve Feature 026's exact-only replacement boundary in `tests/filesystem_boundary_integration.rs`.

### Implementation for User Story 2

- [X] T018 [US2] Make aggregate request construction inspect and reject the full selected-project scope before acquiring mutable execution in `src/lib.rs` and `src/mutation/plan.rs`.
- [X] T019 [US2] Render aggregate dry-run selections, planned complete source states, and blockers deterministically without publishing accepted state in `src/result.rs` and `src/mutation/model.rs`.
- [X] T020 [US2] Enforce aggregate no-follow and destination-leaf-link blocking without changing the existing exact source-winning replacement path in `src/observation/model.rs` and `src/mutation/plan.rs`.

**Checkpoint**: Dry run is fully informative and non-mutating; no aggregate preflight blocker permits partial destination or state changes.

## Phase 5: User Story 3 - Understand the result of an aggregate force attempt (Priority: P3)

**Goal**: An aggregate result precisely reports completed accepted entries, the first failure, and all later unattempted entries without claiming convergence.

**Independent Test**: Inject drift or a typed mutation/publication fault into a later deterministic entry and verify earlier entries retain accepted evidence, the aggregate fails, and later entries are unattempted.

### Tests for User Story 3

- [X] T021 [P] [US3] Add first-failure stop, per-entry publication, and State V4 refresh fault-injection coverage in `tests/push_failure_integration.rs`.
- [X] T022 [P] [US3] Add aggregate result and machine-readable operation-record assertions for completed, failed, and unattempted entries in `tests/operation_record_integration.rs`.
- [X] T023 [US3] Add external pre-action drift coverage proving the affected entry is unaccepted and later entries are not attempted in `tests/push_failure_integration.rs`.

### Implementation for User Story 3

- [X] T024 [US3] Stop aggregate execution at the first revalidation, execution, verification, or publication failure while retaining earlier verified entry publications in `src/mutation/execution.rs`.
- [X] T025 [US3] Persist and expose deterministic aggregate entry statuses, publication generations, failure reason, and later unattempted identities in `src/operation/publication.rs` and `src/operation/model.rs`.
- [X] T026 [US3] Present failed aggregate outcomes without successful-convergence language in human and JSON results in `src/result.rs` and `src/mutation/model.rs`.

**Checkpoint**: A partial aggregate failure is safe, diagnosable, and never represented as fully converged.

## Phase 6: Polish and Cross-Cutting Concerns

**Purpose**: Validate compatibility, performance, documentation, and full feature coherence.

- [X] T027 [P] Preserve and extend aggregate traversal and hashing performance assertions in `tests/performance_acceptance.rs`.
- [X] T028 [P] Document no-selector aggregate force, dry-run behavior, partial-failure semantics, and exact-selector compatibility in `README.md`.
- [X] T029 Run focused aggregate-force tests and the full Rust suite from `tests/push_cli_contract.rs`, `tests/push_planning.rs`, `tests/push_filesystem_integration.rs`, `tests/push_failure_integration.rs`, `tests/operation_record_integration.rs`, and `Cargo.toml`.
- [X] T030 Run repository validation and the manual isolated workflow in `mise.toml` and `specs/027-aggregate-forced-push/quickstart.md`.
- [X] T031 Reconcile artifacts and run the required consistency gate after tasking with `specs/027-aggregate-forced-push/spec.md`, `specs/027-aggregate-forced-push/plan.md`, and `specs/027-aggregate-forced-push/tasks.md` using `$speckit-analyze`.

## Dependencies and Execution Order

```text
Setup (T001)
  -> Foundational (T002-T006)
  -> US1 MVP (T007-T013)
  -> US2 safety/preview (T014-T020)
  -> US3 partial-failure reporting (T021-T026)
  -> Polish and consistency (T027-T031)
```

US2 and US3 use the foundational pipeline and can be developed in parallel after T006. They should merge only after US1 command routing is stable so their tests exercise the final aggregate entry point.

## Parallel Opportunities

- After T001, T002 and T003 can proceed in parallel; T004-T006 then follow their required model and plan dependencies.
- Within US1, T007 and T008 can proceed in parallel; T009 and T010 share an integration file and remain sequential.
- Within US2, T014 and T015 can proceed in parallel; T016 and T017 exercise distinct integration files and can proceed in parallel after their fixtures are ready.
- Within US3, T021 and T022 can proceed in parallel; T023 follows T021 because it extends the same failure suite.
- T027 and T028 can proceed in parallel after all user stories are complete.

## Parallel Example: User Story 2

```text
Task: "Add dry-run contract assertions for omitted-selector force in tests/push_cli_contract.rs"
Task: "Add aggregate preflight planning cases in tests/push_planning.rs"
```

## Implementation Strategy

### MVP First

1. Complete T001-T006 to establish a safe aggregate pipeline.
2. Complete T007-T013 and run the US1 independent test.
3. Stop for review before adding preview/safety refinements and partial-failure reporting.

### Incremental Delivery

1. US1 delivers safe aggregate source-winning execution and exact-selector compatibility.
2. US2 adds full preflight and dry-run observability without changing US1 selection semantics.
3. US3 adds partial-failure accounting and deterministic operator reporting.
4. Polish validates performance, documentation, and cross-artifact consistency.

## Phase 7: Convergence

- [X] T032 Persist a deterministic aggregate entry-outcome summary, including completed, failed, and unattempted identities plus publication generation, in the operation record and cover it in `src/operation/model.rs`, `src/operation/publication.rs`, `src/mutation/execution.rs`, and `tests/operation_record_integration.rs` (FR-010, plan step 6; partial).
