# Tasks: Reverse Synchronization

**Input**: Design documents from `/specs/006-reverse-synchronization/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`

**Tests**: Automated contract, integration, recovery, contention, regression, and performance tests are required by the specification. Each user-story phase starts with tests that must fail before its implementation tasks begin.

**Organization**: Tasks are grouped by user story so each increment can be implemented and validated independently. The P1 preview story is the recommended MVP.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel with other tasks carrying `[P]` in the same dependency wave because the tasks touch different files and have no unfinished dependency on each other
- **[Story]**: Maps the task to a user story from `spec.md` (`US1`, `US2`, `US3`, or `US4`)
- Every task names the exact file or files it changes or validates

## Phase 1: Setup (Shared Structure)

**Purpose**: Establish the direction-neutral module boundaries and pull entry point required by the implementation plan.

- [X] T001 Create the module skeletons and registrations for the shared mutation core and pull adapter in `src/mutation/mod.rs`, `src/pull/mod.rs`, and `src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Extract the existing push safety pipeline into reusable direction-neutral components while preserving all current push behavior. No user-story implementation starts until this phase is complete.

**⚠️ CRITICAL**: Finish this phase and restore the existing push regression suite before beginning any pull story.

- [X] T002 Extract `MutationDirection`, direction-neutral plan/action/evidence types, and mapping-role-to-transfer-role rules from the push model into `src/mutation/model.rs`, retaining compatibility re-exports in `src/push/mod.rs`
- [X] T003 [P] Add direction-neutral mapping, registry, filesystem, operation-record, and result assertion helpers based on the shared model for later pull tests in `tests/support/mod.rs` after T002
- [X] T004 [P] Generalize canonical plan serialization, deterministic ordering, and plan digest construction in `src/mutation/plan.rs` after T002
- [X] T005 [P] Move descriptor-relative inspection, staging, replacement, verification, and cleanup primitives into `src/mutation/filesystem.rs` after T002
- [X] T006 Generalize recovery metadata and recovery-area operations around transfer origin and target roles in `src/mutation/recovery.rs`, preserving the existing on-disk recovery contract used by `src/push/mod.rs`
- [X] T007 [P] Extract a direction-neutral Operation Record V1 identity and initialization seam while preserving the currently accepted push values and strict field rejection in `src/operation/model.rs` and `src/operation/publication.rs` after T002
- [X] T008 [P] Replace push-only mutation error categories with direction-neutral typed failures and side-effect evidence while preserving public error compatibility in `src/error.rs` after T002
- [X] T009 Assemble the shared lock, revalidation, execution, verification, recovery, and checkpoint pipeline in `src/mutation/execution.rs`, leaving `src/push/mod.rs` as a thin direction adapter
- [X] T010 Migrate push orchestration and result projection to the shared mutation core, add the shared direction field while preserving the push schema contract, and prove unchanged behavior with `src/lib.rs`, `src/result.rs`, `tests/push_cli_contract.rs`, `tests/push_planning.rs`, `tests/push_filesystem_integration.rs`, `tests/push_failure_integration.rs`, `tests/push_recovery_integration.rs`, and `tests/push_contention_integration.rs`

**Checkpoint**: The shared mutation foundation is usable and every pre-existing push test passes without relaxing its assertions.

---

## Phase 3: User Story 1 - Preview Destination Changes (Priority: P1) 🎯 MVP

**Goal**: Let a user preview exactly which accepted destination-side changes would replace managed source content, without mutating either side or importing unmanaged content.

**Independent Test**: Prepare accepted mappings with unchanged, source-only, destination-only, both-changed, missing-source, missing-destination, type-mismatch, unsupported-type, unreadable, and unmanaged-destination cases; run `grip pull --dry-run`; verify deterministic classifications, complete blocker reporting, no filesystem or accepted-state mutation, and no operation record.

### Tests for User Story 1

- [X] T011 [P] [US1] Add failing table-driven tests for every pull disposition, deterministic plan ordering and digesting, unmanaged destination reporting including an unsafe collision at a managed path, hierarchical `.gripignore`, and complete blocker accumulation in `tests/pull_planning.rs`
- [X] T012 [P] [US1] Add failing CLI tests for `grip pull [PATH]`, `-n`, `--dry-run`, `--destination`, `--`, invalid selector combinations, human preview output, shared-schema JSON preview output with `direction: pull`, and dry-run non-mutation in `tests/pull_cli_contract.rs`

### Implementation for User Story 1

- [X] T013 [US1] Implement the pull disposition policy so only accepted `destination_only_change` entries become replacement actions, unmanaged destination entries remain itemized non-actions, and an unmanaged item blocks only when it independently creates an unsafe collision at a managed path in `src/mutation/plan.rs` and `src/pull/mod.rs`
- [X] T014 [US1] Add `PullArgs`, selector parsing, dry-run flags, output options, and clap validation matching the push selection contract in `src/cli.rs`
- [X] T015 [US1] Add complete registry loading, selection, filesystem preflight, and side-effect-free pull preview orchestration in `src/lib.rs`
- [X] T016 [US1] Render deterministic pull preview entries, counts, blockers, scope, plan identity, action intent, and the foundation-provided shared `direction: pull` field for human and machine consumers in `src/result.rs`
- [X] T017 [US1] Complete the US1 acceptance matrix and make the preview contract tests pass without weakening assertions in `tests/pull_planning.rs` and `tests/pull_cli_contract.rs`

**Checkpoint**: User Story 1 is independently usable as a safe pull preview and is the minimum viable feature increment.

---

## Phase 4: User Story 2 - Pull Managed Changes Safely (Priority: P2)

**Goal**: Apply an eligible pull plan by replacing managed source content from destination content under the same safety guarantees as push.

**Independent Test**: Preview an accepted destination-only change, apply it, and verify that the source exactly matches the destination, the destination remains unchanged, recovery evidence preserves the prior source, the accepted baseline advances only after full verification, and a second pull is a no-op.

### Tests for User Story 2

- [X] T018 [P] [US2] Add failing file, directory-tree, symlink, metadata, destination-stability, no-source-parent-creation, verification, and second-run-idempotence tests in `tests/pull_filesystem_integration.rs`
- [X] T019 [P] [US2] Add failing tests that successful pull replacement preserves the prior source in the existing recovery layout with verifiable metadata in `tests/pull_recovery_integration.rs`
- [X] T020 [P] [US2] Add failing tests for the shared mutation lock across pull/pull and push/pull contenders, including bounded ownership, actionable deterministic loser results, unchanged state, and stale-owner recovery, in `tests/pull_contention_integration.rs`
- [X] T021 [P] [US2] Add failing tests for immutable direction-bound pull plans, strict Operation Record V1 acceptance of only `push` or `pull`, pull-prefixed operation identities, action evidence, and complete-success records in `tests/operation_record_integration.rs`

### Implementation for User Story 2

- [X] T022 [US2] Implement destination-origin staging, source-target atomic replacement, source verification, and cleanup without creating a missing source parent in `src/mutation/filesystem.rs`
- [X] T023 [US2] Preserve and verify the prior source before replacement and bind recovery evidence to the pull plan and action in `src/mutation/recovery.rs`
- [X] T024 [US2] Initialize and publish pull operation records with direction-bound plan identity, action evidence, recovery references, and complete-success terminal state in `src/operation/model.rs` and `src/operation/publication.rs`
- [X] T025 [US2] Execute pull actions under the shared mutation lock with just-in-time revalidation, verified recovery, atomic publication, and post-publication verification in `src/mutation/execution.rs` and `src/pull/mod.rs`
- [X] T026 [US2] Wire mutating pull execution and all-or-nothing accepted baseline publication into `src/lib.rs`, performing final acceptance-relevant revalidation of the complete selected result, updating only verified actioned identities, and preserving selected no-action, out-of-scope, and pending-retirement State V2 records
- [X] T027 [US2] Complete the US2 acceptance matrix, including dry-run versus mutating-plan parity over equivalent unchanged starting evidence and State V2 preservation assertions, and make safe-apply, recovery, contention, stale-lock recovery, operation-record, and idempotence tests pass in `tests/pull_filesystem_integration.rs`, `tests/pull_recovery_integration.rs`, `tests/pull_contention_integration.rs`, and `tests/operation_record_integration.rs`

**Checkpoint**: User Stories 1 and 2 independently provide safe preview and safe application for eligible managed destination changes.

---

## Phase 5: User Story 3 - Understand and Recover From Pull Failure (Priority: P3)

**Goal**: Stop safely on any failed or stale action, preserve the last verified target state, retain actionable evidence, and leave the accepted baseline unchanged.

**Independent Test**: Inject failures at every pre-publication, publication, verification, checkpoint, and cleanup boundary, plus stale-plan and interrupted-operation cases; verify no later actions run, the source is either prior verified or newly verified content, recovery and operation evidence remain available, and the accepted baseline does not advance.

### Tests for User Story 3

- [X] T028 [P] [US3] Add failing pre-action drift tests for mapping identity, accepted membership, destination evidence, source evidence, source ancestry, accepted baseline, and plan assumptions plus staging, preservation, replacement, visibility-without-confirmed-durability, verification, baseline-publication, accepted-generation-retention, checkpoint, cleanup, no-later-action, and post-failure public `grip status` tests in `tests/pull_failure_integration.rs`
- [X] T029 [P] [US3] Add failing tests for recovery verification failures and restoration to the prior verified source after incomplete publication in `tests/pull_recovery_integration.rs`
- [X] T030 [P] [US3] Add failing tests proving interrupted and failed pull records remain immutable, are never resumed or rolled back, and do not block a later independent pull that passes fresh complete inspection in `tests/operation_record_integration.rs`

### Implementation for User Story 3

- [X] T031 [US3] Add direction-neutral fault-injection phases and lifecycle checkpoints for staging, recovery, publication visibility, durability confirmation, result verification, baseline publication, and cleanup in `src/mutation/mod.rs` and `src/mutation/execution.rs`
- [X] T032 [US3] Emit typed pull failure categories with failed mapping identity, phase, expected and observed evidence, recovery disposition, and safe operator guidance in `src/error.rs`
- [X] T033 [US3] Implement stop-on-first-action-failure, verified-source recovery, durable failure checkpoints, and baseline suppression in `src/mutation/execution.rs` and `src/operation/publication.rs`
- [X] T034 [US3] Complete the US3 fault matrix and make baseline-publication, accepted-generation-retention, public-status observability, recovery, interruption, and subsequent-independent-pull tests pass in `tests/pull_failure_integration.rs`, `tests/pull_recovery_integration.rs`, and `tests/operation_record_integration.rs`

**Checkpoint**: User Story 3 independently demonstrates bounded failure, recovery evidence, and safe retry behavior.

---

## Phase 6: User Story 4 - Use Pull Results in Automation (Priority: P4)

**Goal**: Give scripts a stable versioned result and deterministic exit semantics while keeping human output semantically equivalent.

**Independent Test**: Exercise successful preview, blocked preview, successful apply, no-op apply, partial failure, and output-delivery failure in human and JSON modes; verify schema version, `direction: pull`, scope, entries, actions, blockers, operation record, baseline disposition, recovery evidence, and exit status agree across renderers.

### Tests for User Story 4

- [X] T035 [P] [US4] Add failing shared-schema, `direction: pull`, deterministic ordering, human/JSON semantic parity, exit-status, and output-delivery-failure tests in `tests/pull_cli_contract.rs`
- [X] T036 [US4] Add failing tests that machine results expose the correct finalized or nonterminal operation-record reference for success, partial failure, and result-delivery failure in `tests/operation_record_integration.rs`

### Implementation for User Story 4

- [X] T037 [US4] Complete the foundation-provided shared versioned mutation result model with stable completion, action, recovery, publication, verification, baseline, and error-category projections while preserving mapping-role meanings and push compatibility in `src/result.rs`
- [X] T038 [US4] Implement deterministic human and JSON renderers for preview, success, no-op, blocked, failed, recovery, operation-record, and baseline outcomes in `src/result.rs`
- [X] T039 [US4] Finalize actionful operation records before emitting results and report output-delivery failure without misrepresenting mutation state in `src/lib.rs` and `src/operation/publication.rs`
- [X] T040 [US4] Complete the US4 automation matrix and make result-schema, parity, validation, and delivery-failure tests pass in `tests/pull_cli_contract.rs` and `tests/operation_record_integration.rs`

**Checkpoint**: All four stories are independently demonstrable and automation can distinguish every supported pull outcome.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Complete documentation, performance acceptance, repository-wide validation, and post-implementation reconciliation.

- [X] T041 [P] Document pull preview, apply-by-default behavior, selectors, unmanaged reporting, blockers, recovery, machine output, and deliberate v1 exclusions in `README.md`
- [X] T042 [P] Extend the 10,000-entry fixture and p95 measurement harness to cover pull planning and complete preflight in `tests/performance_acceptance.rs`
- [X] T043 Add concise Rust documentation for public direction-neutral mutation types and adapter contracts in `src/mutation/mod.rs`, `src/mutation/model.rs`, `src/pull/mod.rs`, and `src/push/mod.rs`
- [X] T044 Execute every documented preview, apply, blocker, unmanaged, selector, recovery, and failure workflow and correct any drift found in `specs/006-reverse-synchronization/quickstart.md`
- [X] T045 Run `mise run validate` and resolve only Feature 006 regressions in `Cargo.toml`, `src/`, and `tests/`
- [X] T046 Run `mise run performance`, verify 10,000-entry pull planning and preflight meet the p95 threshold, and record reproducible evidence in `specs/006-reverse-synchronization/quickstart.md`
- [X] T047 Run the post-implementation `$speckit-converge` workflow and reconcile any genuinely unbuilt requirements into `specs/006-reverse-synchronization/tasks.md` against `specs/006-reverse-synchronization/spec.md` and `specs/006-reverse-synchronization/plan.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately.
- **Foundational (Phase 2)**: Depends on Phase 1 and blocks every user story.
- **US1 (Phase 3)**: Depends on Phase 2. This is the MVP and should be completed first.
- **US2 (Phase 4)**: Depends on Phase 2 and reuses US1's pull plan and CLI path.
- **US3 (Phase 5)**: Depends on Phase 2 and exercises the mutation path delivered by US2.
- **US4 (Phase 6)**: Depends on Phase 2; its complete outcome matrix requires the US1-US3 outcomes.
- **Polish (Phase 7)**: Depends on every selected user story; T047 runs only after implementation and validation are complete.

