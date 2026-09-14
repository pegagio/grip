---
description: "Implementation tasks for Feature 021 large-file operation performance"
---

# Tasks: Large-File Operation Performance

**Input**: Design documents from `specs/021-large-file-performance/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/large-file-performance.md](contracts/large-file-performance.md), and [quickstart.md](quickstart.md)

**Tests**: Required. The specification requires isolated regression coverage for the representative workload and preservation of existing safety and command semantics.

**Organization**: Tasks are grouped by user story so each outcome can be implemented and verified independently.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the isolated reusable fixture and release-command support needed by the representative workloads.

- [X] T001 Add reusable isolated approximately 19 MiB unequal regular-file fixture builders and workload metadata to `tests/performance_acceptance.rs`.
- [X] T002 Add deterministic release-command construction, warm-up, 100-sample measurement, and p50/p95/maximum reporting helpers to `tests/performance_acceptance.rs`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Make the performance harness safe and deterministic before it is used as evidence or a regression gate.

**⚠️ CRITICAL**: Do not begin user-story work until the isolated fixture, 100-sample measurement helpers, and explicit preflight behavior are complete.

- [X] T003 Make `tests/performance_acceptance.rs` fail with specific diagnostics when the release binary or isolated temporary fixture cannot be prepared, and prevent any fallback to an operator-managed path.

**Checkpoint**: The isolated release harness can measure both specified workflows and report trustworthy distributions without accessing operator files.

---

## Phase 3: User Story 1 - Diagnose a Slow Large-File Operation (Priority: P1) 🎯 MVP

**Goal**: Give maintainers reproducible release-build evidence for the reported `grip add` and `grip status` latency and its cause.

**Independent Test**: Run the ignored release harness against its temporary, unequal approximately 19 MiB fixtures and inspect the workload conditions, deterministic command results, and p50/p95/maximum distributions for both operations.

### Implementation for User Story 1

- [X] T004 [US1] Add the representative `grip add` and JSON `grip status` workload cases to `tests/performance_acceptance.rs`, recording 100 warm-run p50, p95, maximum, expected exit status, and expected output for each case.
- [X] T005 [US1] Run the new baseline workloads and append the measured environment, distributions, and demonstrated causal finding to `specs/021-large-file-performance/research.md`; do not select an optimization unless an over-target workflow confirms its cause.

**Checkpoint**: The baseline evidence identifies whether the reported workflows exceed the target and establishes whether an observation change is warranted.

---

## Phase 4: User Story 2 - Complete Large-File Work Promptly (Priority: P2)

**Goal**: Remove only the baseline-confirmed redundant work while retaining full file evidence, stability checks, add fencing, and established command results.

**Independent Test**: Run focused observation and CLI-contract tests using unequal regular endpoints; `grip add` must publish the same mapping and initial state without changing either payload, while `grip status` returns the existing classification and preserves drift failures.

### Tests for User Story 2

- [X] T006 [P] [US2] Add focused observation tests for reuse of discovery-backed normal file-mapping evidence, direct-observation fallback when no discovery record exists, and retention of independent stable-observation passes in `src/observation/mod.rs`.
- [X] T007 [P] [US2] Add an unequal large regular-file `grip add` regression covering mapping publication, initial comparison state, unchanged source and destination bytes, and fenced-add behavior in `tests/mapping_cli_contract.rs`.
- [X] T008 [P] [US2] Add an unequal large regular-file JSON `grip status` regression covering the established selection, classification, output, and changed-during-observation failure behavior in `tests/classification_cli_contract.rs`.

### Implementation for User Story 2

- [X] T009 [US2] After T005 records an over-target duplicate-observation cause, refactor normal file-mapping handling in `src/observation/mod.rs` to reuse the discovery-backed complete endpoint observations within each pass, retain direct observation for discovery-absent mappings, and preserve descriptor-bound SHA-256, metadata, no-follow, and two-pass validation behavior.
- [X] T010 [US2] Run the focused tests and release workloads after the observation change, then append before-and-after distributions and retained safety evidence to `specs/021-large-file-performance/research.md`.

**Checkpoint**: `grip add` and `grip status` retain their safety and semantic contracts while only the measured redundant complete-file work is removed.

---

## Phase 5: User Story 3 - Keep the Improvement Durable (Priority: P3)

**Goal**: Turn the representative workload into a repeatable release-build performance acceptance gate.

**Independent Test**: Run `mise run performance`; it must measure both workflows with 100 warm samples and fail with a clear preflight diagnostic or the recorded distribution if it cannot establish p95 at or below one second, without touching non-temporary paths.

### Implementation for User Story 3

- [X] T011 [US3] Make the representative `grip add` and `grip status` cases in `tests/performance_acceptance.rs` fail explicitly when their 100-sample release-build p95 exceeds one second while continuing to report p50, p95, maximum, workload conditions, and preflight failures.
- [X] T012 [US3] Update the expected performance-gate output and safe-environment guidance in `specs/021-large-file-performance/quickstart.md` to match the implemented release-binary and isolated-fixture preflight behavior.

**Checkpoint**: A later slowdown or unsafe measurement setup in either reported workflow is visible as an isolated, actionable performance-gate failure.

---

## Phase 6: Polish & Cross-Cutting Validation

**Purpose**: Reconcile evidence and run the feature's complete validation sequence.

- [X] T013 Reconcile the measured decisions, before-and-after evidence, and selected operation-local remedy with `specs/021-large-file-performance/spec.md` and its linked design artifacts, correcting `specs/021-large-file-performance/research.md` if the implementation differs from the plan.
- [X] T014 Validate the commands and expected results documented in `specs/021-large-file-performance/quickstart.md` by running `cargo fmt --check`, focused tests for `src/observation/mod.rs`, `tests/mapping_cli_contract.rs`, and `tests/classification_cli_contract.rs`, `mise run performance`, `mise run validate`, and `git diff --check`.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: T001 → T002.
- **Foundational (Phase 2)**: T003 depends on T001 and T002 and blocks all user-story work.
- **User Story 1 (P1)**: T004 depends on T003; T005 depends on T004 and an executed baseline run.
- **User Story 2 (P2)**: T006, T007, and T008 depend on T003 and can proceed in parallel; T009 depends on T005, T006, T007, and T008; T010 depends on T009.
- **User Story 3 (P3)**: T011 depends on T005 and, when T009 is performed, T010; T012 depends on T011.
- **Polish (Phase 6)**: T013 depends on T005 and, when T009 is performed, T010; T014 depends on T011, T012, and T013.

### User Story Dependencies

- **US1 (P1)** provides the baseline and causal evidence; it has no dependency on another user story after the foundation is ready.
- **US2 (P2)** may start its regression tests after the foundation, but its implementation is gated on the US1 baseline finding and is independently testable through the focused observation and CLI-contract suites.
- **US3 (P3)** makes the US1 workload durable and therefore depends on the completed measurement and post-change evidence from US2.

### Parallel Opportunities

- T006, T007, and T008 modify separate files and can run in parallel after T003.
- T005's baseline evidence collection may run while T006–T008 are being prepared, provided it records the unmodified baseline revision.
- T012 can be drafted while T011 is under review, but it must be finalized after the implemented harness behavior is known.

## Parallel Example: User Story 2

```text
Task: "T006 Add observation reuse and fallback tests in src/observation/mod.rs"
Task: "T007 Add unequal-file add publication regression in tests/mapping_cli_contract.rs"
Task: "T008 Add unequal-file status regression in tests/classification_cli_contract.rs"
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001–T003 to establish a clean, deterministic representative harness.
2. Complete T004 and run T005 to record the baseline and causal diagnosis.
3. Stop if the measured baseline does not establish an over-target duplicate-observation cause; do not make the proposed code change without new approved direction.

### Incremental Delivery

1. Build and measure the isolated workload (US1).
2. If the evidence supports it, add semantic regression coverage and make the operation-local observation change (US2).
3. Record post-change evidence, then enforce the one-second p95 acceptance gate (US3).
4. Complete the validation sequence before seeking review.

## Notes

- `[P]` tasks touch different files and can proceed in parallel.
- The task list deliberately excludes persistent caches, indexes, background work, parallel hashing, broad locks, schema changes, and public CLI changes.
- The ignored performance harness is a representative-workstation acceptance check; it is not a substitute for the normal correctness suite.
