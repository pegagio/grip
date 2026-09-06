# Tasks: Baselines, Classification, and Status

**Input**: Design documents from `specs/004-baselines-classification-status/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, and `quickstart.md`

**Tests**: Required by FR-035, the success criteria, and the constitution. Story tests are written first and must fail for the intended missing behavior before implementation begins.

**Organization**: Tasks are grouped by user story so each story produces a distinct, independently testable increment over the shared observation, state, and classification foundations.

## Table of Contents

- [Phase 1: Setup and consistency](#phase-1-setup-and-consistency)
- [Phase 2: Foundational observation and state infrastructure](#phase-2-foundational-observation-and-state-infrastructure)
- [Phase 3: User Story 1 - Understand current synchronization state](#phase-3-user-story-1---understand-current-synchronization-state-priority-p1--mvp)
- [Phase 4: User Story 2 - Establish and refresh accepted evidence](#phase-4-user-story-2---establish-and-refresh-accepted-evidence-priority-p2)
- [Phase 5: User Story 3 - Use classification in automation](#phase-5-user-story-3---use-classification-in-automation-priority-p3)
- [Phase 6: User Story 4 - Explain differences safely](#phase-6-user-story-4---explain-differences-safely-priority-p4)
- [Phase 7: Polish and cross-cutting validation](#phase-7-polish-and-cross-cutting-validation)
- [Dependencies and execution order](#dependencies-and-execution-order)
- [Parallel examples](#parallel-examples)
- [Implementation strategy](#implementation-strategy)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it targets different files and does not depend on incomplete work.
- **[Story]**: Maps a task to User Story 1, 2, 3, or 4.
- Every task names the exact file or files it changes or validates.

## Phase 1: Setup and consistency

**Purpose**: Clear the constitutional implementation gate and establish the approved module boundary without changing dependencies.

- [X] T001 Run `$speckit-analyze` against `specs/004-baselines-classification-status/spec.md`, `specs/004-baselines-classification-status/plan.md`, and `specs/004-baselines-classification-status/tasks.md`; resolve every CRITICAL or HIGH issue in those files before source changes
- [X] T002 Create documented module skeletons in `src/observation/mod.rs`, `src/observation/model.rs`, `src/observation/fingerprint.rs`, `src/classification/mod.rs`, `src/classification/model.rs`, and `src/baseline.rs`, then export `observation`, `classification`, and `baseline` from `src/lib.rs`

**Checkpoint**: The artifacts pass analysis and the approved single-crate module structure compiles without adding a framework or dependency.

---

## Phase 2: Foundational observation and state infrastructure

**Purpose**: Implement strict state evolution, durable mapping validation, safe fingerprints, shared identity, selection, and typed result models that block every user story.

**CRITICAL**: No user story implementation begins until this phase passes its focused tests.

- [X] T003 [P] Add failing V1 predecessor, V2 round-trip/integrity, absent-state, canonical-order, duplicate-key, field-applicability, and corrupt/unsupported-state tests in `tests/state_integration.rs` and `src/state/mod.rs`
- [X] T004 [P] Add failing tests proving durable registry ownership/topology validation succeeds for deleted accepted source endpoints while mapping-add validation still requires source presence in `tests/registry_integration.rs` and `tests/mapping_registry_integration.rs`
- [X] T005 [P] Add failing descriptor-bound streamed SHA-256, permission-mask, length, mtime-diagnostic, unsupported-node no-open, and pre/post-file-drift tests in `src/observation/fingerprint.rs`
- [X] T006 Implement version-neutral `AcceptedState`, strict unchanged State Envelope V1 decoding, strict State Envelope V2 DTOs, canonical integrity, baseline semantic validation, and V2 encoding in `src/state/mod.rs`
- [X] T007 Separate durable canonical mapping/topology validation from current endpoint-presence inspection while preserving add/update evidence checks in `src/path_policy.rs` and `src/registry/publication.rs`
- [X] T008 Implement Mapping Snapshot, Entry Identity, Supported State, Observed Entry, raw-byte ordering, and diagnostic evidence models in `src/observation/model.rs`
- [X] T009 Implement no-follow descriptor-relative and exact-root regular-file opening, streamed SHA-256, ordinary-node revalidation, `0o7777` permission capture, and diagnostic mtime capture in `src/observation/fingerprint.rs` and `src/discovery/filesystem.rs`
- [X] T010 Implement shared source/destination selection normalization for all, mapping, entry, and component-boundary subtree scopes over current mappings and retained baseline identities in `src/observation/model.rs` and `src/path_policy.rs`
- [X] T011 Implement typed Classification Record, changed-dimension triplet, direction, stable 18-category counts, attention/blocking totals, and Classification Result DTOs in `src/classification/model.rs`
- [X] T012 Extend isolated fixture builders and byte/metadata/state snapshot helpers for V1, V2, mappings, baselines, raw paths, modes, and before/after audits in `tests/support/mod.rs`
- [X] T013 Run the focused foundational tests in `src/state/mod.rs`, `src/observation/fingerprint.rs`, `tests/state_integration.rs`, `tests/registry_integration.rs`, and `tests/mapping_registry_integration.rs`; confirm V1 compatibility, V2 rejection boundaries, deleted-root durable validation, and safe hashing all pass

**Checkpoint**: State, identity, selection, fingerprint, and result primitives are explicit, deterministic, and independently verified before classification commands exist.

---

## Phase 3: User Story 1 - Understand Current Synchronization State (Priority: P1) 🎯 MVP

**Goal**: Expose deterministic, read-only `status [PATH]` over the complete current/baseline identity union, including every required classification, selector scope, deletion, ignored retirement, and untracked retirement state.

**Independent Test**: Hand-author isolated absent, V1, and V2 state fixtures covering every source/destination/baseline combination; run all-scope and selected `status`, then verify exact classifications, directions, counts, ordering, stale-evidence rejection, retained untracked records, and byte/metadata-identical registry, state, policy, source, destination, recovery, and Git fixtures.

### Tests for User Story 1

- [X] T014 [P] [US1] Add failing exhaustive table-driven tests for all no-baseline, baseline-present, safety-override, changed-dimension, attention, blocking, and direction outcomes in `tests/classification_matrix.rs`
- [X] T015 [P] [US1] Add failing isolated content-only, mode-only, combined, timestamp-only, converged, deletion, ignored, unsupported, unsafe-collision, raw-path, and mapped-root deletion status tests in `tests/classification_filesystem_integration.rs`
- [X] T016 [P] [US1] Add failing `status` grammar, all/mapping/entry/subtree scope, source/destination path-space, `--`, extra-selector, deleted, ignored, destination-only unmanaged, and outside-scope selector, deterministic human/JSON, full-count, and zero-mutation tests in `tests/classification_cli_contract.rs`

### Implementation for User Story 1

- [X] T017 [US1] Implement the pure exhaustive three-way classifier, supported-state comparison, safety overrides, directions, reasons, and classification ordering in `src/classification/mod.rs` using `src/classification/model.rs`
- [X] T018 [US1] Implement current observation and accepted-baseline union joining, including merge of discovery categories by Entry Identity and newly ignored/untracked pending-retirement derivation, in `src/observation/mod.rs`
- [X] T019 [US1] Refactor Feature 003 discovery into a stable evidence-bearing two-pass observation reused by classification while preserving `mapping inspect` behavior in `src/observation/mod.rs`, `src/discovery/mod.rs`, and `src/discovery/model.rs`
- [X] T020 [US1] Complete selector resolution against current and baseline-only identities, including deleted paths, removed mappings, destination interpretation, component boundaries, ignored/unmanaged reporting, and outside-scope errors in `src/path_policy.rs` and `src/observation/model.rs`
- [X] T021 [US1] Add top-level `status [--destination] [--] [PATH]` grammar with the shared selector argument type in `src/cli.rs`
- [X] T022 [US1] Route status through stable registry/state loading, observation, classification, and typed human/JSON result rendering in `src/lib.rs` and `src/result.rs`
- [X] T023 [US1] Preserve state-blind mapping removal and classify retained complete mapping snapshots as untracked pending retirement without reopening removed ownership in `src/lib.rs`, `src/observation/mod.rs`, and `tests/mapping_registry_integration.rs`
- [X] T024 [US1] Run `tests/classification_matrix.rs`, `tests/classification_filesystem_integration.rs`, `tests/classification_cli_contract.rs`, and mapping/discovery regression suites; confirm User Story 1 passes independently from fixture-created accepted state without invoking baseline acceptance

**Checkpoint**: The MVP provides complete, stable, read-only status and makes all required synchronization states observable without payload or accepted-state mutation.

---

## Phase 4: User Story 2 - Establish and Refresh Accepted Evidence (Priority: P2)

**Goal**: Add explicit state-only `baseline accept [PATH]` with complete eligibility, copy-on-write scope preservation, locked revalidation, idempotent no-op behavior, recovery, and truthful atomic publication.

**Independent Test**: Accept equivalent initial and converged fixtures from absent, V1, and V2 state; verify generation transitions, scoped preservation, no-op behavior, complete rejection, stale revalidation, lock contention, recovery, publication faults, and zero payload/registry/policy/Git mutation.

### Tests for User Story 2

- [X] T025 [P] [US2] Add failing initial acceptance, converged refresh, already-current no-op, empty-scope no-op, scoped copy-on-write preservation, pending-retirement preservation, and whole-request ineligibility tests in `tests/baseline_integration.rs`
- [X] T026 [P] [US2] Add failing expected-state drift, registry drift, ignore-policy and membership drift, file content/metadata drift, ordered lock contention, recovery collision, staging substitution/corruption, pre-rename failure, post-rename visibility, and retry tests in `tests/state_integration.rs`
- [X] T027 [P] [US2] Add failing `baseline accept` grammar, selector, human/JSON accepted/no-op/rejection, generation, state-contention/13, safe-path error, and output-channel tests in `tests/classification_cli_contract.rs`

### Implementation for User Story 2

- [X] T028 [US2] Harden owner/type/mode checks, exact expected snapshots, attempt-owned staging, immutable recovery verification, staged-path identity, candidate decode/equality, rename, directory sync, and truthful post-rename failure in `src/state/publication.rs`
- [X] T029 [US2] Expose bounded nonblocking registry and state publication guards with one registry-then-state acquisition order and stable contention mapping in `src/registry/publication.rs` and `src/state/lock.rs`
- [X] T030 [US2] Implement complete selected eligibility evaluation, full-state copy-on-write candidate construction, exact semantic no-op detection, and generation selection in `src/baseline.rs`
- [X] T031 [US2] Implement locked expected-registry/state reload, fresh selected two-pass observation, candidate rebuild, final registry revalidation, recovery, and state publication transaction in `src/baseline.rs` and `src/state/publication.rs`
- [X] T032 [US2] Add nested `baseline accept [--destination] [--] [PATH]`, route it through the acceptance coordinator, and render typed accepted/already-current/full-rejection results in `src/cli.rs`, `src/lib.rs`, and `src/result.rs`
- [X] T033 [US2] Activate the reserved `state_contention`/13 category and stable stale-baseline/publication reasons while preserving existing error exits and visibility fields in `src/error.rs` and `src/result.rs`
- [X] T034 [US2] Run `tests/baseline_integration.rs`, `tests/state_integration.rs`, `tests/classification_cli_contract.rs`, and Feature 001/002 state/registry regression suites; confirm User Story 2 publishes only accepted Grip state and never mutates payloads

**Checkpoint**: Users can deliberately establish or refresh accepted evidence with scoped, recoverable, race-aware state publication and trustworthy no-op behavior.

---

## Phase 5: User Story 3 - Use Classification in Automation (Priority: P3)

**Goal**: Add `check [PATH]` with a stable completed-attention result distinct from invalid input, corrupt state, unsupported schema, and operational failure.

**Independent Test**: Run human and JSON check over clean, initial-match, addition, drift, conflict, deletion, unsupported, pending-retirement, invalid, corrupt, incompatible, and failed fixtures; verify full results, top-level completion status, symbolic code, exit, diagnostics separation, and zero mutation.

### Tests for User Story 3

- [X] T035 [P] [US3] Add failing clean/0, attention-required/1, invalid-usage/2, invalid-configuration/10, unsupported-schema/11, corrupt-state/12, operational-failure/20, initial-match, and human/JSON parity tests in `tests/classification_cli_contract.rs`

### Implementation for User Story 3

- [X] T036 [US3] Add `AttentionRequired` with status `ok`, code `attention_required`, and exit `1` without changing existing category mappings in `src/error.rs`
- [X] T037 [US3] Add top-level `check [--destination] [--] [PATH]` grammar and route the shared classification result to clean or attention outcome without reimplementing classification in `src/cli.rs` and `src/lib.rs`
- [X] T038 [US3] Render check from the same typed records and counts as status while preserving complete-result status and diagnostic separation in `src/result.rs`
- [X] T039 [US3] Run the check-focused cases in `tests/classification_cli_contract.rs` plus status and baseline regressions; confirm User Story 3 distinguishes attention from failure and leaves all filesystem/state snapshots unchanged

**Checkpoint**: Scripts can consume stable clean, attention, and failure boundaries without parsing prose or losing complete classifications.

---

## Phase 6: User Story 4 - Explain Differences Safely (Priority: P4)

**Goal**: Add `diff [PATH]` that explains supported changed dimensions for all three source/baseline/destination comparisons without exposing content or inventing unavailable comparisons.

**Independent Test**: Run diff over content-only, mode-only, combined, timestamp-only, absent, unsupported, wrong-kind, and converged fixtures; verify `null` versus empty comparison semantics, safe fingerprints, three-way human/JSON parity, deterministic order, exit `0` after complete differences, output failure, and zero mutation.

### Tests for User Story 4

- [X] T040 [P] [US4] Add failing three-comparison, changed-dimension order, `null` versus empty, safe-fingerprint, no-content-leak, complete-difference/0, human/JSON parity, selector, and output-failure tests in `tests/classification_cli_contract.rs` and `tests/classification_filesystem_integration.rs`

### Implementation for User Story 4

- [X] T041 [US4] Add top-level `diff [--destination] [--] [PATH]` grammar and route the shared classification result without a command-specific classifier in `src/cli.rs` and `src/lib.rs`
- [X] T042 [US4] Render source-to-baseline, destination-to-baseline, and source-to-destination dimensions plus safe supported fingerprints while suppressing payload content in `src/result.rs`
- [X] T043 [US4] Preserve unavailable comparison, unsupported evidence, absent-side, and wrong-kind semantics through typed serialization and human rendering in `src/classification/model.rs` and `src/result.rs`
- [X] T044 [US4] Run diff-focused cases in `tests/classification_cli_contract.rs` and `tests/classification_filesystem_integration.rs` plus status/check regressions; confirm User Story 4 explains differences without mutation or content disclosure

**Checkpoint**: All four stories work together while sharing one observation, classification, selection, and result truth.

---

## Phase 7: Polish and cross-cutting validation

**Purpose**: Close compatibility, performance, documentation, flow-back, and acceptance obligations across the complete feature.

- [X] T045 [P] Add whole-feature read-only mutation audits, current `mapping inspect` parity, V1 validation compatibility, mapping add/remove/list/show regressions, 100-run unchanged-evidence determinism checks for status/check/diff, and result-channel separation across `tests/classification_filesystem_integration.rs`, `tests/classification_cli_contract.rs`, `tests/discovery_cli_contract.rs`, `tests/mapping_cli_contract.rs`, and `tests/state_integration.rs`
- [X] T046 [P] Extend the ignored release harness with an accepted 10,000-equivalent-file paired tree, 100 byte-equivalent warm JSON status runs, environment reporting, and two-second p95 assertion in `tests/performance_acceptance.rs`
- [X] T047 [P] Document status, check, diff, baseline acceptance, exit 1 attention, source/destination selectors, first metadata set, State V2 behavior, and pending retirement in `README.md`
- [X] T048 Run every scenario in `specs/004-baselines-classification-status/quickstart.md`, record observed platform and outcomes in that file, and correct any discovered drift across `specs/004-baselines-classification-status/contracts/`
- [X] T049 Run `mise run validate`, then run the ignored release performance harness and record the environment and measurements in `specs/004-baselines-classification-status/quickstart.md`
- [X] T050 Reconcile accepted implementation discoveries through `specs/004-baselines-classification-status/spec.md`, `specs/004-baselines-classification-status/plan.md`, `specs/004-baselines-classification-status/data-model.md`, `specs/004-baselines-classification-status/contracts/`, and `specs/004-baselines-classification-status/tasks.md`, then rerun `$speckit-analyze` until no blocking inconsistency remains
- [X] T051 Run `$speckit-converge` against `specs/004-baselines-classification-status/` after implementation and append or complete any remaining tasks until specification, plan, tasks, contracts, tests, and implementation have no unresolved gaps

---

## Dependencies and Execution Order

### Phase Dependencies

- **Phase 1 — Setup and consistency**: Starts immediately. T001 is the constitutional implementation gate; T002 follows successful artifact analysis.
- **Phase 2 — Foundational infrastructure**: Depends on Phase 1. T003–T005 can begin in parallel. T006 follows T003, T007 follows T004, and T009 follows T005 and T008. T010 depends on T006–T008; T011 can proceed after T002; T012 can proceed after wire/domain shapes stabilize; T013 is the foundation gate.
- **Phase 3 — User Story 1**: Depends on Phase 2. T014–T016 are written and failed first. T017–T23 then follow domain-to-orchestration-to-CLI order, and T024 is the independent story gate.
- **Phase 4 — User Story 2**: Depends on User Story 1's stable observation/classification result. T025–T027 are written and failed first; T028 and T029 establish publication primitives; T030–T033 integrate acceptance; T034 is the story gate.
- **Phase 5 — User Story 3**: Depends on User Story 1's classification result but not on successful baseline mutation; fixtures may provide accepted state directly. T035 fails first, T036–T038 implement check, and T039 is the story gate.
- **Phase 6 — User Story 4**: Depends on User Story 1's classification records but not on User Story 2 or 3 behavior. T040 fails first, T041–T043 implement diff, and T044 is the story gate.
- **Phase 7 — Polish**: Depends on all four stories. Full Feature 004 completion requires T048–T051 after cross-cutting tests, performance, and documentation are ready.

### User Story Dependency Graph

```text
Setup and constitutional analysis
              |