### Foundational Dependency Graph

1. T001 → T002.
2. After T002, T003, T004, T005, T007, and T008 can proceed in parallel.
3. T005 → T006.
4. T003 + T004 + T006 + T007 + T008 → T009.
5. T009 → T010.
6. T010 unlocks all user-story phases.

### User Story Dependencies

- **US1**: T011 and T012 can run in parallel; both must fail for the expected reasons before T013-T016; T017 closes the story.
- **US2**: T018-T021 can run in parallel after US1; T022 and T023 then prepare target and recovery mechanics, T024 prepares publication, T025 integrates execution, T026 publishes state, and T027 closes the story.
- **US3**: T028-T030 can run in parallel after US2; T031-T033 implement the failure contract; T034 closes the story.
- **US4**: T035 and T036 establish the automation contract after US3; T037-T039 implement it; T040 closes the story.

### Within Each User Story

1. Write the story's automated tests and confirm they fail because the behavior is absent.
2. Implement models and policies before orchestration and rendering.
3. Make the story-specific tests pass without weakening pre-existing push or earlier pull assertions.
4. Stop at the story checkpoint and demonstrate the independent test before proceeding.

---

## Parallel Execution Examples

### User Story 1

```text
Task T011: Add the pull disposition and planning matrix in tests/pull_planning.rs
Task T012: Add the pull grammar and preview contract in tests/pull_cli_contract.rs
```

