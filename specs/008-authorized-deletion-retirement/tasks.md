# Tasks: Authorized Deletion and Retirement

**Input**: Design documents from `specs/008-authorized-deletion-retirement/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`

**Tests**: The specification explicitly requires isolated automated tests for every payload, authority, recovery, cleanup, concurrency, partial-failure, and non-mutation behavior. Test tasks must be completed first and observed failing before their corresponding implementation tasks.

**Organization**: Tasks are grouped by user story so each story can be implemented and validated as an independent increment after the shared foundation is complete.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it touches different files and has no dependency on another incomplete task in the same phase
- **[Story]**: Maps the task to User Story 1, 2, 3, or 4 from `spec.md`
- Every task names the exact repository file or files it changes

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish the command-owned module boundaries and test scaffolding without changing behavior.

- [X] T001 [P] Create the `delete`, `retire`, and public `recovery` module skeletons and exports in `src/delete/mod.rs`, `src/retire/mod.rs`, `src/recovery/mod.rs`, and `src/lib.rs`
- [X] T002 [P] Add isolated retained-identity, directory-tree, recovery-entry, and fault-injection fixture builders in `tests/support/mod.rs`
- [X] T003 Extend the fixture support from T002 with reusable snapshot assertions proving preview and blocked workflows do not mutate payload, registry, state, recovery, operation, or lock artifacts in `tests/support/mod.rs`

**Checkpoint**: The new modules compile as empty boundaries and tests can construct all Feature 008 fixture shapes.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the shared typed evidence, operation journal, recovery metadata, filesystem, and result primitives required by every user story.

**CRITICAL**: No user-story implementation begins until this phase is complete.

### Foundational tests

- [X] T004 [P] Add strict Recovery Ref grammar, canonical ordering, manifest integrity, tombstone integrity, layout-binding, unknown-field, and malformed-component tests in `src/recovery/model.rs`
- [X] T005 [P] Add operation-record compatibility and interruption tests for typed delete, retire, recovery-restore, and recovery-remove plans, proving every nonterminal record remains immutable, is never resumed, completed, or rolled back, and does not block a freshly inspected operation while preserving historical push, pull, sync, and resolve records in `tests/operation_record_integration.rs`
- [X] T006 [P] Add private recovery publication tests for immutable manifests, legacy Recovery Metadata V1 preservation, collision handling, directory synchronization, and crash-visible staging in `tests/recovery_storage_integration.rs`
- [X] T007 [P] Add descriptor-safe private unlink, empty-directory removal, no-follow traversal, mount-boundary, containment, and restore-staging tests in `tests/recovery_filesystem_integration.rs`
- [X] T008 [P] Add Result Envelope V1 and exit-category tests for all four new operations, including visibility, verification, durability, operation-record, and accepted-authority fields in `src/result.rs`

### Foundational implementation

- [X] T009 Implement `RecoveryRef`, recovery kinds, Recovery Manifest V1, Cleanup Tombstone V1, integrity envelopes, canonical encoders, and strict validators in `src/recovery/model.rs`
- [X] T010 Generalize immutable operation initialization around a typed plan projection and admit command-specific validators for the four new operation kinds in `src/operation/model.rs` and `src/operation/publication.rs`
- [X] T011 Implement immutable Recovery Manifest V1 publication for new payload, Registry V1, and State V2 recovery while preserving all historical recovery bytes and metadata in `src/mutation/recovery.rs`, `src/registry/publication.rs`, and `src/state/publication.rs`
- [X] T012 Implement descriptor-relative file unlink, verified-empty directory removal, private recovery-byte unlink, target staging, parent synchronization, and post-action verification primitives in `src/mutation/filesystem.rs`
- [X] T013 Add stable blocker and operational-failure variants for deletion, retirement, provenance, restore compatibility, cleanup lifecycle, and private evidence corruption in `src/error.rs`
- [X] T014 Extend Result Envelope V1 typed details and human/JSON rendering for deletion authority, retirement force, recovery identities, action milestones, visibility, verification, durability, and accepted authority in `src/result.rs`
- [X] T015 Wire deterministic test-only faults for preservation, unlink, directory sync, absence verification, final observation, registry/state publication, recovery-byte removal, tombstone publication, and result delivery in `src/mutation/filesystem.rs`, `src/mutation/recovery.rs`, `src/registry/publication.rs`, `src/state/publication.rs`, and `src/result.rs`

