# Tasks: Project-Scoped Initialization and Portable Mappings

**Input**: Design documents from `specs/010-project-scoped-initialization/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`

**Tests**: The specification and constitution require isolated automated coverage for initialization, selection, portable mappings, state isolation and rebinding, all existing synchronization guarantees, dry-run purity, output parity, legacy-interface removal, and performance. Complete each story's tests first and observe the expected failures before implementing that story.

**Organization**: Tasks are grouped by user story so each product increment can be reviewed and tested at its checkpoint. The stories intentionally build in order because later workflows consume the project, selection, and portable identity boundaries established by earlier stories.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it changes different files and has no dependency on another incomplete task in the same phase
- **[Story]**: Maps the task to User Story 1, 2, 3, 4, or 5 from `spec.md`
- Every task names the exact repository file or files it changes

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish compile-safe module and isolated-fixture boundaries without changing supported command behavior.

- [X] T001 Create the `project` module boundary and compile-safe exports in `src/project/mod.rs`, `src/project/init.rs`, and `src/lib.rs`
- [X] T002 [P] Add an isolated `ProjectFixture` builder with separate project root, home root, portable descriptor writer, cwd invocation, explicit-project invocation, legacy `GRIP_HOME` trap, and whole-root snapshot helpers in `tests/support/project.rs` and `tests/support/mod.rs`
- [X] T003 Extend the project fixture with safe-node modes, nested-project layouts, deep ancestor trees, clone/copy layouts, lock holders, and fault-injection hooks in `tests/support/project.rs`

**Checkpoint**: The new modules compile, and tests can construct project-scoped scenarios without inspecting or mutating real user paths.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the portable path, mapping, project context, descriptor, durable identity, schema, and result primitives required by every user story.

**CRITICAL**: No user-story implementation begins until this phase is complete.

### Foundational tests

- [X] T004 [P] Add unit tests for project-relative and home-relative grammar, tree-root `.`, forbidden `.grip`, normalization, traversal, environment expressions, tilde-user forms, and non-UTF-8 inputs in `tests/portable_mapping_model.rs`
- [X] T005 [P] Add strict Descriptor V2 decode, encode, ordering, unknown-field, absolute-path, topology, and Descriptor V1 rejection tests in `tests/registry_integration.rs`
- [X] T006 [P] Add State V4 portable identity, binding, integrity, ordering, generation, unknown-field, and State V1-V3 rejection tests in `tests/state_integration.rs`
- [X] T007 [P] Add portable Mutation Recovery V2, Recovery Manifest V2, and Recovery Metadata V3 identity, integrity, target-role, and old-schema rejection tests in `tests/recovery_storage_integration.rs`
- [X] T008 [P] Add Operation Record V2 portable action identity, endpoint-role, diagnostic-path, integrity, and Operation Record V1 rejection tests in `tests/operation_record_integration.rs`
- [X] T009 [P] Add Result Envelope V1 tests for safe project identity, declared/resolved mapping separation, project discovery reasons, and rebinding outcomes in `src/result.rs`

### Foundational implementation

- [X] T010 Implement `ProjectContext`, `ProjectEvidence`, `ProjectSelection`, derived metadata/state paths, and command-scoped revalidation interfaces in `src/project/mod.rs`
- [X] T011 Narrow home selection to canonical invoking-user destination-home resolution and remove `GRIP_HOME` and default global metadata selection in `src/home.rs`
- [X] T012 Implement `ProjectRelativePath`, `HomeRelativePath`, source/destination selector parsing, canonical serialization, and rooted non-following resolution in `src/path_policy.rs`
- [X] T013 Implement `PortableMapping`, `ResolvedMapping`, canonical tuple identity, and resolved ownership namespaces in `src/mapping.rs`
- [X] T014 Replace Registry V1 with strict deterministic Descriptor V2 using portable mappings and complete resolved-topology validation in `src/registry/mod.rs`
- [X] T015 Replace absolute `MappingSnapshot` authority with portable mapping identity and context-bound entry path construction in `src/observation/model.rs`
- [X] T016 Implement State Envelope V4, `ProjectBindingV1`, portable baseline identities, pending-retirement identity, integrity, and strict rejection of State V1-V3 in `src/state/mod.rs`
- [X] T017 Implement Operation Record V2 and the portable mutation/recovery schema cutovers in `src/operation/model.rs`, `src/mutation/recovery.rs`, and `src/recovery/model.rs`
- [X] T018 Add stable project-selection and rebinding reason values plus shared project and declared/resolved mapping result details in `src/error.rs` and `src/result.rs`