### User Story 2

```text
Task T018: Add source replacement tests in tests/pull_filesystem_integration.rs
Task T019: Add recovery preservation tests in tests/pull_recovery_integration.rs
Task T020: Add shared-lock tests in tests/pull_contention_integration.rs
Task T021: Add successful operation-record tests in tests/operation_record_integration.rs
```

### User Story 3

```text
Task T028: Add injected failure tests in tests/pull_failure_integration.rs
Task T029: Add failed recovery tests in tests/pull_recovery_integration.rs
Task T030: Add interruption and restart tests in tests/operation_record_integration.rs
```

### User Story 4

```text
Task T035: Add result and delivery contract tests in tests/pull_cli_contract.rs
Task T036: Add strict direction validation tests in tests/operation_record_integration.rs
```

---

## Implementation Strategy

### MVP First

1. Complete Setup and Foundational work while keeping all push tests green.
2. Complete US1 and demonstrate a deterministic, side-effect-free pull preview.
3. Pause for review before enabling mutation.

### Incremental Delivery

1. **US1** adds safe visibility without mutation.
2. **US2** adds verified source replacement and accepted-state advancement.
3. **US3** adds explicit failure containment and recovery evidence.
4. **US4** stabilizes automation and human-result contracts across every outcome.
5. Polish validates documentation, performance, and cross-artifact completeness.

### Review Discipline

- Keep each task or tightly coupled dependency wave small enough for focused review.
- Do not combine missing-source creation, deletion propagation, unmanaged import, conflict resolution, repair, rollback, or bidirectional sync into Feature 006.
- Treat passing automation as evidence, not intended-user acceptance; retain explicit review at each story checkpoint.
