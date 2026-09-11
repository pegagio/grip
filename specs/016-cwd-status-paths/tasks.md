# Tasks: Current-Directory Status Paths

**Input**: Design documents from `/specs/016-cwd-status-paths/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [status-paths.md](contracts/status-paths.md), and [quickstart.md](quickstart.md)

**Tests**: Automated tests are required by the feature specification and constitution because this feature changes CLI presentation, selector interpretation, ownership boundaries, and JSON compatibility.

**Organization**: Tasks are grouped by user story so each delivered behavior has an independently verifiable contract.

## Phase 1: Setup

**Purpose**: Establish the feature’s focused test fixtures and preserve existing command-invocation coverage.

- [X] T001 Review and reuse controlled-working-directory command helpers in `tests/support/project.rs` for nested, sibling, and explicitly selected-project scenarios.

---

## Phase 2: Foundational Path Handling

**Purpose**: Provide the shared path-display and safe CWD-resolution primitives required by all user stories.

**⚠️ CRITICAL**: Complete this phase before changing human status output or source-side push behavior.

- [X] T002 Add failing unit coverage for CWD-relative source display and Git-style C quoting in `src/path_policy.rs`.
- [X] T003 Add failing unit coverage for lexical and canonical selected-project containment of CWD-resolved source selectors in `src/path_policy.rs`.
- [X] T004 Implement raw-path relative display, Git-style human escaping, and contained CWD-relative source-selector resolution in `src/path_policy.rs`.
- [X] T005 Capture the invocation directory once and make the non-serialized status display projection available to command execution in `src/lib.rs`.

**Checkpoint**: Shared helpers retain raw path authority, do not alter serialized records, and reject an escaping candidate before ordinary selection logic.

---

## Phase 3: User Story 1 - Copy a source path into push (Priority: P1) 🎯 MVP

**Goal**: An operator can run `grip status` from a given directory and use an ordinary displayed source path unchanged in `grip push` from that directory.

**Independent Test**: From the project root, nested and sibling directories, and an externally located directory with an explicit project, assert that a displayed pushable source label selects the same managed source entry when supplied to normal, dry-run, and forced `grip push` with the same project selection.

- [X] T006 [P] [US1] Add project-root, nested-directory, and sibling-directory default-human status path assertions in `tests/classification_cli_contract.rs`.
- [X] T007 [P] [US1] Add normal, dry-run, and forced status-to-push round-trip coverage, including an explicit project selected from outside its root, in `tests/portable_mapping_integration.rs`.
- [X] T008 [US1] Render default-human status source labels from the non-serialized invocation-directory projection while retaining destination labels in `src/result.rs`.
- [X] T009 [US1] Resolve relative source selectors for normal, dry-run, and forced `push` variants from the invocation directory before the existing managed-entry selection pipeline, while retaining absolute-selector rejection, in `src/lib.rs`.
- [X] T010 [US1] Update CWD-aware command fixture use needed by the round-trip integration cases in `tests/support/project.rs`.

**Checkpoint**: An ordinary displayed pushable source path round-trips to the same entry from the unchanged working directory.

---

## Phase 4: User Story 2 - Read concise status without losing endpoint context (Priority: P2)

**Goal**: Status remains recognizably concise: source labels change only in their CWD-relative representation, while section grammar and destination labels stay intact.

**Independent Test**: From a nested directory, exercise push, pull, conflict, needs-baseline, clean, and empty-scope status results; verify their established headings, symbols, order, and destination rendering.

- [X] T011 [US2] Add regression coverage for push, pull, conflict, baseline-needed, clean, and empty default-human status rendering in `tests/classification_cli_contract.rs` and `src/result.rs`.
- [X] T012 [US2] Preserve the established status grouping, stable ordering, relation symbols, blocker visibility, clean message, and destination rendering while applying source display projection in `src/result.rs`.

**Checkpoint**: The concise Feature 015 status grammar remains intact and only source-side human labels depend on the invocation directory.

---

## Phase 5: User Story 3 - Preserve automation and safe selection boundaries (Priority: P3)

**Goal**: Machine-readable status remains compatible, and CWD-relative `push` cannot inspect or mutate a path outside the selected project.

**Independent Test**: Assert unchanged JSON source-path values and exit behavior, then verify traversal and symlink escape candidates fail before mutation or unmanaged-path inspection.

- [X] T013 [P] [US3] Add JSON-status regression assertions retaining canonical source-path values and existing exit behavior in `tests/classification_cli_contract.rs`.
- [X] T014 [P] [US3] Add absolute-selector rejection plus traversal and in-project-symlink escape rejection coverage for CWD-relative `push` in `tests/portable_mapping_integration.rs`.
- [X] T015 [US3] Enforce lexical and canonical selected-project containment before source selection, inspection, or mutation for CWD-relative pushes in `src/lib.rs`.
- [X] T016 [US3] Preserve destination-space selector parsing and portable project-relative selector behavior for non-push commands in `tests/portable_mapping_integration.rs`.

**Checkpoint**: JSON remains a stable automation interface, and no CWD-relative source selector can escape the selected project boundary.

---

## Phase 6: Polish and Cross-Cutting Validation

**Purpose**: Document the operator workflow and verify the complete feature without broadening scope.

- [X] T017 [P] Document nested-directory `status` output and its ordinary `push` follow-up in `README.md`.
- [X] T018 [P] Clarify command-specific source-selector semantics and unchanged destination behavior in `docs/product-definition.md`.
- [X] T019 Run the documented nested-directory workflow and complete the full validation suite recorded in `specs/016-cwd-status-paths/quickstart.md`.

---

## Dependencies and Execution Order

### Phase Dependencies

- **Phase 1** has no dependencies.
- **Phase 2** depends on Phase 1 and blocks every user story because it establishes safe path handling and presentation primitives.
- **User Story 1 (P1)** depends on Phase 2 and is the MVP.
- **User Story 2 (P2)** depends on User Story 1 because it verifies the renderer introduced for the round trip.
- **User Story 3 (P3)** depends on the CWD-resolution path introduced for User Story 1 and may proceed alongside final User Story 2 regression work once that resolver exists.
- **Phase 6** depends on the desired user stories being complete.

### User Story Dependencies

- **US1**: Requires the foundational display and CWD-resolution helpers; no dependency on later stories.
- **US2**: Builds on the default-human renderer delivered by US1.
- **US3**: Builds on the CWD-relative `push` resolver delivered by US1 but validates a separate JSON and ownership-boundary contract.

### Parallel Opportunities

- T006 and T007 can be developed in parallel because they cover distinct test files.
- T013 and T014 can be developed in parallel because they cover distinct test files.
- T017 and T018 can be developed in parallel because they change distinct documentation files.

## Parallel Examples

### User Story 1

```text
Task: "Add nested-directory and sibling-directory default-human status path assertions in tests/classification_cli_contract.rs"
Task: "Add status-to-push round-trip coverage in tests/portable_mapping_integration.rs"
```

### User Story 3

```text
Task: "Add JSON-status regression assertions in tests/classification_cli_contract.rs"
Task: "Add traversal and symlink escape rejection coverage in tests/portable_mapping_integration.rs"
```

## Implementation Strategy

### MVP First

1. Complete the shared path-handling foundation.
2. Deliver User Story 1 and run its nested, sibling, and explicit-project round-trip tests.
3. Stop to verify that an operator can act directly on ordinary status output.

### Incremental Delivery

1. Add User Story 2 regression coverage to protect the concise status grammar.
2. Add User Story 3 automation and ownership-boundary protection.
3. Update documentation and run the full validation suite.