**Checkpoint**: Portable intent, resolved endpoints, project context, local binding, and every durable authority format are explicit and strictly validated without a compatibility reader.

---

## Phase 3: User Story 1 - Initialize a Portable Grip Project (Priority: P1) MVP

**Goal**: Initialize an existing directory with complete version-controllable metadata, exact state exclusion, no local state, and safe exact reinitialization.

**Independent Test**: Initialize an isolated empty directory by cwd and explicit path, verify exact Descriptor V2 and `/state/` bytes plus the reported canonical root, and prove missing, nested, unsafe, partial, concurrent, and non-equivalent cases fail without state, Git, mapping, or payload changes.

### Tests for User Story 1

- [X] T019 [P] [US1] Add CLI contract tests for omitted and explicit targets, exact metadata bytes, initialized/already-initialized output parity, no state creation, and no implicit Git or payload work in `tests/init_cli_contract.rs`
- [X] T020 [P] [US1] Add initialization security tests for missing targets, files, final-component symlinks, unsafe ownership/modes, inaccessible directories, partial metadata, altered `.gitignore`, unsupported config, and nested projects in `tests/project_metadata_security_integration.rs`
- [X] T021 [US1] Add concurrent initialization, atomic visibility, no-replace publication, injected failure, staging cleanup, and concurrent equivalent-winner tests in `tests/project_metadata_security_integration.rs`

### Implementation for User Story 1

- [X] T022 [US1] Implement initialization target selection, canonical target evidence, enclosing-project rejection, and exact existing-layout classification in `src/project/init.rs`
- [X] T023 [US1] Implement private staging, exclusive file creation, exact mode and byte verification, durability sync, atomic no-replace `.grip` publication, winner reload, and invocation-owned cleanup in `src/project/init.rs`
- [X] T024 [US1] Add `grip init [PATH]`, reject inapplicable `--project`, and route initialization without project or home discovery in `src/cli.rs` and `src/lib.rs`
- [X] T025 [US1] Render identical human and JSON initialization outcomes with a safe canonical project root and no generated identifier in `src/result.rs`
- [X] T026 [US1] Run the focused initialization suites and flow accepted behavioral, technical, or work discoveries through `specs/010-project-scoped-initialization/spec.md`, `specs/010-project-scoped-initialization/plan.md`, and `specs/010-project-scoped-initialization/tasks.md`

**Checkpoint**: User Story 1 independently delivers the initialization MVP and creates only commit-eligible project metadata.

---

## Phase 4: User Story 2 - Resolve and Scope Every Command to One Project (Priority: P1)

**Goal**: Resolve one exact project by explicit root or exhaustive ancestor discovery, keep help/version independent, and retain the same context through command execution and result finalization.

**Independent Test**: Invoke project-dependent read-only and dry-run commands from roots, descendants, and unrelated directories against two isolated projects; verify exact selection or fail-closed reasons before mapping, state, payload, or legacy-global access.

### Tests for User Story 2

- [X] T027 [P] [US2] Add implicit root, descendant, 100-level descendant, zero-candidate, multiple-candidate, invalid-inner-boundary, deleted-cwd, and inaccessible-ancestor tests in `tests/project_selection_integration.rs`
- [X] T028 [P] [US2] Add global `--project` placement, relative exact-root, descendant rejection, invalid explicit root, no fallback, and command-applicability tests in `tests/cli_contract.rs`
- [X] T029 [P] [US2] Add help, command-help, clap-version, application-version, and populated legacy `GRIP_HOME` trap tests proving zero project and home discovery in `tests/project_independent_cli_contract.rs`
- [X] T030 [US2] Add root, metadata, descriptor, ignore-file, home, and descriptor-byte substitution tests between selection, dispatch, writer locking, and result delivery in `tests/project_selection_integration.rs`

