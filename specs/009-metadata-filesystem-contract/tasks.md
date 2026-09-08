# Tasks: Metadata and Filesystem Contract Completion

**Input**: Design documents from `specs/009-metadata-filesystem-contract/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`

**Tests**: The specification and constitution explicitly require isolated automated tests for metadata equality, mutation, authorization, recovery, drift, unsupported nodes, filesystem compatibility, migration, output parity, and performance. Complete each story's tests first and observe the expected failures before implementing that story.

**Organization**: Tasks are grouped by user story so each story can be implemented and validated as an independent increment after the shared metadata and storage foundation is complete.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it changes different files and has no dependency on another incomplete task in the same phase
- **[Story]**: Maps the task to User Story 1, 2, 3, or 4 from `spec.md`
- Every task names the exact repository file or files it changes

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish the approved native-binding boundary and isolated macOS/APFS test scaffolding without changing synchronization behavior.

- [X] T001 Present the direct `libc 0.2` dependency proposal for explicit user approval and record the approved direct-dependency or project-owned FFI choice in `specs/009-metadata-filesystem-contract/plan.md`
- [X] T002 [P] Create the platform-neutral metadata module and macOS adapter boundaries in `src/metadata/mod.rs`, `src/metadata/model.rs`, `src/metadata/macos.rs`, and `src/lib.rs`
- [X] T003 Implement the choice approved in T001 by either declaring direct `libc 0.2` use in `Cargo.toml` and `Cargo.lock` or adding only the required project-owned system interface declarations in `src/metadata/macos.rs`
- [X] T004 [P] Add isolated file, directory, xattr, ACL, BSD-flag, timestamp, APFS capability, legacy-state, and fault-injection fixture builders in `tests/support/mod.rs`
- [X] T005 Extend the fixtures from T004 with whole-root snapshots that prove status, diff, dry-run, blocked, and migration-inspection workflows do not mutate payload, registry, state, recovery, operation, or lock artifacts in `tests/support/mod.rs`

**Checkpoint**: The selected native binding strategy is approved and explicit, the new module boundaries compile, and tests can construct every Feature 009 fixture without accessing real user files.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the shared complete-state model, strict schema evolution, recovery evidence, and result primitives required by every user story.

**CRITICAL**: No user-story implementation begins until this phase is complete.

### Foundational tests

- [X] T006 [P] Add unit tests for explicit `Evidence<T>` states, complete file and directory metadata validation, mtime bounds, xattr fingerprints, ordered ACLs, and BSD-flag sets in `tests/metadata_model.rs`
- [X] T007 [P] Add strict State Envelope V3 canonical encoding, integrity, generation, unknown-field, complete-state, V2 legacy decode, no-silent-upgrade, and downgrade-rejection tests in `tests/state_integration.rs`
- [X] T008 [P] Add Recovery Metadata V2 action binding, complete prior-state, xattr payload reference, private-object security separation, integrity, strict decode, and V1 compatibility tests in `tests/recovery_storage_integration.rs`
- [X] T009 [P] Add Operation Record V1 compatibility tests for complete expected metadata, compatibility findings, flag-clear steps, recovery V2 references, and verification details in `tests/operation_record_integration.rs`
- [X] T010 [P] Add Result Envelope V1 tests for metadata dimensions, evidence states, compatibility findings, safe xattr reporting, recovery authority, verification, durability, and accepted-generation fields in `src/result.rs`

### Foundational implementation