**Checkpoint**: Shared schemas and primitives pass their focused tests; existing operation records, recovery artifacts, registry documents, State V2 documents, and result consumers remain compatible.

---

## Phase 3: User Story 1 - Preview and Authorize a Directional Deletion (Priority: P1) MVP

**Goal**: Preview and explicitly execute source- or destination-authoritative deletion with complete preflight, child-first removal, verified recovery, and final baseline retirement.

**Independent Test**: In isolated accepted source-side and destination-side deletion fixtures, compare preview and execution plans, verify ordinary synchronization never deletes, confirm unmanaged descendants block the complete plan, and prove successful execution preserves recovery, removes only the remaining peer, verifies both sides absent, and retires exactly the selected baselines.

### Tests for User Story 1

- [X] T016 [P] [US1] Add delete grammar, authority exclusivity, selector, option-terminator, dry-run parity, human/JSON parity, exit-category, and ordinary-command non-deletion tests in `tests/delete_cli_contract.rs`
- [X] T017 [P] [US1] Add exhaustive deletion eligibility, authority-direction, blocker accumulation, plan-identity, raw-byte ordering, child-before-parent dependency, duplicate-selection, and deterministic-input tests in `tests/delete_planning.rs`
- [X] T018 [P] [US1] Add descriptor-safe file and directory deletion tests covering unmanaged descendants, differently classified children, unsupported nodes, mount boundaries, symlink substitution, concurrent children, fresh empty enumeration, and scoped neighbors in `tests/delete_filesystem_integration.rs`
- [X] T019 [P] [US1] Add verified pre-removal payload and directory-node recovery tests, manifest bindings, preservation drift, collision, and recovery-retention assertions in `tests/delete_recovery_integration.rs`
- [X] T020 [P] [US1] Add injected preservation, unlink, sync, absence-verification, final-observation, state-publication, and result-delivery failure tests with completed/failed/unattempted action evidence in `tests/delete_failure_integration.rs`
- [X] T021 [P] [US1] Add mutation-lock contention, stale-owner recovery, lock-held plan drift, registry/state drift, and per-action recreation/replacement tests in `tests/delete_contention_integration.rs`

### Implementation for User Story 1

- [X] T022 [US1] Implement deletion authority, request, disposition, action, plan, blocker, milestone, count, and result types with semantic plan identity validation in `src/delete/model.rs`
- [X] T023 [US1] Implement complete-scope deletion eligibility, blocker aggregation, managed-descendant ownership checks, child-first canonical ordering, dependencies, and deterministic planning in `src/delete/plan.rs`
- [X] T024 [US1] Implement lock-free preview and actionful deletion orchestration with lock-held plan reconstruction, operation checkpoints, per-action recovery/revalidation/removal, final observation, and one State V2 retirement publication in `src/delete/execution.rs`
- [X] T025 [US1] Add `grip delete (--source|--destination) [-n|--dry-run] [--] PATH` parsing and usage validation in `src/cli.rs`
- [X] T026 [US1] Route deletion requests through the command-owned planner/executor and preserve no-delete behavior in push, pull, sync, resolve, baseline, mapping, and ignore workflows in `src/lib.rs`
- [X] T027 [US1] Render equivalent deletion authority, plan, blockers, recovery references, ordered outcomes, visibility, verification, durability, and accepted-state authority in `src/result.rs`

**Checkpoint**: User Story 1 passes all focused tests and is a complete independently demonstrable MVP.

---

## Phase 4: User Story 2 - Retire No-Longer-Managed Entries Deliberately (Priority: P2)

**Goal**: Remove selected accepted history for ignored, untracked, or convergently deleted entries without changing either payload copy.

**Independent Test**: In isolated newly ignored, removed-mapping, converged-deletion, active, and differing-survivor fixtures, verify path and `--all` selection, force-required reporting, exact State V2 publication, semantic no-op behavior, and byte-identical payloads before and after every outcome.

### Tests for User Story 2

