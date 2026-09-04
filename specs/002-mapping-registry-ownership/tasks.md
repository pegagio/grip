# Tasks: Mapping Registry and Ownership Validation

**Input**: Design documents from `specs/002-mapping-registry-ownership/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), and [quickstart.md](quickstart.md)

**Tests**: Tests are required by FR-020 and SC-001 through SC-005. Write each story's tests first, confirm they fail for the missing behavior, then implement the story.

**Organization**: Tasks are grouped by user story so add, inspect, and remove behavior can be built and validated as independent increments over shared registry infrastructure.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it changes different files and does not depend on incomplete work
- **[Story]**: Maps the task to User Story 1, 2, or 3
- Every task names the exact repository path it changes or validates

## Phase 1: Setup and Consistency Gate

**Purpose**: Confirm artifact consistency and prepare the existing single-crate layout without adding dependencies.

- [x] T001 Run `$speckit-analyze` across `specs/002-mapping-registry-ownership/spec.md`, `specs/002-mapping-registry-ownership/plan.md`, and `specs/002-mapping-registry-ownership/tasks.md`; resolve every blocking finding before T002
- [x] T002 Move `src/registry.rs` to `src/registry/mod.rs`, preserve Feature 001 behavior, and verify the crate still compiles before declaring new modules
- [x] T003 [P] Extend isolated mapping fixture and filesystem snapshot helpers in `tests/support/mod.rs` without reading the real Grip home or developer files

---

## Phase 2: Foundational Registry Infrastructure

**Purpose**: Establish shared domain, path, registry, publication, and error boundaries required by all three stories.

**⚠️ CRITICAL**: No user-story implementation begins until this phase is complete.

- [x] T004 [P] Define `MappingKind`, `Mapping`, `Namespace`, component-aware path relations, and deterministic `OwnershipConflict` ordering in `src/mapping.rs`
- [x] T005 [P] Implement absolute UTF-8 input validation, parent-traversal rejection, longest-existing-prefix canonicalization, non-following endpoint inspection, and revalidation evidence in `src/path_policy.rs`
- [x] T006 Declare `src/mapping.rs`, `src/path_policy.rs`, and `src/registry/mod.rs` from `src/lib.rs`, then implement strict Registry V1 mapping wire types plus deterministic decode/encode conversion in `src/registry/mod.rs` and verify the crate compiles
- [x] T007 Implement complete accepted-registry and candidate validation, including all source/source, destination/destination, and cross-side topology checks, in `src/registry/mod.rs`
- [x] T008 Implement the stable owner-only `.registry.lock`, accepted-byte and path-evidence revalidation, content-addressed prior-registry recovery creation/reuse/verification, same-directory staging, exact safe-mode preservation, rename, directory sync, and fault-injection seams in `src/registry/publication.rs`
- [x] T009 [P] Add structured mapping failure reasons and mapping-aware result-detail builders while preserving Feature 001 categories and exits in `src/error.rs` and `src/result.rs`

**Checkpoint**: Complete registries can be decoded, canonicalized, validated, rendered deterministically, and atomically published without any user-story command being wired.

---

## Phase 3: User Story 1 - Declare Mapping Intent Safely (Priority: P1) 🎯 MVP

**Goal**: Add valid file and tree mapping intent with verified prior-registry recovery but without discovery, payload changes, or synchronization-state changes, and reject invalid paths or ownership graphs before publication.

**Independent Test**: In isolated roots, add one file mapping and one tree mapping, decode the accepted registry, and verify the exact canonical tuples, lock boundary, prior-registry recovery generations, and zero payload or synchronization-state mutation; reject every required unsafe path and topology without changing accepted bytes.

### Tests for User Story 1

> Write these tests first and confirm they fail for missing Feature 002 behavior.

- [x] T010 [P] [US1] Add failing parser and human/JSON add-command contract tests for file and tree variants, argument counts, and `--` termination in `tests/mapping_cli_contract.rs`
- [x] T011 [P] [US1] Add failing canonicalization, absent-destination, endpoint-symlink, node-kind, non-UTF-8, traversal, and path-evidence tests in `tests/mapping_registry_integration.rs`
- [x] T012 [P] [US1] Add failing exhaustive equal, containment, overlap, component-boundary, and transitive cross-mapping rejection tests in `tests/mapping_topology_integration.rs`
- [x] T013 [US1] Add failing atomic-publication, deterministic TOML, accepted-byte staleness, path-evidence staleness, lock contention, prior-registry recovery creation/reuse/collision/failure, interruption, unsafe-node, exact-mode preservation, and zero-unintended-mutation tests in `tests/mapping_registry_integration.rs`

### Implementation for User Story 1

- [x] T014 [P] [US1] Add the nested `mapping add file` and `mapping add tree` parser types and conversion to Grip-owned commands in `src/cli.rs`
- [x] T015 [US1] Implement add-candidate construction, duplicate detection, complete ownership validation, and intent-only publication orchestration in `src/lib.rs`
- [x] T016 [US1] Render add success and all specified stable failure details in human and JSON modes through `src/result.rs`
- [x] T017 [US1] Run the User Story 1 tests in `tests/mapping_cli_contract.rs`, `tests/mapping_registry_integration.rs`, and `tests/mapping_topology_integration.rs`; confirm the accepted registry changes only on valid add and no payload or synchronization-state path changes

**Checkpoint**: User Story 1 is independently usable as the MVP mapping-intent writer.

---

## Phase 4: User Story 2 - Inspect Mapping Ownership (Priority: P2)

**Goal**: List every mapping deterministically or show exactly one mapping selected by canonical source identity without changing any file.

**Independent Test**: Load a fixture registry directly, invoke list and show in both output modes, and verify canonical ordering, complete tuples, distinct not-found behavior, complete-registry validation, and a byte-for-byte unchanged filesystem snapshot.

### Tests for User Story 2

> Write these tests first and confirm they fail for missing inspection behavior.

- [x] T018 [US2] Add failing parser and human/JSON contracts for `mapping list` and `mapping show SOURCE` in `tests/mapping_cli_contract.rs`
- [x] T019 [US2] Add failing deterministic ordering, empty-list, exact canonical-source selection, not-found, and invalid-unrelated-mapping tests in `tests/mapping_registry_integration.rs`

### Implementation for User Story 2

- [x] T020 [US2] Add `mapping list` and `mapping show` parser types and Grip-owned command conversion in `src/cli.rs`
- [x] T021 [US2] Implement read-only list/show services over a fully validated registry and return complete mapping values or stable not-found details in `src/lib.rs`
- [x] T022 [US2] Run the User Story 2 tests in `tests/mapping_cli_contract.rs` and `tests/mapping_registry_integration.rs`; confirm deterministic results and zero filesystem mutation without relying on User Story 1 to create fixtures

**Checkpoint**: User Story 2 can be demonstrated independently from a hand-authored valid registry.

---

## Phase 5: User Story 3 - Remove Intent Without Removing Data (Priority: P3)

**Goal**: Atomically remove exactly one mapping selected by canonical source identity while preserving both payload sides and all synchronization state.

**Independent Test**: Load a fixture registry, remove one mapping, and verify only the accepted registry tuple changes; missing identity, stale evidence, contention, and publication failure preserve the prior registry and every payload/state path.

### Tests for User Story 3

> Write these tests first and confirm they fail for missing removal behavior.

- [x] T023 [US3] Add failing parser and human/JSON contracts for `mapping remove SOURCE`, including `--` termination and missing arguments, in `tests/mapping_cli_contract.rs`
- [x] T024 [US3] Add failing successful removal, missing identity, invalid pre-existing registry, stale evidence, contention, prior-registry recovery, recovery-failure, publication-failure, and zero-payload/synchronization-state-mutation tests in `tests/mapping_registry_integration.rs`

### Implementation for User Story 3

- [x] T025 [US3] Add the `mapping remove` parser type and Grip-owned command conversion in `src/cli.rs`
- [x] T026 [US3] Implement remove-candidate construction, full pre/post candidate validation, atomic publication reuse, and removed-tuple results in `src/lib.rs`
- [x] T027 [US3] Run the User Story 3 tests in `tests/mapping_cli_contract.rs` and `tests/mapping_registry_integration.rs`; confirm only registry intent changes and prior accepted bytes survive every rejected operation

**Checkpoint**: All three mapping lifecycle stories are independently functional and jointly compatible.

---

## Phase 6: Polish and Cross-Cutting Validation

**Purpose**: Close performance, documentation, regression, and end-to-end acceptance obligations across all stories.

- [x] T028 [P] Extend the ignored release harness with deterministic validation/listing of 1,000 mappings over 100 warm runs in `tests/performance_acceptance.rs`
- [x] T029 [P] Document mapping syntax, canonical source identity, UTF-8 path policy, intent-only add/remove behavior, canonical TOML rewrites, accepted-registry mode policy, registry recovery, and the registry lock in `README.md`
- [x] T030 Add focused module-level tests for relation boundaries, conflict ordering, canonical serialization, and structured mapping details beside `src/mapping.rs`, `src/path_policy.rs`, `src/registry/mod.rs`, and `src/result.rs`
- [x] T031 Run every scenario in `specs/002-mapping-registry-ownership/quickstart.md`, including prior-registry recovery and exact safe-mode checks, record observed outcomes in its validation-results section, and correct any documentation drift
- [x] T032 Run `mise run validate` against `Cargo.toml`, `src/`, and `tests/`, then run the ignored release performance harness and record the environment and measurements in `specs/002-mapping-registry-ownership/quickstart.md`
- [x] T033 Reconcile any implementation discoveries through `specs/002-mapping-registry-ownership/spec.md`, `specs/002-mapping-registry-ownership/plan.md`, `specs/002-mapping-registry-ownership/contracts/`, and `specs/002-mapping-registry-ownership/tasks.md`, then rerun `$speckit-analyze` until no blocking inconsistency remains
- [x] T034 Run `$speckit-converge` against `specs/002-mapping-registry-ownership/` after implementation and append/complete any remaining work until specification, plan, tasks, tests, and implementation have no unresolved gaps

---

## Dependencies and Execution Order

### Phase Dependencies

- **Phase 1 — Setup and consistency**: Starts immediately; T001 must clear before source changes, while T003 can proceed after the gate independently of T002.
- **Phase 2 — Foundational infrastructure**: Depends on T002 and T003. T004, T005, and T009 can run in parallel; T006 depends on T004 and T005; T007 depends on T006; T008 depends on T005, T006, T007, and T009.
- **Phase 3 — User Story 1**: Depends on Phase 2. Tests T010–T013 precede implementation T014–T016; T017 is the story gate.
- **Phase 4 — User Story 2**: Executes after User Story 1 to avoid concurrent edits to shared CLI and integration-test files. Hand-authored fixtures keep inspection independently testable without invoking add.
- **Phase 5 — User Story 3**: Executes after User Story 2 to avoid concurrent edits to shared CLI and integration-test files. Hand-authored fixtures keep removal independently testable without invoking add.
- **Phase 6 — Polish**: Depends on whichever user stories are included in the delivery; full feature completion requires all three.

### User Story Dependency Graph

```text
Setup and consistency
        |
