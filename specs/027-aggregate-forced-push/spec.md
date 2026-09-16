# Feature Specification: Aggregate Forced Push

**Feature Branch**: `027-aggregate-forced-push`

**Created**: 2026-09-16

**Status**: Complete

**Input**: User description: "Allow `grip push --force` without a path to force push all managed changes in the selected project."

## Clarifications

### Session 2026-09-16

- Q: After one aggregate forced-push entry fails, should Grip stop before attempting the remaining entries? → A: Stop after the first failed entry; earlier verified entries remain accepted.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Force all managed source changes (Priority: P1)

An operator with several managed entries in different source-to-destination states can deliberately make the source-complete state win across the selected project with one explicit no-selector forced push.

**Why this priority**: The feature removes repetitive exact-entry force commands when the operator has already decided that the selected project's source state is authoritative.

**Independent Test**: In an isolated project with managed entries that include divergent content and a missing destination peer, run a no-selector forced push and confirm that each eligible destination receives its source-complete state.

**Acceptance Scenarios**:

1. **Given** a selected project with multiple managed entries requiring source-winning resolution, **When** the operator runs `grip push --force` without a selector, **Then** Grip prepares one aggregate source-winning operation covering every managed entry in the selected project.
2. **Given** the aggregate contains a divergent entry and a source-present, destination-missing entry, **When** the operator confirms the no-selector forced push, **Then** Grip applies the source-complete state to both eligible destinations rather than requiring separate exact-entry commands.
3. **Given** an operator supplies a source or destination selector, **When** the operator runs a forced push, **Then** the existing exact-entry selector contract remains in effect.

---

### User Story 2 - Review the aggregate force operation safely (Priority: P2)

An operator can review exactly what an aggregate forced push would change before allowing it to mutate managed destinations.

**Why this priority**: A project-wide source-winning action is consequential and must be as inspectable as existing directional operations.

**Independent Test**: In an isolated project with several forceable entries, run the dry-run form and verify that it reports every intended action while leaving source, destination, and accepted state unchanged.

**Acceptance Scenarios**:

1. **Given** an aggregate forced push would change one or more managed destinations, **When** the operator requests a dry run, **Then** Grip reports the same selected entries and intended source-winning states without mutating payloads or accepted state.
2. **Given** any managed entry has an unsupported node, unsafe symlink ancestry, ownership violation, topology violation, or invalid selected project, **When** the operator requests aggregate force, **Then** Grip reports the blocker and does not treat force as permission to bypass it.

---

### User Story 3 - Understand the result of an aggregate force attempt (Priority: P3)

An operator receives a concise, deterministic account of the aggregate forced push outcome and any entry that prevented or interrupted it.

**Why this priority**: Operators need to distinguish a fully accepted aggregate from blocked, drifted, or partially completed work before deciding what to do next.

**Independent Test**: Introduce a controlled safety blocker or external drift into one managed entry and verify that the result identifies the affected entry and does not claim successful convergence for it.

**Acceptance Scenarios**:

1. **Given** evidence changes after the aggregate plan is created, **When** Grip reaches the affected action, **Then** Grip reports the drift precisely, does not publish that entry as accepted convergence, and does not attempt later entries.
2. **Given** the aggregate operation completes successfully, **When** Grip presents the result, **Then** the human and machine-readable results distinguish the selected aggregate scope and every completed source-winning change.

### Edge Cases