### Implementation for User Story 2

- [X] T031 [US2] Implement exact explicit selection and exhaustive fail-closed ancestor discovery with stable zero, multiple, invalid-candidate, and changed-project outcomes in `src/project/mod.rs`
- [X] T032 [US2] Add global `--project PATH` parsing and command applicability validation while preserving clap short-circuit behavior in `src/cli.rs`
- [X] T033 [US2] Refactor central dispatch to resolve one optional `ProjectContext`, pass it to every project-dependent handler, and keep init/version independent in `src/lib.rs`
- [X] T034 [US2] Bind `validate` to `.grip/config.toml`, `.grip/.gitignore`, and optional local state under the retained context in `src/lib.rs`
- [X] T035 [US2] Revalidate project and home evidence at the mutation boundary without rediscovery or replacement of the selected context in `src/project/mod.rs`
- [X] T036 [US2] Decorate successes and post-selection failures with the same safe project root while leaving discovery failures and independent commands unbound in `src/result.rs` and `src/error.rs`
- [X] T037 [US2] Preserve the retained context through operation-result delivery so startup, execution, journaling, and final output cannot resolve different projects in `src/lib.rs` and `src/operation/publication.rs`
- [X] T038 [US2] Run the focused selection suites and flow accepted discoveries through `specs/010-project-scoped-initialization/spec.md`, `specs/010-project-scoped-initialization/plan.md`, and `specs/010-project-scoped-initialization/tasks.md`

**Checkpoint**: User Story 2 independently scopes every project-dependent command and leaves project-independent commands completely independent.

---

## Phase 5: User Story 3 - Commit and Reuse Portable Mapping Intent (Priority: P1)

**Goal**: Store normalized project-relative sources and `~` destinations, resolve them per clone and home, expose declared versus resolved values, and reserve `.grip` from payload ownership.

**Independent Test**: Record portable mappings, copy identical committed metadata to different roots and homes, verify correct local endpoints without descriptor changes, and reject every escaping, ambiguous, overlapping, recursive, or metadata-owning declaration before publication or payload access.

### Tests for User Story 3

- [X] T039 [P] [US3] Add mapping add/list/show/remove/inspect tests for portable declarations, deterministic Descriptor V2 bytes, project-relative source lookup, and `~` destination resolution in `tests/portable_mapping_integration.rs`
- [X] T040 [P] [US3] Add absolute, traversal, dot-component, repeated/trailing-separator, environment-expression, tilde-user, non-UTF-8, symlink-escape, and explicit selector rejection tests in `tests/portable_path_security_integration.rs`
- [X] T041 [P] [US3] Add project-root tree, forbidden `.grip` file/tree source, structural metadata pruning before `.gripignore`, metadata selectors, and metadata-overlap tests in `tests/project_reserved_metadata_integration.rs`
- [X] T042 [P] [US3] Add two-root and two-home clone portability, byte-identical descriptor, distinct resolved endpoint, source identity, topology, and output parity tests in `tests/project_clone_portability_integration.rs`
- [X] T043 [P] [US3] Add resolved equal, nested, overlapping, escaping, cross-recursive, duplicate-source, and duplicate-tuple ownership tests in `tests/mapping_topology_integration.rs`

### Implementation for User Story 3

- [X] T044 [US3] Refactor descriptor snapshots, loading, endpoint resolution, stale-byte evidence, recovery preservation, staging, and atomic publication around `ProjectContext` in `src/registry/publication.rs`
- [X] T045 [US3] Refactor mapping add/list/show/remove/inspect to consume portable arguments, publish declarations only, and match source selectors by normalized project-relative identity in `src/lib.rs`
- [X] T046 [US3] Apply source-space and destination-space selector grammars without changing directional operation semantics in `src/path_policy.rs`, `src/push/mod.rs`, `src/pull/mod.rs`, `src/sync/mod.rs`, and `src/resolve/mod.rs`
- [X] T047 [US3] Enforce complete resolved ownership topology and stable portable tuple ordering for descriptor load and publication in `src/mapping.rs` and `src/registry/mod.rs`
- [X] T048 [US3] Prune `.grip` structurally before ignore-policy evaluation and reject metadata membership during discovery and selection in `src/discovery/filesystem.rs`, `src/discovery/ignore_policy.rs`, and `src/discovery/mod.rs`
- [X] T049 [US3] Render declared and resolved mappings as distinct safe values in mapping, discovery, selector, and validation results in `src/result.rs`
- [X] T050 [US3] Run the focused portability suites and flow accepted discoveries through `specs/010-project-scoped-initialization/spec.md`, `specs/010-project-scoped-initialization/plan.md`, and `specs/010-project-scoped-initialization/tasks.md`

