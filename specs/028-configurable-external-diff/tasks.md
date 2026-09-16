# Tasks: Configurable External Diff Program

**Input**: Design documents from `/specs/028-configurable-external-diff/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [external-diff.md](./contracts/external-diff.md), and [quickstart.md](./quickstart.md)

**Tests**: Automated tests are required by the specification for direct argv handoff, configuration precedence, JSON/non-launch behavior, endpoint safety, and child exit handling. Use isolated temporary homes and projects only.

**Organization**: Tasks are grouped by user story so each increment remains independently testable after the shared descriptor and configuration foundation is complete.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Task can proceed in parallel with other marked tasks after its stated dependencies are complete.
- **[Story]**: User story traceability label.

## Phase 1: Setup

**Purpose**: Establish focused reusable fixtures and a single contract-test home for external-diff behavior.

- [X] T001 Extend isolated command and recording-executable helpers for external-diff argv, sentinel, exit-code, and signal assertions in `tests/support/project.rs`

**Checkpoint**: Tests can create a disposable selected project, disposable `HOME`, controlled `PATH`, and recording diff program without accessing real user configuration.

## Phase 2: Foundational Configuration and Safety

**Purpose**: Add the compatibility-preserving profile representation and safe two-layer resolution that every external-diff story requires.

**⚠️ CRITICAL**: Complete this phase before launching external programs from `grip diff`.

- [X] T002 Add optional deferred `[diff]` and `[difftool.<name>]` profile preservation to Descriptor V2, retaining strict mapping validation and allowed-key checks in `src/registry/mod.rs`
- [X] T003 Preserve deferred project profile data through registry loading, candidate construction, add/remove publication, and descriptor encoding in `src/registry/publication.rs` and `src/lib.rs`
- [X] T004 Add a read-only, current-user-owned, no-follow-safe machine-wide profile loader for `~/.grip/config.toml` in `src/registry/mod.rs` and `src/home.rs`
- [X] T005 Implement typed layer merge and effective tool resolution: global then project field replacement, whole-array `args` replacement, `GRIP_EXTERNAL_DIFF` bypass, and `diff` fallback in `src/registry/mod.rs`
- [X] T006 [P] Add Descriptor V2/profile round-trip, deferred-invalid-profile, strict-key, and mapping-update-preservation coverage in `tests/registry_integration.rs`
- [X] T007 [P] Add isolated global-profile absent/malformed/unsafe/symlink/no-follow coverage in `tests/project_metadata_security_integration.rs`

**Checkpoint**: Existing mapping-only V2 projects still load and publish safely; external-diff configuration is preserved, safely read, and resolvable without altering mappings or state.

## Phase 3: User Story 1 - Open a Selected Comparison (Priority: P1) 🎯 MVP

**Goal**: Let a human-readable exact selected `grip diff` open one safe managed file or directory comparison with direct source and destination endpoints.

**Independent Test**: In a disposable project, select a managed file and directory using source and destination selector forms, invoke `grip diff`, and verify the recording or fallback executable receives exactly the resolved endpoint pair. Verify unselected human `grip diff` does not invoke it.

- [X] T008 [US1] Add failing exact-selection contract coverage for default `diff`, configured named-tool argv ordering, source/destination selectors, mapping-root directory handoff, and no-selector non-launch in `tests/diff_cli_contract.rs`
- [X] T009 [US1] Refactor inspection execution to expose one typed exact-selection handoff candidate while preserving existing classification result construction in `src/lib.rs`
- [X] T010 [US1] Implement direct endpoint eligibility/revalidation for selected file, tree member, and mapping-root directory handoffs without temporary material in `src/lib.rs` and `src/path_policy.rs`
- [X] T011 [US1] Add process-entrypoint routing that renders the human detailed inspection, executes one direct `std::process::Command` with literal source/destination arguments, and leaves ordinary commands on the existing result path in `src/lib.rs`
- [X] T012 [US1] Connect the effective named-tool resolver to eligible selected human diff handoffs, including selected-tool validation and configuration diagnostics, in `src/lib.rs`
- [X] T013 [US1] Render deterministic human external-comparison completion details without changing existing classification or JSON renderers in `src/result.rs`
- [X] T014 [US1] Update existing detailed-diff regression expectations for human no-selector and destination selection behavior in `tests/classification_cli_contract.rs`

**Checkpoint**: A selected safe human comparison launches once with direct endpoints; unselected human diff remains the broad read-only diagnostic.

## Phase 4: User Story 2 - Configure a Safe Comparison Program (Priority: P2)

**Goal**: Let operators select a named global/project tool or a per-invocation environment override while preserving literal argument and configuration safety.

**Independent Test**: In disposable global and selected-project configuration files, prove project-over-global selection/definition inheritance, whole-array argument replacement, literal shell-like argument delivery, `GRIP_EXTERNAL_DIFF` two-argument bypass, and precise invalid/unavailable diagnostics.

- [X] T015 [US2] Add failing global/project precedence, inherited-definition, whole-array replacement, `GRIP_EXTERNAL_DIFF` bypass, literal-token, and unavailable-program scenarios in `tests/diff_cli_contract.rs`
- [X] T016 [US2] Verify profile parsing is deferred for unrelated mapping/JSON operations and profile data survives add/remove publication in `tests/diff_cli_contract.rs` and `tests/registry_integration.rs`
- [X] T017 [US2] Document global/project TOML syntax, precedence, `GRIP_EXTERNAL_DIFF` wrapper guidance, direct invocation, and no-shell boundary in `README.md` and `docs/product-definition.md`

**Checkpoint**: Operators can safely configure and override one comparison executable without changing mapping, synchronization, or shell-execution behavior.

## Phase 5: User Story 3 - Preserve Automation and Safety Expectations (Priority: P3)

**Goal**: Preserve stable JSON inspection and no-launch safety behavior, while returning tool exit information faithfully for human selected comparisons.

**Independent Test**: Verify JSON, missing endpoint, unsupported node, and unsafe-link selected diff scenarios do not invoke the recording program; verify normal exits `0`, `1`, and another nonzero code propagate unchanged and a SIGTERM helper returns `143` with a signal diagnostic.

- [X] T018 [US3] Add failing JSON/non-launch, missing/unsupported/unsafe endpoint, normal-exit, launch-failure, and signal-termination scenarios in `tests/diff_cli_contract.rs`
- [X] T019 [US3] Return normal child exit codes unchanged and map Unix signal termination to `128 + signal` with a precise diagnostic in `src/lib.rs`
- [X] T020 [US3] Prove JSON schema/state snapshots and existing no-follow detailed inspection results remain unchanged when external launch is ineligible in `tests/classification_cli_contract.rs` and `tests/diff_cli_contract.rs`

**Checkpoint**: Automation remains process-free and schema-stable; unsafe comparisons never spawn; human selected comparison returns the actual child completion status.

## Phase 6: Polish and Cross-Cutting Validation

**Purpose**: Reconcile documentation, task evidence, and full validation without broadening the feature.

- [X] T021 [P] Reconcile the external-diff examples and behavioral boundaries with the feature contract in `specs/028-configurable-external-diff/contracts/external-diff.md` and `specs/028-configurable-external-diff/quickstart.md`
- [X] T022 Run focused external-diff and classification tests, then complete `cargo test` and `mise run validate`; record any remaining gap in `specs/028-configurable-external-diff/tasks.md`

## Dependencies and Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: T001 can begin immediately.
- **Foundational (Phase 2)**: T002–T005 depend on T001 in order; T006 and T007 can run in parallel once their exercised interfaces exist. This phase blocks all user stories.
- **US1 (Phase 3)**: T008–T014 depend on foundational completion. T009 → T010 → T011 → T012 → T013 is the production path; T008 and T014 validate it.
- **US2 (Phase 4)**: Depends on the foundation and US1 handoff route. T015 extends named-tool coverage with cross-layer precedence and environment override; T016 follows it; T017 can proceed after the final configuration contract is stable.
- **US3 (Phase 5)**: Depends on the US1 child route and is best completed after US2 so all launch paths share one safety boundary. T018 precedes T019; T020 verifies regression behavior after it.
- **Polish (Phase 6)**: T021 follows finalized behavior; T022 follows all implementation and test tasks.

### User Story Dependencies

- **US1 (P1)**: Delivers the MVP after foundational work; it has no dependency on US2 or US3.
- **US2 (P2)**: Extends US1's direct handoff with configured selection and must not change its default behavior.
- **US3 (P3)**: Verifies and finalizes the automation, error, and completion contracts shared by the earlier stories.

## Parallel Opportunities

- T006 and T007 can run in parallel after their respective configuration surfaces exist.
- T017 documentation and T016 regression coverage can run in parallel once T015's behavior is stable.
- T021 can run in parallel with final test stabilization before T022.

## Parallel Example: Foundational Safety Tests

```text
Task: "Add Descriptor V2/profile preservation tests in tests/registry_integration.rs"
Task: "Add global-profile safe-file tests in tests/project_metadata_security_integration.rs"
```

## Implementation Strategy

### MVP First

1. Complete T001–T007 so project compatibility and safe configuration boundaries are available.
2. Complete T008–T014 to deliver default/named exact external handoff with no-selector protection.
3. Run the US1 independent test before adding broader profile precedence or child-exit behavior.

### Incremental Delivery

1. Foundation → safe preserved configuration representation.
2. US1 → one selected safe comparison handoff.
3. US2 → global/project/environment selection with literal arguments.
4. US3 → stable JSON/non-launch behavior and exact completion semantics.
5. Polish → documentation and full validation.

## Format Validation

All 22 tasks use the required checkbox, sequential ID, optional parallel marker, required user-story label in story phases, and concrete repository file path.

## Phase 7: Convergence

- [X] T023 Revalidate both prepared handoff endpoints with no-follow-safe metadata immediately before external process spawn, suppressing launch with a Grip-owned diagnostic if either changes or becomes ineligible, per FR-008, FR-010, and Constitution III (partial)
- [X] T024 Add isolated acceptance coverage for the default `diff` fallback, project tool-selection override, malformed global profile and empty environment override, missing/unsupported endpoint non-launch, and normal child exits `0` and `1`, per SC-001, SC-004, SC-005, and the plan validation strategy (partial)
