# Tasks: Wiki Health Checks

**Input**: Design documents from `specs/004-wiki-health-checks/`

**Tests**: Use static checks and disposable fixtures in `quickstart.md`; add no framework.

## Phase 1: Setup

- [X] T001 Verify lint registration and mechanical-fix defaults against `specs/004-wiki-health-checks/contracts/wiki-lint-command.md` in `extension.yml`

## Phase 2: Foundational

- [X] T002 Define scope validation, untrusted wiki text, deterministic finding identity, and analyze-before-apply sequencing in `commands/speckit.wiki.lint.md`

## Phase 3: User Story 1 - Diagnose Wiki Health (Priority: P1) 🎯 MVP

- [X] T003 [US1] Define the six checks, exact evidence requirements, severities, deduplication, and bounded contradiction pairs in `commands/speckit.wiki.lint.md`
- [X] T004 [US1] Define deterministic report rows, No findings output, per-check counts, and actual fix disposition in `commands/speckit.wiki.lint.md`

## Phase 4: User Story 2 - Apply Only Safe Mechanical Repairs (Priority: P2)

- [X] T005 [US2] Define the mechanical repair allowlist, ambiguity refusal, semantic preservation, and all-or-nothing fix application in `commands/speckit.wiki.lint.md`

## Phase 5: User Story 3 - Run Focused, Bounded Maintenance (Priority: P3)

- [X] T006 [US3] Define named-check and page-neighborhood scope plus one highest-value next action in `commands/speckit.wiki.lint.md`

## Phase 6: Polish

- [X] T007 Update six-check, fix-boundary, untrusted-text, deterministic-report, and failure behavior guidance in `README.md`
- [X] T008 Validate `commands/speckit.wiki.lint.md`, `config-template.yml`, `extension.yml`, and `README.md` against `specs/004-wiki-health-checks/quickstart.md` and run `git diff --check`

## Dependencies & Execution Order

T001 precedes T002; T002 blocks all stories. Story edits share one command and execute sequentially. Documentation and final validation follow.

## Implementation Strategy

Deliver evidenced diagnostics first, then allowlisted repairs, then focused scope and prioritization. Time Machine owns commit and push gates.