**Checkpoint**: User Story 3 independently delivers version-controllable mapping intent that resolves safely in different project and home locations.

---

## Phase 6: User Story 4 - Keep Operational Evidence Local and Isolated (Priority: P2)

**Goal**: Keep all baselines, locks, staging, recovery, and operation records in ignored project-local state and completely revalidate retained state after a filesystem move or copy.

**Independent Test**: Establish state in two clones, prove disjoint state and contention boundaries, copy or move one project with state, and verify eligible rebinding is read-only until final authorized publication while stale, incompatible, incomplete, or contradictory evidence blocks acceptance and mutation.

### Tests for User Story 4

- [X] T051 [P] [US4] Add lazy state creation, exact `.gitignore` behavior, private permissions, all-locks-under-state, no read-only/dry-run artifacts, and legacy-global trap tests in `tests/project_state_layout_integration.rs`
- [X] T052 [P] [US4] Add same-project contention, unrelated-project independence, two-clone baseline, operation, recovery, and state byte-isolation tests in `tests/project_isolation_integration.rs`
- [X] T053 [P] [US4] Add moved/copied root and changed-home tests for complete portable identity resolution, equivalent accepted evidence, `rebind_eligible`, read-only byte preservation, and final binding publication in `tests/project_state_rebinding_integration.rs`
- [X] T054 [US4] Add descriptor drift, missing and ambiguous mapping, source drift, destination drift, unsafe ancestry, incomplete baseline, corrupt binding, and contradictory accepted-evidence blockers in `tests/project_state_rebinding_integration.rs`
- [X] T055 [P] [US4] Add portable operation, unfinished-action, payload recovery, descriptor recovery, accepted-state recovery, restore, cleanup, and stale-target rebinding tests in `tests/project_recovery_rebinding_integration.rs`

### Implementation for User Story 4

- [X] T056 [US4] Relocate mutation, registry, and state locks beneath `.grip/state/locks/` with lazy private directory creation, stable ordering, and project-local contention in `src/state/mutation_lock.rs`, `src/state/lock.rs`, and `src/registry/publication.rs`
- [X] T057 [US4] Refactor accepted-state loading, recovery generations, staging, atomic publication, and binding persistence around the selected state root and State V4 in `src/state/publication.rs`
- [X] T058 [US4] Implement binding comparison, complete in-memory portable identity matching, accepted-entry observation, eligibility/blocker classification, locked revalidation, and no read-time rewrite in `src/state/rebinding.rs`, `src/state/mod.rs`, and `src/lib.rs`
- [X] T059 [US4] Publish and finalize portable Operation Record V2 exclusively beneath the selected project state root in `src/operation/publication.rs`
- [X] T060 [US4] Bind mutation recovery preservation and verification to portable identities and project-relative private references in `src/mutation/recovery.rs`
- [X] T061 [US4] Refactor recovery inventory, restore, and cleanup to resolve references only within selected state and derive live targets from portable identity and endpoint role in `src/recovery/inventory.rs`, `src/recovery/restore.rs`, and `src/recovery/cleanup.rs`
- [X] T062 [US4] Integrate binding eligibility and blockers into baseline acceptance and every state-authorizing command without publishing on read-only or dry-run paths in `src/baseline.rs` and `src/lib.rs`
- [X] T063 [US4] Render deterministic bound, uninitialized, rebind-eligible, and rebind-blocked state with safe current and prior binding evidence in `src/result.rs`
- [X] T064 [US4] Run the focused state and rebinding suites and flow accepted discoveries through `specs/010-project-scoped-initialization/spec.md`, `specs/010-project-scoped-initialization/plan.md`, and `specs/010-project-scoped-initialization/tasks.md`