State, durable registry, fingerprints, identity, selection, and result models
              |
US1: Read-only status and complete classification (MVP)
       |                     |                    |
US2: Baseline accept   US3: Automation check   US4: Difference explanation
       |                     |                    |
       +---------------------+--------------------+
                             |
             Polish, validation, analysis, and convergence
```

### User Story Dependencies

- **User Story 1 (P1)**: Begins after foundational infrastructure and has no dependency on another story. Fixture-created accepted state makes it independently testable.
- **User Story 2 (P2)**: Reuses US1 observation and classification to decide eligibility, but independently delivers explicit accepted-state creation and refresh.
- **User Story 3 (P3)**: Reuses US1 classifications and independently adds automation-specific completion semantics; it can use fixture-created baselines without US2.
- **User Story 4 (P4)**: Reuses US1 comparison records and independently adds safe three-way difference presentation; it can use fixture-created baselines without US2 or US3.

### Within Each User Story

- Write contract, matrix, and filesystem tests first and confirm they fail for the intended missing behavior.
- Implement pure domain behavior before filesystem orchestration, command routing, and rendering where file dependencies require it.
- Preserve complete registry/state validation and stable two-pass evidence before returning results.
- Run the independent story gate before treating that story as complete.

### Parallel Opportunities

- T003–T005 target state, registry, and fingerprint boundaries in different files.
- T014–T016 target classifier, filesystem, and CLI tests independently.
- T025–T027 target baseline domain, state publication, and CLI tests independently.
- After US1, US3 and US4 may proceed in parallel because their primary implementation paths separate check category/routing from diff presentation, with coordination required for `src/cli.rs`, `src/lib.rs`, and `src/result.rs`.
- T045–T047 target regression tests, performance, and documentation independently.

## Parallel Examples

### User Story 1

```text
Task T014: Add the exhaustive pure matrix in tests/classification_matrix.rs
Task T015: Add real filesystem classification cases in tests/classification_filesystem_integration.rs
Task T016: Add status grammar and result cases in tests/classification_cli_contract.rs
```

### User Story 2

```text
Task T025: Add acceptance eligibility and scope tests in tests/baseline_integration.rs
Task T026: Add expected-snapshot and publication fault tests in tests/state_integration.rs
Task T027: Add baseline CLI contract tests in tests/classification_cli_contract.rs
```

### User Story 3

```text
Task T035: Add the complete check exit matrix in tests/classification_cli_contract.rs
Task T036: Add AttentionRequired in src/error.rs after T035 fails
```

### User Story 4

```text
Task T040: Add diff contracts in tests/classification_cli_contract.rs and tests/classification_filesystem_integration.rs
Task T043: Implement typed unavailable-comparison semantics in src/classification/model.rs while T042 owns final rendering in src/result.rs
```

## Implementation Strategy

### MVP First: User Story 1

1. Complete T001–T013 to establish artifact consistency and shared state/observation foundations.
2. Write and fail T014–T016.
3. Complete T017–T023.
4. Run T024 and stop at the independently usable read-only status checkpoint.

### Incremental Delivery

1. **US1**: Deliver complete read-only status over fixture-created accepted state.
2. **US2**: Add explicit, scoped, recoverable accepted-state publication.
3. **US3**: Add automation-friendly check behavior without changing classification.
4. **US4**: Add safe three-way difference presentation without changing classification.
5. **Polish**: Prove regressions, performance, documentation, quickstart behavior, artifact consistency, and convergence.

### Parallel Team Strategy

After shared foundations and US1 stabilize, separate implementers may build US2, US3, and US4 against the same typed Classification Result. Changes to `src/cli.rs`, `src/lib.rs`, `src/result.rs`, and `tests/classification_cli_contract.rs` require sequencing or explicit non-overlapping ownership; `[P]` never authorizes concurrent edits to the same file.

## Notes

- `[P]` means different files and no dependency on incomplete work; it does not override phase gates.
- State Envelope V1 remains unchanged and readable; only semantic baseline changes publish V2.
- Mtime remains diagnostic-only; file equality is node kind, SHA-256 content, and `0o7777` permission mode; directory equality is node-kind presence.
- `mapping inspect` remains the Feature 003 membership interface and must not acquire state semantics.
- Tests are mandatory because the feature defines persisted accepted state, filesystem hashing, classification, concurrency, and future mutation boundaries.
- No task may copy, replace, delete, retire, resolve, or otherwise mutate payloads; add automatic recovery, multiple selectors, a daemon, watcher, broad payload lock, cache, persistent index, parallel traversal, or new dependency without accepted flow-back.
- Do not commit, push, publish, merge, run privileged filesystem fixtures, or alter real user files unless the user separately authorizes it.

## Phase 8: Convergence

This phase closes implementation gaps found by the post-implementation convergence audit.

- [X] T052 Harden V2 state staging so verification rereads the attempt-owned descriptor, revalidates pathname identity immediately before rename, and removes only the original attempt-owned inode on failure; add deterministic regular-file and symlink substitution tests that preserve unexpected nodes and prior accepted state per FR-030 and Constitution III (partial)
- [X] T053 Validate `<GRIP_HOME>/state` as a current-user-owned non-symlink directory with exact owner-only mode before treating accepted state as present or absent; add read-only load and command tests for symlinked, wrong-mode, and wrong-kind state directories per FR-032 and the storage plan (partial)
- [X] T054 Emit stable structured operation and reason fields for accepted-state, registry, membership/policy, metadata, and content-evidence drift during classification and baseline publication; add deterministic transaction-level drift tests proving prior state and payload preservation per US2/AC4, FR-024, FR-029, T026, and T033 (partial)
- [X] T055 Render changed dimensions and record reasons, including blocking reasons, in human status, check, and diff output and add semantic parity tests against JSON output per FR-025, T038, and T043 (partial)
- [X] T056 Replace baseline generation-overflow panic with a precise non-success result and add a maximum-generation publication test proving no state, recovery, staging, registry, policy, or payload mutation per FR-030 and FR-032 (partial)

## Phase 9: Convergence

This phase closes the remaining baseline transaction drift gap found after Phase 8.

- [X] T057 Compare the locked semantic classification records with the initial accepted candidate before baseline publication, reject any acceptance-relevant content, mode, policy/membership, registry, or state drift with stable structured reasons, and add deterministic coordinator-level tests proving the prior baseline remains authoritative per US2/AC4, FR-029, and FR-035 (partial)
