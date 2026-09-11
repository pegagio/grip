---

description: "Task list for Feature 014 relative destination paths"
---

# Tasks: Relative Destination Paths

**Input**: Design documents from `specs/014-relative-destination-paths/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [destination-paths.md](contracts/destination-paths.md), and [quickstart.md](quickstart.md)

**Tests**: Required. The feature specification requires isolated filesystem coverage for declaration parsing, persistence, CWD invariance, copied-project portability, existing destination forms, and unchanged ownership/topology behavior.

**Organization**: Tasks are grouped by user story after a small shared foundation. Each story has an independently executable acceptance test.

## Phase 1: Setup

**Purpose**: Establish the exact pre-change behavior and fixture boundary without changing project configuration or adding dependencies.

- [X] T001 Review the existing destination declaration, runtime resolution, and source-input boundaries in src/path_policy.rs, src/mapping.rs, and tests/portable_mapping_model.rs before changing the contract

---

## Phase 2: Foundational Path and Persistence Boundaries

**Purpose**: Make every consumer resolve a destination declaration from the correct stable base and ensure publication never reconstructs a retained declaration from its resolved endpoint.

**⚠️ CRITICAL**: Complete this phase before the user-story acceptance work. It preserves the existing ownership, topology, state, and publication safety boundaries for every declaration form.

- [X] T002 Extend the destination declaration parser and resolver for exact persisted relative spellings with project-root-based operational resolution in src/path_policy.rs
- [X] T003 Thread project-root-aware destination resolution through portable mapping resolution in src/mapping.rs
- [X] T004 Update destination-space selector and add preflight resolution to use the selected project root in src/lib.rs
- [X] T005 Update state binding, runtime identity reconstruction, and rebinding to resolve relative destinations from the project root in src/state/mod.rs and src/state/rebinding.rs
- [X] T006 Preserve retained portable declarations during remove publication rather than reconstructing destinations from runtime endpoints in src/registry/publication.rs

**Checkpoint**: Every command path can interpret a stored project-relative destination with the same selected-project base, while existing resolved-endpoint safety checks remain in force.

---

## Phase 3: User Story 1 - Add a Portable Relative Destination (Priority: P1) 🎯 MVP

**Goal**: An operator can add and retain a relative destination such as `../grip-dst/app/`, and the same declaration has stable meaning regardless of CWD or project selection method.

**Independent Test**: In isolated temporary projects, `grip add ./app/ ../grip-dst/app/` records the exact relative destination; `list` and destination-space selection use the same resolved endpoint from project root and not CWD.

### Tests for User Story 1

- [X] T007 [P] [US1] Add accepted/rejected form, exact-spelling persistence, and project-root resolution tests in tests/portable_mapping_model.rs
- [X] T008 [US1] Add CLI add and descriptor round-trip coverage for ./app/ and ../grip-dst/app/ in tests/mapping_cli_contract.rs and tests/portable_mapping_integration.rs
- [X] T009 [P] [US1] Add CWD-invariance and explicit-project destination-space selector coverage in tests/portable_mapping_integration.rs
- [X] T010 [P] [US1] Add resolved-equivalence, equal-endpoint, and recursive-topology coverage for relative destinations in tests/mapping_topology_integration.rs

### Implementation and verification for User Story 1

- [X] T011 [US1] Reconcile parser diagnostics and destination declaration behavior with the accepted and rejected forms in specs/014-relative-destination-paths/contracts/destination-paths.md across src/path_policy.rs and src/lib.rs
- [X] T012 [US1] Run the focused User Story 1 suites in tests/portable_mapping_model.rs, tests/mapping_cli_contract.rs, tests/portable_mapping_integration.rs, and tests/mapping_topology_integration.rs

**Checkpoint**: `grip add ./app/ ../grip-dst/app/` is independently functional, records the exact destination declaration, and preserves all resolved topology protections.

---

## Phase 4: User Story 2 - Retain Existing Destination Choices (Priority: P2)

**Goal**: Adding project-relative destinations does not alter absolute, `~`, or `~/` declaration behavior, and retained mappings keep their original declaration form through state and remove workflows.

**Independent Test**: Add or reload absolute, home, home-relative, and project-relative mappings; copy the project layout; remove an unrelated mapping; and verify each retained declaration remains unchanged while all forms resolve correctly.

### Tests for User Story 2

- [X] T013 [P] [US2] Add compatibility and descriptor-reload coverage for absolute, ~, and ~/ destination declarations alongside relative declarations in tests/portable_mapping_model.rs and tests/portable_mapping_integration.rs
- [X] T014 [P] [US2] Add copied-project state/rebinding coverage for relative destinations in tests/project_clone_portability_integration.rs and tests/project_state_rebinding_integration.rs
- [X] T015 [P] [US2] Add remove-publication regression coverage proving retained relative, absolute, and home-relative declarations are not rewritten in tests/registry_integration.rs and tests/mapping_registry_integration.rs

### Implementation and verification for User Story 2

- [X] T016 [US2] Update the accepted destination-form and portability contract in README.md and docs/product-definition.md
- [X] T017 [US2] Run the focused User Story 2 suites in tests/portable_mapping_model.rs, tests/portable_mapping_integration.rs, tests/project_clone_portability_integration.rs, tests/project_state_rebinding_integration.rs, tests/registry_integration.rs, and tests/mapping_registry_integration.rs

**Checkpoint**: All four destination forms preserve their distinct declarations and remain usable after descriptor reload, state rebinding, copied-project use, and removal of another mapping.

---

## Phase 5: Polish and Cross-Cutting Validation

**Purpose**: Validate the complete implementation, documentation, and manual end-to-end behavior without adding scope.

- [X] T018 Run the Feature 014 manual validation scenarios from specs/014-relative-destination-paths/quickstart.md
- [X] T019 Run the full formatting, lint, test, and release-build validation with mise.toml
- [X] T020 Reconcile completed implementation evidence, task checkboxes, and all Feature 014 artifacts in specs/014-relative-destination-paths/

---

## Dependencies and Execution Order

### Phase dependencies

- **Setup (Phase 1)**: Starts immediately.
- **Foundational (Phase 2)**: Depends on T001 and blocks story acceptance work.
- **User Story 1 (Phase 3)**: Depends on T002–T006 and is the MVP.
- **User Story 2 (Phase 4)**: Depends on the destination declaration foundation and may proceed after Phase 2; run it after US1 when working sequentially to preserve the highest-value acceptance path first.
- **Polish (Phase 5)**: Depends on both user stories.

### User story dependencies

- **US1 (P1)**: Requires T002–T006. It establishes project-relative declaration acceptance, persistence, and stable resolution.
- **US2 (P2)**: Requires T002–T006. It verifies retained compatibility for the existing destination forms and publication/state behavior. It does not need a new public interface beyond the shared foundation.

### Parallel opportunities

- T007, T009, and T010 modify distinct test files and can proceed in parallel after the foundation; T008 is sequential because it shares tests/portable_mapping_integration.rs with T009.
- T013–T015 modify separate compatibility, state, and registry test areas and can proceed in parallel after the foundation.
- T016 documentation can proceed in parallel with the Phase 4 test tasks after the behavior is settled.

## Parallel Example: User Story 1

```text
Task: "Add parser and resolution coverage in tests/portable_mapping_model.rs"
Task: "Add add/descriptor coverage in tests/mapping_cli_contract.rs and tests/portable_mapping_integration.rs"
Task: "Add topology coverage in tests/mapping_topology_integration.rs"
```

## Implementation Strategy

### MVP first

1. Complete T001–T006 to establish project-root-aware resolution and declaration-preserving publication.
2. Complete T007–T012 to prove `grip add ./app/ ../grip-dst/app/` works independently.
3. Stop and validate the US1 checkpoint before expanding compatibility coverage.

### Incremental delivery

1. Add US1 to provide the requested portable project-relative destination workflow.
2. Add US2 to prove that existing absolute and home-relative mappings retain their behavior through every persistence and state path.
3. Complete the quickstart and full validation before considering the feature ready for review.

## Notes

- Every task uses the required checklist format with a sequential ID and exact file path.
- `[P]` marks only tasks that touch independent test or documentation areas after their shared prerequisites are complete.
- The task list intentionally includes no migration, new flag, remote-path syntax, or current-working-directory-relative mode.

## Phase 6: Convergence

- [X] T021 Select the removed mapping detail by resolved source rather than registry index, and add mixed file/tree remove-output coverage in src/lib.rs and tests/mapping_registry_integration.rs per FR-007 (partial)
