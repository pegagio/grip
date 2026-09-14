# Feature Specification: Force Mapping Replacement

**Feature Branch**: `working`

**Created**: 2026-09-14

**Status**: Complete

**Input**: User description: "Allow `grip add --force SOURCE DESTINATION` to replace one existing conflicting mapping, while a later ordinary add succeeds after removal of that exact mapping."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Replace an Active Mapping Deliberately (Priority: P1)

As a Grip operator, I want to explicitly replace a mapping that already owns my chosen destination so that I can switch a managed executable or file to a new source without manually reconstructing project configuration.

**Why this priority**: The current ownership rejection prevents an intentional, well-scoped replacement even when the operator has selected the exact desired destination.

**Independent Test**: In an isolated project, create one file mapping, then run `grip add --force` with a different source and the same resolved destination; verify that exactly the prior mapping is replaced, endpoint payloads are unchanged, and the new mapping has the established initial comparison behavior.

**Acceptance Scenarios**:

1. **Given** one active file mapping whose destination resolves to a selected destination, **When** an operator runs `grip add --force` with a different valid file source and that destination, **Then** Grip replaces exactly that mapping and reports the replacement.
2. **Given** the same active conflicting mapping, **When** an operator runs ordinary `grip add`, **Then** Grip retains the existing ownership-conflict rejection and does not change mappings, state, or payloads.
3. **Given** a successful forced replacement, **When** the operator inspects mappings and status, **Then** the old source is no longer managed at that destination and the new source follows the established add-time comparison behavior.

---

### User Story 2 - Re-add After Exact Removal Without Force (Priority: P2)

As a Grip operator, I want an ordinary add to succeed after I remove the exact conflicting mapping so that I am not forced to use a destructive-looking option when no active mapping remains.

**Why this priority**: A successful removal must completely retire the mapping’s ownership and current comparison evidence.

**Independent Test**: In an isolated project, add a mapping, remove that same source mapping, then add a replacement source to the former destination without `--force`; verify that the add succeeds and no stale ownership conflict remains.

**Acceptance Scenarios**:

1. **Given** an operator removes the exact mapping that owns a destination, **When** the operator later adds a valid replacement source to that destination without `--force`, **Then** the add succeeds without an ownership-conflict error.
2. **Given** an operator removes a different mapping, **When** another active mapping still owns the requested destination, **Then** ordinary add retains the ownership-conflict error.
3. **Given** removal or replacement cannot complete safely, **When** Grip reports the failure, **Then** the previously accepted mapping and its current evidence remain authoritative.

---

### User Story 3 - Preserve Explicit Ownership Boundaries (Priority: P3)

As a Grip operator, I want forced add to remain narrowly scoped so that it cannot silently replace multiple mappings, bypass unrelated topology rules, or modify payloads.

**Why this priority**: A force option must make one deliberate registry decision, not become a broad ownership override.

**Independent Test**: Exercise file, tree, overlapping, and ambiguous mapping fixtures; verify that only one exact equal-destination file mapping is replaceable and every other ownership or topology conflict remains blocked.

**Acceptance Scenarios**:

1. **Given** the request matches zero or more than one replaceable active mapping, **When** an operator runs `grip add --force`, **Then** Grip fails without changing mappings, state, or payloads.
2. **Given** a requested mapping has a source-overlap, tree, nested, or other non-equal-destination ownership conflict, **When** an operator runs `grip add --force`, **Then** Grip retains the existing conflict rejection.
3. **Given** a force-add attempt succeeds or fails, **When** an operator compares source and destination payloads, **Then** `add` has not copied, deleted, or otherwise changed either endpoint.

### Edge Cases