- [X] T028 [P] [US2] Add retire grammar, mandatory path-or-all selection, destination-space selector, force, dry-run, no-op, output-parity, and usage rejection tests in `tests/retire_cli_contract.rs`
- [X] T029 [P] [US2] Add exhaustive retirement classification, survivor-comparison, force-required, blocker, scope-ordering, deterministic-plan, and selected-record preservation tests in `src/retire/plan.rs`
- [X] T030 [P] [US2] Add isolated ignored, untracked, converged-deletion, active, stale-policy, stale-membership, contention, State V2 publication, publication-failure, result-delivery, and payload-non-mutation tests in `tests/retire_integration.rs`

### Implementation for User Story 2

- [X] T031 [US2] Implement retirement request, disposition, plan, survivor-difference, force authorization, blocker, and result types with semantic plan identity validation in `src/retire/model.rs`
- [X] T032 [US2] Extend retained ignored and untracked identity observation to capture safe live evidence from both sides without changing membership classification in `src/observation/mod.rs` and `src/classification/mod.rs`
- [X] T033 [US2] Implement exact-path and explicit-all retirement selection, complete eligibility/blocker evaluation, force-required differences, canonical ordering, and no-op detection in `src/retire/plan.rs`
- [X] T034 [US2] Implement state-only retirement execution with mutation coordination, lock-held plan reconstruction, operation checkpoints, cloned-record removal, and exactly one atomic State V2 publication in `src/retire/execution.rs`
- [X] T035 [US2] Add `grip retire [-n|--dry-run] [--destination] [--force] (--all|[--] PATH)` parsing and usage validation in `src/cli.rs`
- [X] T036 [US2] Route retirement requests and render retirement reasons, survivor differences, force state, operation evidence, visibility, durability, and authoritative generation in `src/lib.rs` and `src/result.rs`

**Checkpoint**: User Story 2 independently retires only eligible accepted records and produces zero payload mutations.

---

## Phase 5: User Story 3 - Inspect and Restore Recovery Evidence (Priority: P3)

**Goal**: Enumerate and inspect all retained recovery evidence and safely restore one exact payload, Registry V1, or State V2 entry to its bound target.

**Independent Test**: Produce current and legacy recovery records, verify deterministic list/show output without payload disclosure, preview and execute safe absent/exact-post-state restores, and prove stale, ambiguous, corrupt, incompatible, unsafe, or provenance-incomplete requests block before mutation while valid missing/corrupt authority targets can be repaired from exact compatible bytes.

### Tests for User Story 3

- [X] T037 [P] [US3] Add recovery list/show/restore grammar, exact-reference, option-terminator, operation-reference restriction, dry-run, output-parity, content-nondisclosure, and exit-category tests in `tests/recovery_cli_contract.rs`
- [X] T038 [P] [US3] Add deterministic cross-store enumeration, strict selected-show verification, legacy projection, corrupt-entry, cleaned/missing/incomplete availability, metadata-only list, and no-central-index tests in `tests/recovery_inventory_integration.rs`
- [X] T039 [P] [US3] Add payload restore tests for absent and exact post-action targets, displacement recovery, original binding, parent-before-child directory composition, source-recovery retention, mutation-lock contention and fresh retry, stale evidence, unsafe ancestry, unsupported nodes, injected preservation/staging/publication/verification/result-delivery failures, truthful partial effects, and no baseline acceptance in `tests/recovery_restore_integration.rs`
- [X] T040 [P] [US3] Add exact Registry V1 and State V2 recovery tests for digest/schema/integrity, provenance, ownership graph, path safety, remaining authority, live payload compatibility, missing/corrupt target repair, different-valid-target blocking, mutation/registry/state lock contention, injected staging/publication/verification/result-delivery failures, and truthful visibility and durability in `tests/recovery_authority_restore_integration.rs`

### Implementation for User Story 3

