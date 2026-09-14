---

description: "Implementation tasks for Feature 022 force mapping replacement"
---

# Tasks: Force Mapping Replacement

**Input**: Design documents from `specs/022-force-mapping-replacement/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), and [CLI contract](contracts/force-add-replacement.md)

**Tests**: Required. The specification explicitly requires isolated regression coverage for replacement, removal/re-add, ownership boundaries, payload non-mutation, and publication/drift safety.

**Organization**: Tasks are grouped by user story. Shared publication safety is completed before story work so descriptor and State V4 membership cannot diverge.

## Phase 1: Setup (Shared Contract Surface)

**Purpose**: Expose the explicit CLI intent without changing ordinary add behavior.

- [X] T001 Add the `-f`/`--force` boolean argument to `AddArgs` in `src/cli.rs`, defaulting to ordinary add behavior when omitted.

**Checkpoint**: `grip add --help` accepts the documented option, while existing add invocations parse unchanged.

## Phase 2: Foundational (Recoverable Descriptor/State Transitions)

**Purpose**: Make the existing bounded add fence capable of protecting the complete metadata transitions required by forced replacement and exact removal.

**⚠️ CRITICAL**: Complete this phase before publishing any replacement or removal candidate.

- [X] T002 Refactor the fence model and persistence validation in `src/state/add_fence.rs` to record a descriptor/state transition’s prior and candidate pairs, protected mapping selector, operation/result context, and verified completion-or-restoration behavior.
- [X] T003 Update fenced-transition resume, restoration, and candidate-current checks in `src/lib.rs` so add and remove retries can complete or restore the exact recorded pair without reporting a mixed descriptor/State V4 result as success.
- [X] T004 Update descriptor-bound State V4 candidate preparation and validation seams in `src/state/mod.rs` and `src/state/publication.rs` so a candidate may retire selected mapping identities while retaining unrelated accepted baselines.
- [X] T005 Add focused publication-fence fault and recovery fixtures in `tests/mapping_cli_contract.rs` and `tests/state_integration.rs` that prove a failed descriptor/state transition restores or completes its complete prior/candidate pair.

**Checkpoint**: The shared publication mechanism can safely represent a replacement or removal transition, with no new lock, cache, service, or payload mutation.

## Phase 3: User Story 1 - Replace an Active Mapping Deliberately (Priority: P1) 🎯 MVP

**Goal**: Allow an operator to replace exactly one active equal-destination file mapping through explicit `grip add --force`.

**Independent Test**: In a temporary project, add `target/debug/grip -> ~/.local/bin/grip`, force-add `target/release/grip` to the same resolved destination, and prove that only the release declaration and its initial comparison evidence remain while both endpoint payloads are byte-for-byte unchanged.

### Tests for User Story 1

- [X] T006 [US1] Add exact equal-destination file-replacement and alternate-destination-spelling CLI fixtures in `tests/mapping_cli_contract.rs`.
- [X] T007 [US1] Add human and JSON replacement-result assertions, including `mapping` and `replaced_mapping`, in `tests/mapping_cli_contract.rs`.
- [X] T008 [US1] Add replacement initial-comparison and endpoint-payload non-mutation fixtures in `tests/mapping_cli_contract.rs`.

### Implementation for User Story 1

- [X] T009 [US1] Add replacement-specific structured outcome details and `Mapping replaced:` human rendering in `src/result.rs` while retaining the ordinary add result shape.
- [X] T010 [US1] Implement exact displaced-file selection and full replacement-candidate descriptor resolution in `src/lib.rs`, removing only one distinct equal-canonical-destination declaration before reusing complete registry ownership validation.
- [X] T011 [US1] Build the replacement State V4 candidate in `src/lib.rs` by retiring only the displaced mapping identities, applying add-time baseline construction to the requested mapping, and publishing the descriptor/state pair through the fenced transition.
- [X] T012 [US1] Preserve replacement outcome context across fenced retry and stale-candidate restoration in `src/lib.rs` and `src/state/add_fence.rs` so a recovered successful replacement does not render as an ordinary add.

**Checkpoint**: The P1 workflow succeeds only for one exact file owner, preserves payloads, and reports both mappings deterministically.

## Phase 4: User Story 2 - Re-add After Exact Removal Without Force (Priority: P2)

**Goal**: Ensure exact removal retires both mapping ownership and current comparison evidence so a normal subsequent add may reuse that destination.

**Independent Test**: In a temporary project, remove the exact source that owns a destination, add a different source to that destination without `--force`, and verify success; remove a different source instead and verify the active owner still rejects the add.

### Tests for User Story 2

- [X] T013 [US2] Add exact-remove then different-source ordinary-re-add fixtures, including State V4 baseline absence and new initial evidence, in `tests/mapping_cli_contract.rs`.
- [X] T014 [US2] Add unrelated-remove regression fixtures that retain the actual owner’s conflict, state, and endpoint payloads in `tests/mapping_cli_contract.rs`.
- [X] T015 [US2] Add removal transition fault/retry fixtures in `tests/mapping_cli_contract.rs` that reject partial descriptor/state retirement.

### Implementation for User Story 2

- [X] T016 [US2] Change exact removal in `src/lib.rs` to construct a descriptor-bound state candidate that filters only the selected mapping declaration and baseline identities before fenced publication.
- [X] T017 [US2] Handle fenced exact-removal retry, restoration, and successful `Mapping removed:` result completion in `src/lib.rs` and `src/state/add_fence.rs`.

**Checkpoint**: Exact removal leaves no stale evidence, while removal of a different mapping never authorizes reuse of another mapping’s destination.

## Phase 5: User Story 3 - Preserve Explicit Ownership Boundaries (Priority: P3)

**Goal**: Keep force narrowly limited to one exact equal-destination file replacement and preserve every other existing validation boundary.

**Independent Test**: Exercise zero-match, multiple-match, source-overlap, tree, nested, unsafe, and incompatible fixtures with `--force`; each fails with no mapping, State V4, fence, or endpoint-payload change.

### Tests for User Story 3

- [X] T018 [US3] Add zero-match and multiple-eligible-match force rejection fixtures in `tests/mapping_cli_contract.rs`.
- [X] T019 [P] [US3] Add force rejection fixtures for source-overlap, tree, nested, and incompatible topology cases in `tests/mapping_topology_integration.rs`.
- [X] T020 [P] [US3] Add ordinary conflicting-add regression assertions for unchanged error category, descriptor bytes, State V4 bytes, fence absence, and endpoint payloads in `tests/mapping_cli_contract.rs`.

### Implementation for User Story 3

- [X] T021 [US3] Tighten force-admission and error mapping in `src/lib.rs` so zero, multiple, non-file, and residual-conflict candidates fail before any fence or metadata publication while ordinary add retains `destination_overlap` behavior.
- [X] T022 [US3] Verify complete-candidate ownership conflict reporting remains deterministic for forced requests in `src/mapping.rs`, `src/registry/mod.rs`, and `src/lib.rs` without weakening existing source/tree/nested validation.

**Checkpoint**: `--force` is a single precise membership replacement, not a generalized ownership override.

## Phase 6: Polish and Cross-Cutting Validation

**Purpose**: Make the new behavior discoverable and prove the complete change set meets project quality gates.

- [X] T023 [P] Document the `grip add --force` replacement boundary, exact-removal re-add behavior, and payload non-mutation guarantee in `README.md`.
- [X] T024 Run the focused Feature 022 suites from `specs/022-force-mapping-replacement/quickstart.md` and resolve any failures in the owning `src/` or `tests/` file.
- [X] T025 Run `mise run validate` and the repository’s Feature 022 quickstart checks in `mise.toml` and `specs/022-force-mapping-replacement/quickstart.md`.

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1**: Can start immediately.
- **Phase 2**: Depends on T001; blocks metadata-changing user-story implementation.
- **US1 (Phase 3)**: Depends on Phase 2. This is the MVP.
- **US2 (Phase 4)**: Depends on Phase 2 and shares its transition machinery with US1; its behavior is independently testable once implemented.
- **US3 (Phase 5)**: Depends on T010 because it verifies the force-admission path; its rejection fixtures may be prepared after Phase 2.
- **Polish (Phase 6)**: Depends on all desired user stories.

### User Story Completion Order

```text
Setup → Foundational transition safety → US1 (MVP)
                                      ├→ US2 exact removal/re-add
                                      └→ US3 retained ownership boundaries
