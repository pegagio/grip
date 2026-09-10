---

description: "Implementation tasks for the Git-inspired Grip command hierarchy"
---

# Tasks: Git-Inspired Command Hierarchy

**Input**: Design documents from `/specs/011-git-command-hierarchy/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/cli.md](contracts/cli.md), and [quickstart.md](quickstart.md)

**Tests**: Automated CLI, state-lifecycle, and filesystem tests are required by the specification and constitution. Each behavior-changing task includes an isolated test task before or alongside its implementation.

**Organization**: Tasks are grouped by user story. Foundational work provides the shared parser, state, and publication safety boundaries that every story needs.

## Phase 1: Setup (Shared CLI and result infrastructure)

**Purpose**: Establish the root invocation grammar and common dispatch/result behavior used by every flat command.

- [X] T001 Update the root `Cli`, `OutputArg`, and `Command` definitions for the ten-command flat surface and global `-p`, `-o`, and repeatable `-v` options in `src/cli.rs`.
- [X] T002 Update JSON pre-parse detection for `-o json`, `-ojson`, `--output json`, and `--output=json` parse failures in `src/cli.rs`.
- [X] T003 Refactor project-option validation, command dispatch, and command-result finalization for the flat command enum in `src/lib.rs`.
- [X] T004 Update human/JSON result rendering and result-category exit mapping for ordinary status attention versus operational errors in `src/result.rs`.
- [X] T005 [P] Add a parameterized retained-command help, root-parser, global-option, output-mode, and `init`/`version` contract matrix in `tests/cli_contract.rs`.
- [X] T006 [P] Update project-selection rejection coverage for `-p`/`--project` on `init` and `version` in `tests/project_independent_cli_contract.rs`.

**Checkpoint**: The crate parses only the new top-level grammar, renders deterministic human or JSON results, and enforces the project-option boundary.

## Phase 2: Foundational (Baseline membership and safe publication)

**Purpose**: Replace retirement/recovery state with active-membership baselines and retain only bounded, verified publication mechanics.

**⚠️ CRITICAL**: Complete this phase before implementing user-facing synchronization, mapping, or force behavior.

- [X] T007 Replace direct legacy recovery/delete/resolve/retirement tests with explicit rejection or forced-direction coverage before removing their public implementations in `tests/legacy_interface_contract.rs`, `tests/recovery_*`, `tests/delete_cli_contract.rs`, `tests/resolve_*`, `tests/retire_*`, `tests/operation_record_integration.rs`, `tests/project_recovery_rebinding_integration.rs`, `src/lib.rs`, `src/recovery/`, `src/resolve/`, `src/retire/`, and `src/mutation/recovery.rs`.
- [X] T008 Replace pending-retirement state with active-mapping baselines carrying `.gripignore` content and filesystem-change revision evidence in `src/state/mod.rs`.
- [X] T009 Implement validation, publication, rebinding, and bounded completion of coordinated registry/state pruning transitions in `src/state/publication.rs`, `src/state/rebinding.rs`, and `src/state/mutation_lock.rs`.
- [X] T010 Implement active-membership derivation and baseline eligibility for ignored, removed, policy-stale, and newly reintroduced tree entries in `src/discovery/ignore_policy.rs`, `src/observation/mod.rs`, and `src/classification/mod.rs`.
- [X] T011 Remove pending-retirement classifications and recovery-oriented planning branches from `src/classification/model.rs` and `src/mutation/plan.rs`.
- [X] T012 Refactor publication execution to use only temporary staging, pre-action revalidation, post-publication verification, and baseline publication without retained payload recovery artifacts in `src/mutation/execution.rs`, `src/mutation/filesystem.rs`, and `src/operation/publication.rs`.
- [X] T013 Adapt the internal deletion action to publish a force-selected absent winner without exposing a public delete command in `src/delete/model.rs`, `src/delete/plan.rs`, and `src/delete/execution.rs`.
- [X] T014 [P] Replace baseline/state schema, policy-revision invalidation, mapping-removal pruning, and dry-run state-preservation coverage in `tests/state_integration.rs` and `tests/project_state_rebinding_integration.rs`.
- [X] T015 [P] Replace ignored-entry, newly discovered, and no-stale-baseline classification coverage in `tests/classification_matrix.rs` and `tests/gripignore_conformance.rs`.
- [X] T016 [P] Update isolated staging, revalidation, drift, and post-publication verification coverage in `tests/push_contention_integration.rs`, `tests/pull_contention_integration.rs`, and `tests/sync_contention_integration.rs`.

**Checkpoint**: Baseline evidence belongs only to current managed membership, ignore-policy changes cannot revive stale evidence, and publication has no user-recoverable payload history.

## Phase 3: User Story 1 - Inspect and synchronize a mapped project (Priority: P1) 🎯 MVP

**Goal**: Let an operator inspect state, view differences, and apply only unambiguous non-absent directional synchronization through `status`, `diff`, `push`, `pull`, and `sync`.

**Independent Test**: In isolated file and tree mappings, verify source-only and destination-only changes synchronize in the correct direction, divergent and one-sided-absent entries block all ordinary operations, `status -e` reports the same findings with a nonzero attention result, and `diff` does not alter baseline state.

### Tests for User Story 1

- [X] T017 [P] [US1] Add `status`, `status -e`, destination-selector, JSON, and invalid-metadata contract coverage in `tests/classification_cli_contract.rs`.
- [X] T018 [P] [US1] Add non-mutating detailed-difference and destination-selector coverage in `tests/classification_cli_contract.rs`.
- [X] T019 [P] [US1] Add ordinary push eligibility, one-sided-absence blocking, and dry-run complete-plan-equivalence/non-mutation coverage in `tests/push_cli_contract.rs` and `tests/push_planning.rs`.
- [X] T020 [P] [US1] Add ordinary pull eligibility, one-sided-absence blocking, and dry-run complete-plan-equivalence/non-mutation coverage in `tests/pull_cli_contract.rs` and `tests/pull_planning.rs`.
- [X] T021 [P] [US1] Add mixed-direction sync, all-or-nothing conflict blocking, unbaselined-difference blocking, and dry-run complete-plan-equivalence/non-mutation coverage in `tests/sync_cli_contract.rs` and `tests/sync_planning.rs`.
- [X] T022 [P] [US1] Add file and tree filesystem acceptance coverage for successful ordinary publication and baseline refresh in `tests/push_filesystem_integration.rs`, `tests/pull_filesystem_integration.rs`, and `tests/sync_filesystem_integration.rs`.

### Implementation for User Story 1

- [X] T023 [US1] Implement `status` validation, destination-space selection, ordinary attention reporting, and `-e` exit behavior in `src/lib.rs` and `src/result.rs`.
- [X] T024 [US1] Implement pure `diff` inspection with source/destination detail and destination-space selection in `src/lib.rs` and `src/observation/mod.rs`.
- [X] T025 [US1] Implement normal non-absent source-to-destination planning, execution, and baseline refresh for `push` in `src/push/mod.rs` and `src/mutation/plan.rs`.
- [X] T026 [US1] Implement normal non-absent destination-to-source planning, execution, and baseline refresh for `pull` in `src/pull/mod.rs` and `src/mutation/plan.rs`.
- [X] T027 [US1] Implement conservative mixed-direction `sync` planning and atomic selected-scope execution in `src/sync/mod.rs` and `src/mutation/plan.rs`.
- [X] T028 [US1] Route `-d`/`--destination` selection consistently through `status`, `diff`, `push`, `pull`, and `sync` in `src/cli.rs` and `src/lib.rs`.

**Checkpoint**: User Story 1 works independently with no conflict winner selected implicitly and no mutation from `diff` or dry-run commands.

## Phase 4: User Story 2 - Declare and manage mappings with flat commands (Priority: P1)

**Goal**: Let an operator add, list, and remove mappings directly while preserving endpoint payloads and baseline lifecycle rules.

**Independent Test**: In isolated projects, add file and tree mappings from either existing endpoint, reject two missing endpoints and incompatible kinds, list all or one source mapping, and remove a mapping without changing payloads or permitting baseline reuse after re-addition.

### Tests for User Story 2

- [X] T029 [P] [US2] Add flat `add`, `list`, and `remove` parser/help/JSON contract coverage in `tests/mapping_cli_contract.rs`.
- [X] T030 [P] [US2] Add endpoint-presence, kind-inference, incompatible-kind, and no-copy/no-create registration coverage in `tests/mapping_registry_integration.rs` and `tests/mapping_topology_integration.rs`.
- [X] T031 [P] [US2] Add coordinated remove, endpoint-preservation, baseline-pruning, and re-add-as-new coverage in `tests/mapping_cli_contract.rs` and `tests/state_integration.rs`.
- [X] T032 [P] [US2] Add direct `.gripignore` exclusion and later-unignore-as-new coverage for tree mappings in `tests/gripignore_conformance.rs`.

### Implementation for User Story 2

- [X] T033 [US2] Implement two-endpoint discovery for `add`, requiring one existing endpoint, inferring a compatible file/tree kind, rejecting two absent or incompatible endpoints, and creating no peer payload in `src/mapping.rs`.
- [X] T034 [US2] Implement baseline establishment only for equivalent endpoints during `add` and unbaselined declaration for differing or absent peers in `src/mapping.rs` and `src/baseline.rs`.
- [X] T035 [US2] Implement direct `list [SOURCE]` behavior and source-space filtering without a separate inspect/show command in `src/mapping.rs` and `src/lib.rs`.
- [X] T036 [US2] Implement `remove SOURCE` as a coordinated mapping-descriptor and baseline-pruning transition without endpoint publication in `src/mapping.rs`, `src/registry/publication.rs`, and `src/state/publication.rs`.

**Checkpoint**: User Story 2 works independently, exposes no nested mapping family, and never carries baseline evidence across a removal or ignore-policy lifecycle break.

## Phase 5: User Story 3 - Resolve a conflict through an explicit direction (Priority: P2)

**Goal**: Allow a deliberate, exact-one-entry `push -f` or `pull -f` to make the named direction authoritative, including intentional absence propagation.

**Independent Test**: In isolated divergent, one-sided-absent, and selected-tree-entry scenarios, verify `push -f` and `pull -f` change only one exact managed entry, make the selected complete winner prevail, refresh baseline evidence, and reject omitted, broad, ambiguous, or multi-entry selection before endpoint mutation.

### Tests for User Story 3

- [X] T037 [P] [US3] Add `-f`/`--force` parsing, exact-selector, and `sync -f` rejection coverage in `tests/push_cli_contract.rs`, `tests/pull_cli_contract.rs`, and `tests/sync_cli_contract.rs`.
- [X] T038 [P] [US3] Add divergent complete-winner and baseline-refresh plan coverage in `tests/push_planning.rs`, `tests/pull_planning.rs`, and `tests/classification_matrix.rs`.
- [X] T039 [P] [US3] Add forced one-sided-absence propagation, unchanged-unselected-entry, and post-publication verification coverage in `tests/push_filesystem_integration.rs` and `tests/pull_filesystem_integration.rs`.
- [X] T040 [US3] Add force preflight rejection coverage for mapping-wide, tree-wide, ambiguous, and multi-entry selectors in `tests/push_planning.rs` and `tests/pull_planning.rs` after T038.

### Implementation for User Story 3

- [X] T041 [US3] Add `-f`/`--force` only to direct `push` and `pull` arguments, including command-specific help text, in `src/cli.rs`.
- [X] T042 [US3] Implement exact-one-managed-entry force selection and source/destination complete-winner planning in `src/push/mod.rs`, `src/pull/mod.rs`, and `src/observation/model.rs`.
- [X] T043 [US3] Execute forced payload or absence publication through the internal deletion/publish path with revalidation, verification, and baseline refresh in `src/mutation/plan.rs`, `src/mutation/execution.rs`, and `src/delete/execution.rs`.

**Checkpoint**: User Story 3 resolves only an explicitly targeted conflict through its named direction and does not provide a separate resolve or delete interface.

## Phase 6: Polish and cross-cutting completion

**Purpose**: Remove the superseded product surface, align current documentation, and validate the complete replacement.

- [X] T044 [P] Update the command hierarchy, output, mapping, inspection, directional force, `.gripignore`, and no-recovery guidance in `README.md` and `docs/product-definition.md`.
- [X] T045 [P] Update current wiki command, mapping, baseline, synchronization, state, deletion/recovery, safety, and product-model pages in `wiki/pages/command-line-and-path-selection.md`, `wiki/pages/mappings-and-managed-membership.md`, `wiki/pages/baseline-classification-and-status.md`, `wiki/pages/synchronization-and-conflicts.md`, `wiki/pages/configuration-and-state.md`, `wiki/pages/deletion-retirement-and-recovery.md`, `wiki/pages/safety-and-recovery-model.md`, and `wiki/pages/grip-product-model.md`.
- [X] T046 Run the feature quickstart scenarios and the full repository quality gate with `mise run validate` and `mise run performance`, recording any intentional performance-test outcome in `specs/011-git-command-hierarchy/quickstart.md`.
- [X] T047 Run `/speckit-analyze` against `specs/011-git-command-hierarchy/` and reconcile every reported artifact inconsistency before implementation handoff.

**Checkpoint**: The repository exposes only the approved ten-command interface, all current documentation agrees with it, and all required automated validation passes.

## Dependencies and execution order

### Phase dependencies

- **Phase 1** establishes the CLI grammar and result layer; it can start immediately.
- **Phase 2** depends on Phase 1 and blocks all user stories because each uses the same state and publication invariants.
- **User Story 1** depends on Phase 2 and is the MVP.
- **User Story 2** depends on Phase 2. It may proceed after Phase 2 alongside User Story 1, but its mapping lifecycle must be complete before final acceptance of the synchronization workflow.
- **User Story 3** depends on Phase 2 and the ordinary push/pull publication path from User Story 1.
- **Phase 6** depends on all selected user stories.

### User story dependency graph

```text
Setup → Foundational → US1 (MVP) → US3
                     └→ US2