- [X] T011 Implement `Evidence<T>`, `MetadataState`, `ModificationTime`, `XattrFingerprint`, ordered ACL, BSD-flag, endpoint-capability, compatibility-finding, collision, transition, and qualification types with strict validation in `src/metadata/model.rs`
- [X] T012 Extend complete `SupportedState`, diagnostic evidence, observed entries, and fingerprint construction for file and directory metadata in `src/observation/model.rs` and `src/observation/fingerprint.rs`
- [X] T013 Implement strict State Envelope V3 encode, decode, integrity, publication, and explicit State V2 legacy-incomplete loading without read-time rewrite in `src/state/mod.rs` and `src/state/publication.rs`
- [X] T014 Implement Recovery Metadata V2 envelopes, complete prior-state bindings, xattr evidence references, and strict Recovery Metadata V1 read compatibility in `src/recovery/model.rs` and `src/mutation/recovery.rs`
- [X] T015 Extend changed dimensions, classification records, action evidence, mutation actions, blockers, plans, and qualification details for complete metadata in `src/classification/model.rs` and `src/mutation/model.rs`
- [X] T016 Extend Operation Record V1 payload validation to admit complete metadata action details while preserving every historical operation record in `src/operation/model.rs` and `src/operation/publication.rs`
- [X] T017 Extend Result Envelope V1 typed details and shared human/JSON projections for metadata evidence, differences, capabilities, recovery, verification, durability, and accepted authority in `src/result.rs`
- [X] T018 Add stable error variants and reason codes for unavailable, unsupported, unreadable, unauthorized, unknown-xattr, protected-flag, metadata-migration, APFS-collision, sparse, hard-link, and mount-boundary findings in `src/error.rs`

**Checkpoint**: Complete metadata can be represented, validated, persisted in State V3, preserved in Recovery Metadata V2, journaled in Operation Record V1, and rendered through Result Envelope V1 while all historical state and recovery fixtures remain readable.

---

## Phase 3: User Story 1 - Synchronize the Promised Metadata Contract (Priority: P1) MVP

**Goal**: Inspect, preview, synchronize, recover, verify, and accept every supported file and directory metadata field with complete-entry conflict semantics and explicit State V2 migration.

**Independent Test**: In isolated APFS roots, change only each supported field on a regular file or directory, exercise push and pull plus converged and divergent changes, verify exact dry-run plans and whole-entry resolution, inject drift and failure boundaries, restore the losing complete state, and confirm State V3 publishes only after complete verification.

### Tests for User Story 1

- [X] T019 [P] [US1] Add canonical and exact-value tests for mode, numeric ownership, nanosecond mtime, the synchronized xattr allowlist, ordered ACL semantics, and supported BSD flags in `tests/metadata_model.rs`
- [X] T020 [P] [US1] Add descriptor-bound file and directory metadata observation tests covering absent and present values, empty and large xattrs, resource forks, unstable xattr reads, ACL inheritance, and nanosecond mtime in `tests/metadata_filesystem_integration.rs`
- [X] T021 [P] [US1] Add complete-state three-way tests for every metadata-only source change, destination change, converged change, divergent conflict, and content-plus-metadata conflict in `tests/classification_matrix.rs`
- [X] T022 [P] [US1] Add State V2 equal-copy migration-ready, unequal-copy migration-conflict, state-only acceptance, source-wins, destination-wins, read-only non-rewrite, failed migration, and verified State V3 publication tests in `tests/metadata_migration_integration.rs`
- [X] T023 [P] [US1] Add deterministic metadata action, immutable/append clearing, complete-scope blocker, whole-entry winner, directory finalizer, deepest-first dependency, and semantic plan-identity tests in `tests/push_planning.rs`, `tests/pull_planning.rs`, `tests/sync_planning.rs`, and `tests/resolve_planning.rs`
- [X] T024 [US1] Add bidirectional metadata-only execution tests for files and directories, full replacement metadata, final directory mtime, accepted State V3 publication, scoped neighbors, and no-op convergence in `tests/metadata_filesystem_integration.rs`
- [X] T025 [P] [US1] Add complete prior-metadata preservation and restore tests for metadata-only changes, replacements, empty and large xattrs, resource forks, ACLs, flags, V2 manifests, and V1 read compatibility in `tests/metadata_recovery_integration.rs`
- [X] T026 [P] [US1] Add per-action drift and injected-failure tests for mode, ownership, mtime, xattr, ACL, flags, staging, recovery, application, verification, durability, final observation, State V3 publication, and result delivery in `tests/push_failure_integration.rs`, `tests/pull_failure_integration.rs`, `tests/sync_failure_integration.rs`, and `tests/resolve_filesystem_integration.rs`
- [X] T027 [P] [US1] Add status, check, diff, baseline-accept, push, pull, sync, resolve, dry-run, field-difference, migration, blocker, and human/JSON parity tests in `tests/metadata_cli_contract.rs`

