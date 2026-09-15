---

description: "Task list for Contained-Source Tree Mappings"
---

# Tasks: Contained-Source Tree Mappings

**Input**: Design documents from `specs/025-contained-source-tree-mappings/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contained-source mapping contract](contracts/contained-source-mappings.md), and [quickstart.md](quickstart.md)

**Tests**: Required. The specification and constitution require isolated filesystem evidence for ownership, traversal bounds, rebinding, dry-run non-mutation, drift, force, deletion, and baseline publication behavior.

**Organization**: Tasks are grouped by user story. Test tasks precede the behavior they prove, and every filesystem scenario must use disposable roots rather than the developer's real project or home.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel with other marked tasks after shared prerequisites are complete because it modifies different files.
- **[Story]**: Maps a task to one feature user story.
- Every task names the exact file or files it changes or validates.

## Phase 1: Setup (Shared Test Infrastructure)

**Purpose**: Establish one reusable isolated fixture for a project and source tree nested beneath a disposable destination home.

- [X] T001 Add a contained-home fixture builder that exposes the disposable home, nested Grip project, `home/` source, command environment, and payload snapshots in `tests/support/project.rs`.

**Checkpoint**: Tests can construct `home/` to `~/` layouts without reading or mutating the developer's real home or project metadata.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Introduce the shared runtime types and accepted-state plumbing needed by every contained-source story without changing persisted descriptor or state schemas.

**⚠️ CRITICAL**: No user-story implementation begins until these compile-safe foundations are complete.

- [X] T002 Define component-aware resolved-root and managed-member relation types plus additive recursive-member diagnostic detail in `src/mapping.rs`, `src/discovery/model.rs`, and `src/classification/model.rs`.
- [X] T003 Define the ephemeral managed-identity set and retained ignored-prefix coverage types, explicitly excluding the tree root anchor, in `src/discovery/model.rs` and `src/discovery/ignore_policy.rs`.
- [X] T004 Thread accepted active identities through `discovery::inspect`, candidate inspection, two-pass inspection, and their observation/add call sites while preserving current behavior in `src/discovery/mod.rs`, `src/observation/mod.rs`, and `src/lib.rs`.

**Checkpoint**: The crate compiles with runtime-only topology and managed-set models, and discovery can receive retained accepted identities without a descriptor V2 or state V4 migration.

---

## Phase 3: User Story 1 - Map a Repository Home Tree to the User's Home (Priority: P1) 🎯 MVP

**Goal**: Accept the narrow tree-source-beneath-destination root shape and support ordinary add, inspection, push, pull, and sync for safe members while every previously unsafe root shape remains rejected.

**Independent Test**: In a disposable home containing a nested Grip project and `home/` source, add `home/` to `~/`, verify add changes no endpoint payload, then preview and execute ordinary push and pull for `.bashrc`, `.gitconfig`, and `.local/bin/tool`; separately verify equal roots, destination-beneath-source roots, and contained file mappings still fail without publication.

### Tests for User Story 1

Write these tests first and verify that the newly admitted contained-tree cases fail before implementation.

- [X] T005 [P] [US1] Add the complete same-mapping root matrix for disjoint trees, source-beneath-destination trees, equal roots, destination-beneath-source trees, contained files, deep containment, and component-boundary siblings in `tests/mapping_topology_integration.rs`.
- [X] T006 [P] [US1] Add isolated happy-path and rejection scenarios for non-mutating add, status, diff, push, pull, sync, and accepted-state publication in `tests/contained_source_tree_integration.rs`.

### Implementation for User Story 1

- [X] T007 [US1] Implement the narrow tree source-strictly-beneath-destination admission and keep every other same-mapping and complete-registry root conflict unchanged in `src/mapping.rs` and `src/registry/mod.rs`.
- [X] T008 [US1] Carry an admitted contained mapping through candidate observation and baseline construction before descriptor or state publication, preserving add non-mutation and existing initial-comparison rules in `src/lib.rs` and `src/baseline.rs`.
- [X] T009 [US1] Run the focused root-topology and contained-source integration targets and reconcile only P1 failures in `tests/mapping_topology_integration.rs` and `tests/contained_source_tree_integration.rs`.

**Checkpoint**: A safe `home/` to `~/` mapping works end to end in an isolated home, while equal, reverse-contained, and file-contained roots remain rejected.

---

## Phase 4: User Story 2 - Leave the Rest of the Home Directory Alone (Priority: P2)

**Goal**: Make destination work proportional to current or retained managed identities so unrelated home content is never enumerated, reported, baselined, selected, or used as drift evidence.

**Independent Test**: Populate the disposable destination with 10,000 unrelated entries, unreadable directories, links, special nodes, and between-pass churn; verify add, status, diff, previews, and synchronization ignore them, while exact retained source-deletion and paired unsafe-path evidence still classify or block correctly.

### Tests for User Story 2

Write these tests first and verify the current recursive destination walk violates the new bounded-scope expectations.

- [X] T010 [P] [US2] Replace arbitrary tree destination-only expectations with managed-identity-only assertions in `tests/mapping_cli_contract.rs`, `tests/pull_cli_contract.rs`, `tests/sync_planning.rs`, and `tests/product_acceptance.rs`.
- [X] T011 [P] [US2] Add bounded-inspection scenarios covering 10,000 unrelated entries, unreadable and unsupported subtrees, unrelated links, and unrelated between-pass churn in `tests/contained_source_tree_integration.rs` and `src/discovery/mod.rs`.
- [X] T012 [P] [US2] Add retained source-deletion, converged-deletion, destination-change, conflict, missing-root, and paired unsafe-ancestor or leaf scenarios in `tests/managed_identity_retention_integration.rs`.
- [X] T013 [P] [US2] Extend managed-name collision and capability coverage so only current and retained active identities participate without sibling enumeration in `tests/apfs_name_compatibility_integration.rs`.
- [X] T014 [P] [US2] Add an ignored release-mode workload with 100 managed members, 10,000 unrelated destination entries, 100 warm status samples, deterministic output checks, and a one-second p95 assertion gated to the supported macOS ARM64 release platform in `tests/performance_acceptance.rs`.

### Implementation for User Story 2

- [X] T015 [US2] Populate the deterministic managed-identity set from non-ignored source discovery plus retained active baselines, preserving raw relative bytes and canonical order, in `src/discovery/mod.rs` and `src/observation/mod.rs`.
- [X] T016 [US2] Track accepted identities beneath pruned `.gripignore` prefixes as ignored retirement evidence without opening or fabricating descendants in `src/discovery/ignore_policy.rs` and `src/discovery/mod.rs`.
- [X] T017 [US2] Implement one shared cached descriptor-relative target probe with explicit absent, supported, and blocking ancestor or leaf outcomes in `src/discovery/filesystem.rs`, then refactor `src/observation/fingerprint.rs` to reuse it instead of introducing a second traversal path.
- [X] T018 [US2] Replace `inspect_tree_destination` and `walk_destination` with exact managed-target probes and remove unrelated child-name evidence from two-pass stability checks in `src/discovery/mod.rs`.
- [X] T019 [US2] Scope APFS comparison-key grouping and destination capability qualification to current eligible and retained active identities, explicitly excluding identities in ignored-retirement membership, in `src/discovery/mod.rs` and `src/metadata/macos.rs`.
- [X] T020 [US2] Preserve exact observation and existing classifications for active retained identities whose source is absent or whose paired destination is blocking in `src/observation/mod.rs` and `src/classification/mod.rs`.
- [X] T021 [US2] Run every P2 target added or changed by T010-T013, including bounded-discovery unit tests, retained-deletion coverage, and product acceptance, and reconcile only P2 failures in `src/discovery/mod.rs`, `tests/contained_source_tree_integration.rs`, `tests/managed_identity_retention_integration.rs`, `tests/gripignore_conformance.rs`, `tests/apfs_name_compatibility_integration.rs`, `tests/mapping_cli_contract.rs`, `tests/pull_cli_contract.rs`, `tests/sync_planning.rs`, and `tests/product_acceptance.rs`.

**Checkpoint**: Unrelated destination breadth, permissions, node types, and churn are operationally invisible, while every current or retained managed target preserves existing safety and classification behavior.

---

## Phase 5: User Story 3 - Block Only Genuinely Recursive Members (Priority: P3)

**Goal**: Reject current or retained members whose paired destination equals, contains, or falls within the source tree, while allowing disjoint members and explicit source-policy exclusions.

**Independent Test**: Exercise the complete `R` versus `P` matrix, ignored unsafe subtrees, a retained identity made unsafe by rebinding, and a newly introduced unsafe sibling between planning and action; verify deterministic details and zero payload or accepted-state mutation.

### Tests for User Story 3

Write these tests first and verify unsafe managed members are not yet blocked at every required gate.

- [X] T022 [P] [US3] Add equal, ancestor, descendant, disjoint, component-sibling, ignored-current, and ignored-retained member scenarios with add and inspection non-mutation assertions in `tests/contained_source_tree_integration.rs` and `tests/gripignore_conformance.rs`.
- [X] T023 [P] [US3] Add disjoint-to-contained, contained-to-disjoint, changed-`P`, and retained-identity-becomes-recursive rebinding scenarios that preserve accepted evidence on failure in `tests/project_state_rebinding_integration.rs`.
- [X] T024 [P] [US3] Add mutation and deletion fault-hook scenarios that introduce an unsafe sibling after planning and before lock or action revalidation in `tests/push_failure_integration.rs`, `tests/sync_failure_integration.rs`, and `tests/contained_source_topology_drift_integration.rs`.

### Implementation for User Story 3

- [X] T025 [US3] Assess every active managed identity against the runtime containment-relative path after source policy and block unsafe candidate addition before fence, descriptor, baseline, or payload publication in `src/mapping.rs`, `src/discovery/mod.rs`, `src/lib.rs`, and `src/baseline.rs`.
- [X] T026 [US3] Emit stable `recursive_member_topology` human and JSON detail for mapping, relative member, resolved paths, relation, and explicit `.gripignore` remediation without changing envelope versions in `src/classification/model.rs`, `src/error.rs`, and `src/result.rs`.
- [X] T027 [US3] Topology-check retained identities before fingerprint probing during binding changes and preserve state on blockers in `src/state/rebinding.rs` and `src/state/mod.rs`.
- [X] T028 [US3] Revalidate the complete selected mapping before its first push, pull, sync, force, or deletion action, then retain existing per-action evidence checks in `src/mutation/execution.rs` and `src/delete/execution.rs`.
- [X] T029 [US3] Run every P3 target added or changed by T022-T024, including the dedicated topology-drift target, and reconcile only P3 failures in `tests/contained_source_tree_integration.rs`, `tests/contained_source_topology_drift_integration.rs`, `tests/gripignore_conformance.rs`, `tests/project_state_rebinding_integration.rs`, `tests/push_failure_integration.rs`, and `tests/sync_failure_integration.rs`.

**Checkpoint**: Every unsafe current or retained member blocks before mutation with actionable typed detail, and ordinary explicit ignore policy is the only exclusion mechanism.

---

## Phase 6: User Story 4 - Preserve Established Ownership Boundaries (Priority: P4)

**Goal**: Prove that the contained-tree exception does not create a general topology bypass, broaden force authority, affect unrelated mappings, or weaken dry-run and schema compatibility.

**Independent Test**: Run equal, nested, duplicate, source-overlap, destination-overlap, cross-mapping recursion, selector, force, deletion, dry-run, and state-layout cases beside contained-source fixtures; verify only the approved same-mapping tree relation and arbitrary tree destination enumeration changed.

### Tests for User Story 4

Write these regression tests before any final compatibility corrections.

- [X] T030 [P] [US4] Expand complete-registry, cross-mapping recursion, duplicate, nested, component-boundary, project-metadata, and `add --force` ownership cases in `tests/mapping_topology_integration.rs` and `tests/project_reserved_metadata_integration.rs`.
- [X] T031 [P] [US4] Add exact-entry selector, forced direction and absence, unrelated-mapping independence, deletion authorization, and dry-run non-mutation cases for contained mappings in `tests/contained_source_tree_integration.rs` and `tests/force_resolution_guidance_contract.rs`.
- [X] T032 [P] [US4] Add compatibility assertions that descriptor V2, state V4, command syntax, selector interpretation, and machine-output envelope versions remain unchanged in `tests/state_integration.rs`, `tests/portable_mapping_model.rs`, and `tests/metadata_cli_contract.rs`.

### Implementation for User Story 4

- [X] T033 [US4] Constrain recursive-member blocker propagation to the selected mapping while keeping unrelated mappings independently operable in `src/observation/model.rs` and `src/observation/mod.rs`.
- [X] T034 [US4] Preserve exact-entry force, deletion authorization, complete-registry validation, and dry-run behavior without permitting root or member topology overrides in `src/lib.rs`, `src/mutation/plan.rs`, and `src/delete/plan.rs`.
- [X] T035 [US4] Run the ownership, selector, force, deletion, state-schema, dry-run, filesystem-safety, and publication regression targets and reconcile only explicitly superseded assertions in `tests/`.

**Checkpoint**: Feature 025 is a narrow contained-tree capability, not a general override of Grip ownership or synchronization authority.

---

## Requirement Traceability

This map keeps every functional requirement attached to at least one implementation or verification task without duplicating the detailed acceptance scenarios.

| Requirements | Primary tasks |
|---|---|
| FR-001-FR-002 | T005-T008 |
| FR-003-FR-006 | T003, T015-T016, T022, T025-T026 |
| FR-007-FR-010 | T010-T012, T015-T020 |
| FR-011-FR-012 | T006, T012-T013, T017-T020 |
| FR-013-FR-015 | T023-T028 |
| FR-016-FR-017 | T030-T035 |
| FR-018 | T036-T037 |
| SC-001-SC-005 and SC-007 | T005-T013, T021-T035, T038 |
| SC-006 | T014, T039 |

---

## Phase 7: Polish & Cross-Cutting Validation

**Purpose**: Align user guidance, prove the representative performance claim, and pass the required Spec Kit and repository gates.

- [X] T036 [P] Document the supported `home/` to `~/` layout, managed-identity inspection boundary, recursive-member diagnostics, explicit `.gripignore` remediation, and unchanged unsafe root cases in `README.md` and `docs/product-definition.md`.
- [X] T037 Execute every scenario in `specs/025-contained-source-tree-mappings/quickstart.md` against disposable roots, correct only command or presentation errors in that file, flow any behavioral contradiction back through `specs/025-contained-source-tree-mappings/spec.md`, `specs/025-contained-source-tree-mappings/plan.md`, and `specs/025-contained-source-tree-mappings/tasks.md` before continuing, and leave no fixture state behind.
- [X] T038 Run the complete formatting, Clippy, unit, integration, and release-build gate defined by `mise run validate` in `mise.toml` and resolve regressions without weakening Feature 025 contracts.
- [X] T039 Run the ignored contained-home release performance workload defined by `mise run performance` in `mise.toml`, record any supported-platform limitation in `specs/025-contained-source-tree-mappings/quickstart.md`, and do not add caching or parallelism without evidence.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately.
- **Foundational (Phase 2)**: Depends on T001 and blocks every user story.
- **User Story 1 (Phase 3)**: Depends on T002-T004 and establishes the admitted contained-tree root shape.
- **User Story 2 (Phase 4)**: Depends on US1 because bounded destination inspection must run against an admitted contained tree.
- **User Story 3 (Phase 5)**: Depends on US2 because recursive-member validation and drift checks use the authoritative managed-identity set.
- **User Story 4 (Phase 6)**: Depends on US1-US3 so compatibility tests exercise the completed exception and blockers.
- **Polish (Phase 7)**: Depends on all selected user-story phases.

### User Story Dependencies

- **US1 (P1)**: Foundation only. It independently proves the safe contained-root happy path and prohibited-root matrix.
- **US2 (P2)**: Builds on US1. It independently proves that destination work is bounded while retained identities remain observable.
- **US3 (P3)**: Builds on US2's managed set. It independently proves member topology, ignore remediation, rebinding, and stale-plan blocking.
- **US4 (P4)**: Integrates the completed capability with established ownership, force, deletion, dry-run, and persistence contracts.

### Within Each User Story

- Add and run the story's failing tests before implementing its behavior.
- Implement domain models and safety decisions before presentation changes.
- Revalidate candidate-add and mutation boundaries before accepting happy-path output.
- Finish each story's focused test task before proceeding to the next priority.

### Parallel Opportunities

- T005 and T006 can run in parallel after the foundation because they modify separate test targets.
- T010-T014 can run in parallel after US1 because they establish distinct contract, integration, APFS, and performance evidence.
- T022-T024 can run in parallel after US2 because they cover separate member, rebinding, and fault-hook targets.
- T030-T032 can run in parallel after US3 because they cover distinct ownership, operation, and persistence compatibility surfaces.
- T036 can proceed in parallel with final test reconciliation after behavior and diagnostic wording stabilize.

## Parallel Example: User Story 2

```text
Task: "Update destination-only contract assertions in mapping, pull, sync, and product acceptance tests"
Task: "Add bounded destination traversal and unrelated-churn integration scenarios"
Task: "Add managed-only APFS name compatibility scenarios"
Task: "Add the 100-managed and 10,000-unrelated performance workload"
```

## Parallel Example: User Story 3

```text
Task: "Add the managed-member relation and ignore-policy matrix"
Task: "Add topology-changing rebinding scenarios"
Task: "Add pre-action unsafe-sibling fault-hook scenarios"
```

## Implementation Strategy

### MVP First (User Story 1)

1. Complete T001-T004 to establish isolated fixtures, runtime models, and retained-state plumbing.
2. Complete T005-T009 to admit and exercise safe `home/` to `~/` mappings.
3. Stop and validate the P1 slice in disposable roots.

US1 is the smallest demonstrable capability, but it is not release-ready by itself. The ownership and performance contract requires US2-US4 before Feature 025 can be accepted.

### Incremental Delivery

1. Add US1: narrow root admission and safe synchronization.
2. Add US2: bounded managed-identity destination inspection.
3. Add US3: recursive-member, rebinding, and stale-plan blockers.
4. Add US4: compatibility and non-regression proof.
5. Complete documentation, quickstart, performance, repository validation, and cross-artifact analysis.

## Notes

- No task authorizes a descriptor or accepted-state schema version change.
- No task authorizes an implicit ignore, topology force flag, destination inventory, cache, watcher, daemon, broad lock, persistent index, or parallel traversal.
- Retained accepted identities remain the durable index for source-deletion and ignore-retirement behavior.
- Existing historical Feature 002 and Feature 003 artifacts remain unchanged; Feature 025 carries the superseding behavior forward.
- Commit, push, publication, roadmap-status changes, and wiki ingestion are outside this task list unless separately requested.
