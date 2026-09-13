# Feature Specification: Source Path Input Normalization

**Feature Branch**: `013-source-path-input`

**Created**: 2026-09-11

**Status**: Complete

**Input**: User description: "Grip should accept `./app/` as a source path when it resolves within the Grip project, while continuing to store a normalized project-relative source declaration."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Add a Mapping with an Ordinary Relative Source Path (Priority: P1)

An operator can add a file or tree mapping using the relative source spelling they naturally use in a shell, including a leading current-directory component or a trailing separator.

**Why this priority**: Mapping creation is blocked today even though the requested source is within the selected Grip project.

**Independent Test**: In an initialized project with an `app` directory, run `grip add ./app/ <valid-destination>` and verify that Grip records the mapping without changing either endpoint.

**Acceptance Scenarios**:

1. **Given** an initialized project containing an `app` directory, **When** the operator runs `grip add ./app/ <valid-destination>`, **Then** Grip accepts the source and records it as `app`.
2. **Given** an initialized project containing `nested/app`, **When** the operator adds it using a relative spelling with harmless dot components, **Then** Grip records the same normalized source declaration as it would for `nested/app`.
3. **Given** a source spelling that resolves outside the selected project or into its `.grip` metadata directory, **When** the operator runs `grip add`, **Then** Grip rejects the request before recording a mapping.

### User Story 2 - Reuse Relative Source Spellings in Source-Space Commands (Priority: P2)

An operator can use the same accepted relative source spelling to select an existing mapping or source-side scope after it has been recorded.

**Why this priority**: A mapping created through an accepted shell-style path should remain addressable through the same user-facing path convention.

**Independent Test**: Add a mapping for `app`, then invoke source-space mapping selection commands using `./app/` and verify that they select the recorded mapping or scope.

**Acceptance Scenarios**:

1. **Given** a mapping declared as `app`, **When** the operator selects it using `./app/` in a source-space command, **Then** Grip selects the `app` mapping or source-side scope.
2. **Given** a selector that resolves outside the project or into `.grip`, **When** the operator uses it in a source-space command, **Then** Grip rejects it without changing mappings, state, or payloads.

### Edge Cases

- A source spelling consisting only of current-directory components continues to be valid only when the resolved project root is used as a tree mapping; it remains invalid for a file mapping.
- Repeated separators, current-directory components, and parent components are accepted only when lexical normalization produces a path within the project and outside `.grip`.
- Existing descriptor files remain strict: stored source declarations must already be normalized and must not be silently rewritten merely because this input behavior changes.
- Absolute source paths, non-UTF-8 source paths, environment-expansion forms, and source paths resolving outside the project remain rejected.
- Destination-path acceptance is unchanged; a relative destination is still invalid even when the source input is accepted.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST accept a source argument containing ordinary relative lexical components when it resolves to a location within the selected project and outside `.grip`.
- **FR-002**: Grip MUST normalize every accepted source argument to one canonical project-relative declaration before mapping identity, lookup, or persistence is evaluated.
- **FR-003**: `grip add ./app/ <valid-destination>` MUST produce the same declared source identity as `grip add app <valid-destination>`.
- **FR-004**: Every CLI command that interprets an argument in source space MUST apply the same acceptance, normalization, and rejection rules.
- **FR-005**: Grip MUST reject a source argument when its normalized result is absolute, escapes the project, identifies `.grip` or a descendant of `.grip`, is empty, or is not valid UTF-8.
- **FR-006**: Grip MUST preserve the existing rule that the project root can represent only a tree mapping.
- **FR-007**: Grip MUST continue to reject a descriptor whose stored source declaration is non-normalized, invalid, outside the project, or inside `.grip`.
- **FR-008**: The change MUST NOT broaden accepted destination forms, mutate either mapping endpoint during `add`, or change destination-space selector behavior.
- **FR-009**: User-facing documentation MUST distinguish accepted source input spellings from the normalized source declaration retained by Grip.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can add a mapping for an existing `app` directory with `./app/` on the first attempt, and the declaration identifies `app`.
- **SC-002**: All supported source-space command families resolve `./app/` to the same mapping or scope as `app` in automated acceptance coverage.
- **SC-003**: Automated coverage confirms that every rejected escaping or `.grip` source spelling leaves mappings, state, and endpoint payloads unchanged.
- **SC-004**: Existing valid normalized descriptor declarations remain readable without migration or modification.

## Assumptions

- The selected project root is already established by Grip's existing project-selection rules before source input is interpreted.
- Lexical normalization is sufficient for the input convention; existing endpoint validation continues to enforce filesystem safety boundaries.
- This feature supersedes Feature 012 only where its explicit exclusion of source-path form changes conflicts with the requirements above. Feature 012 remains historical evidence for destination-path behavior.
- No new command, option, storage schema, migration, or compatibility alias is required.

## Constitutional Alignment

- **Explicit Ownership and Least Surprise**: Accepted input must still resolve only within the selected project and never into Grip-owned metadata.
- **Validate, Revalidate, and Verify**: Input normalization must occur before mapping identity and existing endpoint validation, preserving the established safety boundary.
- **Fast, Observable, and Testable**: The behavior requires deterministic isolated tests covering accepted normalization and rejected boundary crossings without adding state or background work.