### Implementation for User Story 1

- [X] T028 [US1] Implement allowlisted xattr enumeration, exact-value reads, bounded size-race retry, ordered ACL reads, BSD-flag reads, numeric ownership, mode, and nanosecond mtime observation in `src/metadata/macos.rs`
- [X] T029 [US1] Integrate complete file and directory metadata observation and fingerprint reuse into managed discovery without following links or reading excluded xattr values as equality state in `src/discovery/filesystem.rs`, `src/observation/mod.rs`, and `src/observation/fingerprint.rs`
- [X] T030 [US1] Implement complete-state equality, field-level changed dimensions, converged two-sided metadata changes, indivisible divergent conflicts, and metadata-only direction selection in `src/classification/mod.rs`
- [X] T031 [US1] Implement `metadata_migration_ready` and `metadata_migration_conflict`, explicit baseline-accept eligibility, whole-entry winner requirements, and no read-only State V3 publication in `src/classification/mod.rs` and `src/baseline.rs`
- [X] T032 [US1] Implement metadata-only actions, complete-state replacement expectations, explicit supported immutable/append clearing, full-scope blocker accumulation, and deepest-first directory finalizer dependencies in `src/mutation/plan.rs`
- [X] T033 [US1] Implement descriptor-bound metadata application in owner/group, ACL, allowlisted-xattr, final-mode, mtime, and final-BSD-flag order with full-state reread and verification in `src/mutation/filesystem.rs` and `src/metadata/macos.rs`
- [X] T034 [US1] Implement complete metadata recovery preservation, private recovery-object xattr retention, Recovery Metadata V2 verification, and complete-state restore in `src/mutation/recovery.rs` and `src/recovery/restore.rs`
- [X] T035 [US1] Integrate metadata actions, flag clearing, per-action revalidation, recovery, directory finalizers, partial-failure evidence, durability, and final State V3 publication into `src/mutation/execution.rs`
- [X] T036 [US1] Route complete metadata through the existing status, check, diff, baseline accept, push, pull, sync, resolve, and recovery workflows without adding commands or flags in `src/lib.rs` and `src/cli.rs`
- [X] T037 [US1] Render equivalent migration state, field differences, complete-entry conflicts, action ordering, recovery references, verification, and accepted State V3 authority in `src/result.rs`
- [X] T038 [US1] Run the focused User Story 1 suites and reconcile any accepted behavioral or technical discovery across `specs/009-metadata-filesystem-contract/spec.md`, `specs/009-metadata-filesystem-contract/plan.md`, and `specs/009-metadata-filesystem-contract/tasks.md`

**Checkpoint**: User Story 1 independently delivers the complete metadata synchronization MVP for ordinary files and directories on qualified APFS roots.

---

## Phase 4: User Story 2 - Understand Filesystem Compatibility Before Mutation (Priority: P2)

**Goal**: Report whether each concrete APFS endpoint can inspect, represent, apply, and verify every selected transition, including authorization and filename compatibility, before any mutation occurs.

**Independent Test**: Exercise eligible and unavailable metadata capabilities, unauthorized ownership, protected metadata, excluded and unknown xattrs, cross-volume APFS mappings, case-sensitive and case-insensitive targets, and canonical Unicode collisions; verify precise read-only findings, complete preflight blocking, and zero side effects.

### Tests for User Story 2

- [X] T039 [P] [US2] Add endpoint capability profile, evidence-state, mtime precision, ACL/xattr/flag support, numeric ownership authorization, and complete-scope blocker tests in `tests/metadata_capability_integration.rs`
- [X] T040 [US2] Add synchronized, excluded, unknown, protected, volatile, empty, large, and race-changing xattr policy tests for files and directories in `tests/metadata_capability_integration.rs`
- [X] T041 [US2] Add same-volume and cross-volume APFS tests proving logical fidelity without hard-link, sparse, clone, compression, or block-layout promises in `tests/metadata_capability_integration.rs`
- [X] T042 [P] [US2] Add case-sensitive, case-insensitive, case-only collision, canonical Unicode-equivalence, raw-byte preservation, multi-identity reporting, and inconclusive-comparator tests in `tests/apfs_name_compatibility_integration.rs`
- [X] T043 [P] [US2] Add paired human/JSON tests for endpoint, path, field, required value, observed capability, reason, blocker status, excluded metadata, safe value nondisclosure, and the corrective user choice available from read-only output in `tests/metadata_cli_contract.rs`

