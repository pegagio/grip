---

description: "Implementation tasks for Source-Authoritative Mapping Addition"
---

# Tasks: Source-Authoritative Mapping Addition

**Input**: Design documents from `specs/018-source-authoritative-add/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [add-initial-authority.md](contracts/add-initial-authority.md), and [quickstart.md](quickstart.md)

**Tests**: Required. The feature changes state construction, descriptor/state publication, and synchronization behavior; use isolated temporary roots for every new test.

**Organization**: Tasks are grouped by user story. Foundational state and publication primitives must complete before the user-story phases.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: May run in parallel because it uses a different file and has no incomplete-task dependency.
- **[Story]**: User story served by the task.

## Phase 1: Setup

**Purpose**: Establish the focused test and validation entry points without changing product behavior.

- [X] T001 Review the current add baseline and publication seams in `src/lib.rs`, `src/baseline.rs`, `src/registry/publication.rs`, and `src/state/publication.rs` against `specs/018-source-authoritative-add/research.md` before editing.
- [X] T002 [P] Confirm the existing add, status, push, pull, sync, state, and tree test fixtures in `tests/mapping_cli_contract.rs`, `tests/classification_cli_contract.rs`, `tests/push_cli_contract.rs`, `tests/pull_cli_contract.rs`, `tests/sync_cli_contract.rs`, and `tests/state_integration.rs` support isolated temporary roots.

---

## Phase 2: Foundational Publication and State Primitives

**Purpose**: Provide the add-specific accepted-state candidate and bounded compound-publication support required by every user story.

**⚠️ CRITICAL**: Complete this phase before wiring the new `add` behavior.

- [X] T003 Add an add-specific initial-baseline candidate builder in `src/baseline.rs` that records source state for equal pairs, destination state for unequal pairs, and no state for source-only or unmanaged entries while leaving ordinary baseline acceptance unchanged.
- [X] T004 Add bounded compound add-publication support and a mapping-scoped incomplete-add publication fence in `src/registry/publication.rs`, `src/state/mod.rs`, and `src/state/publication.rs`: create and verify the fence before descriptor/state publication; bind its canonical mapping identity and prior/candidate descriptor and State V4 digests; publish descriptor then State V4; verify both candidate digests; and clear the fence only after that verification.
- [X] T005 Add focused endpoint-evidence, fence creation/clearing, state/registry publication, verification, and retry/restoration test seams in `src/registry/publication.rs` and `src/state/publication.rs` for isolated integration tests.

**Checkpoint**: Initial-state construction and descriptor/state publication can support a coherent new mapping without altering classification or mutation planning.

---

## Phase 3: User Story 1 - Add a source that is ready to push (Priority: P1) 🎯 MVP

**Goal**: Adding an unequal file pair leaves both payloads unchanged and makes the source immediately available to ordinary push and sync.

**Independent Test**: Add a differing file pair in a temporary project; confirm status displays a push, push/sync dry runs select it, pull selects no action, and ordinary push makes the pair current.

### Tests for User Story 1

- [X] T006 [P] [US1] Extend unequal-file add and re-add coverage in `tests/mapping_cli_contract.rs` to assert complete content, supported-metadata, and node-presence non-mutation; destination-derived accepted state; the existing `source_only_change` JSON representation and unchanged result shape for an unaffected mapping; and ordinary push instead of forced initial-collision resolution.
- [X] T007 [P] [US1] Add dry-run direction assertions for a newly added unequal file in `tests/push_cli_contract.rs`, `tests/pull_cli_contract.rs`, and `tests/sync_cli_contract.rs`.

### Implementation for User Story 1

- [X] T008 [US1] Wire `execute_add` and its replacement for `establish_added_baseline` in `src/lib.rs` to inspect the candidate mapping, build add-specific state, and invoke the compound publication path under the existing mutation lock.
- [X] T009 [US1] Run the focused mapping, classification, push, pull, and sync tests in `tests/mapping_cli_contract.rs`, `tests/classification_cli_contract.rs`, `tests/push_cli_contract.rs`, `tests/pull_cli_contract.rs`, and `tests/sync_cli_contract.rs` to verify the P1 contract.

**Checkpoint**: A differing file add presents a normal source-to-destination action and succeeds through ordinary push.

---

## Phase 4: User Story 2 - Preserve source authority for managed trees (Priority: P2)

**Goal**: A mixed tree applies source authority per managed member without taking ownership of ignored or destination-only content.

**Independent Test**: Add a tree containing equal, unequal, source-only, ignored, and destination-only entries; verify each member’s status and dry-run behavior with no endpoint mutations during add.

### Tests for User Story 2

- [X] T010 [US2] Add a mixed-tree add fixture in `tests/mapping_cli_contract.rs` covering equal, unequal, source-only, ignored, and destination-only members with complete content, supported-metadata, and node-presence snapshots before and after add.

### Implementation for User Story 2

- [X] T011 [US2] Refine add-time observation and initial-baseline selection in `src/lib.rs` and `src/baseline.rs` so it applies the data-model table independently to every source-defined non-ignored tree member.
- [X] T012 [US2] Run the mixed-tree add coverage in `tests/mapping_cli_contract.rs` and ownership/topology regressions in `tests/mapping_topology_integration.rs` and `tests/gripignore_conformance.rs`.

**Checkpoint**: Tree members have the intended per-entry direction, while ignored and destination-only content remains outside active ownership.

---

## Phase 5: User Story 3 - Retain conservative behavior outside the new initial state (Priority: P3)

**Goal**: Equivalent additions remain current, later destination drift conflicts, and a failed add never exposes a usable incomplete unequal mapping.

**Independent Test**: Add equal and unequal mappings, introduce later drift, and inject descriptor/state publication failures; confirm safe classification and descriptor/state coherence without payload changes.

### Tests for User Story 3

- [X] T013 [P] [US3] Add later-destination-drift and equivalent-add regression coverage in `tests/classification_cli_contract.rs` and `tests/classification_matrix.rs`.
- [X] T014 [P] [US3] Add isolated add-failure coverage in `tests/mapping_registry_integration.rs` and `tests/state_integration.rs` for fence-creation failure with unchanged descriptor/state; registry/state publication and verification failures that retain a fence only for its mapping; same-add retry that completes or restores; fence-clearing failure; and visible matching descriptor/state outcomes.

### Implementation for User Story 3

- [X] T015 [US3] Complete error propagation, mapping-scoped incomplete-add fence detection/retry, descriptor restoration, and visible-publication handling for add in `src/lib.rs`, `src/registry/publication.rs`, and `src/state/publication.rs`: commands explicitly selecting only unfenced mappings continue, while any selected fenced mapping blocks under the existing integrity category without changing unaffected JSON result shapes.
- [X] T016 [US3] Run the focused classification, registry, and state failure suites in `tests/classification_cli_contract.rs`, `tests/classification_matrix.rs`, `tests/mapping_registry_integration.rs`, and `tests/state_integration.rs`.

**Checkpoint**: Existing equivalent, conflict, and publication-safety behaviors remain conservative and test-proven.

---

## Phase 6: Polish and Cross-Cutting Concerns

**Purpose**: Document the new add semantics and verify all regressions before review.

- [X] T017 [P] Update the add semantics and source-authoritative first-push guidance in `README.md` without changing the existing status-output and `diff` documentation.
- [X] T018 [P] Update the mapping and state model description in `docs/product-definition.md` to replace the old unequal-add unbaselined rule.
- [X] T019 Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo build --release` from `mise.toml`, then record any required task flow-back in `specs/018-source-authoritative-add/`.