Foundational registry infrastructure
        |
        |
US1: Add mapping intent (MVP)
        |
US2: Inspect mappings independently from fixtures
        |
US3: Remove mapping intent independently from fixtures
        |
Polish, analysis, convergence, and acceptance
```

### Within Each User Story

- Write contract and integration tests first and confirm they fail for the intended missing behavior.
- Reuse only completed foundational domain, path, registry, publication, and result boundaries.
- Implement parser conversion before wiring command execution.
- Implement domain/service behavior before final result rendering.
- Run the independent story gate before moving to the next sequential priority.

### Parallel Opportunities

- T004, T005, and T009 operate on separate foundational modules.
- T010, T011, and T012 cover distinct User Story 1 test surfaces before T013 joins publication cases.
- T014 can proceed alongside the service-side implementation after the failing tests exist.
- User Stories 2 and 3 remain independently testable from hand-authored fixtures but execute sequentially because they share CLI and integration-test files with earlier stories.
- T028 and T029 can proceed in parallel after the story behavior stabilizes.

## Parallel Examples

### User Story 1

```text
Task T010: Add add-command contract tests in tests/mapping_cli_contract.rs
Task T011: Add mapping path integration tests in tests/mapping_registry_integration.rs
Task T012: Add exhaustive topology tests in tests/mapping_topology_integration.rs
```

User Stories 2 and 3 intentionally have no file-safe parallel task pairs; execute each phase in task order.

## Implementation Strategy

### MVP First: User Story 1

1. Complete T001–T009 to establish the consistency and shared safety boundaries.
2. Write and fail T010–T013.
3. Complete T014–T016.
4. Run T017 and stop at the independently usable add-intent checkpoint.

### Incremental Delivery

1. **US1**: Add validated file and tree intent without payload mutation.
2. **US2**: Inspect complete ownership deterministically from existing registry data.
3. **US3**: Remove intent atomically without removing data.
4. **Polish**: Measure the simple design, document the full lifecycle, run the quickstart, and close consistency gates.

### Parallel Team Strategy

Use parallelism only within the explicitly marked foundational, User Story 1, and polish tasks. Execute User Stories 2 and 3 sequentially because they intentionally extend shared `src/cli.rs`, `src/lib.rs`, and integration-test files.

## Notes

- `[P]` means different files and no dependency on incomplete work; it does not override the stated phase gates.
- Tests are first-class tasks because this feature changes persistent user intent and validates filesystem ownership boundaries.
- `config.toml` remains the only accepted registry; `.registry.lock` and `.config.tmp-*` never contain accepted mapping data.
- Do not add `toml_edit`, a path-indexing structure, async execution, broad locks, tree traversal, payload mutation, or synchronization state to satisfy these tasks.
- Do not commit, push, publish, or merge unless the user separately authorizes it.

## Phase 7: Convergence

**Purpose**: Close implementation and acceptance-evidence gaps found by the post-implementation convergence audit.

- [x] T035 CRITICAL Refactor registry loading and publication in `src/lib.rs`, `src/path_policy.rs`, `src/registry/publication.rs`, and focused tests so descriptor-backed non-following registry/path evidence is captured once, redundant endpoint inspection is eliminated, and accepted bytes, node safety, and path evidence are revalidated at the final pre-rename boundary per Constitution III, Constitution V, FR-016, and FR-025 (partial)
- [x] T036 Render deterministic conflicting canonical paths and relationships for human-mode ownership failures in `src/result.rs` and cover them in `tests/mapping_cli_contract.rs` per FR-013, FR-018, and T016 (partial)
- [x] T037 Normalize invalid-registry and registry-publication failures into stable operation, reason, kind, and safe-path JSON details, and represent post-rename directory-sync failure as visible-but-not-durability-confirmed in `src/error.rs`, `src/registry/publication.rs`, `src/result.rs`, and integration tests per FR-017, FR-019, T009, and T016 (partial)
- [x] T038 Decode the staged registry and require semantic equality with the candidate, then add final-reread, rename, and directory-sync fault seams and preservation assertions in `src/registry/publication.rs` and `tests/mapping_registry_integration.rs` per the plan registry-publication decision, T008, and T013 (partial)
- [x] T039 Extend isolated filesystem snapshots and lifecycle tests in `tests/support/mod.rs` and `tests/mapping_registry_integration.rs` to compare payload content and metadata plus synchronization, baseline, and registry state across successful and rejected add/remove cases, including proof that stale publication preserves the newer accepted registry per SC-003, SC-004, T013, and T024 (partial)
- [x] T040 Add a complete file/tree and human/JSON lifecycle output matrix with exact canonical kind, source, and destination assertions for add, list, show, and remove in `tests/mapping_cli_contract.rs` per SC-001, T010, T018, and T023 (partial)
- [x] T041 Add a table-driven topology conformance matrix covering equal paths, duplicate identities, source and destination containment in both operand orders and kind combinations, overlap, component boundaries, and cross-mapping recursion in `tests/mapping_topology_integration.rs` per SC-002 and T012 (partial)
- [x] T042 Compare every measured 1,000-mapping list result with the warmed canonical result while retaining the 100-run p95 timing assertion in `tests/performance_acceptance.rs` per SC-005 and T028 (partial)
- [x] T043 Add reader/writer registry permission-policy coverage for read-only acceptance, mutation rejection without owner-write permission, unsafe modes, symlinks, wrong node types, and ownership where testable in `tests/mapping_registry_integration.rs` per FR-025 and T013 (partial)
- [x] T044 Add an independent empty-registry `mapping list` success and zero-mutation assertion in `tests/mapping_registry_integration.rs` per T019 (missing)

## Phase 8: Convergence

**Purpose**: Close the remaining staging-node identity gap at both registry publication boundaries.

- [x] T045 CRITICAL Refactor accepted-registry and recovery staging in `src/registry/publication.rs` to retain descriptor-backed identity from exclusive creation through write, permission application, sync, reread, and semantic verification; immediately before each rename, prove the staging pathname still names that attempt-owned current-user regular file with the expected mode and reject symlink, wrong-node, or identity substitution; remove a failed attempt's staging pathname only while that identity still matches, and add focused substitution and cleanup-preservation tests in `tests/mapping_registry_integration.rs` per Constitution III, Constitution V, FR-016, FR-017, FR-024, FR-025, and the storage-contract publication sequence (partial)

## Phase 9: Convergence

**Purpose**: Enforce the complete-registry validation boundary before command-specific mapping input handling.

- [x] T046 CRITICAL Reorder `mapping add`, `mapping show`, and `mapping remove` orchestration in `src/lib.rs` so the selected home and complete accepted registry are loaded and validated before resolving command-specific source or destination paths, preserve operation, kind, and safe registry-path error details, and add precedence tests in `tests/mapping_registry_integration.rs` proving an invalid unrelated registry blocks each operation even when its supplied path is independently invalid per Constitution III, FR-009, and storage-contract publication step 1 (contradicts)

## Phase 10: Convergence

**Purpose**: Make canonical source selectors produce the specified not-found lifecycle result even when the selected source is absent.

- [x] T047 Allow `mapping show` and `mapping remove` selectors in `src/path_policy.rs` to canonicalize an absent absolute source identity from a safe existing ancestor while retaining UTF-8, traversal, symlink, ancestry, and supported existing-node checks, and add isolated human/JSON zero-mutation tests in `tests/mapping_registry_integration.rs` proving absent selectors return the distinct `mapping_not_found` result per US2/AC3, US3/AC2, FR-014, and the data-model lifecycle rules (partial)

## Phase 11: Convergence

**Purpose**: Restore the established operational-failure category for registry publication and recovery failures.

- [x] T048 Classify structured mapping failures with reasons `publication_failure` and `registry_recovery_failure` as `operational_failure` with exit 20 in `src/error.rs` while preserving their mapping detail fields, and add focused category and JSON-envelope tests in `src/result.rs` and `tests/mapping_registry_integration.rs` per FR-018 and the CLI-contract exit table (contradicts)

## Phase 12: Convergence

**Purpose**: Enforce canonical stored identities during complete accepted-registry validation.

- [x] T049 CRITICAL Strengthen accepted-registry loading in `src/registry/publication.rs` so every stored source and destination must equal its resolved canonical `PathEvidence` identity before the registry is accepted, then validate the resulting canonical ownership graph so textual aliases cannot evade duplicate, overlap, or recursive-topology checks; return stable safe-path mapping details and add isolated intermediate-symlink alias and duplicate-canonical-source tests in `tests/mapping_registry_integration.rs` per Constitution II, Constitution III, FR-004, FR-008, FR-009, FR-010, FR-011, FR-012, and FR-013 (partial)

## Phase 13: Convergence

**Purpose**: Restore representative-workstation performance after canonical accepted-registry validation.

- [x] T050 Remove the redundant raw-path all-pairs ownership pass from accepted-registry loading by separating strict Registry V1 wire decoding from domain acceptance in `src/registry/mod.rs` and `src/registry/publication.rs`, while preserving `registry::decode` as a fully validating public boundary and performing exactly one canonical ownership-graph validation during publication loading; retain canonical-alias rejection coverage and rerun the 100-run release harness below the one-second p95 thresholds per Constitution I, Constitution V, SC-005, and the plan performance decision (contradicts)

## Phase 14: Convergence

**Purpose**: Apply unexpected-staging coordination policy consistently to recovery publication.

- [x] T051 Reject unexpected pre-existing `.config.tmp-*` nodes in the selected recovery-generation directory before creating recovery staging in `src/registry/publication.rs`, reuse the existing stable corrupt-state behavior without accepting or deleting the unexpected node, and extend `tests/mapping_registry_integration.rs` to prove a retry after recovery staging substitution preserves both the accepted registry and substituted node per Constitution III and the storage-contract coordination and cleanup rules (partial)

## Phase 15: Convergence

**Purpose**: Make accepted-registry node-safety and I/O failures preserve their contracted reasons and exit categories.

- [x] T052 Refine accepted-registry loading in `src/registry/publication.rs` and mapping error conversion in `src/error.rs` so a symlink, wrong node type, unsafe ownership, or unsafe permission mode returns stable `unsafe_registry_mode` details with invalid-configuration exit 10, missing or malformed registry data remains `invalid_registry`, and genuine inaccessible open, metadata, or read I/O returns `operational_failure` exit 20 without losing the mapping operation or safely available `config.toml` path; strengthen the reader/writer permission-policy matrix in `tests/mapping_registry_integration.rs` with exact JSON reason, code, exit, and zero-mutation assertions per FR-018, FR-019, FR-025, the plan error decision, and the CLI-contract exit table (partial)

## Phase 16: Convergence

**Purpose**: Complete accepted-registry safety classification and structured schema-failure reporting across every mapping lifecycle operation.

- [x] T053 Refine accepted-registry inspection in `src/registry/publication.rs` so bounded non-following preclassification rejects every unsupported node type and unsafe ownership or permission mode before attempting to read it, descriptor identity verification still detects substitution, unreadable unsafe modes and non-openable FIFO or socket nodes return stable `unsafe_registry_mode` details with invalid-configuration exit 10, and otherwise-safe registries blocked by genuine access or descriptor I/O failures retain `operational_failure` exit 20; extend `tests/mapping_registry_integration.rs` with exact JSON and zero-mutation coverage per FR-025, T052, and the Constitution product-boundary requirement for precise unsupported-node handling (partial)
- [x] T054 Preserve `unsupported_schema` exit 11 when converting accepted-registry schema failures into structured mapping errors in `src/error.rs` and `src/result.rs`, retaining the lifecycle operation, supplied mapping kind where applicable, stable `invalid_registry` reason, and safely available `config.toml` path; add isolated exact-JSON and zero-mutation tests for add, list, show, and remove against an unsupported registry version in `tests/mapping_registry_integration.rs` per FR-018, FR-019, the plan error decision, and the CLI-contract exit table (partial)

## Phase 17: Convergence

**Purpose**: Complete the single-inspection path-evidence boundary required by the filesystem safety and performance model.

- [x] T055 CRITICAL Refactor `src/path_policy.rs` so endpoint resolution and evidence capture share one bounded non-following inspection result instead of repeating longest-existing-prefix discovery and endpoint or anchor metadata reads across `canonicalize`, `resolve_endpoint`, and `inspect_endpoint`; preserve canonical identities, absent-destination handling, node-kind and symlink rejection, revalidation semantics, stable mapping errors, and public selector behavior, strengthen focused call-boundary and integration coverage, and rerun the 1,000-mapping release harness per Constitution V, FR-016, T005, T035, and the plan performance decision (partial)

## Phase 18: Convergence

**Purpose**: Complete safe canonicalization through an intermediate symlink ancestor for absent mapping identities.

- [x] T056 Update the shared path inspection in `src/path_policy.rs` so an absent destination or source selector beneath an existing symlink-to-directory prefix resolves through that intermediate alias to a canonical target identity, verifies and captures the resolved directory anchor without accepting a final symlink endpoint or non-directory target, and add isolated integration coverage for add, show, and remove identity behavior plus zero unintended mutation in `tests/mapping_registry_integration.rs` per FR-008, the plan safe-ancestry decision, T005, and T047 (partial)

## Phase 19: Convergence

**Purpose**: Preserve submitted-path evidence through publication so ancestry aliases are genuinely revalidated.

- [x] T057 HIGH Refactor add-path inspection and registry publication across `src/lib.rs`, `src/path_policy.rs`, and `src/registry/publication.rs` so the initial source and destination evidence retains each submitted path, the exact evidence used to construct the canonical candidate is carried into the locked final-revalidation boundary instead of being reconstructed only from canonical registry values, and retargeting an intermediate symlink alias is rejected as `stale_path_evidence` without changing the accepted registry or payloads; add focused path-policy and publication coverage per FR-008, FR-016, the plan path-canonicalization and revalidation decisions, the data-model submitted-path field, and storage-contract step 5 (partial)
