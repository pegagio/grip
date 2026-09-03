# Tasks: Knowledge Ingestion

**Input**: Design documents from `specs/002-knowledge-ingestion/`

**Tests**: Use static contract validation and the disposable-project scenarios in `quickstart.md`; do not introduce a new test framework.

## Phase 1: Setup

- [X] T001 Verify ingestion registration and shared limits against `specs/002-knowledge-ingestion/contracts/wiki-ingest-command.md` in `extension.yml`

## Phase 2: Foundational

- [X] T002 Define source validation, local containment, untrusted-content handling, and prepare-before-commit failure behavior in `commands/speckit.wiki.ingest.md`

## Phase 3: User Story 1 - Ingest Durable Knowledge (Priority: P1) 🎯 MVP

**Independent Test**: Run Quickstart Scenarios 1 and 2.

- [X] T003 [US1] Define explicit and default source resolution, stable normalization, directory handling, and source registration in `commands/speckit.wiki.ingest.md`
- [X] T004 [US1] Define durable extraction, index-first page selection, citations, metadata, and reciprocal linking in `commands/speckit.wiki.ingest.md`

## Phase 4: User Story 2 - Refresh Without Erasing Disagreement (Priority: P2)

**Independent Test**: Run Quickstart Scenario 3.

- [X] T005 [US2] Define re-ingestion identity preservation, duplicate suppression, stale-claim refresh, and conflict retention in `commands/speckit.wiki.ingest.md`

## Phase 5: User Story 3 - Bound Each Ingestion (Priority: P3)

**Independent Test**: Run Quickstart Scenarios 4 and 5.

- [X] T006 [US3] Define page-cap prioritization, page splitting, explicit deferrals, state synchronization, and completion reporting in `commands/speckit.wiki.ingest.md`

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T007 Update source trust, supported source types, stable provenance, failure atomicity, and bounded-run guidance in `README.md`
- [X] T008 Validate `commands/speckit.wiki.ingest.md`, `extension.yml`, and `README.md` against `specs/002-knowledge-ingestion/quickstart.md` and run `git diff --check`

## Dependencies & Execution Order

- T001 precedes T002.
- T002 blocks every user story.
- US1, US2, and US3 are independently testable after T002 but edit the same command and therefore execute sequentially.
- T007 and T008 follow all user stories.

## Implementation Strategy

Complete T001–T004 for the MVP, validate new-source and hostile-content fixtures, then add re-ingestion and bounded overflow behavior before public documentation and final validation.

## Notes

- No task commits or pushes; Time Machine owns those gates.
- Each task names the exact file it may change.