- The requested source and destination are invalid, missing, incompatible, unsupported, or unsafe; force does not weaken existing input and filesystem validation.
- The prior mapping has incomplete or stale add evidence; Grip must not leave a partially replaced mapping or combine old and new evidence.
- A concurrent Grip operation or external change invalidates the replacement evidence; Grip fails precisely before accepting a partial replacement.
- The old and requested mappings use distinct destination spellings that resolve to the same endpoint; ownership is evaluated using the resolved endpoint while accepted declaration spelling remains accurate.
- Human and machine-readable results distinguish an ordinary ownership conflict, a successful forced replacement, and a failed force-add safety check.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST accept `grip add --force SOURCE DESTINATION` as an explicit mapping-replacement request.
- **FR-002**: A forced add MUST replace an existing mapping only when exactly one active file mapping has the same resolved destination as the requested mapping and all existing add validation succeeds.
- **FR-003**: A forced replacement MUST retire the displaced mapping’s declaration and current membership evidence before publishing the replacement mapping’s accepted evidence.
- **FR-004**: A successful `grip remove` of the exact mapping that owns a destination MUST leave no ownership or current comparison evidence that blocks a later ordinary add to that destination.
- **FR-005**: Ordinary `grip add` MUST continue to reject an active ownership conflict, including when an operator removed a different mapping.
- **FR-006**: Forced add MUST retain existing rejection for source-overlap, tree, nested, ambiguous, incompatible, unsupported, or otherwise unsafe ownership and topology conflicts.
- **FR-007**: Neither ordinary nor forced add MUST copy, delete, or modify source or destination payloads.
- **FR-008**: Replacement and removal MUST be all-or-nothing: a failure or detected drift MUST leave the previously accepted mapping and current evidence authoritative, or leave the successfully removed mapping fully absent.
- **FR-009**: Human and machine-readable results MUST identify a successful forced replacement and preserve existing error categories and detail for rejected or failed requests.
- **FR-010**: The feature MUST add isolated regression coverage for successful exact replacement, ordinary add after exact removal, removal of a different mapping, rejected non-exact conflicts, payload non-mutation, and detected-drift or partial-failure safety.

### Key Entities

- **Displaced mapping**: The one active file mapping that force-add is authorized to retire because it has the same resolved destination as the requested mapping.
- **Replacement mapping**: The requested mapping that becomes active only after the displaced mapping is fully retired and the candidate is safely accepted.
- **Exact removal invariant**: The requirement that removing a mapping clears its declaration and current comparison evidence so a later ordinary add does not encounter a stale conflict.
- **Force-add request**: An explicit add request that asks Grip to replace one precisely identified active mapping; it is not a payload synchronization operation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In isolated exact equal-destination file fixtures, 100% of valid forced-add requests replace exactly one mapping and leave both endpoint payloads byte-for-byte unchanged.
- **SC-002**: In isolated fixtures, 100% of ordinary adds after successful removal of the exact prior mapping succeed without force or an ownership-conflict result.
- **SC-003**: In isolated non-exact conflict fixtures, 100% of forced-add requests are rejected without changing accepted mappings, current evidence, or endpoint payloads.
- **SC-004**: In isolated failure and drift fixtures, 100% of interrupted or invalidated replacement attempts leave no partial mapping replacement or mixed prior/candidate evidence.
- **SC-005**: Existing mapping-add, removal, status, ownership, JSON-result, and force-resolution regression suites retain their accepted behavior outside the new exact replacement case.

## Constitution Alignment

- **Principle II — Explicit Ownership and Least Surprise**: Force-add replaces only one exact active file mapping; all unrelated ownership and topology validation remains authoritative.
- **Principle III — Validate, Revalidate, and Verify**: Replacement is all-or-nothing, detects drift, and publishes accepted mapping and current evidence only after safe completion.
- **Principle IV — Bounded Concurrency**: The feature uses narrow Grip-owned publication coordination and evidence revalidation rather than locking endpoints or trees.
- **Principle V — Fast, Observable, and Testable**: Results remain deterministic for people and automation, and every changed behavior is validated in isolated filesystem fixtures.
- **Product boundaries**: The feature does not mutate payloads during add, introduce a history or recovery interface, bypass ownership rules, add background services, or elevate privileges.

## Assumptions

- `--force` on `grip add` is the operator’s explicit confirmation; the feature does not add a second interactive confirmation or a separate preview mode.
- Initial delivery supports replacement only for one exact equal-destination file mapping. Tree, source-overlap, nested, and multi-mapping conflicts remain rejected unless a later feature explicitly expands authority.
- The replacement mapping uses the existing add-time comparison policy for equal, unequal, source-only, and destination-only endpoints.
- The specification amends the add-time mapping behavior introduced by Feature 018 and preserves the exact-entry force boundary established by Feature 019.
