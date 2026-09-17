---

description: "Task list for Destination Adoption implementation"
---

# Tasks: Destination Adoption

**Input**: Design documents from `specs/036-destination-adoption/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [pull-adopt.md](contracts/pull-adopt.md), and [quickstart.md](quickstart.md)

**Tests**: Required. This feature changes payloads and accepted state; every mutation path must have isolated temporary-root coverage.

**Organization**: Tasks are grouped by user story so each increment remains independently testable.

## Phase 1: Setup

**Purpose**: Extend the existing isolated test harness with reusable fixtures and assertions for a destination-only member, its ancestor chain, and accepted-state evidence.

- [X] T001 Extend destination-adoption fixture and accepted-state assertion helpers in `tests/support/mod.rs`

## Phase 2: Foundational

**Purpose**: Establish typed adoption mode and exact destination ownership resolution before any mutation behavior is added.

**⚠️ CRITICAL**: Complete this phase before user-story implementation.

- [X] T002 Add mutually exclusive `-a`/`--adopt` and `-s`/`--source` parsing and exact-path validation to `src/cli.rs`
- [X] T003 [P] Add typed adoption operation, request state, plan/result identity, and advisory-warning representation to `src/mutation/model.rs`
- [X] T004 Add exact destination-tree ownership, raw relative identity, source-absence, and ancestor-chain evidence types to `src/observation/model.rs`
- [X] T005 Implement no-follow exact adoption inspection and bounded source-ancestor-chain collection without destination-tree enumeration in `src/observation/mod.rs`

**Checkpoint**: Adoption has a typed, exact, source-defined ownership boundary; normal discovery semantics are unchanged.

## Phase 3: User Story 1 - Adopt an existing destination file (Priority: P1) 🎯 MVP

**Goal**: Explicitly adopt one eligible destination regular file into its paired source path, including only required ancestors and the accepted ignored-path override, then verify and baseline the adoption set.

**Independent Test**: In isolated roots, `grip pull --adopt DESTINATION` and `-a` create an equivalent source file, preserve destination bytes, baseline the target and newly-created ancestors, and leave `status` current. `--dry-run` changes neither endpoint nor state. An ignored target is rejected normally but succeeds with `--adopt --force`, does not alter `.gripignore`, and emits a verified retention recommendation.

### Tests for User Story 1

- [X] T006 [P] [US1] Add long/short adoption option, destination-selector, incompatible-flag, required-path, dry-run, and forced-ignored-adoption CLI contract coverage in `tests/pull_cli_contract.rs`
- [X] T007 [P] [US1] Add direct-parent and grandparent ancestor-chain adoption, destination-preservation, baseline-publication, current-status, and dry-run filesystem coverage in `tests/pull_filesystem_integration.rs`
- [X] T008 [P] [US1] Add global and nested ignore-policy override, unchanged-policy, ancestor-exemption recommendation, and ordinary-discovery retention coverage in `tests/gripignore_conformance.rs`

### Implementation for User Story 1

- [X] T009 [US1] Build deterministic ancestor-first adoption actions and directory-finalization dependencies from exact evidence in `src/mutation/plan.rs`
- [X] T010 [US1] Add staged source-side application, per-action revalidation, and complete-state verification for adoption actions in `src/mutation/execution.rs`
- [X] T011 [US1] Publish accepted baselines only for the fully verified target and newly-created structural ancestors in `src/baseline.rs`
- [X] T012 [US1] Expose effective ignore-policy evaluation and a non-mutating verified exemption-rule-set recommendation for one source-relative adoption target in `src/discovery/ignore_policy.rs`
- [X] T013 [US1] Route `pull --adopt` through exact inspection, plan, ignore eligibility, revalidation, execution, and publication while keeping ordinary and forced pull behavior intact in `src/lib.rs`
- [X] T014 [US1] Render deterministic human and JSON previews, applied results, and forced-ignored retention guidance with policy-file placement in `src/result.rs`

**Checkpoint**: One destination-only regular file, including an explicitly forced ignored member, can be adopted safely and is immediately current without a nested mapping.

## Phase 4: User Story 2 - Preserve source-defined ownership (Priority: P2)

**Goal**: Keep ordinary operations source-defined and make the exact adoption boundary incapable of admitting unselected destination content.

**Independent Test**: Ordinary `pull`, `status`, and `diff` do not import destination-only siblings; adopting one exact sibling admits only its target/ancestor set.

### Tests for User Story 2

- [X] T015 [P] [US2] Add one-selected-sibling and ordinary destination-only non-import regression coverage in `tests/contained_source_tree_integration.rs`
- [X] T016 [P] [US2] Add ordinary `pull`, `status`, and `diff` destination-only non-import regression coverage in `tests/pull_cli_contract.rs`

### Implementation for User Story 2

- [X] T017 [US2] Preserve the ordinary source-defined selection path and prohibit any adoption inspection fallback or sibling enumeration outside `--adopt` in `src/lib.rs` and `src/observation/mod.rs`

**Checkpoint**: Explicit adoption remains the only destination-membership exception, and unselected destination siblings remain unmanaged.

## Phase 5: User Story 3 - Receive safe adoption diagnostics (Priority: P3)

**Goal**: Refuse every ambiguous or unsafe exact adoption with a concrete reason and no endpoint or accepted-state mutation.

**Independent Test**: Isolated attempts for existing source, no source ancestor, tree root/directory/non-regular destination, outside or reserved/conflicting mapping, unsupported metadata, incompatible flags, and stale evidence all identify the requested path/reason and preserve payload/state.

### Tests for User Story 3

- [X] T018 [P] [US3] Add selector-space, existing-source, missing-ancestor, tree-root, directory, outside-tree, and conflicting/reserved-mapping rejection coverage in `tests/pull_cli_contract.rs`
- [X] T019 [P] [US3] Add unsupported-node, managed-metadata, symlink/no-follow, stale-evidence, copy/verification failure, and accepted-state non-publication coverage in `tests/pull_filesystem_integration.rs`

### Implementation for User Story 3

- [X] T020 [US3] Classify and report adoption-specific ownership, topology, metadata, and stale-evidence blockers with exact source/destination context in `src/lib.rs`
- [X] T021 [US3] Ensure adoption plan and execution failures preserve destination payload and suppress publication for every unverified entry in `src/mutation/plan.rs` and `src/mutation/execution.rs`
- [X] T022 [US3] Render actionable human and JSON blocked outcomes for adoption-specific error categories in `src/result.rs`

**Checkpoint**: Unsafe adoption cannot partially copy, claim ownership, or obscure the reason for refusal.

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Keep operator documentation, all contract surfaces, and validation aligned with the implemented behavior.

- [X] T023 [P] Document `pull --adopt`, `-a`, `--force` ignore override, required ancestor behavior, and no-policy-mutation guidance in `README.md`
- [X] T024 [P] Update source-defined membership and exact-adoption product behavior in `docs/product-definition.md`
- [X] T025 Reconcile implementation behavior, examples, and acceptance criteria across `specs/036-destination-adoption/spec.md`, `specs/036-destination-adoption/plan.md`, and `specs/036-destination-adoption/contracts/pull-adopt.md`
- [X] T026 Run formatting and focused adoption suites for `src/cli.rs`, `src/lib.rs`, `src/observation/mod.rs`, `src/mutation/plan.rs`, `src/mutation/execution.rs`, `src/result.rs`, `tests/pull_cli_contract.rs`, `tests/pull_filesystem_integration.rs`, `tests/contained_source_tree_integration.rs`, and `tests/gripignore_conformance.rs`
- [X] T027 Run the complete Rust test suite and record any unrelated pre-existing failure separately from `specs/036-destination-adoption/tasks.md`

Validation note: `cargo test` reaches unrelated existing failures in `contained_source_topology_drift_integration::unsafe_sibling_introduced_before_deletion_revalidation_preserves_the_target`, `diff_cli_contract::verbose_selected_diff_explains_grip_inspection_on_stderr_without_polluting_tool_output`, and modification-time expectations in `metadata_filesystem_integration`. The adoption-focused suites pass, and all remaining tests pass when those unrelated known failures are skipped.

## Phase 7: Convergence

**Purpose**: Close evidence and diagnostic gaps found by post-implementation convergence against the feature specification and constitution.

- [X] T028 Preserve effective `.gripignore` evidence for an adoption request and reject policy drift before action or publication per FR-007 and plan: bounded revalidation (partial)
- [X] T029 Revalidate the complete exact source and destination ancestor chain without following replacement links for every adoption action, with stale-swap non-mutation coverage per FR-004, FR-009, and Constitution III (partial)
- [X] T030 Report the selected path and concrete unsupported metadata or topology reason in adoption failures, and cover the remaining unsafe selector/node/source-state matrix in isolated tests per FR-009, FR-010, SC-003, and Constitution V (partial)

## Dependencies & Execution Order

```text
Setup (T001)
  -> Foundational (T002-T005)
    -> US1 / MVP (T006-T014)
      -> US2 (T015-T017)
        -> US3 (T018-T022)
          -> Polish and validation (T023-T027)