### Implementation for User Story 2

- [X] T044 [US2] Implement operation-scoped APFS filesystem type, identity, mount-flag, returned-attribute, case-behavior, mtime-precision, and metadata-operation capability discovery in `src/metadata/macos.rs`
- [X] T045 [US2] Implement conservative numeric owner and group authorization proofs plus ACL, xattr, mode, mtime, and flag transition eligibility without probe writes or privilege elevation in `src/metadata/mod.rs` and `src/metadata/macos.rs`
- [X] T046 [US2] Implement exact synchronized and excluded xattr policies, unknown-attribute blockers, protected-flag findings, and deterministic raw-name ordering in `src/metadata/mod.rs`
- [X] T047 [US2] Implement endpoint-qualified APFS case and Unicode comparison evidence, exact path-byte preservation, complete collision sets, and inconclusive-capability blocking in `src/metadata/macos.rs` and `src/path_policy.rs`
- [X] T048 [US2] Integrate complete operation-scoped capability and authorization preflight into inspection and mutation planning before operation initialization in `src/observation/mod.rs` and `src/mutation/plan.rs`
- [X] T049 [US2] Render deterministic compatibility profiles, findings, and actionable corrective choices through existing status, check, diff, and dry-run output without claiming other Unix or filesystem support in `src/result.rs`
- [X] T050 [US2] Run the focused User Story 2 suites and reconcile accepted discoveries across `specs/009-metadata-filesystem-contract/spec.md`, `specs/009-metadata-filesystem-contract/plan.md`, and `specs/009-metadata-filesystem-contract/tasks.md`

**Checkpoint**: User Story 2 independently proves eligibility or supplies a precise non-mutating blocker for every selected macOS/APFS transition.

---

## Phase 5: User Story 3 - Reject Unsupported Nodes Without Side Effects (Priority: P3)

**Goal**: Detect unsupported nodes, hard links, sparse files, and nested mounts without following or opening them as payload, while leaving ignored and destination-only unmanaged content untouched.

**Independent Test**: Place every unsupported node and boundary at a managed source, managed destination, ignored source path, and destination-only unmanaged path; inspect and preview mutation; verify exact classification, blocking scope, non-following behavior, and byte-identical unrelated content.

### Tests for User Story 3

- [X] T051 [P] [US3] Add non-following symbolic-link, socket, FIFO, character-device, block-device, whiteout, and unknown-node discovery tests with target-access sentinels in `tests/filesystem_boundary_integration.rs`
- [X] T052 [US3] Add authoritative hard-link and sparse-file detection tests, including post-inspection link-count and sparse-state drift, in `tests/filesystem_boundary_integration.rs`
- [X] T053 [US3] Add explicit mapping-root and nested mounted-directory boundary tests with device/filesystem and mount-status drift in `tests/filesystem_boundary_integration.rs`
- [X] T054 [P] [US3] Add managed-source, managed-destination, ignored-source, destination-only unmanaged, replacement, selected-scope, and unrelated-neighbor blocker tests in `tests/classification_filesystem_integration.rs`
- [X] T055 [P] [US3] Add human/JSON unsupported-node, link-count, sparse, mount, safe-path, blocker, and no-payload-access contract tests in `tests/metadata_cli_contract.rs`

### Implementation for User Story 3

- [X] T056 [US3] Implement non-following node-kind, link-count, authoritative sparse-flag, filesystem identity, and directory mount-status inspection before payload access in `src/discovery/filesystem.rs` and `src/metadata/macos.rs`
- [X] T057 [US3] Implement managed, ignored, and destination-only ownership-aware unsupported-node classification and complete selected-scope blocking in `src/discovery/mod.rs` and `src/classification/mod.rs`
- [X] T058 [US3] Revalidate node kind, link count, sparse indicator, filesystem identity, mount status, and target ancestry immediately before every applicable action in `src/mutation/execution.rs` and `src/mutation/filesystem.rs`
- [X] T059 [US3] Run the focused User Story 3 suites and reconcile accepted discoveries across `specs/009-metadata-filesystem-contract/spec.md`, `specs/009-metadata-filesystem-contract/plan.md`, and `specs/009-metadata-filesystem-contract/tasks.md`

