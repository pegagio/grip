# Tasks: CLI, Configuration, and State Foundation

**Input**: Design documents from `specs/001-cli-state-foundation/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), and [quickstart.md](quickstart.md)

**Tests**: Tests are required because the specification mandates automated contract and isolated-filesystem coverage. Within each user-story phase, write the listed tests first, verify that they fail for the missing behavior, and then implement the story.

**Organization**: Tasks are grouped by user story so each increment has a defined user-visible goal and independent acceptance boundary.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it changes different files and has no dependency on another incomplete task in the same phase
- **[Story]**: Maps implementation work to US1, US2, or US3 from [spec.md](spec.md)
- Every task names the exact file or files it changes

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish the pinned Rust application, dependency boundary, and planned source layout without implementing feature behavior.

- [X] T001 [P] Pin Rust 1.98.0 and its formatting and lint components in `mise.toml` and `rust-toolchain.toml`
- [X] T002 [P] Define the `grip` Rust 2024 application package, `rust-version = "1.98"`, runtime dependencies, and `tempfile` dev dependency in `Cargo.toml`
- [X] T003 Generate and review the application dependency lockfile in `Cargo.lock` after T001 and T002, retaining only dependencies justified by `research.md`
- [X] T004 Create the single-crate module skeleton and test directories in `src/main.rs`, `src/lib.rs`, `src/cli.rs`, `src/error.rs`, `src/home.rs`, `src/registry.rs`, `src/result.rs`, `src/state/mod.rs`, `src/state/lock.rs`, `src/state/publication.rs`, and `tests/support/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Define shared domain result/error boundaries and isolated test support required by every user story.

**Critical gate**: No user-story implementation begins until these types compile without embedding presentation strings or process-global test state in domain behavior.

- [X] T005 [P] Implement the exhaustive public Grip result-category mapping for exits 0, 2, 10, 11, 12, and 20 plus the reserved internal state-contention publication error in `src/error.rs`
- [X] T006 Implement version-neutral command outcomes and the five-field `ResultEnvelopeV1` wire type, including category/status consistency validation, in `src/result.rs`
- [X] T007 [P] Implement isolated temporary-root, child-environment, subprocess, filesystem-snapshot, and injected-writer helpers in `tests/support/mod.rs`
- [X] T008 Export the shared error, result, and command execution boundaries without exposing CLI framework types from `src/lib.rs`

**Checkpoint**: Shared domain contracts and isolated test infrastructure compile; story phases can begin.

---

## Phase 3: User Story 1 - Invoke Grip Predictably (Priority: P1) MVP

**Goal**: Users can invoke Grip help and version behavior successfully without a Grip home, registry, state, or filesystem mutation.

**Independent Test**: Run `--help`, `--version`, and the human-readable `version` command in an isolated environment with no Grip home; verify deterministic text, exit `0`, and an unchanged filesystem snapshot.

### Tests for User Story 1

- [X] T009 [US1] Add failing black-box tests for conventional `--help`/`--version`, the human-readable `version` command, exit `0`, independence from Grip-home resolution, and total non-mutation in `tests/cli_contract.rs`

### Implementation for User Story 1

- [X] T010 [US1] Define the `version` command and conventional parser metadata displays without Grip-home resolution in `src/cli.rs`
- [X] T011 [US1] Implement the human-readable version outcome without consulting configuration or state in `src/lib.rs`
- [X] T012 [US1] Wire version/help parsing, locked stdout/stderr writers, and cleanup-preserving `ExitCode` return through the thin entrypoint in `src/main.rs`

**Checkpoint**: User Story 1 passes independently and provides a complete non-mutating help/version MVP.

---

## Phase 4: User Story 2 - Validate Configuration and State Safely (Priority: P2)

**Goal**: Users can validate strict versioned registry and machine-state documents, while the internal state writer proves atomic, integrity-checked, owner-only publication under bounded coordination.

**Independent Test**: Place valid, absent, malformed, unknown-field, unsupported-version, integrity-failed, contended, interrupted, and unsafe-node documents in isolated Grip homes; run validation/publication tests and verify exact categories, old-or-new accepted state, required permissions, and no payload mutation.

### Tests for User Story 2