**Checkpoint**: User Story 4 independently proves clone isolation and safe retained-state rebinding without a project-instance key or global lookup.

---

## Phase 7: User Story 5 - Preserve Existing Synchronization Guarantees Within a Project (Priority: P2)

**Goal**: Carry project context and portable identity through every existing read, plan, mutation, deletion, retirement, recovery, verification, and truthful-publication workflow without cross-project access or weakened safety.

**Independent Test**: Run the complete existing behavioral matrix against two isolated projects and verify unchanged classifications, deterministic plans, dry-run purity, pre-action revalidation, recovery, partial-failure truth, final verification, and accepted-state publication while no workflow touches the other project or a global trap.

### Tests for User Story 5

- [X] T065 [P] [US5] Convert mapping, registry, discovery, `.gripignore`, and topology suites from global-home fixtures to portable project fixtures in `tests/mapping_cli_contract.rs`, `tests/mapping_registry_integration.rs`, `tests/discovery_cli_contract.rs`, `tests/discovery_filesystem_integration.rs`, `tests/gripignore_conformance.rs`, and `tests/mapping_topology_integration.rs`
- [X] T066 [P] [US5] Convert classification, status/check/diff, baseline, metadata migration, and matrix suites to project-scoped State V4 fixtures in `tests/classification_cli_contract.rs`, `tests/classification_filesystem_integration.rs`, `tests/classification_matrix.rs`, `tests/baseline_integration.rs`, and `tests/metadata_migration_integration.rs`
- [X] T067 [P] [US5] Convert push CLI, planning, filesystem, recovery, contention, and failure suites to portable project fixtures in `tests/push_cli_contract.rs`, `tests/push_planning.rs`, `tests/push_filesystem_integration.rs`, `tests/push_recovery_integration.rs`, `tests/push_contention_integration.rs`, and `tests/push_failure_integration.rs`
- [X] T068 [P] [US5] Convert pull CLI, planning, filesystem, recovery, contention, and failure suites to portable project fixtures in `tests/pull_cli_contract.rs`, `tests/pull_planning.rs`, `tests/pull_filesystem_integration.rs`, `tests/pull_recovery_integration.rs`, `tests/pull_contention_integration.rs`, and `tests/pull_failure_integration.rs`
- [X] T069 [P] [US5] Convert sync and resolve CLI, planning, filesystem, recovery, contention, and failure suites to portable project fixtures in `tests/sync_cli_contract.rs`, `tests/sync_planning.rs`, `tests/sync_filesystem_integration.rs`, `tests/sync_recovery_integration.rs`, `tests/sync_contention_integration.rs`, `tests/sync_failure_integration.rs`, `tests/resolve_cli_contract.rs`, `tests/resolve_planning.rs`, and `tests/resolve_filesystem_integration.rs`
- [X] T070 [P] [US5] Convert delete and retire CLI, planning, filesystem, recovery, contention, and failure suites to portable project fixtures in `tests/delete_cli_contract.rs`, `tests/delete_planning.rs`, `tests/delete_filesystem_integration.rs`, `tests/delete_recovery_integration.rs`, `tests/delete_contention_integration.rs`, `tests/delete_failure_integration.rs`, `tests/retire_cli_contract.rs`, and `tests/retire_integration.rs`
- [X] T071 [P] [US5] Convert recovery authority, inventory, storage, restore, cleanup, filesystem, and CLI suites to project-local portable recovery fixtures in `tests/recovery_authority_restore_integration.rs`, `tests/recovery_inventory_integration.rs`, `tests/recovery_storage_integration.rs`, `tests/recovery_restore_integration.rs`, `tests/recovery_cleanup_integration.rs`, `tests/recovery_filesystem_integration.rs`, and `tests/recovery_cli_contract.rs`
- [X] T072 [P] [US5] Convert metadata, APFS capability, unsupported-node, result-delivery, and product acceptance suites to project-scoped fixtures in `tests/metadata_model.rs`, `tests/metadata_filesystem_integration.rs`, `tests/metadata_capability_integration.rs`, `tests/metadata_recovery_integration.rs`, `tests/metadata_cli_contract.rs`, `tests/apfs_name_compatibility_integration.rs`, `tests/filesystem_boundary_integration.rs`, `tests/operation_record_integration.rs`, and `tests/product_acceptance.rs`