```

### User Story Dependencies

- **US1 (P1)** depends only on the foundational exact-resolution and typed-operation work and includes every one of its acceptance scenarios.
- **US2 (P2)** depends on US1 because it verifies the boundary added by adoption; its ordinary-operation regressions remain independently testable.
- **US3 (P3)** depends on US1's adoption path and validates its refusal boundaries; it may start after US1 but must reconcile with US2 before final validation.

## Parallel Opportunities

- T002 and T003 can proceed in parallel after T001 because they modify independent command and model files.
- T006, T007, and T008 can proceed in parallel after T005 because they cover independent CLI, filesystem, and policy-conformance files.
- T015 and T016 can proceed in parallel after US1 because they cover independent ownership and CLI regression files.
- T018 and T019 can proceed in parallel after US2 because they cover independent diagnostic surfaces.
- T023 and T024 can proceed in parallel after user-story behavior is complete.

## Parallel Example: User Story 1

```text
Task: "Add adoption CLI contract coverage in tests/pull_cli_contract.rs"
Task: "Add adoption filesystem coverage in tests/pull_filesystem_integration.rs"
Task: "Add forced-ignore adoption coverage in tests/gripignore_conformance.rs"
```

## Implementation Strategy

### MVP First

1. Complete T001-T005.
2. Complete T006-T014.
3. Run the US1 independent test criteria before moving to source-membership regressions.

### Incremental Delivery

1. Deliver the complete explicit adoption contract, including the narrow ignored-path override.
2. Verify ordinary operations continue to exclude destination-only content.
3. Add the complete failure matrix and cross-cutting documentation/validation.

Before starting implementation, run `speckit-analyze` as required by the project constitution.