US1 + US2 + US3 → Polish and full validation
```

### Parallel Opportunities

- T019 may proceed in parallel with T018 because it changes `tests/mapping_topology_integration.rs` while T018 changes `tests/mapping_cli_contract.rs`.
- T006–T008, T013–T015, and T018/T020 intentionally remain sequential because each group changes the same CLI contract suite.
- T023 may proceed while the final focused test runs are being prepared; T024 and T025 remain sequential validation gates.

## Parallel Examples

### User Story 1

```text
No safe same-story parallel implementation tasks: T006–T008 deliberately share `tests/mapping_cli_contract.rs` and must be sequenced.
```

### User Story 2

```text
No safe same-story parallel implementation tasks: T013–T015 deliberately share `tests/mapping_cli_contract.rs` and must be sequenced.
```

### User Story 3

```text
Task: "Add force zero/multiple-match fixtures in tests/mapping_cli_contract.rs"
Task: "Add force topology-rejection fixtures in tests/mapping_topology_integration.rs"
```

## Implementation Strategy

### MVP First

1. Complete T001–T005 to establish recoverable metadata transitions.
2. Complete T006–T012 to deliver and validate one exact forced file replacement.
3. Stop and run the User Story 1 independent test before extending removal behavior or rejection coverage.

### Incremental Delivery

1. Add US1 as the deliberate replacement path without altering ordinary add.
2. Add US2 so exact removal and later normal re-add share the same descriptor/state safety boundary.
3. Add US3’s full rejection matrix to prove force has not broadened ownership authority.
4. Update the README and run the focused and full validation gates.

## Format Validation

All 25 tasks use the required checkbox, sequential ID, optional parallel marker, user-story label for story tasks, and an exact repository file path.