### Implementation for User Story 5

- [X] T073 [US5] Propagate `ProjectContext` and resolved mappings through discovery, observation, and classification without changing membership or conflict semantics in `src/discovery/mod.rs`, `src/observation/mod.rs`, `src/observation/fingerprint.rs`, and `src/classification/mod.rs`
- [X] T074 [US5] Propagate project-scoped state and selectors through status, check, diff, and baseline acceptance with complete final revalidation in `src/lib.rs` and `src/baseline.rs`
- [X] T075 [US5] Propagate portable identities and selected state/recovery roots through mutation planning, filesystem actions, execution, verification, and partial-failure reporting in `src/mutation/model.rs`, `src/mutation/plan.rs`, `src/mutation/filesystem.rs`, and `src/mutation/execution.rs`
- [X] T076 [US5] Integrate project-scoped planning, dry-run, mutation, verification, and final State V4 publication for push, pull, sync, and resolve in `src/push/mod.rs`, `src/pull/mod.rs`, `src/sync/mod.rs`, and `src/resolve/mod.rs`
- [X] T077 [US5] Integrate project-scoped portable identities, recovery, dry-run, and publication for explicit deletion and retirement in `src/delete/mod.rs`, `src/delete/model.rs`, `src/delete/plan.rs`, `src/delete/execution.rs`, `src/retire/mod.rs`, `src/retire/model.rs`, `src/retire/plan.rs`, and `src/retire/execution.rs`
- [X] T078 [US5] Preserve Feature 009 metadata, capability, unsupported-node, APFS collision, non-following, and pre-action revalidation behavior under project-resolved endpoints in `src/metadata/mod.rs`, `src/metadata/macos.rs`, `src/discovery/filesystem.rs`, and `src/mutation/execution.rs`
- [X] T079 [US5] Remove remaining production `GripHome`, global config/state paths, absolute durable mapping authority, and post-command rediscovery while preserving independent version/help in `src/lib.rs`, `src/home.rs`, `src/main.rs`, and `src/result.rs`
- [X] T080 [US5] Remove obsolete `command_with_grip_home`, `minimal_home`, absolute registry writers, and global-home fixture branches after all suites use `ProjectFixture` in `tests/support/mod.rs` and `tests/support/project.rs`
- [X] T081 [US5] Run the complete User Story 5 regression matrix and flow accepted discoveries through `specs/010-project-scoped-initialization/spec.md`, `specs/010-project-scoped-initialization/plan.md`, and `specs/010-project-scoped-initialization/tasks.md`

**Checkpoint**: User Story 5 proves that the project correction preserves every established synchronization safety and behavior contract.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Remove the superseded current interface, update authoritative documentation, qualify performance, and complete consistency and convergence gates.

