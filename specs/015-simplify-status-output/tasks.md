# Tasks: Simplify Status Output

**Input**: Design documents from `specs/015-simplify-status-output/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [status-output contract](contracts/status-output.md), and [quickstart.md](quickstart.md)

**Tests**: Required. The specification defines observable human-output, JSON-compatibility, exit-behavior, and safety requirements. Write each listed regression test before its corresponding implementation work.

**Organization**: Tasks are grouped by user story. The one shared renderer boundary is isolated in the foundational phase so each story changes a bounded presentation concern.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it touches a different file and has no incomplete dependency.
- **[Story]**: The user story served by the task.

## Phase 1: Setup

**Purpose**: No project bootstrap, dependency, schema, or configuration change is required. The existing Rust CLI, test fixtures, and validation task are the feature's setup.

## Phase 2: Foundational

**Purpose**: Create the status-only presentation boundary without changing classification, JSON serialization, or the detailed `diff` renderer.

- [X] T001 Extract a dedicated default-human status rendering path in src/result.rs while preserving CommandOutcome classification details, JSON rendering, and the existing detailed renderer for diff

**Checkpoint**: `grip status` can use a specialized human renderer, while `grip diff` and JSON continue through their existing detail paths.

## Phase 3: User Story 1 - Identify Work That Needs Attention (Priority: P1) 🎯 MVP

**Goal**: Operators can identify pushable changes, pullable changes, conflicts, and every nonblocking non-directional reconciliation state from compact, path-pair output.

**Independent Test**: Build isolated fixtures for source-only, destination-only, divergent, equal-but-unbaselined, deletion-convergence, and metadata-migration-ready records; assert the default human result renders only nonempty directional sections and the required symbols.

### Tests for User Story 1

- [X] T002 [US1] Add failing exact human-status contract coverage for Changes to push, Changes to pull, Conflicts, and Needs baseline in tests/classification_cli_contract.rs, including initial-match, baseline-refresh, deletion-convergence, and metadata-migration-ready records as `>-<` without a payload-match assertion

### Implementation for User Story 1

- [X] T003 [US1] Implement deterministic classification grouping, source/destination pair rendering, nonempty section suppression, and symbol selection in src/result.rs, mapping every nonblocking no-direction attention record to `Needs baseline`

**Checkpoint**: The P1 mixed-status output uses the approved four-section grammar without exposing current entries individually or choosing a conflict winner.

## Phase 4: User Story 2 - Confirm That a Scope Is Current (Priority: P2)

**Goal**: Operators receive an immediately understandable clean summary or an unambiguous empty-scope result.

**Independent Test**: Run status on all-current and zero-record isolated scopes and verify their summaries contain no itemized technical detail or misleading clean state.

### Tests for User Story 2

- [X] T004 [US2] Add failing default-human status coverage for all-current summaries, count partitioning, and no-managed-entry scopes in tests/classification_cli_contract.rs

### Implementation for User Story 2

- [X] T005 [US2] Render the concise current-count and empty-scope summary variants in the specialized status path in src/result.rs

**Checkpoint**: Clean and empty scopes are distinguishable at a glance and never emit per-entry diagnostics.

## Phase 5: User Story 3 - Preserve Operational Safety and Automation Contracts (Priority: P3)

**Goal**: The concise human view retains real blockers while hiding no-action diagnostics and leaving existing JSON and exit semantics untouched.

**Independent Test**: Exercise an unknown metadata blocker and an excluded informational attribute, then compare human output, JSON records, and `status -e` behavior against their existing contracts.

### Tests for User Story 3

- [X] T006 [P] [US3] Add failing human-output coverage for visible plain-language blocking metadata and suppressed excluded metadata diagnostics in tests/metadata_cli_contract.rs
- [X] T007 [P] [US3] Add regression coverage that status JSON records, endpoint capabilities, classifications, and exit-code behavior remain unchanged in tests/classification_cli_contract.rs

### Implementation for User Story 3

- [X] T008 [US3] Filter no-action compatibility and capability details from the default status renderer while retaining plain-language blocked metadata explanations and leaving structured output untouched in src/result.rs

**Checkpoint**: Safety blockers remain visible, no-action diagnostics remain quiet, and automation receives the existing structured status interface.

## Phase 6: Polish and Cross-Cutting Concerns

**Purpose**: Document the finished default view and validate the complete feature against its public contract.

- [X] T009 [P] Document concise clean, directional, conflict, and baseline-needed status examples in README.md
- [X] T010 [P] Align the inspection-and-synchronization guidance with the default human status contract in docs/product-definition.md
- [X] T011 Run every scenario in specs/015-simplify-status-output/quickstart.md and the full validation task from mise.toml

## Dependencies and Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No work is required; the existing project tooling is sufficient.
- **Foundational (Phase 2)**: T001 must complete before any story implementation because it isolates `status` presentation from `diff` and JSON behavior.
- **User Story 1 (Phase 3)**: T002 must fail before T003 is implemented; T003 depends on T001.
- **User Story 2 (Phase 4)**: T004 must fail before T005 is implemented; T005 depends on T001 and may follow the P1 renderer.
- **User Story 3 (Phase 5)**: T006 and T007 may run in parallel after the status renderer is available; T008 depends on both tests.
- **Polish (Phase 6)**: T009 and T010 depend on the final renderer contract. T011 depends on all preceding tasks.

### User Story Dependencies

- **US1 (P1)**: Depends on T001 only. It is the MVP and proves the primary directional status grammar.
- **US2 (P2)**: Depends on T001. It verifies the same renderer's clean and empty branches and can be delivered after the P1 grammar.
- **US3 (P3)**: Depends on T001 and the status renderer behavior. It protects the safety and automation boundary after the concise presentation exists.

### Parallel Opportunities

- T006 and T007 can proceed in parallel because they edit different test files.
- T009 and T010 can proceed in parallel because they edit different documentation files.
- No other tasks are marked parallel: `src/result.rs` is intentionally changed through a single ordered sequence to keep the status contract coherent.

## Parallel Example: User Story 3

```text
Task: "Add failing human-output coverage for visible plain-language blocking metadata and suppressed excluded metadata diagnostics in tests/metadata_cli_contract.rs"
Task: "Add regression coverage that status JSON records, endpoint capabilities, classifications, and exit-code behavior remain unchanged in tests/classification_cli_contract.rs"
```

## Implementation Strategy

### MVP First

1. Complete T001 to isolate the status-specific human renderer.
2. Complete T002 and T003.
3. Verify the P1 directional, conflict, and baseline-needed status grammar independently.
4. Stop for review before adding clean/empty and metadata compatibility refinements.

### Incremental Delivery

1. Add US1 for the visible directional sections.
2. Add US2 for clean and empty scope clarity.
3. Add US3 to preserve metadata-safety, JSON, and exit contracts.
4. Update documentation and run the quickstart plus full repository validation.

## Phase 7: Convergence

**Purpose**: Close the remaining documentation gap found after implementation.

- [X] T012 Add a concise clean default-human `grip status` example to README.md per FR-011 (partial)
