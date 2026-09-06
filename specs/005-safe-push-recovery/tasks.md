# Tasks: Safe Push and Recovery

**Input**: Design documents from `specs/005-safe-push-recovery/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`

**Tests**: Feature 005 requires automated tests for every payload, recovery, concurrency, failure, output, and baseline-publication boundary. Within each story phase, write the listed tests first and confirm they fail for the intended missing behavior before implementation.

**Organization**: Tasks are grouped by user story so preview, safe execution, failure recovery, and automation contracts can be validated as distinct increments.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it targets different files and does not depend on an incomplete task in the same phase
- **[Story]**: Maps the task to User Story 1, 2, 3, or 4
- Every task names its concrete repository path

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish the new module and fixture boundaries without changing public behavior.

- [X] T001 Create the `push` and `operation` module roots and exports in `src/push/mod.rs`, `src/operation/mod.rs`, and `src/lib.rs`
- [X] T002 [P] Add isolated push fixture builders, complete payload/state snapshots, and strict partitioned operation/recovery readers in `tests/support/mod.rs`
- [X] T003 Add shared test-only output-writer and deterministic fault-hook support in `tests/support/mod.rs` and `src/push/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Implement typed shared models, strict operation encoding, and the global writer-coordination boundary required by every user story.

**⚠️ CRITICAL**: Complete this phase before beginning any user-story implementation.

- [X] T004 Define `PushRequest`, `PushPlan`, `EntryDisposition`, `PushAction`, milestone, blocker, count, baseline-outcome, and result types from `data-model.md` in `src/push/model.rs`
- [X] T005 [P] Define strict immutable-plan, bounded operation-summary, sparse action-checkpoint, mutation-lock owner, validation, and state-transition models from Partitioned Operation Record V1 in `src/operation/model.rs`
- [X] T006 Write failing unit tests for every Partitioned Operation Record V1 envelope round trip, canonical integrity, unknown fields, invalid transitions, plan/action binding, missing-as-unattempted derivation, and escaping recovery references in `src/operation/model.rs`
- [X] T007 Implement deterministic Partitioned Operation Record V1 encoding, decoding, integrity verification, component binding, and semantic validation in `src/operation/model.rs`
- [X] T008 [P] Write failing tests for advisory ownership, active contention, unlocked stale metadata, unsafe nodes, and owner-detail parsing in `src/state/mutation_lock.rs`
- [X] T009 Implement the persistent owner-aware `.mutation.lock` with nonblocking advisory acquisition and safe metadata publication in `src/state/mutation_lock.rs`
- [X] T010 Apply the global `mutation → registry → state` acquisition order to mapping add/remove and changed baseline acceptance in `src/lib.rs`, `src/registry/publication.rs`, and `src/baseline.rs`
- [X] T011 Add regression coverage proving mapping publication, baseline publication, read-only commands, and semantic no-ops honor the new outer lock boundary in `tests/mapping_registry_integration.rs` and `tests/baseline_integration.rs`

**Checkpoint**: Shared types decode strictly, one safe outer writer lock coordinates every existing writer, and existing read-only/no-op behavior remains lock-free.

---

## Phase 3: User Story 1 - Preview a Safe Push (Priority: P1) 🎯 MVP

**Goal**: Produce a complete deterministic push preview with every selected entry, action, no-action reason, and blocker while changing nothing.

**Independent Test**: In an isolated mixed scope, `grip push --dry-run` and `grip push -n` return equivalent plan identities and ordered results, report all blockers, and leave payloads, accepted state, recovery, journals, registry, and lock state unchanged.

### Tests for User Story 1

- [X] T012 [P] [US1] Write failing grammar and selector tests for `push`, `-n`, `--dry-run`, `--destination`, `--`, and extra arguments in `tests/push_cli_contract.rs`
- [X] T013 [P] [US1] Write the failing 18-classification disposition matrix and complete-blocker aggregation tests in `tests/push_planning.rs`
- [X] T014 [US1] Write failing dependency-DAG, shared-parent deduplication, raw-byte ordering, and cycle-invariant tests in `tests/push_planning.rs`
- [X] T015 [US1] Write failing dry-run alias parity, deterministic `plan_id`, repeated-output, and complete non-mutation tests in `tests/push_cli_contract.rs`

### Implementation for User Story 1

- [X] T016 [US1] Add action-specific `PushArgs` and `push` grammar without altering push direction in `src/cli.rs`
- [X] T017 [US1] Implement pure classification-to-disposition planning for all 18 inherited categories in `src/push/plan.rs`
- [X] T018 [US1] Implement synthetic missing-parent discovery, authorization links, deduplication, dependency validation, and stable topological ordering in `src/push/plan.rs`
- [X] T019 [US1] Implement canonical mode-neutral plan serialization and SHA-256 `plan_id` generation in `src/push/plan.rs`
- [X] T020 [P] [US1] Add typed Push Result projection into Result Envelope V1 with complete counts, entries, actions, blockers, operation availability, and baseline outcome in `src/result.rs`
- [X] T021 [US1] Add human preview, blocked-plan, and no-op rendering from the typed Push Result in `src/result.rs`
- [X] T022 [US1] Route `push` through complete registry/state load, selector resolution, observation, classification, planning, and lock-free preview/no-op handling in `src/lib.rs`
- [X] T023 [US1] Run and reconcile `tests/push_planning.rs`, `tests/push_cli_contract.rs`, and all existing read-only classification regressions against `contracts/cli.md` and `contracts/push.md`

**Checkpoint**: User Story 1 is independently usable as a safe deterministic preview and is the recommended MVP boundary.

---

## Phase 4: User Story 2 - Push Source Changes Safely (Priority: P2)

**Goal**: Execute eligible source additions and replacements through lock-held revalidation, sibling staging, verified recovery, result verification, and one truthful baseline publication.

**Independent Test**: An isolated actionful push creates missing parents, adds and replaces regular files with supported modes, preserves verified prior replacement content, leaves unmanaged neighbors untouched, verifies every destination, and publishes exactly one matching State V2 generation.

### Tests for User Story 2

- [X] T024 [P] [US2] Write failing descriptor-relative addition, replacement, source-mode, unsupported-node, and unmanaged-neighbor tests in `tests/push_filesystem_integration.rs`
- [X] T025 [US2] Write failing explicit parent-action, `0700` creation, dependency order, concurrent appearance, and parent-sync tests in `tests/push_filesystem_integration.rs`
- [X] T026 [P] [US2] Write failing operation-directory allocation, immutable plan and bounded summary initialization, sparse atomic action checkpoint, permissions, integrity, cleanup, and bounded-serialization tests in `tests/push_recovery_integration.rs`
- [X] T027 [US2] Write failing verified replacement recovery, addition-without-backup, collision, path containment, and recovery-permission tests in `tests/push_recovery_integration.rs`
- [X] T028 [P] [US2] Write failing post-lock plan-drift and per-action source/destination/ancestry/policy/state revalidation tests in `tests/push_filesystem_integration.rs`
- [X] T029 [P] [US2] Write failing complete-success, scoped-baseline update, out-of-scope preservation, no-partial-generation, and post-rename durability tests in `tests/baseline_integration.rs`

### Implementation for User Story 2

- [X] T030 [US2] Implement descriptor-relative no-follow destination ancestry, exclusive child creation, safe opened-node identity, rename, sync, and cleanup primitives in `src/push/filesystem.rs`
- [X] T031 [US2] Implement descriptor-bound source streaming, sibling staging, supported-mode application, sync, reread, fingerprint verification, and attempt-owned cleanup in `src/push/filesystem.rs`
- [X] T032 [US2] Implement synthetic parent and managed-directory execution with exact absence revalidation and containing-directory durability in `src/push/execution.rs`
- [X] T033 [US2] Implement private numeric-action recovery staging, exclusive publication, metadata binding, descriptor-bound verification, and collision handling in `src/push/recovery.rs`
- [X] T034 [US2] Implement safe operation-directory allocation plus atomic integrity-checked immutable-plan, bounded-summary, and sparse action-checkpoint publication in `src/operation/publication.rs`
- [X] T035 [US2] Implement actionful push lock acquisition, complete lock-held reload/reinspection/replanning, semantic plan comparison, and initial journal publication in `src/push/execution.rs`
- [X] T036 [US2] Implement add-file and replace-file execution with pre-action checkpoint, immediate revalidation, recovery gate, staging publication, visibility/durability tracking, and final destination verification in `src/push/execution.rs`
- [X] T037 [US2] Extend accepted-baseline candidate construction to update only verified actioned identities and preserve every other accepted record in `src/baseline.rs`
- [X] T038 [US2] Implement final complete selected observation, registry/state revalidation, inner lock acquisition, and one State V2 publication in `src/push/execution.rs`
- [X] T039 [US2] Implement successful applied/no-op action counts, operation availability, recovery state, and baseline generation results in `src/push/model.rs` and `src/result.rs`
- [X] T040 [US2] Integrate the complete execute path and operation receipt into command orchestration in `src/lib.rs`

**Checkpoint**: User Story 2 safely performs and accepts complete source-to-destination additions and replacements without implementing reverse direction or deletion.

---

## Phase 5: User Story 3 - Understand and Recover From Failure (Priority: P3)

**Goal**: Stop at the first execution failure, preserve truthful journal and recovery evidence, leave later actions unattempted, publish no partial baseline, and permit a later independently planned push after interruption.

**Independent Test**: Deterministic faults at every side-effect boundary yield exact completed/failed/unattempted states, correct visibility/verification/durability, preserved recovery, unchanged pre-rename baseline authority, immutable interrupted records, and no automatic rollback or resume.

### Tests for User Story 3

- [X] T041 [P] [US3] Write failing fault-matrix tests for staging, recovery, payload publication, directory sync, verification, and baseline publication in `tests/push_failure_integration.rs`
- [X] T042 [US3] Write failing before/after-side-effect bounded action-checkpoint failure tests and last-durable-state assertions in `tests/push_failure_integration.rs`
- [X] T043 [P] [US3] Write failing process-interruption, immutable nonterminal record, fresh-plan/new-ID, preserved-recovery, and unrelated-history-nontraversal tests in `tests/push_recovery_integration.rs`
- [X] T044 [US3] Write failing unsafe, corrupt, unsupported-schema, and inconsistent live operation/recovery component tests in `tests/push_recovery_integration.rs`

### Implementation for User Story 3

- [X] T045 [US3] Add typed hidden push fault phases and hook invocation points without production CLI or environment activation in `src/push/mod.rs` and `src/push/execution.rs`
- [X] T046 [US3] Persist bounded per-action `in_progress`, recovery, staging, visibility, durability, verification, completed, and failed checkpoints plus bounded terminal operation summaries in `src/push/execution.rs`
- [X] T047 [US3] Implement first-failure stopping with completed/failed/unattempted accounting, preserved side effects, and baseline non-publication in `src/push/execution.rs`
- [X] T048 [US3] Build partial and failed Push Results with first failure, recovery availability, last-known action evidence, and authoritative baseline state in `src/push/model.rs` and `src/result.rs`
- [X] T049 [US3] Implement exclusive fresh operation allocation that preserves prior nonterminal records and recovery entries without enumerating unrelated retained history in `src/operation/publication.rs`
- [X] T050 [US3] Integrate existing State V2 pre-rename and post-rename publication faults without misreporting baseline visibility or durability in `src/push/execution.rs`
- [X] T051 [US3] Run and reconcile `tests/push_failure_integration.rs` and `tests/push_recovery_integration.rs` against every state transition in `data-model.md`

**Checkpoint**: User Story 3 makes every partial or interrupted outcome inspectable and recoverable later without pretending it was accepted.

---

## Phase 6: User Story 4 - Use Push Results in Automation (Priority: P4)

**Goal**: Expose stable human and JSON outcomes, exit categories, contention details, and output-failure finalization for every push terminal state.

**Independent Test**: CLI fixtures for preview, blocked, no-op, applied, stale, contended, partial, baseline-failed, and output-failed outcomes produce contract-exact envelopes or the documented failed-channel behavior, with human/JSON semantic parity and diagnostics isolated on stderr.

### Tests for User Story 4

- [X] T052 [P] [US4] Write failing JSON schema, complete array/count, opaque operation ID, safe-path, human parity, and exit `0/2/10/11/12/20` tests in `tests/push_cli_contract.rs`
- [X] T053 [P] [US4] Write failing exit `13`, stable contention reason, owner-detail, stale-metadata, and diagnostic-channel tests in `tests/push_contention_integration.rs`
- [X] T054 [US4] Write failing result-writer tests proving exit `20`, retained published baseline, and durable `result_delivery` finalization in `tests/push_cli_contract.rs`

### Implementation for User Story 4

- [X] T055 [US4] Complete stable human and JSON rendering for planned, blocked, no-op, applied, partial, failed, and publication-visible outcomes in `src/result.rs`
- [X] T056 [US4] Add structured push blocker/failure projection and generalize state contention so it reports the active operation instead of hardcoding baseline acceptance in `src/error.rs` and `src/result.rs`
- [X] T057 [US4] Add an internal operation receipt and bounded-summary render-failure finalization path that records `prepared`, best-effort `failed`, and non-retroactive `delivered` states in `src/lib.rs` and `src/operation/publication.rs`
- [X] T058 [US4] Expose safe mutation-lock owner metadata and stale-unlocked metadata behavior through typed contention results in `src/state/mutation_lock.rs` and `src/result.rs`
- [X] T059 [US4] Reconcile CLI help, grammar, result messages, safe path encoding, and exit behavior with `contracts/cli.md` in `src/cli.rs`, `src/lib.rs`, and `src/result.rs`
- [X] T060 [US4] Run and reconcile `tests/push_cli_contract.rs` and `tests/push_contention_integration.rs` across every public terminal outcome

**Checkpoint**: User Story 4 gives scripts a stable, truthful contract without coupling them to private recovery paths or diagnostics.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Close repository-wide compatibility, performance, documentation, and validation obligations after all selected stories are complete.

- [X] T061 [P] Extend the ignored 10,000-entry harness with 100 dry-run and 100 pre-mutation execute-mode planning measurements, equivalent-plan and deterministic non-mutation assertions, and the dry-run p95 gate in `tests/performance_acceptance.rs`
- [X] T062 [P] Add State V1/V2, registry, discovery, classification, baseline, non-UTF-8, and unsupported-node regression coverage affected by push in `tests/state_integration.rs`, `tests/discovery_filesystem_integration.rs`, and `tests/classification_filesystem_integration.rs`
- [X] T063 [P] Update the public push command, safety boundary, recovery evidence, output, and exit documentation in `README.md`
- [X] T064 Audit FR-001 through FR-032 and SC-001 through SC-009 traceability against implementation and tests, flowing accepted corrections through `specs/005-safe-push-recovery/spec.md`, `plan.md`, and `tasks.md`
- [X] T065 Execute every disposable-root scenario in `specs/005-safe-push-recovery/quickstart.md` and record measured results in that file without adding real user paths
- [X] T066 Run `mise run validate` and `mise run performance`, then reconcile any formatting, Clippy, test, release-build, or p95 failures in the files changed by Feature 005

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies; begin immediately.
- **Foundational (Phase 2)**: Depends on Setup and blocks all user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational; delivers the preview MVP.
- **User Story 2 (Phase 4)**: Depends on User Story 1's planner and result model.
- **User Story 3 (Phase 5)**: Depends on User Story 2's executor, journal, recovery, and baseline path.
- **User Story 4 (Phase 6)**: Depends on terminal results from User Stories 1 through 3.
- **Polish (Phase 7)**: Depends on every user story selected for delivery.

### User Story Dependencies

```text
Setup -> Foundational -> US1 Preview -> US2 Safe Execution -> US3 Failure Evidence -> US4 Automation
                                                                      \
                                                                       -> Polish
