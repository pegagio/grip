# Feature Specification: Current-Directory Status Paths

**Feature Branch**: `016-cwd-status-paths`

**Created**: 2026-09-11

**Status**: Complete

**Input**: User description: "With `grip status`, the source files' paths should be relative to the current working directory. The user should be able to `grip status` and see files that need to be pushed, and then copy the source paths to use in `grip push`."

## Clarifications

### Session 2026-09-11

- Q: Should source paths containing shell-special characters be rendered as shell-quoted copy-paste tokens? → A: Follow Git-style relative human display; do not promise universally shell-safe tokens.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Copy a source path into push (Priority: P1)

An operator runs `grip status` from any directory and can use an ordinary listed source path directly in a subsequent `grip push` invocation from that same directory.

**Why this priority**: The status view should lead directly to the next safe operator action without manually translating an absolute or project-root-relative path.

**Independent Test**: From a nested source directory, create an eligible source-side change, run `grip status`, copy the source side of its `Changes to push` row, and use it as the selector to `grip push` from that same directory. The selected entry is the one shown by status.

**Acceptance Scenarios**:

1. **Given** a pushable source entry beneath the current working directory, **When** the operator runs `grip status`, **Then** its source label is relative to that directory and can be used unchanged as the `grip push` selector from the same directory.
2. **Given** a pushable source entry outside the current working directory, **When** the operator runs `grip status`, **Then** its source label uses a relative parent traversal when necessary and can be used unchanged as the `grip push` selector from the same directory.
3. **Given** a project selected explicitly while the process runs outside that project, **When** the operator runs `grip status` and then `grip push` from that unchanged directory, **Then** an ordinary displayed source selector still round-trips to the shown managed entry.

---

### User Story 2 - Read concise status without losing endpoint context (Priority: P2)

An operator retains the concise status grammar while being able to distinguish the copyable source selector from the destination endpoint.

**Why this priority**: Feature 015's concise direction-oriented status view must remain recognizable and safe.

**Independent Test**: Exercise push, pull, conflict, and needs-baseline records from a nested directory and verify that every row preserves its existing directional symbol and destination representation while source labels are relative to the invocation directory.

**Acceptance Scenarios**:

1. **Given** any nonempty default-human status section, **When** a row is rendered, **Then** only its source-side path is expressed relative to the invocation directory.
2. **Given** a current entry or an empty selected scope, **When** the operator runs `grip status`, **Then** the existing concise summary behavior is unchanged.

---

### User Story 3 - Preserve automation and safe selection boundaries (Priority: P3)

An operator gains copy-pasteable source paths without changing machine-readable output or allowing a copied selector to escape the selected project.

**Why this priority**: Human usability must not weaken project ownership or automation contracts.

**Independent Test**: Compare JSON status before and after the feature for the same fixture, then attempt to use a relative selector that resolves outside the selected project and verify that `grip push` rejects it without mutation.

**Acceptance Scenarios**:

1. **Given** a script requesting JSON status, **When** the feature is delivered, **Then** source-path fields and the structured status schema remain unchanged.
2. **Given** a copied-looking relative selector that resolves outside the selected project, **When** the operator runs `grip push`, **Then** Grip rejects it before inspecting or mutating an unmanaged path.
3. **Given** a source path containing spaces or unusual printable characters, **When** it is shown by default human status, **Then** Git-style human quoting preserves an unambiguous relative source representation without promising a universally shell-safe token.

### Edge Cases

- The current directory can be the project root, a nested source directory, a sibling directory, or outside an explicitly selected project.
- A source entry can be absent, at the selected project root, or outside the current directory; its human status label must still be a relative selector that does not imply ownership outside the project.
- Destination-space selectors and destination output remain distinct from copyable source selectors.
- Paths requiring display escaping use Git-style human quoting; this display convention does not promise a universally shell-safe token for every possible filename.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Default human `grip status` output MUST render every source-side path in an actionable row relative to the process current working directory, including `..` components when required.
- **FR-002**: Default human `grip status` MUST use Git-style human path display for relative source labels: ordinary paths remain unquoted and paths requiring display escaping use conventional C-style quoting. This display convention MUST NOT promise a universally shell-safe token for every possible filename.
- **FR-003**: An ordinary source-side path emitted by default human `grip status` MUST be accepted unchanged as the source selector of `grip push` when both commands use the same current working directory and project selection.
- **FR-004**: `grip push` MUST resolve a relative source selector from the process current working directory, then require the resolved path to remain inside the selected project before applying normal managed-entry selection.
- **FR-005**: Default human status MUST preserve Feature 015's summary, section names, directional symbols, stable ordering, blocker visibility, and destination-side rendering.
- **FR-006**: The feature MUST preserve JSON status records and their source-path values, status exit behavior, mapping declarations, destination-space selection, and filesystem mutation rules.
- **FR-007**: A relative source selector that resolves outside the selected project MUST fail before unmanaged-path inspection or mutation.
- **FR-008**: User-facing documentation MUST show a nested-directory status example and its ordinary copy-pasteable `grip push` follow-up.

### Key Entities *(include if feature involves data)*

- **Copyable source selector**: The source component rendered in default human status relative to the process current working directory and accepted by `grip push` from that same directory.
- **Invocation directory**: The process current working directory that defines both source-label rendering and copied-selector interpretation.
- **Selected project boundary**: The project root that a resolved source selector must remain within before normal managed-entry selection proceeds.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In fixtures invoked from the project root, a nested source directory, a sibling directory, and outside an explicitly selected project, 100% of displayed ordinary pushable source labels select the same entry when copied into `grip push` from the unchanged invocation directory.
- **SC-002**: In representative push, pull, conflict, and baseline-needed status fixtures, 100% of default-human rows retain their existing section and directional symbol while displaying source labels relative to the invocation directory.
- **SC-003**: Existing JSON status contract and exit-behavior tests pass without changing expected structured records or exit codes.
- **SC-004**: Every tested source selector resolving outside the selected project is rejected before mutation.

## Assumptions

- This feature follows Feature 015 and changes only the default human source-path presentation plus the `grip push` source-selector interpretation needed for ordinary copy-paste round trips.
- The destination-side status path remains in its existing representation because it is not the selector copied into `grip push`.
- Relative source selectors may contain `..` when the invocation directory is outside the displayed source path, but their resolved path remains subject to the selected project boundary.
- Existing source declaration input and source-selector behavior for commands other than `grip push` remain unchanged unless later clarification expands the scope.
- Git-style human display uses conventional C-style quoting for paths requiring display escaping; it is a readability convention rather than a cross-shell argument-escaping guarantee.
