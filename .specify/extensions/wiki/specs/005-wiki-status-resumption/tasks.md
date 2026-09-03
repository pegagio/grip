# Tasks: Wiki Status and Resumption

**Input**: Design documents from `specs/005-wiki-status-resumption/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

## Phase 1: Setup

- [X] T001 Verify `/speckit.wiki.status` registration and declared inputs in `extension.yml` and `commands/speckit.wiki.status.md`

## Phase 2: Foundational

- [X] T002 Define argument validation, untrusted-data handling, structural-only reads, missing-artifact behavior, and the strict zero-write boundary in `commands/speckit.wiki.status.md`

## Phase 3: User Story 1 - Resume From a Compact Snapshot (P1)

**Goal**: Render an honest one-screen snapshot from structural metadata alone.

**Independent Test**: Use populated and malformed fixtures; verify bounded summaries, explicit unknown/invalid values, zero page-body reads, and unchanged checksums.

- [X] T003 [US1] Define scope, counts, bounded recent-page and source slices, and structural-only freshness derivation in `commands/speckit.wiki.status.md`
- [X] T004 [US1] Define prioritized issue rendering and the compact output format in `commands/speckit.wiki.status.md`

## Phase 4: User Story 2 - Receive Exactly One Next Action (P2)

**Goal**: Select one concrete, evidence-backed continuation deterministically.

**Independent Test**: Use conflict, stale-lint, empty-wiki, un-ingested-artifact, healthy-wiki, and no-wiki fixtures; verify exactly one stable recommendation.

- [X] T005 [US2] Implement recommendation candidates, priority, evidence requirements, deterministic tie-breaking, and the single-action output rule in `commands/speckit.wiki.status.md`

## Phase 5: User Story 3 - Filter Without Expanding the Boundary (P3)

**Goal**: Support page-type and bounded full views without additional read authority.

**Independent Test**: Compare default, type-filtered, full, and invalid inputs; verify filtering, threefold bounds, no page-body reads, and zero writes.

- [X] T006 [US3] Implement configured page-type filtering, bounded `full` expansion, and invalid-input reporting in `commands/speckit.wiki.status.md`

## Phase 6: Documentation and Validation

- [X] T007 Document structural-only status, honest unknowns, bounded filters, and evidence-backed next actions in `README.md`
- [X] T008 Validate `commands/speckit.wiki.status.md` against `specs/005-wiki-status-resumption/quickstart.md` and run `git diff --check`

## Dependencies & Execution Order

T001 precedes T002. T002 blocks all user stories. Complete T003-T004, then T005, then T006. T007 follows the command contract; T008 completes the feature.