- [X] T013 [P] [US2] Add failing isolated-filesystem tests for default and exact `GRIP_HOME` resolution, invalid values, absent roots, inaccessible roots, ownership, symlink rejection, wrong node types, and instrumented proof that resolution never reads `/etc/grip/` or another path outside the selected root in `tests/home_integration.rs`
- [X] T014 [P] [US2] Add failing registry tests for a missing `config.toml`, required V1 fields, empty mappings, two-stage version dispatch, unknown fields, malformed TOML, non-empty mappings, no machine-wide fallback, and read-only behavior in `tests/registry_integration.rs`
- [X] T015 [P] [US2] Add failing state validation and publication tests for absent state, V1 canonical digest, generation, verified prior-generation recovery, retry reuse and collision, recovery-write failure, stable-lock contention/crash release, interruption, injected failures, exact permissions, symlinks, and wrong node types in `tests/state_integration.rs`

### Implementation for User Story 2

- [X] T016 [P] [US2] Implement exact `GRIP_HOME` and default `~/.grip` resolution, non-following existing-root policy validation, and an injectable access boundary constrained to the selected root in `src/home.rs`
- [X] T017 [P] [US2] Add the `validate` command and global output/verbosity arguments to the established parser metadata behavior in `src/cli.rs`
- [X] T018 [P] [US2] Implement strict two-stage `RegistryV1` TOML decoding and conversion to the empty version-neutral registry in `src/registry.rs`
- [X] T019 [P] [US2] Implement strict `StateEnvelopeV1`, canonical SHA-256 integrity input, validation states, and generation rules in `src/state/mod.rs`
- [X] T020 [US2] Implement stable owner-only lock-file validation and nonblocking descriptor-backed publication coordination in `src/state/lock.rs`
- [X] T021 [US2] Implement verified prior-generation recovery retention followed by same-directory exclusive staging, restrictive creation modes, sync/reread/verify, atomic rename, directory sync, and pre-rename cleanup in `src/state/publication.rs`
- [X] T022 [US2] Implement complete `validate` execution for the selected home, required `config.toml`, and optional `state/state.json`; classify absent home or registry as exit `10`; and avoid missing-path creation or machine-wide fallback in `src/lib.rs`
- [X] T023 [US2] Complete black-box validation cases for registry/state success and every publicly reachable Feature 001 exit category while asserting stdout/stderr, selected-root-only access, and filesystem non-mutation in `tests/cli_contract.rs`

**Checkpoint**: User Stories 1 and 2 pass; validation is useful independently and state publication satisfies the storage contract without exposing payload behavior.

---

## Phase 5: User Story 3 - Consume Deterministic Results (Priority: P3)

**Goal**: Humans and automation receive equivalent stable outcomes through separate human/JSON result rendering and opt-in redacted diagnostics.

**Independent Test**: Exercise every application success and failure category in both modes, then verify exact exits, the five JSON fields, semantic details, deterministic rendering, stderr-only diagnostics, parser-display exceptions, and failing-writer behavior.

### Tests for User Story 3

- [X] T024 [US3] Add failing human/JSON contract, valid JSON usage-error, verbosity separation, redaction, deterministic rendering, and closed/failing-writer tests in `tests/cli_contract.rs` and unit tests within `src/result.rs`

### Implementation for User Story 3

- [X] T025 [US3] Implement injected human and single-document JSON result renderers, stable detail objects, newline behavior, and result-writer failure handling in `src/result.rs`
- [X] T026 [US3] Implement valid `--output json` usage-error detection, parser-display exceptions, and verbosity selection without process termination in `src/cli.rs`
- [X] T027 [US3] Implement opt-in redacted diagnostic events on the separate diagnostic writer and preserve domain-result equivalence in `src/result.rs`
- [X] T028 [US3] Wire final output-mode rendering, best-effort output-failure notification, diagnostic emission, and exit mapping in `src/main.rs`
- [X] T029 [US3] Complete end-to-end contract assertions for all six publicly reachable Feature 001 symbolic/numeric categories and both application result modes, while confirming reserved state contention has no public path, in `tests/cli_contract.rs`

**Checkpoint**: All three user stories satisfy the CLI and storage contracts and remain independently testable through their phase criteria.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Complete user documentation, performance evidence, and whole-feature verification without expanding Feature 001 scope.