```

- **US1**: Independently testable as a non-mutating planning product after Foundational.
- **US2**: Reuses US1 planning but is independently testable as a complete successful mutation journey.
- **US3**: Exercises failures in US2 components and is independently testable through injected faults and later inspection.
- **US4**: Projects already-typed results from all earlier stories into stable public interfaces.

### Within Each User Story

- Write the story's tests first and confirm they fail for the intended missing behavior.
- Establish domain state and validation before side-effecting services.
- Revalidate before every action and checkpoint before every side effect.
- Complete focused integration tests before advancing to the next story.

### Parallel Opportunities

- T002 can proceed after T001; T003 follows because both tasks extend `tests/support/mod.rs`.
- T005 and T008 can proceed in parallel after T004; T006 follows T005, T007 follows T006, and T009 follows T008.
- US1 grammar tests T012 and planner matrix tests T013 can be authored in parallel; T014 follows T013 and T015 follows T012 because each pair shares a file.
- US2 filesystem tests T024, operation tests T026, and baseline tests T029 can be authored in parallel; related follow-on tests sharing those files remain sequential.
- US3 fault tests T041 and interruption tests T043 can be authored in parallel; T042 and T044 follow their same-file predecessors.
- US4 result tests T052 and contention tests T053 can be authored in parallel; failed-writer tests T054 follow T052.
- T061–T063 can proceed in parallel after story completion; T064–T066 follow the stable integrated result.

---

## Parallel Example: User Story 1

```text
Task T012: Write CLI grammar and selector tests in tests/push_cli_contract.rs
Task T013: Write classification disposition tests in tests/push_planning.rs
```

## Parallel Example: User Story 2

```text
Task T024: Write addition/replacement filesystem tests in tests/push_filesystem_integration.rs
Task T026: Write partitioned operation-record lifecycle tests in tests/push_recovery_integration.rs
Task T029: Write successful baseline publication tests in tests/baseline_integration.rs
```

## Parallel Example: User Story 3

```text
Task T041: Write the execution fault matrix in tests/push_failure_integration.rs
Task T043: Write process-interruption coexistence tests in tests/push_recovery_integration.rs
```

## Parallel Example: User Story 4

```text
Task T052: Write result-schema and human-parity tests in tests/push_cli_contract.rs
Task T053: Write exit and contention tests in tests/push_contention_integration.rs
```

---

## Implementation Strategy

### MVP First: User Story 1

1. Complete Setup and Foundational work.
2. Implement the pure complete planner and dry-run CLI.
3. Stop and validate US1 independently with exact non-mutation snapshots.
4. Do not begin payload mutation until preview ordering, blockers, and deterministic identity are stable.

### Incremental Delivery

1. **US1**: Complete deterministic preview and blocker reporting.
2. **US2**: Add successful staged mutation, recovery, verification, and accepted baseline publication.
3. **US3**: Harden every failure and interruption boundary without rollback.
4. **US4**: Freeze stable automation and output-delivery contracts.
5. **Polish**: Run traceability, quickstart, full validation, and representative performance gates.

### Recommended Review Boundaries

- Review planner types and matrix before adding filesystem writes.
- Review descriptor and recovery primitives before wiring the executor.
- Review operation-journal transitions before injecting partial failures.
- Review public result schemas before freezing automation tests.
- Run `$speckit-analyze` after this task list and before implementation, as required by the constitution.

## Notes

- `[P]` marks tasks that can proceed without editing the same production file or depending on unfinished behavior.
- Every test uses isolated temporary roots and must not inspect or mutate a real Grip home or user payload.
- Operation IDs and recovery references are opaque public evidence; private storage paths are not public API.
- No task authorizes a commit, push, deletion, deployment, privilege change, or automatic recovery operation.

## Phase 8: Convergence

- [X] T067 CRITICAL Introduce typed filesystem and recovery side-effect outcomes, propagate exact failed phases, visibility, verification, and durability into action checkpoints, and stop on the first failure per Constitution III and FR-014, FR-019, FR-020, FR-029 (partial)
- [X] T068 Route every failure after operation initialization—including final observation, registry/state coordination, candidate construction, baseline publication, and terminal-summary publication—through truthful terminal operation and Push Result handling per FR-019, FR-023, FR-029 (partial)
- [X] T069 Make operation checkpoint and recovery component discovery, reading, verification, replacement, and collision handling descriptor-relative, no-follow, identity-checked, and correctly classified per FR-017, FR-026, FR-029 and the storage contract (partial)
- [X] T070 Complete Push Result and human/JSON rendering parity with stable first-failure reasons and safe paths, outcome-sensitive completion, exact recovery/publication/verification/durability evidence, `not_attempted` preview baselines, and prior successful generation per FR-020, FR-027, FR-028, SC-007, SC-009 (partial)
- [X] T071 Replace planner filesystem reads with synthetic-parent inputs derived from the validated observation and preserve deterministic dependency ordering per FR-003, FR-004 and plan: pure planner (contradicts)
- [X] T072 Add adversarial registry, state, policy, source, destination, ancestry, recovery, staging, publication, verification, journal, baseline, and directory-sync tests with exact last-durable-state assertions, then rerun full validation and the representative performance gate per FR-010, FR-031, SC-004, SC-005 (partial)
