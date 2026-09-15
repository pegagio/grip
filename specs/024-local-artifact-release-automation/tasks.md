---

description: "Task list for Local Artifact Release Automation"
---

# Tasks: Local Artifact Release Automation

**Input**: Design documents from `specs/024-local-artifact-release-automation/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), and [release task contract](contracts/release-task.md)

**Tests**: Required. The specification requires repeatable success and failure evidence for the local release task.

**Organization**: Tasks are grouped by user story. The shell contract test uses isolated temporary Git repositories and must not mutate the developer checkout, local release tags, or remotes.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel with other tasks that modify different files after its dependencies are complete.
- **[Story]**: Maps a task to one feature user story.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish the maintainable task and test entry points without changing Grip runtime code.

- [X] T001 Create the isolated shell-contract test harness, temporary-repository fixture helpers, and cleanup discipline in `tests/release_task_contract.sh`.
- [X] T002 Add the `release` and `test-release` task declarations that invoke the repository-owned scripts in `mise.toml`.

**Checkpoint**: The planned public and test task names exist, and the test harness can run only against disposable repositories.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish the common release candidate checks and shared release identity used by every user story.

**⚠️ CRITICAL**: No release behavior may create final artifacts or tags until this phase is complete.

- [X] T003 Create the release workflow skeleton with stage-specific diagnostics and failure cleanup in `scripts/prepare-release.sh`.
- [X] T004 Implement repository-root resolution, attached-`master` verification, clean-worktree verification, Darwin/arm64 verification, Cargo package-version extraction, and initial tag/artifact collision checks in `scripts/prepare-release.sh`.
- [X] T005 Add shared fixture assertions for captured `HEAD`, versioned artifact paths, existing-tag detection, and final-output collision detection in `tests/release_task_contract.sh`.

**Checkpoint**: The release script rejects every ineligible candidate before validation, final artifact creation, or local tag creation.

---

## Phase 3: User Story 1 - Prepare a Release Artifact (Priority: P1) 🎯 MVP

**Goal**: A maintainer can prepare one validated macOS Apple Silicon archive, checksum, and local annotated tag from an eligible `master` checkout.

**Independent Test**: In an isolated clean `master` repository with an unused package version, run the release task and verify one versioned archive, one matching checksum, and one annotated `v<version>` tag resolving to the captured `HEAD`.

### Tests for User Story 1

- [X] T006 [US1] Add the successful release-task contract scenario covering archive layout, checksum verification, annotated-tag type, and tag target in `tests/release_task_contract.sh`.

### Implementation for User Story 1

- [X] T007 [US1] Invoke the existing validation and performance release gates, then revalidate the captured candidate identity in `scripts/prepare-release.sh`.
- [X] T008 [US1] Build an attempt-owned staging directory, package `grip`, `README.md`, and `LICENSE` into the versioned Darwin/arm64 archive, generate and verify its SHA-256 checksum, and publish final `dist/` outputs without overwrite in `scripts/prepare-release.sh`.
- [X] T009 [US1] Create the annotated local `v<version>` tag only after artifact verification, verify that it resolves to the captured commit, and print artifact/tag handoff details in `scripts/prepare-release.sh`.

**Checkpoint**: The P1 task completes a local release preparation run from an eligible disposable repository.

---

## Phase 4: User Story 2 - Prevent an Unsafe or Ambiguous Release (Priority: P2)

**Goal**: Unsafe candidates and failed preparation attempts are rejected without creating a misleading release identity.

**Independent Test**: Each isolated dirty-tree, wrong-branch, unsupported-platform, invalid-version, existing-tag, existing-artifact, validation-failure, and packaging-failure case exits nonzero and leaves no new local tag or overwritten final artifact.

### Tests for User Story 2

- [X] T010 [US2] Add failure contract scenarios for dirty and wrong-branch candidates, unsupported host, invalid version, and tag/artifact/checksum collisions in `tests/release_task_contract.sh`.
- [X] T011 [US2] Add injected validation, packaging, and checksum failure scenarios that assert staging cleanup, no new tag, and no falsely releasable final output in `tests/release_task_contract.sh`.

### Implementation for User Story 2

- [X] T012 [US2] Add final collision and candidate-identity rechecks immediately before final artifact publication and tag creation in `scripts/prepare-release.sh`.
- [X] T013 [US2] Implement trap-based attempt cleanup and precise failed-stage diagnostics without deleting pre-existing release identities in `scripts/prepare-release.sh`.

**Checkpoint**: All selected unsafe and failure scenarios prove that no new release tag is created and no existing output is replaced.

---

## Phase 5: User Story 3 - Publish Deliberately (Priority: P3)

**Goal**: Maintainers receive clear manual publication instructions while the local release task performs no remote or GitHub mutation.

**Independent Test**: A successful isolated release preparation run leaves configured remote references unchanged, invokes no prohibited Git push, GitHub, upload, or registry-publication command, and identifies separate manual tag-push and GitHub-upload steps.

### Tests for User Story 3

- [X] T014 [US3] Add a no-external-publication contract scenario that records remote refs and uses command spies for prohibited `git push`, GitHub, upload, and registry-publication invocations in `tests/release_task_contract.sh`.

### Implementation for User Story 3

- [X] T015 [US3] Document `mise run release`, artifact inspection, explicit tag push, and manual GitHub Release upload in `README.md`.
- [X] T016 [US3] Ensure the release task's success output states that no push, upload, GitHub API call, or registry publication was performed in `scripts/prepare-release.sh`.

**Checkpoint**: The local workflow provides an inspected artifact/tag handoff but never publishes externally.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Validate the complete feature and reconcile its durable artifacts before implementation handoff.

- [X] T017 [P] Reconcile release-command names, output paths, collision behavior, and manual-publication guidance across `README.md`, `mise.toml`, and `scripts/prepare-release.sh`.
- [X] T018 Run the isolated release-task suite and project release gates referenced by `specs/024-local-artifact-release-automation/quickstart.md`; record any platform-limited validation evidence in `specs/024-local-artifact-release-automation/quickstart.md`.
- [X] T019 Run `speckit-analyze` against `specs/024-local-artifact-release-automation/spec.md`, `specs/024-local-artifact-release-automation/plan.md`, and `specs/024-local-artifact-release-automation/tasks.md`; reconcile any findings before implementation begins.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately.
- **Foundational (Phase 2)**: Depends on T001-T002 and blocks all user stories.
- **User Story 1 (Phase 3)**: Depends on T003-T005; delivers the MVP.
- **User Story 2 (Phase 4)**: Depends on the P1 workflow because it hardens the same local task.
- **User Story 3 (Phase 5)**: Depends on the P1 workflow; its documentation and non-effect test can proceed in parallel with P2 after T009.
- **Polish (Phase 6)**: Depends on all desired user-story tasks.

### User Story Dependencies

- **US1**: Foundation only; independently proves artifact preparation and local tag creation.
- **US2**: Builds on the US1 task to prove failure safety; it does not change the successful release contract.
- **US3**: Builds on the US1 task to prove manual publication remains explicit; it can run in parallel with US2 once US1 succeeds.

### Parallel Opportunities

- T001 and T002 can proceed in parallel because they modify different files.
- T010 and T011 can proceed in parallel after the shared fixture assertions are available.
- T014 and T015 can proceed in parallel after the successful workflow exists.
- T017 can begin once T015-T016 establish stable names and messages; T018 and T019 remain final sequential gates.

## Parallel Example: Post-MVP Hardening

```text
Task: "Add failure contract scenarios in tests/release_task_contract.sh"
Task: "Add no-remote-mutation contract scenario in tests/release_task_contract.sh"
Task: "Document manual publication in README.md"
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001-T005 to establish safe eligibility and test infrastructure.
2. Complete T006-T009 to produce and verify a local artifact set and annotated tag.
3. Run the P1 isolated contract scenario before hardening failure behavior.

