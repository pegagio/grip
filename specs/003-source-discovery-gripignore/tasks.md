# Tasks: Source Discovery and Gripignore

**Input**: Design documents from `specs/003-source-discovery-gripignore/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, and `quickstart.md`

**Tests**: Required by FR-025 and the constitution. Story tests are written first and must fail for the intended missing behavior before implementation begins.

**Organization**: Tasks are grouped by user story so each story can be implemented and validated as a distinct read-only increment.

## Table of Contents

- [Phase 1: Setup and consistency](#phase-1-setup-and-consistency)
- [Phase 2: Foundational discovery infrastructure](#phase-2-foundational-discovery-infrastructure)
- [Phase 3: User Story 1 - Inspect the managed namespace](#phase-3-user-story-1---inspect-the-managed-namespace-priority-p1--mvp)
- [Phase 4: User Story 2 - Apply predictable Gripignore policy](#phase-4-user-story-2---apply-predictable-gripignore-policy-priority-p2)
- [Phase 5: User Story 3 - See unmanaged and unsupported boundaries](#phase-5-user-story-3---see-unmanaged-and-unsupported-boundaries-priority-p3)
- [Phase 6: Polish and cross-cutting validation](#phase-6-polish-and-cross-cutting-validation)
- [Dependencies and execution order](#dependencies-and-execution-order)
- [Parallel examples](#parallel-examples)
- [Implementation strategy](#implementation-strategy)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it targets different files and does not depend on incomplete work.
- **[Story]**: Maps a task to User Story 1, 2, or 3.
- Every task names the exact file or files it changes or validates.

## Phase 1: Setup and consistency

**Purpose**: Clear the constitutional implementation gate and establish the approved dependency and module boundary.

- [X] T001 Run `$speckit-analyze` against `specs/003-source-discovery-gripignore/spec.md`, `specs/003-source-discovery-gripignore/plan.md`, and `specs/003-source-discovery-gripignore/tasks.md`; resolve every CRITICAL or HIGH issue in those files before source changes
- [X] T002 Add the plan-approved `ignore = "0.4.33"` matcher dependency to `Cargo.toml`, regenerate `Cargo.lock`, and review the lockfile diff to confirm only expected transitive packages were introduced
- [X] T003 Create the `src/discovery/mod.rs`, `src/discovery/model.rs`, `src/discovery/filesystem.rs`, and `src/discovery/ignore_policy.rs` module skeletons with module-level documentation, then export `discovery` from `src/lib.rs`

**Checkpoint**: Artifacts are consistent, the dependency boundary is explicit, and the discovery module compiles as an empty integration point.

---

## Phase 2: Foundational discovery infrastructure

**Purpose**: Implement shared models, descriptor-relative inspection, registry evidence, error mapping, and isolated test support that block every user story.

**CRITICAL**: No user story implementation begins until this phase passes its focused tests.

- [X] T004 [P] Add failing unit tests for Safe Path byte preservation, record/category ordering, derived counts, and blocker counts in `src/discovery/model.rs`
- [X] T005 [P] Add failing unit tests for non-following relative stat/open behavior, raw node metadata capture, sorted raw child names, and bounded descriptor depth in `src/discovery/filesystem.rs`
- [X] T006 [P] Add failing read-only registry snapshot comparison tests in `src/registry/publication.rs` proving byte or identity drift is detected without acquiring `.registry.lock`
- [X] T007 Implement Safe Path, Discovery Request, Discovery Record, Node Evidence, Discovery Pass, and Discovery Inventory domain values and deterministic ordering in `src/discovery/model.rs`
- [X] T008 Implement the Unix descriptor-relative read-only adapter with `rustix::fs::openat`, `statat`, and `Dir` in `src/discovery/filesystem.rs`, preserving exact raw names and never following a final child link
- [X] T009 Expose read-only accepted-registry snapshot revalidation in `src/registry/publication.rs` and reuse complete canonical registry validation without creating locks, staging files, or recovery evidence
- [X] T010 [P] Add discovery operation/reason conversion and stable exit-category tests in `src/error.rs`, including invalid policy, unreadable policy, directory I/O, and stale discovery evidence
- [X] T011 [P] Extend isolated raw-byte tree snapshots in `tests/support/mod.rs` and add a deterministic private between-pass fault seam in `src/discovery/mod.rs` without reading sockets, FIFOs, devices, symlink targets, or ordinary payload content unnecessarily

**Checkpoint**: Shared discovery types, non-following inspection, registry revalidation, stable failures, and test controls are independently verified.

---

## Phase 3: User Story 1 - Inspect the Managed Namespace (Priority: P1) 🎯 MVP

**Goal**: Discover exact file mappings and eligible ordinary file/directory members of tree mappings, including empty directories, for all mappings or one canonical source identity with deterministic read-only output.

**Independent Test**: From a hand-authored isolated registry, discover one file mapping and nested tree mappings in all/selected scopes; verify every eligible record and paired destination, deterministic ordering across repeated runs, complete-registry precedence, two-pass stale rejection, and byte/metadata-identical Grip home and payload snapshots.

### Tests for User Story 1

- [X] T012 [P] [US1] Add failing grammar, all-mapping scope, selected-source scope, extra-selector, `--` termination, human output, and JSON envelope tests in `tests/discovery_cli_contract.rs`
- [X] T013 [P] [US1] Add failing isolated discovery tests for exact file mappings, nested tree files/directories, empty directories, absent destinations, stable paired paths, canonical mapping ordering, and zero mutation in `tests/discovery_filesystem_integration.rs`
- [X] T014 [US1] Add failing invalid-registry precedence coverage in `tests/discovery_filesystem_integration.rs` and deterministic private between-pass add/remove/replace/directory-enumeration drift tests in `src/discovery/mod.rs`

### Implementation for User Story 1

- [X] T015 [US1] Add `mapping inspect [SOURCE]` parsing and source-selector conversion while rejecting destination, nested-member, extra, or ambiguous selectors in `src/cli.rs`
- [X] T016 [US1] Implement exact file-mapping records and sequential descriptor-relative ordinary file/directory tree discovery, including empty directories and source-relative paired paths, in `src/discovery/mod.rs` and `src/discovery/filesystem.rs`
- [X] T017 [US1] Implement all/selected mapping orchestration, two complete pass comparison, registry revalidation, deterministic inventory finalization, and `stale_discovery_evidence` rejection in `src/discovery/mod.rs`
- [X] T018 [US1] Route `mapping inspect` through `src/lib.rs` and render deterministic human and JSON scope, counts, blockers, records, and Safe Path fields in `src/result.rs`
- [X] T019 [US1] Run `tests/discovery_cli_contract.rs` and `tests/discovery_filesystem_integration.rs`; confirm the User Story 1 fixtures pass independently with no policy files, destination-only traversal, unsupported-node classification, state publication, or payload mutation

**Checkpoint**: The MVP exposes a complete, deterministic, read-only eligible namespace for exact and tree mappings.

---

## Phase 4: User Story 2 - Apply Predictable Gripignore Policy (Priority: P2)

**Goal**: Apply only root and nested source-side `.gripignore` policy with the specified Gitignore-compatible grammar, precedence, negation, pruning, and unconditional policy-file exclusion.

**Independent Test**: Run a fixed table-driven corpus and authority-poison fixture over isolated trees; compare the exact eligible and ignored records while proving `.gitignore`, `.ignore`, Git excludes, destination policy, global configuration, and hidden status have no effect.

### Tests for User Story 2

- [X] T020 [P] [US2] Add failing table-driven Gitignore grammar tests for blank/comment lines, escaping, trailing spaces, anchors, directory-only rules, `*`, `?`, ranges, all `**` forms, negation, last-rule precedence, BOM, CRLF, final unterminated lines, and unclosed classes in `tests/gripignore_conformance.rs`
- [X] T021 [US2] Add failing root/nested precedence, valid re-inclusion, excluded-parent pruning, ignored-directory reporting, hidden-entry, and policy-only `!.gripignore` tests in `tests/gripignore_conformance.rs`
- [X] T022 [P] [US2] Add failing authority-poison, symlink/special/hard-linked/sparse policy, unreadable policy, invalid UTF-8 policy, parse failure, between-pass policy drift, and zero-mutation tests in `tests/discovery_filesystem_integration.rs`

### Implementation for User Story 2

- [X] T023 [US2] Implement exact no-follow `.gripignore` loading, UTF-8/BOM/CRLF/final-line normalization, fail-closed `GitignoreBuilder` construction, policy evidence, and unconditional policy-only exclusion in `src/discovery/ignore_policy.rs`
- [X] T024 [US2] Integrate the root-to-directory matcher stack, deepest non-neutral precedence, ignored file records, ignored-directory pruning, and hidden-entry inclusion into source traversal in `src/discovery/mod.rs` and `src/discovery/filesystem.rs`
- [X] T025 [US2] Map malformed/non-UTF-8 policy to `invalid_configuration`, unreadable policy to `operational_failure`, and policy drift to `stale_discovery_evidence` with safe structured paths in `src/error.rs` and `src/result.rs`
- [X] T026 [US2] Run `tests/gripignore_conformance.rs`, `tests/discovery_filesystem_integration.rs`, and `tests/discovery_cli_contract.rs`; confirm User Story 2 passes independently over a tree with no destination-only or unsupported payload entries

**Checkpoint**: Source membership follows only the explicit Gripignore contract, reports truthful exclusion roots, and remains read-only.

---

## Phase 5: User Story 3 - See Unmanaged and Unsupported Boundaries (Priority: P3)

**Goal**: Report destination-only overlay content, unsupported source nodes, and unsafe nodes at eligible paired destination paths without following, opening, or mutating them.

**Independent Test**: Populate isolated source/destination trees with ordinary overlay content and every platform-feasible unsupported node; verify exact category, reason, path identity, blocker count, pruning behavior, success envelope, and zero mutation, with pure classification tests covering privileged or unavailable node modes.

### Tests for User Story 3

- [X] T027 [P] [US3] Add failing raw-mode classification tests for symlink, socket, FIFO, character device, block device, whiteout, unknown special, nested mount, hard link, sparse file, wrong kind, ordinary file, and ordinary directory in `src/discovery/filesystem.rs`
- [X] T028 [P] [US3] Add failing isolated destination-only and paired-collision tests for ordinary files/directories, symlinks, FIFOs, sockets, hard links, sparse files, non-UTF-8 names, pruning, stable ordering, and blocker counts in `tests/discovery_filesystem_integration.rs`
- [X] T029 [P] [US3] Add failing JSON/human record-category and complete-success-with-blockers tests in `tests/discovery_cli_contract.rs`, Safe Path `raw_hex` coverage in `src/discovery/model.rs` and `tests/discovery_filesystem_integration.rs`, and output-failure coverage in `src/result.rs`

### Implementation for User Story 3

- [X] T030 [US3] Implement ordered source node allowlist classification, same-device enforcement, hard-link/sparse detection, conditional whiteout recognition, non-UTF-8 identity, and no-open/no-follow pruning in `src/discovery/filesystem.rs`
- [X] T031 [US3] Implement paired-destination inspection and the separate non-following destination-only walk, including wrong-kind/unsupported collisions and nonblocking unsupported overlay boundaries, in `src/discovery/mod.rs` and `src/discovery/filesystem.rs`
- [X] T032 [US3] Render all five stable record categories, unsupported/collision reasons, exact Safe Path objects, deterministic counts, and `blocking_count` while preserving exit `0` for complete inventories in `src/result.rs`
- [X] T033 [US3] Run `tests/discovery_filesystem_integration.rs` and `tests/discovery_cli_contract.rs`; confirm User Story 3 passes independently with an empty Gripignore policy stack and no baseline or synchronization-state assumptions

**Checkpoint**: All three stories are independently testable and jointly produce the complete Feature 003 inventory.

---

## Phase 6: Polish and Cross-Cutting Validation

**Purpose**: Close performance, documentation, regression, flow-back, and acceptance obligations across the complete feature.

- [X] T034 [P] Add whole-feature zero-mutation, complete-registry precedence, result-channel separation, and Feature 001/002 regression coverage across `tests/discovery_filesystem_integration.rs`, `tests/discovery_cli_contract.rs`, `tests/mapping_cli_contract.rs`, and `tests/mapping_registry_integration.rs`
- [X] T035 [P] Extend the ignored 100-run release harness with a deterministic 10,000-entry two-pass discovery fixture and two-second p95 assertion in `tests/performance_acceptance.rs`
- [X] T036 [P] Document `mapping inspect`, source-defined membership, `.gripignore` authority, ignored-directory reporting, destination-only ownership, blockers, and non-mutation in `README.md`
- [X] T037 Run every scenario in `specs/003-source-discovery-gripignore/quickstart.md`, record observed platform and outcomes in that file, and correct any drift across `specs/003-source-discovery-gripignore/contracts/`
- [X] T038 Run `mise run validate`, then run the ignored release performance harness and record the environment and measurements in `specs/003-source-discovery-gripignore/quickstart.md`
- [X] T039 Reconcile accepted implementation discoveries through `specs/003-source-discovery-gripignore/spec.md`, `specs/003-source-discovery-gripignore/plan.md`, `specs/003-source-discovery-gripignore/contracts/`, and `specs/003-source-discovery-gripignore/tasks.md`, then rerun `$speckit-analyze` until no blocking inconsistency remains
- [X] T040 Run `$speckit-converge` against `specs/003-source-discovery-gripignore/` after implementation and append/complete any remaining work until specification, plan, tasks, tests, and implementation have no unresolved gaps

---

## Dependencies and Execution Order

### Phase Dependencies

- **Phase 1 — Setup and consistency**: Starts immediately. T001 is the implementation gate; T002 follows artifact approval, and T003 follows the dependency update.
- **Phase 2 — Foundational infrastructure**: Depends on Phase 1. T004, T005, and T006 can begin in parallel after the module skeleton; T007 depends on T004, T008 on T005, and T009 on T006. T010 and T011 can proceed in parallel with those implementations because they target separate files.
- **Phase 3 — User Story 1**: Depends on Phase 2. T012–T014 are written and failed first; T015–T018 then execute in order where shared files require it; T019 is the story gate.
- **Phase 4 — User Story 2**: Depends on User Story 1's traversal and two-pass inventory. T020–T022 can run in parallel before T023–T025; T026 is the story gate.
- **Phase 5 — User Story 3**: Depends on User Story 1's inventory model but can be developed independently of User Story 2 after Phase 3. T027–T029 can run in parallel before T030–T032; T033 is the story gate.
- **Phase 6 — Polish**: Depends on all included stories. Full Feature 003 completion requires all three stories before T037–T040.

### User Story Dependency Graph

```text
Setup and constitutional analysis
              |
