# Feature Specification: Current Status Entries

**Feature Branch**: `032-current-status-entries`

**Created**: 2026-09-16

**Status**: Implemented

**Input**: User description: "Human `grip status` should show which entries are current, not only their count."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Identify current entries in mixed status output (Priority: P1)

An operator can see every current entry alongside entries that require a push, pull, conflict resolution, or baseline acceptance.

**Why this priority**: A count alone cannot identify the entry that needs no action in a large mapping tree.

**Independent Test**: Render a status result with one current and one push entry and verify that the `Current:` section names the current source and destination.

**Acceptance Scenarios**:

1. **Given** status has current and actionable entries, **When** the operator uses human output, **Then** Grip renders a `Current:` section with the current mapping identities before action sections.
2. **Given** status has multiple current entries, **When** the operator uses human output, **Then** Grip renders every current entry in the existing stable record order.

### User Story 2 - Inspect a fully current scope (Priority: P2)

An operator can identify the managed entries that are already synchronized.

**Why this priority**: A clean summary should remain specific, not merely reassuring.

**Independent Test**: Run human status for an accepted single-entry fixture and verify it retains the no-action summary and lists that entry under `Current:`.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Human `status` MUST render each non-attention record in a `Current:` section using its source and destination identities.
- **FR-002**: Current records MUST use `=` to communicate equivalence and retain deterministic status-record ordering.
- **FR-003**: The `Current:` section MUST be present for both mixed and fully current nonempty scopes.
- **FR-004**: Existing action sections, their symbols, summary counts, JSON output, and status exit behavior MUST remain unchanged.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of mixed-status fixtures identify every current record in human output.
- **SC-002**: 100% of fully current nonempty fixtures retain their no-action summary and identify every current record.
- **SC-003**: JSON status fixture output remains byte-for-byte unchanged for equivalent input.

## Assumptions

- A non-attention classification record is the existing definition of current for human status presentation.