- [X] T041 [US3] Implement public recovery inventory-entry, origin, provenance, integrity, availability, restore-eligibility, and blocker models in `src/recovery/model.rs`
- [X] T042 [US3] Implement read-only descriptor-safe adapters over operation payload, registry, accepted-state, and operation evidence stores with canonical enumeration and conservative legacy projection in `src/recovery/inventory.rs`
- [X] T043 [US3] Implement strict `show` resolution and byte verification that reports metadata and eligibility without printing recovered payload contents in `src/recovery/inventory.rs`
- [X] T044 [US3] Implement payload restore planning and execution with exact bound targets, current post-action equality, displacement recovery, no-follow staging/publication, verification, recovery retention, and no automatic baseline acceptance in `src/recovery/restore.rs`
- [X] T045 [US3] Implement exact Registry V1 restore compatibility checks and strongest-supported publication under `mutation -> registry` lock ordering in `src/recovery/restore.rs` and `src/registry/publication.rs`
- [X] T046 [US3] Implement exact State V2 restore compatibility checks and strongest-supported publication under `mutation -> state` lock ordering in `src/recovery/restore.rs` and `src/state/publication.rs`
- [X] T047 [US3] Add `grip recovery list`, `grip recovery show RECOVERY_REF`, and `grip recovery restore [-n|--dry-run] [--] RECOVERY_REF` parsing and usage validation in `src/cli.rs`
- [X] T048 [US3] Route recovery inspection/restore and render stable references, provenance, integrity, availability, eligibility, bound targets, recovery actions, authority, visibility, verification, and durability in `src/lib.rs` and `src/result.rs`

**Checkpoint**: User Story 3 independently exposes trustworthy recovery inspection and exact bound restoration for every supported recovery kind.

---

## Phase 6: User Story 4 - Clean Recovery Material Explicitly (Priority: P4)

**Goal**: Preview and permanently remove recoverable bytes for explicitly confirmed exact references while retaining immutable manifests, tombstones, and operation provenance.

**Independent Test**: Create multiple payload, registry, and state recovery entries; verify preview parity and confirmation; execute canonical cleanup; prove only selected bytes disappear and show reports `cleaned`; then inject each failure boundary and verify truthful completed/failed/unattempted outcomes plus `cleanup_incomplete` recovery.

### Tests for User Story 4

- [X] T049 [P] [US4] Add recovery-remove grammar, required confirmation, unique exact references, rejected operation references, dry-run parity, canonical ordering, output parity, and no-implicit-selector tests in `tests/recovery_cli_contract.rs`
- [X] T050 [P] [US4] Add cleanup tests for full preflight, active-operation blocking, private containment, byte-count verification, parent sync, immutable tombstones, manifest/provenance survival, already-cleaned/inconsistent states, retry after interrupted tombstone publication, partial failure, and accepted-authority non-interference in `tests/recovery_cleanup_integration.rs`

### Implementation for User Story 4

- [X] T051 [US4] Implement cleanup request, action, plan, blocker, lifecycle milestone, and result models with canonical exact-reference plan identity in `src/recovery/model.rs`
- [X] T052 [US4] Implement cleanup planning with duplicate rejection, kind restrictions, complete selected-entry validation, active-operation checks, byte counts, canonical ordering, and lock-free preview in `src/recovery/cleanup.rs`
- [X] T053 [US4] Implement confirmed cleanup execution with mutation coordination, lock-held plan reconstruction, operation checkpoints, descriptor-bound unlink, synchronization, absence verification, immutable tombstone publication, first-failure stop, and retryable `cleanup_incomplete` handling in `src/recovery/cleanup.rs`
- [X] T054 [US4] Add `grip recovery remove [-n|--dry-run] --confirm RECOVERY_REF...` parsing, exact-reference validation, and usage rejection in `src/cli.rs`
- [X] T055 [US4] Route cleanup and render selected references, byte counts, tombstone state, completed/failed/unattempted actions, operation evidence, verification, and authority non-interference in `src/lib.rs` and `src/result.rs`

**Checkpoint**: User Story 4 independently performs only deliberate bounded cleanup and preserves enough immutable evidence to distinguish cleaned from missing bytes.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Prove feature-wide determinism, performance, documentation accuracy, and regression safety after the desired stories are complete.

- [X] T056 [P] Add 100-run human/JSON deterministic-output and no-mutation regression coverage across delete, retire, recovery restore, and recovery remove in `tests/delete_cli_contract.rs`, `tests/retire_cli_contract.rs`, and `tests/recovery_cli_contract.rs`
- [X] T057 [P] Extend the ignored release harness with documented 10,000-entry deletion-preview and 10,000-entry recovery-inventory fixtures, 100 warm runs, distribution reporting, and p95 assertions in `tests/performance_acceptance.rs`
- [X] T058 Review and update Feature 008 command examples, expected evidence, focused suites, and validation instructions against implemented behavior in `specs/008-authorized-deletion-retirement/quickstart.md`
- [X] T059 Run every focused suite documented in `specs/008-authorized-deletion-retirement/quickstart.md` and reconcile any accepted behavioral or technical discoveries across `specs/008-authorized-deletion-retirement/spec.md`, `specs/008-authorized-deletion-retirement/plan.md`, and `specs/008-authorized-deletion-retirement/tasks.md`
- [X] T060 Run formatting, Clippy, all default tests, release build, and the documented performance acceptance command through `mise.toml`, recording any representative-workstation qualifications in `specs/008-authorized-deletion-retirement/quickstart.md`