- [X] T082 [P] Replace global setup, configuration, mapping, state, command, and recovery instructions with the project-scoped workflow in `README.md`
- [X] T083 [P] Replace the current global product model, absolute mapping contract, state layout, and command-selection text with Feature 010 behavior in `docs/product-definition.md`
- [X] T084 Ingest Feature 010 and reconcile current project-scoped claims and citations in `wiki/pages/grip-product-model.md`, `wiki/pages/command-line-and-path-selection.md`, `wiki/pages/configuration-and-state.md`, `wiki/pages/mapping-registry-publication.md`, `wiki/pages/mappings-and-managed-membership.md`, `wiki/pages/safety-and-recovery-model.md`, `wiki/pages/deletion-retirement-and-recovery.md`, `wiki/pages/implementation-and-delivery-direction.md`, `wiki/INDEX.md`, and `wiki/sources.md`
- [X] T085 [P] Add a current-interface scan that rejects supported runtime, active-test, README, product-definition, and current-wiki dependence on global registry paths, `GRIP_HOME`, absolute stored mappings, or generated project IDs while excluding `specs/001-*` through `specs/009-*` in `tests/legacy_interface_contract.rs`
- [X] T086 [P] Extend the release performance harness with 100-level discovery, 100 samples, p50/p95/maximum and host/revision evidence, the one-second p95 assertion, and converted 10,000-entry project fixtures in `tests/performance_acceptance.rs`
- [X] T087 Audit project selection, initialization, path resolution, state/recovery binding, and publication for symlink following, unsafe ownership/modes, unbounded locks, unintended state creation, global fallback, absolute restoration authority, and cross-project access in `src/project/mod.rs`, `src/project/init.rs`, `src/path_policy.rs`, `src/registry/publication.rs`, `src/state/publication.rs`, `src/state/rebinding.rs`, `src/operation/publication.rs`, `src/recovery/restore.rs`, and `src/mutation/execution.rs`
- [X] T088 Run every manual scenario and focused command from `specs/010-project-scoped-initialization/quickstart.md` and record exact acceptance and performance evidence in `specs/010-project-scoped-initialization/quickstart.md`
- [X] T089 Run formatting, Clippy with warnings denied, all default tests, release build, explicit ignored APFS suites, interface scan, and performance acceptance through `mise.toml`, recording any accepted artifact changes in `specs/010-project-scoped-initialization/tasks.md`
  - **Accepted qualification changes**: Added destination-home-aware descriptor fixture generation for cross-volume projects and updated ignored Feature 009 product fixtures to use Feature 010 project-relative selectors and the current `synchronized` classification label. All five explicit ignored APFS tests pass on distinct disposable case-sensitive and case-insensitive APFS images.
- [X] T090 Run `$speckit-converge` after implementation and resolve every reported unbuilt requirement until no work remains in `specs/010-project-scoped-initialization/tasks.md`

**Checkpoint**: Feature 010 satisfies the project-scoped contracts, full regression suite, current documentation, interface removal scan, performance target, and merge-bounded consistency gates.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 - Setup**: No dependencies. T001 and T002 can proceed in parallel; T003 follows T002.
- **Phase 2 - Foundational**: Depends on Setup and blocks every user story. Write and observe T004-T009 failing before T010-T018.
- **Phase 3 - User Story 1**: Depends on Foundation and is the suggested MVP.
- **Phase 4 - User Story 2**: Depends on Foundation. Its direct project fixtures do not require the init command, but the recommended delivery sequence follows US1.
- **Phase 5 - User Story 3**: Depends on US2 because CLI mapping workflows require one selected context.
- **Phase 6 - User Story 4**: Depends on US3 because state and recovery identities use portable mappings.
- **Phase 7 - User Story 5**: Depends on US2-US4 because it integrates selection, mappings, and local state across existing workflows.
- **Phase 8 - Polish**: Depends on every story selected for delivery. T090 is the constitutionally required post-implementation convergence gate.

Run the separate `$speckit-analyze` workflow immediately after task generation and before implementation begins or resumes.

### User Story Dependency Graph

```text
Setup -> Foundation -> US1 (P1, initialization MVP)
                    -> US2 (P1, project selection)

US2 -> US3 (P1, portable mappings) -> US4 (P2, local state)
                                  \-> US5 (P2, integrated behavior) <- US4

US1 + US2 + US3 + US4 + US5 -> Polish and complete gates
```

US1 and US2 are independently testable after Foundation. US3, US4, and US5 remain independently testable at their checkpoints once their explicit authority dependencies are present.

### Within Each User Story

- Complete and observe the story's tests failing before implementation.
- Implement domain values before storage and services, storage before command routing, and core behavior before output integration.
- Complete inspection and eligibility checks before mutation, recovery before destructive publication, and final verification before State V4 publication.
- Re-run the story's focused suites at its checkpoint.
- Flow behavioral discoveries to `spec.md`, technical discoveries to `plan.md`, and work changes back to this file before proceeding.

### Parallel Opportunities

