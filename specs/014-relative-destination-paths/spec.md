# Feature Specification: Relative Destination Paths

**Feature Branch**: `014-relative-destination-paths`

**Created**: 2026-09-11

**Status**: Complete

**Input**: User description: "Allow relative destination paths and store them as relative paths."

## Clarifications

### Session 2026-09-11

- Q: Should Grip preserve the operator’s accepted relative destination spelling exactly, or normalize it while keeping it relative? → A: Preserve the accepted relative spelling exactly.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Add a portable relative destination (Priority: P1)

An operator declares a mapping whose destination is relative to the Grip project, so the mapping can be used from another checkout that retains the same project-relative layout.

**Why this priority**: Relative destination declarations let an operator express adjacent or project-contained deployment targets without embedding a machine-specific absolute location.

**Independent Test**: In an initialized project, add an eligible source using a destination such as `../grip-dst/app/`, then inspect the recorded mapping and confirm it remains a relative declaration.

**Acceptance Scenarios**:

1. **Given** an initialized Grip project and an eligible source directory, **When** the operator runs `grip add ./app/ ../grip-dst/app/`, **Then** Grip accepts the source and destination and records the destination as a relative declaration.
2. **Given** a mapping with a relative destination, **When** the operator runs Grip from a descendant directory or uses the project selector, **Then** Grip interprets the destination relative to the selected Grip project rather than the command's current directory.
3. **Given** a project copied to a new parent directory with the same relative source and destination layout, **When** the operator uses the copied project, **Then** the relative destination resolves within the copied layout without editing the mapping declaration.

---

### User Story 2 - Retain existing destination choices (Priority: P2)

An operator can continue to declare an absolute or home-relative destination when portability relative to a Grip project is not the desired model.

**Why this priority**: Adding a portable declaration form must not take away established choices for fixed machine locations or home-relative locations.

**Independent Test**: Add mappings with absolute, `~`, `~/`, and project-relative destinations and confirm each is accepted and distinguishable in the recorded declarations.

**Acceptance Scenarios**:

1. **Given** an initialized Grip project and an eligible source, **When** the operator supplies an absolute, `~`, or `~/` destination, **Then** Grip retains the existing acceptance and declaration behavior.
2. **Given** an initialized Grip project and an eligible source, **When** the operator supplies a relative destination containing `.` or `..` components or repeated separators, **Then** Grip accepts and retains the relative declaration without requiring the operator to pre-normalize it.

### Edge Cases

- A relative destination consisting of `.` resolves to the selected project root; ordinary ownership and topology validation determines whether the requested mapping is safe.
- A relative destination containing `..` may resolve outside the selected project; it remains valid because its project-relative declaration is explicit, while ordinary destination validation still applies.
- A relative destination must not change meaning merely because the command is invoked from a different current working directory.
- Empty, non-text, environment-expansion-like, or unsupported home-user destination forms remain invalid.
- Existing mapping ownership, endpoint-kind, topology, reserved-metadata, and baseline rules continue to apply after a destination form is accepted.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST accept a relative destination path for `grip add`.
- **FR-002**: Grip MUST persist an accepted relative destination as a relative destination declaration using its exact accepted spelling and MUST NOT silently convert or normalize it into another declaration form.
- **FR-003**: Grip MUST interpret a persisted relative destination against the selected Grip project root whenever it validates or uses the mapping, independent of the process current working directory.
- **FR-004**: Grip MUST preserve existing acceptance and persistence behavior for absolute, `~`, and `~/`-prefixed destination declarations.
- **FR-005**: Grip MUST accept a relative destination whose spelling contains lexical dot components or repeated separators and retain that accepted declaration without requiring caller normalization.
- **FR-006**: Grip MUST continue to resolve and persist source declarations relative to the Grip project root; accepting a relative destination MUST NOT broaden source-path behavior beyond the existing contract.
- **FR-007**: Grip MUST distinguish the declared relative destination from its operationally resolved location in any output or validation context where both are relevant.
- **FR-008**: Grip MUST apply existing mapping-ownership, endpoint-kind, topology, reserved-metadata, and baseline validation after accepting a relative destination; this feature MUST NOT weaken those protections.
- **FR-009**: Grip MUST provide a diagnostic that identifies the selected project as the base for a relative destination when that declaration cannot be used, rather than suggesting that the process current working directory is the base.
- **FR-010**: Grip MUST update its user-facing mapping documentation to describe absolute, home-relative, and project-relative destination declarations and their persistence behavior.

### Key Entities *(include if feature involves data)*

- **Mapping declaration**: The user-authored association between a project-relative source and a destination declaration, which may be absolute, home-relative, or relative to the Grip project.
- **Relative destination declaration**: A destination path stored without an absolute or home-relative prefix and interpreted from the selected Grip project root when Grip performs an operation.
- **Resolved destination location**: The filesystem location obtained by interpreting a destination declaration for a specific selected Grip project.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In isolated filesystem tests, 100% of representative relative destination declarations, including `./`, `../`, dot components, and repeated separators, are accepted without a destination-form error.
- **SC-002**: In isolated filesystem tests, every persisted relative destination resolves to the same location for the same selected project when Grip is invoked from at least two different working directories.
- **SC-003**: In isolated filesystem tests, a copied project with an equivalent relative layout can use each representative relative destination declaration without editing that declaration.
- **SC-004**: Existing isolated filesystem coverage for absolute and home-relative destinations and for project-relative source paths continues to pass unchanged.
- **SC-005**: Operators can complete `grip add ./app/ ../grip-dst/app/` in one command without converting either path into a normalized or absolute spelling first.

## Assumptions

- A relative destination is project-relative, not current-working-directory-relative; the selected Grip project is the stable, persisted base for that declaration.
- As with existing accepted `~/` forms, Grip retains accepted relative destination spelling in mapping intent and resolves it only for validation and use.
- The feature supersedes Feature 012 only where Feature 012 rejects relative destinations; Feature 012 remains historical evidence for absolute and home-relative destination behavior.
- The command example using `./app/` depends on Feature 013's project-relative source-input behavior; this feature changes destination declarations only.
- Relative destinations may resolve outside the project through `..`; ordinary ownership and topology protections remain responsible for rejecting unsafe or overlapping mappings.
- This feature changes destination forms only. It does not add new commands, automatic migration of existing declarations, remote destination syntax, or a current-working-directory-relative destination mode.