- A no-selector forced push encounters an exact managed destination-leaf symbolic link: only the already authorized exact source-winning replacement behavior may apply; link targets and link ancestors remain untouched and blocking, respectively.
- A managed source entry is absent: the existing source-winning complete-state semantics, including intentional absence where supported, remain applicable without introducing implicit deletion authority.
- A managed entry becomes unsafe, unsupported, or changes after planning: Grip stops the affected work with a precise result rather than adopting the changed evidence.
- A project has no managed entries requiring source-winning change: Grip reports a deterministic no-change result without creating payload or accepted-state changes.
- The operation encounters a failure after one or more entry actions have been verified: accepted evidence remains published for verified completed entries, the aggregate remains failed, and every failed or later entry remains unaccepted and unattempted.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST accept `grip push --force` without a path selector as an explicit request to select the complete source state for all managed entries in the selected project.
- **FR-002**: A no-selector aggregate forced push MUST include managed entries that require source-winning propagation, including divergent entries and source-present entries with missing destination peers, subject to existing supported-node and safety contracts.
- **FR-003**: A forced push with a supplied selector MUST retain the existing requirement that the selector resolve to one exact managed entry.
- **FR-004**: Before mutating destinations, Grip MUST inspect the aggregate scope and reject unsupported nodes, unsafe symbolic-link ancestry, ownership violations, topology violations, invalid project selection, and other established safety blockers without treating force as a bypass.
- **FR-005**: `grip push --force --dry-run` and `grip push --force -n` without a selector MUST report the same aggregate selection and intended complete source states as execution while making no payload or accepted-state changes.
- **FR-006**: Immediately before each aggregate action, Grip MUST revalidate evidence relevant to that action; if it has changed unsafely, Grip MUST stop the affected action and report the drift precisely.
- **FR-007**: Grip MUST verify each applied source-winning destination state before reporting that entry as successfully changed.
- **FR-008**: Grip MUST provide deterministic human and machine-readable results that identify the aggregate scope, completed entries, blocked entries, and operational failures without claiming convergence for an unverified entry.
- **FR-009**: Grip MUST NOT add aggregate `pull --force`, aggregate `sync`, implicit force, automatic conflict merging, new ownership of destination-only content, or any force bypass for existing safety rules.
- **FR-010**: After an aggregate forced push fails, Grip MUST stop before attempting any later entry, MUST publish accepted baseline evidence for each earlier verified completed entry, MUST report the aggregate operation as failed, and MUST identify every failed or unattempted entry without representing it as converged.

## Key Entities *(include if feature involves data)*

- **Aggregate forced-push scope**: The complete set of managed entries in one selected Grip project considered by a no-selector forced push.
- **Aggregate entry result**: The outcome for one managed entry in the aggregate, including its selected source-complete state and whether it was blocked, changed, verified, or failed.
- **Aggregate accepted state**: The accepted synchronization evidence published for each verified completed aggregate entry before the first failure; failed and unattempted entries remain unaccepted.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In an isolated project containing at least three managed entries across divergent, missing-peer, and already-converged states, one no-selector forced push selects every managed entry and applies the source-complete state to every eligible changed entry.
- **SC-002**: Across all supported aggregate-force acceptance scenarios, a dry run leaves 100% of source payloads, destination payloads, and accepted-state records unchanged.
- **SC-003**: Across all aggregate-force safety-blocker scenarios, 100% of unsupported-node, unsafe-ancestry, ownership, topology, and project-selection blockers remain blocking rather than becoming forceable.
- **SC-004**: In 100% of supplied-selector forced-push acceptance scenarios, Grip retains the existing exact-entry selection behavior.
- **SC-005**: In 100% of induced pre-action drift scenarios, Grip reports the affected entry without reporting it as successfully converged, retains accepted evidence for any earlier verified aggregate entries, and leaves every later entry unattempted.

## Assumptions

- The direct active-user decision and roadmap Feature 027 authorize a narrow exception to the constitution's and earlier command contract's exact-entry force rule: only `grip push --force` with no selector may operate over the selected project's aggregate managed scope.
- Existing verified Features 011, 023, and 026 remain authoritative for selected-project resolution, exact selected force behavior, missing-peer restoration, complete-state semantics, and no-follow symbolic-link safety unless this specification expressly changes the no-selector scope.
- A supplied selector, aggregate pull, aggregate sync, and ordinary push remain outside this feature's changed behavior.
- The direct active-user decisions resolve roadmap Q-22 and the aggregate failure boundary: verified completed entries publish accepted evidence individually; the first later failure leaves the overall operation failed and every failed or later entry unaccepted and unattempted.
