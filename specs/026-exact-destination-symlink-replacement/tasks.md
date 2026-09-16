# Tasks: Exact Destination Symlink Replacement

**Input**: Design documents from `/specs/026-exact-destination-symlink-replacement/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/destination-link-replacement.md`, and `quickstart.md`

**Tests**: Tests are required by FR-012 and the Constitution. Write each story's tests before its implementation and confirm they fail for the intended missing behavior.

**Organization**: Tasks are grouped by user story so each story remains independently testable after the shared no-follow observation foundation is complete.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Extend isolated filesystem fixtures without touching real user paths or link targets.

- [X] T001 Add reusable fixture helpers for creating destination leaf symlinks and snapshotting link objects and target contents in `tests/support/project.rs`.
- [X] T002 Add an isolated fixture helper for private sibling-directory publication and injected pre-rename drift in `tests/support/project.rs`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Represent the narrow, runtime-only final-link observation and carry it safely through classification and planning. No user-story work begins until this phase is complete.

- [X] T003 Add ephemeral `DestinationLeafLinkEvidence` with no-follow object identity and safe-parent evidence in `src/observation/model.rs`.
- [X] T004 Extend exact destination probing to return typed final-leaf link evidence while retaining source, root, and ancestor symlinks as blocking outcomes in `src/observation/fingerprint.rs`.
- [X] T005 Thread exact destination-link observations through managed-identity observation without persisting target data, descriptor state, or State V4 content in `src/observation/mod.rs`.
- [X] T006 Add the `unresolved_destination_link` classification and path-specific blocking reason in `src/classification/model.rs` and `src/classification/mod.rs`.
- [X] T007 Preserve unresolved final-link evidence as baseline-free add/action input and add planned file/directory link-replacement action variants in `src/baseline.rs` and `src/mutation/model.rs`.

**Checkpoint**: Foundation ready—only an exact managed destination leaf can carry runtime-only link evidence; all other link positions remain blockers.

---

## Phase 3: User Story 1 - Record a Mapping with a Destination Link (Priority: P1) 🎯 MVP

**Goal**: Admit source-defined file and tree mappings with an exact destination-link leaf without changing the source, link object, or target, and report the member as unresolved.

**Independent Test**: Add isolated file and tree mappings whose exact paired destinations are symlinks; verify descriptor intent is recorded, payloads and targets are unchanged, and status/dry-run report only the unresolved member and source-side guidance.

### Tests for User Story 1

- [X] T008 [P] [US1] Write failing admission coverage for a regular-file mapping with an exact destination symlink, including descriptor/state nonmutation and target preservation, in `tests/mapping_registry_integration.rs`.
- [X] T009 [P] [US1] Write failing contained-source tree coverage for an exact destination-link member, bounded member reporting, and unchanged link target in `tests/contained_source_tree_integration.rs`.
- [X] T010 [P] [US1] Write failing human and JSON status/dry-run guidance coverage for `unresolved_destination_link`, exact link paths, and source-only force advice in `tests/classification_cli_contract.rs` and `tests/force_resolution_guidance_contract.rs`.

### Implementation for User Story 1

- [X] T011 [US1] Admit only a final destination-link leaf during `add` after a supported source establishes its kind, while rejecting linked roots and ancestors, in `src/path_policy.rs` and `src/lib.rs`.
- [X] T012 [US1] Retain admitted link members without accepted baselines while preserving candidate fencing, all-or-nothing descriptor/state publication, and nonmutating add behavior in `src/baseline.rs` and `src/lib.rs`.
- [X] T013 [US1] Render deterministic unresolved-link diagnostics for status and dry run, including a force command only for a single exact source selector, in `src/result.rs` and `src/lib.rs`.
- [X] T014 [US1] Run and pass the User Story 1 focused suites in `tests/mapping_registry_integration.rs`, `tests/contained_source_tree_integration.rs`, `tests/classification_cli_contract.rs`, and `tests/force_resolution_guidance_contract.rs`.

**Checkpoint**: A mapping can record one exact destination-link obstacle safely; ordinary inspection explains it without reading or changing its target.