Foundational models, filesystem adapter, and registry evidence
              |
US1: Eligible managed namespace (MVP)
       |                         |
US2: Gripignore policy     US3: Overlay and unsupported boundaries
       |                         |
       +------------+------------+
                    |
Polish, validation, analysis, and convergence
```

### User Story Dependencies

- **User Story 1 (P1)**: Begins after foundational infrastructure and has no dependency on another story.
- **User Story 2 (P2)**: Reuses US1 traversal, evidence, and inventory; it remains independently testable with ordinary source entries and no destination overlay.
- **User Story 3 (P3)**: Reuses US1 traversal, evidence, and inventory but does not require US2 policy behavior; it remains independently testable with an empty policy stack.

### Within Each User Story

- Write contract and integration tests first and confirm they fail for the intended missing behavior.
- Implement domain and filesystem behavior before CLI orchestration and rendering where their files do not conflict.
- Preserve the two-pass read-only boundary throughout; no story may add state publication or payload reads.
- Run the independent story gate before treating that story as complete.

### Parallel Opportunities

- T004, T005, and T006 test separate foundational modules.
- T010 and T011 target error handling and test support independently of the foundational implementations.
- T012–T014 cover separate US1 contract and integration concerns.
- T020–T022 cover separate matcher, policy, and integration surfaces for US2.
- T027–T029 cover separate classifier, filesystem integration, and CLI output surfaces for US3.
- US2 and US3 can proceed in parallel after US1 if edits to `src/discovery/mod.rs`, `src/discovery/filesystem.rs`, and shared integration tests are coordinated or isolated into non-overlapping commits.
- T034–T036 target regression tests, the performance harness, and documentation independently.

## Parallel Examples

### User Story 1

```text
Task T012: Add mapping discovery CLI contract tests in tests/discovery_cli_contract.rs
Task T013: Add eligible namespace and zero-mutation tests in tests/discovery_filesystem_integration.rs
Task T014: Add registry precedence and stale-pass tests in tests/discovery_filesystem_integration.rs after coordinating non-overlapping test sections with T013
```

### User Story 2

```text
Task T020: Add the Gitignore grammar corpus in tests/gripignore_conformance.rs
Task T021: Add nested precedence and policy-only tests in tests/gripignore_conformance.rs after coordinating non-overlapping test sections with T020
Task T022: Add authority-poison and invalid-policy integration tests in tests/discovery_filesystem_integration.rs
```

### User Story 3

```text
Task T027: Add pure raw-mode classification tests in src/discovery/filesystem.rs
Task T028: Add destination and unsupported-node integration tests in tests/discovery_filesystem_integration.rs
Task T029: Add record and Safe Path rendering tests in tests/discovery_cli_contract.rs
```

## Implementation Strategy

### MVP First: User Story 1

1. Complete T001–T011 to establish artifact consistency and shared read-only infrastructure.
2. Write and fail T012–T014.
3. Complete T015–T018.
4. Run T019 and stop at the independently usable eligible-namespace checkpoint.

### Incremental Delivery

1. **US1**: Deliver deterministic eligible membership for exact and tree mappings without policy or overlay classification.
2. **US2**: Add the complete `.gripignore` authority and conformance boundary without changing US1's command or output envelope.
3. **US3**: Add destination-only and unsupported-boundary records without adding baseline semantics.
4. **Polish**: Measure the sequential two-pass design, document the interface, execute the quickstart, and close analysis/convergence gates.

### Parallel Team Strategy

After shared foundations and US1 stabilize, one implementer can build the matcher stack and conformance corpus while another builds destination/unsupported classification. Both must coordinate changes to `src/discovery/mod.rs`, `src/discovery/filesystem.rs`, and `tests/discovery_filesystem_integration.rs`; the explicitly file-separated tasks remain safe to run concurrently.

## Notes

- `[P]` means different files and no dependency on incomplete work; it does not override phase gates or justify concurrent edits to the same file.
- `ignore` is matching-only; Grip owns traversal, reporting, evidence, and no-follow safety.
- Tests are mandatory because the feature defines filesystem behavior and future ownership boundaries.
- No task may add baseline comparison, content hashing, payload copying, deletion, retirement, state publication, a daemon, watcher, broad lock, persistent inode identity, cache, index, or parallel traversal.
- Generated dependency changes are limited to `Cargo.lock` after the explicit `Cargo.toml` addition and must be reviewed before proceeding.
- Do not commit, push, publish, merge, or run privileged filesystem fixtures unless the user separately authorizes it.