## Dependencies and Execution Order

```text
Setup: T001, T002
  └─ Foundational: T003 → T004 → T005
       └─ US1: T006 + T007 → T008 → T009
            ├─ US2: T010 → T011 → T012
            └─ US3: T013 + T014 → T015 → T016
                 └─ Polish: T017 + T018 → T019
```

## Parallel Opportunities

- **Setup**: T002 may run while T001 reviews the current code paths.
- **US1**: T006 and T007 update distinct test files and may be prepared in parallel after foundational primitives are available.
- **After US1**: T010, T013, and T014 are independent test work in distinct files; complete their respective implementation tasks before claiming each story complete.
- **Polish**: T017 and T018 modify independent documentation files and may run in parallel after all behavior is verified.

## Implementation Strategy

1. Deliver the MVP through US1: use the destination’s complete state as the initial reference for an unequal file and verify ordinary push/sync behavior.
2. Extend the same policy to mixed tree membership through US2, preserving ignore and destination-only boundaries.
3. Complete US3’s drift and publication-failure coverage before documenting or validating the feature.
4. Run the full validation suite, then use `$speckit-analyze` before implementation begins, as required by the constitution.

---

## Phase 7: Convergence

- [X] T020 Revalidate current endpoint evidence before completing a fenced add retry in `src/lib.rs`; when the candidate can no longer be verified, restore the recorded prior descriptor/state safely instead of publishing the stale candidate, per FR-007 and US3/AC5 (partial).
- [X] T021 Render the exact fenced mapping as an actionable incomplete-add integrity condition in `src/lib.rs` and `src/result.rs`, including a same-`grip add` retry instruction while continuing to report unrelated mappings, per FR-007, FR-009, and US3/AC5 (partial).
- [X] T022 Add end-to-end compound add fault-injection coverage in `tests/mapping_registry_integration.rs` and `tests/state_integration.rs` for descriptor publication, State V4 publication, visible-but-unconfirmed state publication, fence clearing, endpoint revalidation, retry completion, and verified restoration, per SC-004 and US3/AC3-AC5 (missing).

## Phase 8: Convergence

- [X] T023 Revalidate the fenced candidate’s destination-derived baseline immediately before its initial State V4 publication in `src/lib.rs`; if it is stale, leave the exact mapping fenced and do not publish the stale state, with regression coverage in `tests/mapping_cli_contract.rs`, per FR-007, SC-004, and Constitution III (partial).
- [X] T024 Render and select the exact mapping recorded only in an incomplete-add fence when descriptor publication did not become visible, so `grip status` and selected mutation commands report an actionable blocked mapping while unrelated mappings remain available, with regression coverage in `src/lib.rs`, `src/result.rs`, and `tests/mapping_cli_contract.rs`, per FR-007, FR-009, US3/AC5, and SC-004 (missing).