US1 + US2 + US3 → Polish and validation
```

### Parallel opportunities

- T005 and T006 can run in parallel after T001 through T004 establish the intended contract.
- T014 through T016 can run in parallel with one another after T007 through T013 define the shared state transition.
- T017 through T022 can run in parallel as test-first work for User Story 1.
- T029 through T032 can run in parallel as test-first work for User Story 2.
- T037 through T039 can run in parallel as test-first work for User Story 3; T040 follows T038 because they share planning files.
- T044 and T045 can run in parallel once command behavior is finalized.

## Parallel example: User Story 1

```text
Task: "Add status and exit-code contracts in tests/classification_cli_contract.rs"
Task: "Add pure diff contracts in tests/classification_cli_contract.rs"
Task: "Add push contracts in tests/push_cli_contract.rs and tests/push_planning.rs"
Task: "Add pull contracts in tests/pull_cli_contract.rs and tests/pull_planning.rs"
Task: "Add sync contracts in tests/sync_cli_contract.rs and tests/sync_planning.rs"
```

## Implementation strategy

### MVP first

1. Complete Phase 1 and Phase 2 so all commands use one safe baseline and publication model.
2. Complete User Story 1 and run its isolated contract and filesystem tests.
3. Demonstrate `status`, `diff`, and unambiguous ordinary `push`, `pull`, and `sync` before beginning mapping lifecycle and forced-direction work.

### Incremental delivery

1. Add User Story 2 to replace nested mapping setup and prove that removal/ignore transitions cannot revive stale state.
2. Add User Story 3 to make deliberate conflict and deletion choices explicit through `push -f` and `pull -f`.
3. Remove the old surface only after its replacements and rejection tests are present, then update current documentation and run all validation.

## Format validation

All 47 tasks use the required checklist format: checkbox, sequential task ID, optional parallel marker only where work is independently parallelizable, user-story label for every story task, and one or more explicit repository file paths.