### Incremental Delivery

1. Add P1: a successful local release preparation workflow.
2. Add P2: collision, failure, and cleanup guarantees.
3. Add P3: explicit manual publication handoff and non-effect proof.
4. Complete final validation and cross-artifact analysis before implementation begins.

## Notes

- Every task follows the required checkbox, sequential ID, story-label, and exact-path format.
- No task authorizes pushing, GitHub publication, package-registry publication, or a change to Grip runtime behavior.
- The task list deliberately treats the local annotated tag as a guarded final step, not as an external publication action.

## Phase 7: Convergence

- [X] T020 Reject malformed prerelease and build identifiers in the release-version guard, with isolated contract coverage, per FR-004 (partial).

## Phase 8: Convergence

- [X] T021 Publish the final archive and checksum with an atomic no-clobber operation, and prove collision-race failures preserve the concurrent output, per FR-009 and plan: bounded concurrency (partial).

## Phase 9: Convergence

- [X] T022 Extract the successful archive in the isolated contract test and verify the delivered executable, README, and license files, per SC-003 (partial).

## Phase 10: Convergence

- [X] T023 Cover existing-checksum and non-arm64-Darwin failure cases in the isolated contract suite, per SC-002 (partial).
- [X] T024 State explicitly that no GitHub API call occurs and assert the complete non-publication handoff message in the success contract test, per FR-011 and T016 (partial).