- [X] T030 [P] Implement the ignored 100-run warm release acceptance harness with environment reporting and 95th-order-statistic checks in `tests/performance_acceptance.rs`
- [X] T031 [P] Document Feature 001 installation, `validate`/`version`, output modes, exact `GRIP_HOME`, minimal `config.toml`, exit codes, and non-goals in `README.md`
- [X] T032 Audit all Feature 001 automated tests and acceptance-validation procedures for isolated roots; assert they never inspect or mutate the real home, default Grip home, payloads, `/etc/grip/`, other privileged paths, or version-control state; and confirm production validation uses only the user-selected root in `tests/support/mod.rs`, `tests/home_integration.rs`, `tests/registry_integration.rs`, and `tests/state_integration.rs`
- [X] T033 Run and resolve `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo build --release` findings across `Cargo.toml`, `src/`, and `tests/`
- [X] T034 Execute every isolated scenario in `specs/001-cli-state-foundation/quickstart.md`, run the release performance harness on a documented representative workstation, and record verified outcomes or remaining acceptance limitations in `specs/001-cli-state-foundation/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately; T003 depends on T001 and T002, and T004 follows the declared package structure.
- **Foundational (Phase 2)**: Depends on Setup and blocks all user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational and establishes complete help/version behavior without requiring configuration or state.
- **User Story 2 (Phase 4)**: Depends on User Story 1's executable boundary, then adds home resolution and the complete `validate` command without exposing a partial command in US1.
- **User Story 3 (Phase 5)**: Depends on the application outcomes produced by User Stories 1 and 2, then adds equivalent renderings and diagnostics without changing their domain classifications.
- **Polish (Phase 6)**: Depends on all selected user stories; T033 follows implementation and T034 follows T030 and T033.

### User Story Dependency Graph

```text
Setup → Foundational → US1 help/version MVP → US2 home/validation/state → US3 result interfaces → Polish
```

The delivery order is intentionally sequential because US2 extends the US1 `validate` command and US3 renders outcomes from both earlier stories. Independence is preserved at the acceptance level: each story has its own test boundary, and later phases must not change earlier domain meanings.

### Within Each User Story

- Write the story's tests first and confirm they fail for the missing behavior.
- Implement wire/domain models before services that consume them.
- Implement the service behavior before entrypoint integration.
- Run the independent test criterion at the checkpoint before starting the next story.
- Do not satisfy a later story by weakening or rewriting an earlier story's accepted contract.

### Parallel Opportunities

- T001 and T002 can run in parallel; T003 joins their results.
- T005 and T007 can run in parallel before T006 and T008 complete the shared foundation.
- T013, T014, and T015 can run in parallel; after they fail as expected, T016, T017, T018, and T019 can begin in parallel.
- T030 and T031 can run in parallel after the user stories are complete.

---

## Parallel Examples

### User Story 1

```text
Task T009: Add help/version CLI contract tests in tests/cli_contract.rs

After T009 fails as expected:
Task T010: Implement version/help grammar in src/cli.rs
Task T011: Implement version behavior in src/lib.rs
Task T012: Wire the entrypoint in src/main.rs
```

### User Story 2

```text
Task T013: Add home-policy and no-system-fallback tests in tests/home_integration.rs
Task T014: Add registry tests in tests/registry_integration.rs
Task T015: Add state tests in tests/state_integration.rs

After all three tests fail as expected:
Task T016: Implement home resolution in src/home.rs
Task T017: Add validate parsing in src/cli.rs
Task T018: Implement registry decoding in src/registry.rs
Task T019: Implement state validation in src/state/mod.rs
```

### Cross-cutting completion

```text
Task T030: Implement performance acceptance in tests/performance_acceptance.rs
Task T031: Document the feature in README.md
```

---

## Implementation Strategy

### MVP First: User Story 1

1. Complete Setup and Foundational phases.
2. Write and fail T009 before implementation.
3. Complete T010 through T012.
4. Stop and run the User Story 1 independent test criterion.
5. Retain this checkpoint as the smallest usable CLI/home-selection increment.

### Incremental Delivery

1. **US1**: Predictable executable and complete help/version behavior without configuration.
2. **US2**: Exact home selection, strict registry/state validation, and safe internal state publication.
3. **US3**: Stable human/JSON results, exits, and separate diagnostics.
4. **Polish**: Performance evidence, user documentation, and full quickstart verification.

Each increment adds capability without introducing mapping, traversal, baseline, payload-copying, synchronization, deletion, remote, daemon, caching, indexing, or parallel payload behavior.

## Notes

- `[P]` identifies only tasks that can change different files concurrently without depending on incomplete same-phase work.
- Tests are first-class implementation tasks because FR-017 and SC-001 through SC-006 require automated filesystem and contract evidence.
- Generated `Cargo.lock` changes in T003 require explicit review because the lockfile is a large generated dependency artifact.
- The state publication API is internal in Feature 001; no command creates state or mappings.
- After this task list is accepted, run `$speckit-analyze` as the constitution-required consistency gate before implementation.
- Do not commit, push, publish, or merge as part of these tasks unless separately authorized.