**Checkpoint**: User Story 3 independently rejects every unsupported managed node and boundary without following, copying, preserving as ordinary payload, or mutating it.

---

## Phase 6: User Story 4 - Qualify the Complete Initial Product (Priority: P4)

**Goal**: Produce attributable acceptance evidence for the complete initial Grip product on current macOS with APFS, including correctness, output parity, recovery, and representative performance.

**Independent Test**: Run the documented matrix on recorded case-insensitive and case-sensitive APFS endpoints, execute paired human/JSON fixtures, and measure 100 status and 100 dry-run invocations over the representative 10,000-entry tree; every promised case passes or is explicitly reported as unsupported, unavailable, or below threshold.

### Tests and qualification for User Story 4

- [X] T060 [P] [US4] Add exact macOS version/build, Darwin kernel, APFS bundle, filesystem type, mount flags, capability masks, binary revision, and build-profile capture to qualification fixtures in `tests/support/mod.rs`
- [X] T061 [P] [US4] Add a complete case-insensitive and case-sensitive APFS product acceptance harness covering every classification and applicable mutation direction from Features 001 through 009, including mapping and baseline flows, one-sided and converged changes, conflict and whole-entry resolution, deletion, retirement, recovery, contention, partial failure, metadata-only changes, migration, capability blockers, unsupported nodes, and path collisions in `tests/product_acceptance.rs`
- [X] T062 [P] [US4] Add paired-result fixtures proving equivalent classifications, field differences, capabilities, blockers, actions, verification, recovery, durability, and accepted authority in 100 percent of human/JSON cases in `tests/metadata_cli_contract.rs`
- [X] T063 [P] [US4] Extend the ignored release harness with the documented 10,000-entry metadata fixture, 100 status samples, 100 dry-run samples, p50/p95/maximum reporting, two-second p95 assertions, and separately reported representative synchronization timing in `tests/performance_acceptance.rs`
- [X] T064 [US4] Add deterministic platform qualification report construction and safe serialization for the acceptance harness in `tests/support/mod.rs` and `tests/performance_acceptance.rs`
- [X] T065 [US4] Run the full APFS qualification matrix and record exact environmental passes, unavailable cases, and unsupported cases in `specs/009-metadata-filesystem-contract/quickstart.md`
- [X] T066 [US4] Run the release performance qualification and record fixture composition, sample distribution, host/build evidence, threshold disposition, and synchronization measurement in `specs/009-metadata-filesystem-contract/quickstart.md`

**Checkpoint**: User Story 4 supplies version-bound, reproducible evidence for every initial product promise without presenting deferred platforms or unexercised cases as qualified.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Prove feature-wide determinism, documentation accuracy, regression safety, and merge-bounded artifact consistency.

- [X] T067 [P] Add 100-run deterministic ordering and read-only non-mutation regression coverage across metadata, capability, collision, unsupported-node, and migration results in `tests/metadata_cli_contract.rs` and `tests/metadata_migration_integration.rs`
- [X] T068 [P] Review public command examples, evidence fields, focused suites, qualification commands, and cleanup instructions against implemented behavior in `specs/009-metadata-filesystem-contract/quickstart.md`
- [X] T069 Audit the final implementation for forbidden privilege elevation, shell metadata utilities, link following, probe writes, unapproved platform claims, caches, parallel traversal, persistent indexes, broad locks, and raw metadata disclosure in `src/metadata/`, `src/discovery/`, `src/mutation/`, `src/recovery/`, `src/result.rs`, and `src/cli.rs`
- [X] T070 Run every focused suite documented in `specs/009-metadata-filesystem-contract/quickstart.md` and flow accepted behavioral, technical, or required-work discoveries back through `specs/009-metadata-filesystem-contract/spec.md`, `specs/009-metadata-filesystem-contract/plan.md`, and `specs/009-metadata-filesystem-contract/tasks.md`
- [X] T071 Run formatting, Clippy with warnings denied, all default tests, release build, explicit macOS/APFS ignored suites, and the performance acceptance command through `mise.toml`, recording the final validation evidence in `specs/009-metadata-filesystem-contract/quickstart.md`
- [X] T072 Run `$speckit-converge` after implementation and resolve every reported unbuilt requirement until no remaining work is appended to `specs/009-metadata-filesystem-contract/tasks.md`

