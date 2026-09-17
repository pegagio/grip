# Tasks: Operational Output and State Rebinding

**Input**: [spec.md](./spec.md), [plan.md](./plan.md), [research.md](./research.md), [data-model.md](./data-model.md), [quickstart.md](./quickstart.md), and [human-output.md](./contracts/human-output.md)

**Tests**: Automated isolated filesystem and CLI tests are required by the specification.

## Phase 1: Setup

**Purpose**: Identify established diff, mutation, blocker, and binding evidence to reuse.

- [X] T001 Confirm existing structured diff, action, blocker, and binding evidence supports requested human output in `src/lib.rs`, `src/result.rs`, and `src/state/rebinding.rs`

## Phase 2: User Story 1 - Read focused external diff output (Priority: P1)

**Goal**: Preserve comparison-program standard output and provide opt-in readable diagnostics.

**Independent Test**: Use a selected recording external diff with and without verbosity.

- [X] T002 [US1] Route eligible selected external handoffs around normal human rendering and emit verbose inspection only to standard error in `src/lib.rs`
- [X] T003 [US1] Render property-oriented verbose diff explanation with managed values, concise capability roles, and no unmanaged compatibility notes in `src/result.rs`
- [X] T004 [US1] Cover standard-output ownership and verbose property explanations in `tests/diff_cli_contract.rs`

## Phase 3: User Story 2 - Understand synchronization results (Priority: P2)

**Goal**: Make payload-file counts and aggregate force blockers actionable.

**Independent Test**: Exercise tree push and blocked aggregate force fixtures.

- [X] T005 [US2] Count only file-level payload actions in human mutation summaries and render aggregate force blocker evidence in `src/result.rs`
- [X] T006 [US2] Cover tree payload-file counts and aggregate blocker path rendering in `tests/push_cli_contract.rs`

## Phase 4: User Story 3 - Change a diff profile without blocking push (Priority: P3)

**Goal**: Let output-only descriptor settings change without invalidating payload authorization.

**Independent Test**: Change source content and an isolated diff profile in a bound project before pushing.

- [X] T007 [US3] Treat unchanged location and resolved mapping identity as rebind-eligible despite a raw descriptor digest change in `src/state/rebinding.rs`
- [X] T008 [US3] Cover source drift plus a valid diff-profile change while retaining drift-blocking regression coverage in `tests/project_state_rebinding_integration.rs`

## Phase 5: Polish and Cross-Cutting Validation

**Purpose**: Document behavior and validate the complete implementation.

- [X] T009 Document external-diff output ownership, verbose inspection, and binding behavior in `README.md`
- [X] T010 Document payload-file counts, aggregate blockers, verbose diff output, and mapping-relevant binding behavior in `docs/product-definition.md`
- [X] T011 Run focused integration suites, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `git diff --check`

## Dependencies and Execution Order

- T001 establishes the existing evidence boundary.
- US1 and US2 depend on T001 and are otherwise independent.
- US3 depends on the existing State V4 binding model and does not weaken US1 or US2.
- T009-T011 follow the completed stories.

## Implementation Strategy

The completed MVP is US1: selected external comparison output is clean and verbose explanation is opt-in. US2 makes mutation transcripts actionable, and US3 restores intended push behavior after diff-profile edits. All tasks are complete; verification remains a separate roadmap-debrief gate.