---

## Phase 4: User Story 2 - Explicitly Replace One Destination Link (Priority: P2)

**Goal**: Permit one exact, source-winning forced push to atomically replace a revalidated final link object with supported file or directory source state, then publish accepted state only after verification.

**Independent Test**: Select one unresolved source identity and force-push it in isolated file and tree fixtures; prove only the checked link object changes, the former target remains unchanged, and accepted state appears only after a verified replacement.

### Tests for User Story 2

- [X] T015 [P] [US2] Write failing exact source-force coverage for regular-file link replacement, dry-run nonmutation, target preservation, and post-verification baseline publication in `tests/push_filesystem_integration.rs`.
- [X] T016 [P] [US2] Write failing directory-source replacement coverage requiring a staged empty sibling directory, atomic link replacement, descendant application, and target preservation in `tests/contained_source_tree_integration.rs`.
- [X] T017 [P] [US2] Write failing selection and guidance coverage that accepts only one exact source-space `push --force` and rejects destination, mapping, subtree, aggregate, and ambiguous force shapes in `tests/force_resolution_guidance_contract.rs`.
- [X] T018 [P] [US2] Write failing partial-failure coverage proving a failed replacement never publishes accepted state in `tests/push_failure_integration.rs`.

### Implementation for User Story 2

- [X] T019 [US2] Extend exact force selection to admit exactly one active, unbaselined source identity only after current observation confirms `unresolved_destination_link` evidence in `src/lib.rs`.
- [X] T020 [US2] Plan replacement only for `Resolve + Source winner + unresolved destination link`, preserving ordinary-operation and generic unsafe-collision rejection in `src/mutation/plan.rs`.
- [X] T021 [US2] Implement descriptor-relative staging of a regular-file sibling and a verified empty private directory sibling, each published by atomic rename over the checked link object, in `src/mutation/filesystem.rs`.
- [X] T022 [US2] Rebuild under the existing mutation lock, revalidate source, safe ancestry, and final link identity before staging and immediately before rename, then verify and publish accepted state only on success in `src/mutation/execution.rs`.
- [X] T023 [US2] Run and pass the User Story 2 focused suites in `tests/push_filesystem_integration.rs`, `tests/contained_source_tree_integration.rs`, `tests/force_resolution_guidance_contract.rs`, and `tests/push_failure_integration.rs`.

**Checkpoint**: One source-winning exact force can replace a verified file or directory destination-link object without dereferencing its former target or authorizing any other entry.

---

## Phase 5: User Story 3 - Preserve Link Safety Boundaries (Priority: P3)

**Goal**: Keep source links, destination-link ancestors, ordinary operations, unsupported force directions, and stale observations blocked with path-specific diagnostics and no payload mutation.

**Independent Test**: Exercise each unsupported placement and operation, including injected drift, and verify the selected endpoints and former target remain untouched with no false accepted-state publication.

### Tests for User Story 3

- [X] T024 [US3] Write failing source-link and destination-ancestor-link regression coverage for add, inspect, plan, and force paths in `tests/filesystem_boundary_integration.rs`.
- [X] T025 [P] [US3] Write failing ordinary push, pull, sync, delete, and destination-winning force coverage for unresolved destination links in `tests/push_filesystem_integration.rs`, `tests/pull_filesystem_integration.rs`, and `tests/sync_filesystem_integration.rs`.
- [X] T026 [US3] Write failing drift coverage for substituted, removed, or ancestry-invalidated link objects before rename and assert no accepted-state publication in `tests/filesystem_boundary_integration.rs` and `tests/push_failure_integration.rs`.

### Implementation for User Story 3

- [X] T027 [US3] Preserve hard source-link, root-link, and ancestor-link blockers and ensure stale final-link evidence produces a path-specific no-mutation result in `src/observation/fingerprint.rs`, `src/path_policy.rs`, and `src/mutation/execution.rs`.
- [X] T028 [US3] Prevent unresolved destination-link actions from ordinary synchronization, delete, destination-winning force, and non-exact selection paths while retaining deterministic diagnostics in `src/mutation/plan.rs`, `src/lib.rs`, and `src/result.rs`.
- [X] T029 [US3] Run and pass the User Story 3 focused suites in `tests/filesystem_boundary_integration.rs`, `tests/push_filesystem_integration.rs`, `tests/pull_filesystem_integration.rs`, `tests/sync_filesystem_integration.rs`, and `tests/push_failure_integration.rs`.