- T001 and T002 can proceed in parallel; T003 follows T002.
- Foundational tests T004-T009 target distinct files and can proceed in parallel.
- US1 test files T019 and T020 can proceed in parallel; T021 follows T020 in the shared security suite.
- US2 tests T027-T029 target distinct files; T030 follows T027 in the selection suite.
- US3 tests T039-T043 target distinct files and can proceed in parallel.
- US4 tests T051-T053 and T055 target distinct files; T054 follows T053.
- US5 fixture conversions T065-T072 target disjoint test groups and can proceed in parallel after the project fixture API stabilizes.
- Documentation T082-T083, interface scanning T085, and performance work T086 target distinct files and can proceed in parallel.
- Production changes sharing `src/lib.rs`, `src/project/mod.rs`, `src/result.rs`, `src/registry/publication.rs`, or durable schema types must be serialized or explicitly coordinated.

---

## Parallel Examples

### User Story 1

```text
Task: T019 Add initialization CLI contract tests in tests/init_cli_contract.rs
Task: T020 Add initialization safety tests in tests/project_metadata_security_integration.rs
```

### User Story 2

```text
Task: T027 Add implicit discovery tests in tests/project_selection_integration.rs
Task: T028 Add global --project CLI tests in tests/cli_contract.rs
Task: T029 Add project-independent command tests in tests/project_independent_cli_contract.rs
```

### User Story 3

```text
Task: T039 Add portable mapping workflow tests in tests/portable_mapping_integration.rs
Task: T040 Add portable path security tests in tests/portable_path_security_integration.rs
Task: T041 Add reserved metadata tests in tests/project_reserved_metadata_integration.rs
Task: T042 Add clone portability tests in tests/project_clone_portability_integration.rs
Task: T043 Add resolved topology tests in tests/mapping_topology_integration.rs
```

### User Story 4

```text
Task: T051 Add project-local state layout tests in tests/project_state_layout_integration.rs
Task: T052 Add clone isolation and contention tests in tests/project_isolation_integration.rs
Task: T053 Add eligible rebinding tests in tests/project_state_rebinding_integration.rs
Task: T055 Add recovery rebinding tests in tests/project_recovery_rebinding_integration.rs
```

### User Story 5

```text
Task: T065 Convert mapping and discovery suites to project fixtures
Task: T066 Convert classification and baseline suites to State V4 project fixtures
Task: T067 Convert push suites to project fixtures
Task: T068 Convert pull suites to project fixtures
Task: T069 Convert sync and resolve suites to project fixtures
Task: T070 Convert deletion and retirement suites to project fixtures
Task: T071 Convert recovery suites to portable project-local fixtures
Task: T072 Convert metadata and product acceptance suites to project fixtures
```

---

## Implementation Strategy

### MVP First: User Story 1

1. Complete Setup and Foundation.
2. Write and observe US1 tests failing for the expected missing behavior.
3. Implement atomic, non-repairing initialization and paired output.
4. Stop and validate that only commit-eligible metadata is created and every failure is non-destructive.

### Incremental Delivery

1. Deliver US1 as the initialization MVP.
2. Add US2 as the one-project command authority boundary.
3. Add US3 as portable committed mapping intent and safe runtime resolution.
4. Add US4 as isolated local state and complete copied-state rebinding.
5. Add US5 as the full existing behavior and safety regression cutover.
6. Complete current documentation, wiki ingestion, interface scanning, performance, validation, analysis, and convergence gates.

### Parallel Team Strategy

After Foundation, US1 and US2 tests can be developed concurrently against direct fixtures. Once the selection API stabilizes, US3 test groups can proceed in parallel. After portable identity stabilizes, US4 test groups can proceed in parallel. US5 test conversion is highly parallel by workflow group, but production integrations through shared dispatch, state, result, and recovery modules must be serialized.

## Notes

- `[P]` never removes test-before-implementation dependencies.
- User-story labels appear only in user-story phases for traceability.
- No task authorizes a new dependency, compatibility reader, migration mode, generated project ID, daemon, cache, watcher, persistent index, broad filesystem lock, privilege elevation, Git operation, or global fallback.
- All filesystem tests use disposable project and home roots and a trap legacy global location.
- Historical Features 001–009 remain unchanged; only current product documentation and cited wiki claims are replaced.
- Run `$speckit-analyze` now before implementation, and `$speckit-converge` after implementation.