**Checkpoint**: The full feature satisfies its contracts, regressions, performance evidence, and merge-bounded artifact consistency.

---

## Dependencies & Execution Order

### Phase dependencies

- **Phase 1 - Setup**: No external dependencies. T001 and T002 touch separate files and may proceed in parallel; T003 follows T002.
- **Phase 2 - Foundational**: Depends on Phase 1 and blocks all user stories. Write T004-T008 before implementing T009-T015.
- **Phase 3 - User Story 1**: Depends only on Phase 2 and is the suggested MVP.
- **Phase 4 - User Story 2**: Depends only on Phase 2; it shares state publication primitives but not User Story 1 behavior.
- **Phase 5 - User Story 3**: Depends only on Phase 2; its fixtures can create recovery directly and do not require directional deletion.
- **Phase 6 - User Story 4**: Depends only on Phase 2; its fixtures can create exact recovery entries directly and do not require restore behavior.
- **Phase 7 - Polish**: Depends on every user story selected for delivery. T057 may run in parallel with documentation review once the measured commands exist.

### User-story dependency graph

```text
Setup -> Foundation -> US1 (P1, MVP)
                    -> US2 (P2)
                    -> US3 (P3)
                    -> US4 (P4)

US1 + US2 + US3 + US4 -> Polish and complete gates
```

The stories are behaviorally independent after Foundation. Priority order remains the recommended single-developer sequence because it delivers the highest-risk user value first and minimizes simultaneous edits to `src/cli.rs`, `src/lib.rs`, and `src/result.rs`.

### Within each user story

- Complete and observe the story's tests failing before implementation.
- Implement typed models before planners and planners before executors.
- Implement filesystem or authority publication behavior before CLI routing that exposes it.
- Re-run the story's focused tests at its checkpoint before starting another story.
- Treat any accepted behavioral discovery as a flow-back update to `spec.md`; technical changes flow back to `plan.md`; required-work changes flow back here.

### Parallel opportunities

- T001 and T002 can proceed in parallel; T003 follows T002 in the same fixture file.
- Foundational tests T004-T008 target separate files and can proceed in parallel.
- US1 tests T016-T021 can proceed in parallel before T022-T027 implementation.
- US2 tests T028-T030 can proceed in parallel before T031-T036 implementation.
- US3 tests T037-T040 can proceed in parallel before T041-T048 implementation.
- US4 tests T049-T050 can proceed in parallel before T051-T055 implementation.
- Once Foundation completes, different developers can implement US1-US4 in parallel, but shared edits to `src/cli.rs`, `src/lib.rs`, and `src/result.rs` must be serialized or coordinated.

---

## Parallel Examples

### User Story 1

```text
Task: T016 Add delete CLI contract tests in tests/delete_cli_contract.rs
Task: T017 Add deletion policy and ordering tests in tests/delete_planning.rs
Task: T018 Add descriptor-safe deletion tests in tests/delete_filesystem_integration.rs
Task: T019 Add deletion recovery tests in tests/delete_recovery_integration.rs
Task: T020 Add deletion failure tests in tests/delete_failure_integration.rs
Task: T021 Add deletion contention tests in tests/delete_contention_integration.rs
```

### User Story 2

```text
Task: T028 Add retirement CLI contract tests in tests/retire_cli_contract.rs
Task: T029 Add retirement policy tests in src/retire/plan.rs
Task: T030 Add retirement integration tests in tests/retire_integration.rs
```

### User Story 3

```text
Task: T037 Add recovery CLI contract tests in tests/recovery_cli_contract.rs
Task: T038 Add recovery inventory tests in tests/recovery_inventory_integration.rs
Task: T039 Add payload restore tests in tests/recovery_restore_integration.rs
Task: T040 Add authority restore tests in tests/recovery_authority_restore_integration.rs
```

