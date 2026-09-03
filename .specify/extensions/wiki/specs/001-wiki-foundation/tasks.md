# Tasks: Wiki Foundation

**Input**: Design documents from `specs/001-wiki-foundation/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/wiki-init-command.md`, `quickstart.md`

**Tests**: No automated test framework or TDD requirement is specified. Validate the prompt contract statically and execute the disposable-project scenarios in `quickstart.md` where the current environment permits.

**Organization**: Tasks are grouped by user story so each behavior remains independently reviewable and testable.

## Phase 1: Setup

**Purpose**: Confirm the existing extension surfaces that implement the feature.

- [X] T001 Verify initialization command registration, shared defaults, and supported Spec Kit version against `specs/001-wiki-foundation/contracts/wiki-init-command.md` in `extension.yml`

---

## Phase 2: Foundational

**Purpose**: Establish the safety rule that applies before any initialization path writes state.

- [X] T002 Add normalized project-boundary validation for the resolved wiki directory before the idempotency check in `commands/speckit.wiki.init.md`

**Checkpoint**: Every initialization path resolves configuration and rejects path escape before reading or writing wiki state.

---

## Phase 3: User Story 1 - Initialize a Project Wiki (Priority: P1) 🎯 MVP

**Goal**: Create the schema, empty index, and empty source registry exactly once without synthesizing knowledge.

**Independent Test**: Run Scenario 1 in `specs/001-wiki-foundation/quickstart.md` and verify all three artifacts, the scope, default policies, and the absence of `pages/`.

- [X] T003 [US1] Align new-wiki artifact creation, initial scope handling, and zero-page behavior with FR-003 through FR-008 in `commands/speckit.wiki.init.md`
- [X] T004 [US1] Align the new-wiki completion report with FR-012 and the New-Wiki Outcome contract in `commands/speckit.wiki.init.md`

**Checkpoint**: A new project receives a complete, empty, citable wiki foundation in one invocation.

---

## Phase 4: User Story 2 - Preserve an Existing Wiki (Priority: P2)

**Goal**: Make repeated initialization safe while allowing an explicitly supplied scope to append.

**Independent Test**: Run Scenarios 2 and 3 in `specs/001-wiki-foundation/quickstart.md`; verify checksum preservation without a scope and a single minimal schema change with a scope.

- [X] T005 [US2] Align the schema-sentinel idempotency path and no-overwrite guarantees with FR-009 in `commands/speckit.wiki.init.md`
- [X] T006 [US2] Align append-only scope handling and existing-wiki status delegation with FR-010 and FR-011 in `commands/speckit.wiki.init.md`

**Checkpoint**: Existing user-authored wiki state remains authoritative across repeated initialization.

---

## Phase 5: User Story 3 - Configure the Foundation (Priority: P3)

**Goal**: Apply predictable configuration precedence and clearly document safe customization.

**Independent Test**: Run Scenarios 4 and 5 in `specs/001-wiki-foundation/quickstart.md`; verify precedence at every layer and rejection of an escaping directory.

- [X] T007 [US3] Make per-setting defaults, saved values, environment overrides, and invocation overrides explicit and validate invalid values in `commands/speckit.wiki.init.md`
- [X] T008 [P] [US3] Document repository-relative directory containment and effective setting constraints in `config-template.yml`

**Checkpoint**: Maintainers can customize the wiki predictably without expanding the write boundary.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Keep public guidance and delivered behavior aligned, then validate the complete feature.

- [X] T009 Update initialization safety, idempotency, and configuration-precedence guidance in `README.md`
- [X] T010 Validate `commands/speckit.wiki.init.md`, `config-template.yml`, `extension.yml`, and `README.md` against `specs/001-wiki-foundation/quickstart.md` and run `git diff --check`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately.
- **Foundational (Phase 2)**: Depends on T001 and blocks every user story.
- **User Story 1 (Phase 3)**: Depends on T002 and delivers the MVP.
- **User Story 2 (Phase 4)**: Depends on T002 and can be validated independently from User Story 1 against an existing fixture.
- **User Story 3 (Phase 5)**: Depends on T002 and can be validated independently using configuration fixtures.
- **Polish (Phase 6)**: Depends on all selected user stories.

### User Story Dependencies

- **US1**: No dependency on another user story after T002.
- **US2**: No implementation dependency on US1; its fixture starts from an already initialized wiki.
- **US3**: No dependency on US1 or US2 after T002.

### Within Each User Story

- Update behavior before validating its completion report or fixture.
- Preserve existing prompt structure and avoid unrelated command changes.
- Complete each story’s independent quickstart scenario before its checkpoint.

### Parallel Opportunities

- T007 and T008 can run in parallel after T002 because they change different files.
- US1, US2, and US3 are behaviorally independent after T002, although overlapping edits to `commands/speckit.wiki.init.md` should be serialized to avoid merge conflicts.

## Parallel Example: User Story 3

```text
Task T007: Make configuration resolution and validation explicit in commands/speckit.wiki.init.md
Task T008: Document directory containment and value constraints in config-template.yml
```

## Implementation Strategy

### MVP First

1. Complete T001 and T002.
2. Complete T003 and T004 for User Story 1.
3. Run Quickstart Scenario 1 and review the MVP before expanding repeated-run and customization behavior.

### Incremental Delivery

1. Foundation safety → project-contained target resolution.
2. US1 → new-wiki creation.
3. US2 → repeated-run preservation.
4. US3 → predictable customization.
5. Polish → public documentation and full validation.

## Notes

- `[P]` marks work on different files with no unfinished dependency.
- Story labels map each task to its independently testable user scenario.
- No commit or push is part of an implementation task; Time Machine owns those later gates.