**Checkpoint**: Feature 009 satisfies its contracts, regression suite, macOS/APFS qualification, performance evidence, and merge-bounded consistency gates.

---

## Dependencies & Execution Order

### Phase dependencies

- **Phase 1 - Setup**: No code dependency. T001 is the explicit approval gate. T002 and T004 may proceed in parallel while the dependency choice is resolved; T003 follows T001 and T002, and T005 follows T004.
- **Phase 2 - Foundational**: Depends on Phase 1 and blocks every user story. Write and observe T006-T010 failing before implementing T011-T018.
- **Phase 3 - User Story 1**: Depends only on Foundation and is the suggested MVP.
- **Phase 4 - User Story 2**: Depends on Foundation. It can be tested independently with direct metadata-transition fixtures, but the recommended sequence follows US1 because both touch the macOS adapter and mutation planner.
- **Phase 5 - User Story 3**: Depends on Foundation. It can be tested independently with discovery fixtures, but the recommended sequence follows US2 because it reuses endpoint capability evidence.
- **Phase 6 - User Story 4**: Depends on all three behavioral stories because it qualifies the integrated product.
- **Phase 7 - Polish**: Depends on all stories selected for delivery. T072 is the constitutionally required post-implementation convergence gate. Run the separate `$speckit-analyze` workflow immediately after task generation and before implementation begins or resumes.

### User-story dependency graph

```text
Setup -> Foundation -> US1 (P1, MVP)
                    -> US2 (P2)
                    -> US3 (P3)

US1 + US2 + US3 -> US4 (P4 qualification) -> Polish and complete gates
```

US1, US2, and US3 are independently testable after Foundation, although a single developer should follow priority order to minimize simultaneous edits to shared metadata, planning, result, and CLI files. US4 is intentionally integrative because its user outcome is qualification of the complete product.

### Within each user story

- Complete and observe the story's tests failing before implementation.
- Implement typed models before observation services, observation before classification, classification before planning, and planning before execution.
- Complete read-only capability preflight before adding mutation paths.
- Implement preservation before destructive or in-place metadata mutation and final verification before accepted-state publication.
- Re-run the story's focused suites at its checkpoint.
- Flow behavioral discoveries to `spec.md`, technical discoveries to `plan.md`, and work changes back to this file before proceeding.

### Parallel opportunities

- T002 and T004 can proceed in parallel while T001 resolves the binding strategy; T003 follows T001 and T002, and T005 follows T004.
- Foundational tests T006-T010 target separate files and can proceed in parallel.
- US1 tests T019-T023 and T025-T027 target distinct focused suites except for coordinated additions to shared planning and failure files; T024 follows T020 in the same integration file.
- US2 tests T039, T042, and T043 target separate capability, collision, and CLI files; T040 and T041 follow T039 in the shared capability integration file.
- US3 tests T051, T054, and T055 target separate boundary, classification, and CLI files; T052 and T053 follow T051 in the shared boundary integration file.
- US4 qualification fixture, output-parity, and performance tasks T060-T063 can proceed in parallel after US1-US3.
- Production integrations touching `src/metadata/macos.rs`, `src/mutation/plan.rs`, `src/result.rs`, `src/lib.rs`, or `src/cli.rs` must be serialized or explicitly coordinated.

---

## Parallel Examples

### User Story 1

```text
Task: T020 Add descriptor-bound metadata observation tests in tests/metadata_filesystem_integration.rs
Task: T021 Add complete-state three-way tests in tests/classification_matrix.rs
Task: T022 Add State V2 migration tests in tests/metadata_migration_integration.rs
Task: T025 Add complete metadata recovery tests in tests/metadata_recovery_integration.rs
Task: T027 Add metadata CLI contract tests in tests/metadata_cli_contract.rs
```

