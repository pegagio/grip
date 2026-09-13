---

description: "Task list for Feature 019 executable force-resolution guidance"
---

# Tasks: Executable Force-Resolution Guidance

**Input**: Design documents from `specs/019-executable-force-guidance/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [quickstart.md](quickstart.md), and [executable-force-guidance.md](contracts/executable-force-guidance.md)

**Tests**: Required. The specification requires isolated CLI-contract coverage proving that every displayed force command passes exact-entry selector validation and that aggregate conflicts offer only inspection guidance.

**Organization**: Tasks are grouped by user story after the shared guidance model and eligibility derivation are in place. The feature remains presentation-only: it must not broaden forced mutation authority or change JSON, diff, classification, or selector semantics.

## Phase 1: Setup

**Purpose**: Confirm the existing test fixture conventions and output contract before changing behavior.

- [X] T001 Add a reusable aggregate tree-conflict fixture for the CLI contract suites in tests/support/mod.rs

## Phase 2: Foundational

**Purpose**: Establish one non-serialized guidance model and one eligibility decision shared by status and blocked mutation output.

**⚠️ CRITICAL**: Complete this phase before rendering or testing user-story behavior so status and mutation guidance cannot diverge.

- [X] T002 Add a non-serialized force-pair, inspect-diff, or no-guidance representation and outcome attachment API in src/result.rs
- [X] T003 Derive per-record and per-blocker guidance from the existing exact-entry selection rules and attach it to status, push, pull, and sync outcomes in src/lib.rs

**Checkpoint**: The renderer can receive guidance that reflects the existing exact-entry force contract without changing serialized result details.

## Phase 3: User Story 1 - Follow a displayed resolution command (Priority: P1) 🎯 MVP

**Goal**: Human output suggests force commands only when they are executable for one exact managed entry, and it guides aggregate conflicts to `grip diff SOURCE` instead.

**Independent Test**: In isolated file and tree fixtures, each displayed force command succeeds through selector validation with `--dry-run`; aggregate tree roots show one `grip diff` command and no force command in status and blocked mutation output.

### Tests for User Story 1

- [X] T004 [P] [US1] Add exact-file and aggregate-tree status guidance contract tests that run every displayed force choice with `--dry-run` and assert aggregate rows show only `grip diff SOURCE` in tests/classification_cli_contract.rs
- [X] T005 [P] [US1] Add blocked push guidance tests that run both displayed exact-entry force choices with `--dry-run` and assert aggregate conflicts show only inspection guidance in tests/push_cli_contract.rs
- [X] T006 [P] [US1] Add blocked pull guidance tests that run both displayed exact-entry force choices with `--dry-run` and assert aggregate conflicts show only inspection guidance in tests/pull_cli_contract.rs
- [X] T007 [P] [US1] Add blocked sync guidance tests that run both displayed exact-entry force choices with `--dry-run` and assert aggregate conflicts show only inspection guidance in tests/sync_cli_contract.rs

### Implementation for User Story 1

- [X] T008 [US1] Render force pairs only for exact guidance and `Run: grip diff SOURCE` for aggregate guidance in default human status and blocked mutation output in src/result.rs

**Checkpoint**: The original tree-root failure is replaced by a copyable `grip diff` next step, while exact file conflicts retain both winner choices.

## Phase 4: User Story 2 - Preserve safe force scope (Priority: P2)

**Goal**: The correction does not expand force resolution or alter machine-readable and diagnostic contracts.

**Independent Test**: Tree-root `push --force` and destination-root `pull --force --destination` remain rejected; JSON status and mutation details, `grip diff`, classifications, and exit categories match their pre-feature contracts.

### Tests for User Story 2

- [X] T009 [US2] Add regression coverage for aggregate force rejection, unchanged JSON schemas and values, classification values, exit categories, and `grip diff` output, plus no invented guidance for technical or compatibility blockers, in tests/force_resolution_guidance_contract.rs

### Implementation for User Story 2

- [X] T010 [US2] Update force-guidance documentation for exact entries and aggregate tree inspection in README.md

**Checkpoint**: Force remains an exact-entry operation, and documentation tells users why an aggregate conflict must be inspected before choosing a winner.

## Phase 5: Polish & Cross-Cutting Verification

**Purpose**: Verify the integrated contract and ensure the feature artifacts describe the delivered behavior.

- [X] T011 Run focused CLI contract suites from Cargo.toml: `cargo test --test classification_cli_contract`, `cargo test --test push_cli_contract`, `cargo test --test pull_cli_contract`, `cargo test --test sync_cli_contract`, and `cargo test --test force_resolution_guidance_contract`
- [X] T012 Run repository formatting, lint, test, and release validation through mise.toml with `mise run validate`
- [X] T013 Verify whitespace and review the final user-facing examples against specs/019-executable-force-guidance/contracts/executable-force-guidance.md using `git diff --check`

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately and confirms how isolated filesystem fixtures are built.
- **Foundational (Phase 2)**: Depends on T001 and blocks all story work because the same eligibility result must feed both render paths.
- **User Story 1 (Phase 3)**: Depends on T002–T003. Its tests may be written in parallel before T008; T008 completes the behavior.
- **User Story 2 (Phase 4)**: Depends on T002–T003 and may begin after its distinct regression test file is prepared. Documentation follows the established behavior from T008.
- **Polish (Phase 5)**: Depends on both user stories.

### User Story Dependencies

- **User Story 1 (P1)**: Depends only on the shared guidance foundation and is the MVP.
- **User Story 2 (P2)**: Depends on the shared foundation and validates the boundaries preserved by User Story 1; it does not introduce a second implementation path.

## Parallel Opportunities

- T004, T005, T006, and T007 modify separate CLI-contract files and can run in parallel after T003.
- T009 targets a new regression contract file and can run in parallel with the User Story 1 test tasks after T003.
- T010 can be drafted in parallel with test execution once T008 establishes the final wording.

## Parallel Example: User Story 1

```text
Task: "Add exact-file and aggregate-tree status guidance contract tests in tests/classification_cli_contract.rs"
Task: "Add blocked push guidance tests in tests/push_cli_contract.rs"
Task: "Add blocked pull guidance tests in tests/pull_cli_contract.rs"
Task: "Add blocked sync guidance tests in tests/sync_cli_contract.rs"
```

## Implementation Strategy

### MVP First

1. Complete T001–T003 to produce one eligibility decision based on the existing exact-entry resolver.
2. Complete T004–T008 and run the focused User Story 1 tests.
3. Confirm the reported aggregate tree-root status output now offers `grip diff` rather than an invalid force command.

### Incremental Delivery

1. Deliver the P1 status and blocked-output correction without changing force semantics.
2. Add P2 regression coverage and documentation to make the retained exact-entry boundary explicit.
3. Run the full repository validation workflow before handoff.

## Format Validation

All 13 tasks use the required checkbox, sequential task ID, optional parallel marker only where files are independent, required user-story label in story phases, and an explicit repository file path.