**Checkpoint**: The exact final-leaf exception does not extend to targets, ancestors, sources, ordinary operations, or stale evidence.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Document the bounded contract, preserve performance evidence, and run repository validation.

- [X] T030 [P] Document exact destination-link admission, source-only forced replacement, target non-interference, and unsupported boundary cases in `README.md` and `docs/product-definition.md`.
- [X] T031 [P] Extend the representative 100-managed-identity and 10,000-unrelated-entry benchmark with exact destination-link diagnostics and bounded-inspection assertions in `tests/performance_acceptance.rs`.
- [X] T032 Validate the quickstart acceptance flow and update command/output expectations if implementation evidence requires it in `specs/026-exact-destination-symlink-replacement/quickstart.md`.
- [X] T033 Run the full formatting, lint, test, and release-build gate defined by `mise.toml` with `mise run validate`.
- [X] T034 Run the supported macOS ARM64 release performance gate defined in `mise.toml` with `mise run performance` and verify its p95 threshold.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies; complete T001 before T002 because both update the shared fixture module.
- **Foundational (Phase 2)**: Depends on the fixture setup; T003 through T007 block every user story.
- **User Story 1 (Phase 3)**: Depends on T003 through T007; it delivers safe admission and diagnostics.
- **User Story 2 (Phase 4)**: Depends on the foundation and User Story 1's unresolved-link classification/output; it adds the only allowed mutation path.
- **User Story 3 (Phase 5)**: Depends on the foundation and exercises the completed force boundary; it can begin test authoring after T007 but its implementation follows T019 through T022.
- **Polish (Phase 6)**: Depends on all story checkpoints.

### User Story Dependencies

- **US1 (P1)**: The MVP; requires only the foundational no-follow observation and classification work.
- **US2 (P2)**: Builds on US1's admitted unresolved entry and exact source-side guidance.
- **US3 (P3)**: Protects the final behavior around US1 and US2; it must not relax their narrow exception.

### Parallel Opportunities

- T008 through T010 and T015 through T018 are independent test files and can proceed in parallel within their story.
- T025 can proceed in parallel with either sequential `tests/filesystem_boundary_integration.rs` task (T024 or T026), but T024 and T026 must not run concurrently.
- T030 and T031 can proceed after the story implementation stabilizes.

## Parallel Example: User Story 2

```text
Task: "Write regular-file exact-force coverage in tests/push_filesystem_integration.rs"
Task: "Write directory exact-force coverage in tests/contained_source_tree_integration.rs"
Task: "Write force selector coverage in tests/force_resolution_guidance_contract.rs"
Task: "Write partial-failure coverage in tests/push_failure_integration.rs"
```

## Phase 7: Convergence

- [X] T035 Add isolated pre-rename regressions for a removed final destination link and invalidated destination ancestry, proving payloads and State V4 remain unpublished per FR-006, FR-012, and SC-004 (partial).

## Implementation Strategy

### MVP First

1. Complete T001 through T007 to establish the runtime-only, no-follow evidence boundary.
2. Complete T008 through T014 and validate safe admission plus deterministic diagnostics.
3. Pause for acceptance of the nonmutating admission behavior before enabling replacement.

### Incremental Delivery

1. Add exact final-leaf admission and report it as unresolved (US1).
2. Add source-only exact force with atomic file and directory publication (US2).
3. Prove every adjacent link and operation boundary remains blocked (US3).
4. Run documentation, performance, and repository validation gates.

### Parallel Team Strategy

After T007, one contributor can finish US1 while another writes US2's isolated force tests and a third writes US3's negative-boundary tests. Merge the tests against the shared typed evidence before applying the sequential force and safety implementations.
