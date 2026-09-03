# Tasks: Cited Wiki Query

**Input**: Design documents from `specs/003-cited-wiki-query/`

**Tests**: Use static validation and disposable fixtures from `quickstart.md`; add no framework.

## Phase 1: Setup

- [X] T001 Verify query registration and shared page/context defaults against `specs/003-cited-wiki-query/contracts/wiki-query-command.md` in `extension.yml`

## Phase 2: Foundational

- [X] T002 Define untrusted question/index/page/registry handling and the absolute read-only contract in `commands/speckit.wiki.query.md`

## Phase 3: User Story 1 - Answer From Cited Wiki Evidence (Priority: P1) 🎯 MVP

- [X] T003 [US1] Define material question parts, index-only candidate ranking, bounded page selection, and selected-page reads in `commands/speckit.wiki.query.md`
- [X] T004 [US1] Define source-ID validation, claim deduplication, material-statement citations, and relied-on page output in `commands/speckit.wiki.query.md`

## Phase 4: User Story 2 - Expose Gaps and Disagreement (Priority: P2)

- [X] T005 [US2] Define conflict presentation and deterministic Covered, Partial, and Uncovered verdict rules in `commands/speckit.wiki.query.md`

## Phase 5: User Story 3 - Keep Queries Bounded and Read-Only (Priority: P3)

- [X] T006 [US3] Define context-budget narrowing, bounded empty-question overview, and structural-gap reporting in `commands/speckit.wiki.query.md`

## Phase 6: Polish

- [X] T007 Update provenance validation, untrusted wiki text, verdict, limits, and read-only guidance in `README.md`
- [X] T008 Validate `commands/speckit.wiki.query.md`, `config-template.yml`, `extension.yml`, and `README.md` against `specs/003-cited-wiki-query/quickstart.md` and run `git diff --check`

## Dependencies & Execution Order

T001 precedes T002; T002 blocks all stories. Story changes share one command and execute sequentially. Documentation and final validation follow all stories.

## Implementation Strategy

Deliver bounded cited answers first, then gap and conflict honesty, then overview and invariant validation. Time Machine owns commit and push gates.