### User Story 2

```text
Task: T039 Add endpoint capability and authorization tests in tests/metadata_capability_integration.rs
Task: T042 Add APFS name compatibility tests in tests/apfs_name_compatibility_integration.rs
Task: T043 Add compatibility output parity tests in tests/metadata_cli_contract.rs
```

### User Story 3

```text
Task: T051 Add non-following unsupported-node tests in tests/filesystem_boundary_integration.rs
Task: T052 Add hard-link and sparse-file tests in tests/filesystem_boundary_integration.rs
Task: T053 Add nested-mount boundary tests in tests/filesystem_boundary_integration.rs
Task: T054 Add ownership-aware classification tests in tests/classification_filesystem_integration.rs
Task: T055 Add unsupported-node CLI contract tests in tests/metadata_cli_contract.rs
```

### User Story 4

```text
Task: T060 Add platform qualification evidence fixtures in tests/support/mod.rs
Task: T061 Add the Feature 001-009 product acceptance harness in tests/product_acceptance.rs
Task: T062 Add complete paired-result fixtures in tests/metadata_cli_contract.rs
Task: T063 Extend the release performance harness in tests/performance_acceptance.rs
```

---

## Implementation Strategy

### MVP first: User Story 1

1. Resolve the native-binding approval gate and complete Setup.
2. Complete Foundation, including State V3 and Recovery Metadata V2 compatibility.
3. Write and observe the US1 tests failing for the expected missing behavior.
4. Implement complete observation, classification, migration, planning, application, recovery, verification, publication, and output.
5. Run the US1 focused suites and stop for independent review of complete-entry semantics, migration authority, recovery, directory ordering, and State V3 publication.

### Incremental delivery

1. Deliver US1 as the complete metadata synchronization MVP.
2. Add US2 as explicit endpoint capability, authorization, xattr-policy, and APFS collision preflight.
3. Add US3 as comprehensive unsupported-node and boundary rejection without side effects.
4. Add US4 as integrated macOS/APFS correctness and performance qualification.
5. Complete deterministic-output, quickstart, full validation, analysis, and convergence gates.

### Parallel team strategy

After Foundation, separate developers may own US1, US2, and US3 tests concurrently. Serialize production changes through the shared macOS adapter, mutation planner, result renderer, CLI, and library routing. US4 starts only after all three behavior stories are integrated.

## Notes

- `[P]` means a task can run concurrently with other marked work in its phase; it does not remove test-before-implementation dependencies.
- User-story labels provide requirement traceability and appear only in story phases.
- No task authorizes a new framework, service, cache, watcher, persistent index, parallel traversal, broad lock, privilege elevation, shell metadata tool, non-APFS qualification claim, or automatic field-level conflict merge.
- All filesystem tests use isolated disposable roots and never inspect or mutate real user files or Grip home.
- T001 is a real approval boundary, not permission to edit dependency files implicitly.
- Run `$speckit-analyze` now, before implementation, as required by the constitution; run `$speckit-converge` after implementation.

## Phase 8: Convergence

**Purpose**: Close implementation gaps found by the post-implementation cross-artifact audit.

- [X] T073 [US3] Add post-inspection filesystem identity and mount-status drift coverage that proves execution stops before payload mutation in `tests/filesystem_boundary_integration.rs`
- [X] T074 [US3] Add paired human/JSON contract coverage for hard-link, sparse-file, mount-boundary, safe-path, blocker, and no-payload-access evidence in `tests/metadata_cli_contract.rs`
- [X] T075 [US4] Extend the macOS/APFS product acceptance harness with the remaining Feature 001-009 flows named by T061: mapping and baseline, one-sided classification, retirement, contention, partial failure, migration, capability blocking, unsupported nodes, and path collisions in `tests/product_acceptance.rs`
- [X] T076 Run the focused convergence suites, full validation, explicit APFS qualification commands, and `$speckit-converge` again; reconcile every discovery until no additional tasks are appended