### User Story 4

```text
Task: T049 Add recovery-remove CLI tests in tests/recovery_cli_contract.rs
Task: T050 Add recovery cleanup tests in tests/recovery_cleanup_integration.rs
```

---

## Implementation Strategy

### MVP first: User Story 1

1. Complete Setup and Foundation.
2. Complete US1 tests and observe them fail for the expected missing behavior.
3. Implement the typed deletion model, complete planner, child-first executor, CLI, routing, and results.
4. Run the six focused US1 suites and the existing full validation gate.
5. Stop for independent review of explicit authorization, unmanaged-descendant protection, verified recovery, partial-failure truth, and baseline authority.

### Incremental delivery

1. Deliver US1 as the explicit directional-deletion MVP.
2. Add US2 as a state-only membership-retirement increment and revalidate payload non-mutation.
3. Add US3 as a read-only recovery inventory plus exact bound restore increment.
4. Add US4 as deliberate recovery-byte cleanup with immutable tombstones.
5. Complete deterministic-output, performance, quickstart, full validation, and artifact-consistency gates.

### Parallel team strategy

After Foundation, separate developers may own one story each. Coordinate shared interface files explicitly: land story-owned models, planners, executors, and tests first; then serialize the small `src/cli.rs`, `src/lib.rs`, and `src/result.rs` integrations in priority order.

## Notes

- `[P]` means the task can run concurrently with other marked work in its phase; it does not remove test-before-implementation dependencies.
- User-story labels provide requirement traceability and appear only in story phases.
- No task introduces a new dependency, cache, central recovery index, archive format, automatic retention policy, broad payload-tree lock, arbitrary restore destination, or automatic rollback.
- All filesystem tests use isolated temporary roots and never inspect or mutate real user files.
- Before implementation begins, run `$speckit-analyze` as required by the constitution.

## Phase 8: Convergence

- [X] T061 CRITICAL revalidate registry, accepted membership, authoritative absence, remaining-side evidence, target ancestry, baseline, and dependency assumptions immediately before every deletion action in `src/delete/execution.rs` per FR-008 and Constitution III (contradicts)
- [X] T062 CRITICAL finalize every post-initialization delete, retire, recovery-restore, and recovery-remove failure with truthful completed, failed, and unattempted action checkpoints, operation summary, baseline or authority outcome, visibility, verification, durability, recovery references, and operation-record identity in equivalent human and JSON results across `src/error.rs`, `src/result.rs`, `src/delete/execution.rs`, `src/retire/execution.rs`, `src/recovery/restore.rs`, and `src/recovery/cleanup.rs` per FR-029, FR-030, FR-031, SC-006, SC-011, SC-013, and Constitution III (partial)
- [X] T063 perform deterministic full live compatibility planning for payload, Registry V1, and State V2 restore previews, require every non-target authority to validate, and reject missing, corrupt, stale, unsafe, or incompatible evidence before operation initialization in `src/recovery/restore.rs` per FR-021, FR-022, FR-023, FR-024, US3/AC2, US3/AC4, and US3/AC5 (partial)
- [X] T064 verify durable operation-record components instead of inferring integrity from a directory name, and report truthful origin, creation time, integrity, availability, and restore eligibility in `src/recovery/inventory.rs` and `src/operation/model.rs` per FR-018, FR-019, and US3/AC1 (contradicts)
- [X] T065 implement isolated exact-reference recovery lookup so unrelated corrupt store entries do not block `recovery show`, restore planning, or selected cleanup, while preserving deterministic complete inventory behavior in `src/recovery/inventory.rs` per FR-018, FR-026, and plan: exact-reference inspection (partial)
- [X] T066 add isolated integration coverage for per-action deletion drift, stale mutation-lock recovery, mount boundaries and concurrent directory children, ignored and force-authorized retirement, authority-restore lock and fault boundaries, multi-reference cleanup partial failure, operation-evidence corruption, exact-reference isolation, and equivalent failure-result evidence in `tests/delete_contention_integration.rs`, `tests/delete_filesystem_integration.rs`, `tests/retire_integration.rs`, `tests/recovery_authority_restore_integration.rs`, `tests/recovery_cleanup_integration.rs`, `tests/recovery_inventory_integration.rs`, and the Feature 008 CLI contract suites per FR-033 and Constitution V (partial)
