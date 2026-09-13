# Feature Specification: Executable Force-Resolution Guidance

**Feature Branch**: `019-executable-force-guidance`

**Created**: 2026-09-13

**Status**: Complete

**Input**: User description: "`grip status` printed `grip pull --force --destination DESTINATION` for a tree mapping, but the command failed because Grip requires one exact established managed entry. Every displayed force-resolution instruction must be executable."

## Clarifications

### Session 2026-09-13

- Q: What should Grip show for an aggregate tree conflict that cannot be force-resolved as one exact entry? → A: Show `Run: grip diff SOURCE`.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Follow a displayed resolution command (Priority: P1)

An operator sees a source-winning or destination-winning force command in default human output and can copy and run it from the same directory without an invalid-selector or exact-entry error.

**Why this priority**: Suggested commands are part of Grip’s safety guidance. A command that fails immediately defeats the operator’s next safe action.

**Independent Test**: Create isolated file and tree conflicts; run every force command that default human `status`, `push`, `pull`, or `sync` displays; verify it passes selector validation and resolves exactly the displayed managed entry.

**Acceptance Scenarios**:

1. **Given** a force-resolvable conflict for one exact managed file, **When** an operator runs `grip status`, **Then** each displayed source-winning and destination-winning command is accepted by the corresponding forced command from the invocation directory.
2. **Given** a conflict reported for a tree mapping root or other aggregate scope that cannot be selected as one exact established entry, **When** an operator runs `grip status`, **Then** Grip does not display either force command for that row.
3. **Given** such an aggregate conflict, **When** an operator reads its status row, **Then** Grip provides a valid inspection command that identifies the exact conflict entries needed before a forced choice.

---

### User Story 2 - Preserve safe force scope (Priority: P2)

An operator receives truthful guidance without changing the existing requirement to choose one exact managed entry for a forced directional resolution.

**Why this priority**: Making an aggregate tree selector force-resolvable would turn presentation repair into a potentially broader overwrite authority.

**Independent Test**: Exercise force guidance for file and tree conflicts, including destination-path selection; verify the existing selected entry, mutation scope, JSON result, and exit behavior remain unchanged.

**Acceptance Scenarios**:

1. **Given** an aggregate tree conflict, **When** the feature is delivered, **Then** `grip push --force TREE` and `grip pull --force --destination TREE_DESTINATION` retain their existing exact-entry validation rather than force-resolving the whole mapping.
2. **Given** any affected default human output, **When** the feature is delivered, **Then** JSON output, `grip diff` detail, classification, selection, and mutation behavior are unchanged.

### Edge Cases

- A tree may have several exact conflicted members; no aggregate row may imply that one force command will resolve all of them.
- A source or destination display path may be relative to the invocation directory or absolute; guidance must use the form accepted by its corresponding command.
- Initial collisions without accepted entry state must not be presented as force-resolvable merely because their classification is a conflict.
- Technical, compatibility, deletion, and incomplete-publication blockers remain without invented force guidance.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Default human output MUST display source-winning and destination-winning force commands only when the corresponding displayed selector resolves to one exact established managed entry accepted by the existing forced-resolution contract.
- **FR-002**: For every displayed source-winning or destination-winning command, running that command from the same invocation directory MUST pass selector validation before any normal conflict-resolution planning.
- **FR-003**: For a conflict row that represents an aggregate mapping or otherwise cannot be force-resolved as one exact entry, default human output MUST omit both force commands and MUST provide `Run: grip diff SOURCE`, using the displayed source selector, to find exact conflicted entries.
- **FR-004**: The feature MUST preserve exact-entry forced-resolution scope; it MUST NOT make a tree root, subtree, mapping, or multi-entry conflict force-resolvable solely to match displayed output.
- **FR-005**: The rule in FR-001 and FR-003 MUST apply consistently to default human `status` and default human blocked mutation output wherever force guidance is rendered.
- **FR-006**: The feature MUST preserve command syntax, JSON schemas and values, JSON and human exit behavior, `grip diff` output, classification, selector interpretation, mutation planning, and filesystem-safety behavior.
- **FR-007**: User-facing documentation MUST describe that force guidance is shown only for an exact resolvable entry and that aggregate tree conflicts require inspecting exact entries first.

### Key Entities

- **Executable force guidance**: A displayed forced `push` or `pull` command whose selector satisfies Grip’s existing exact-established-entry requirement.
- **Aggregate conflict**: A reported conflict row representing a mapping, subtree, tree root, or other scope that is not one force-resolvable entry.
- **Inspection next step**: A read-only command shown for an aggregate conflict so an operator can identify the exact entry before choosing a winner.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In isolated file and tree conflict fixtures, 100% of force commands printed in default human output pass selector validation when copied from that invocation directory.
- **SC-002**: In aggregate tree-conflict fixtures, default human output contains zero force commands that target the aggregate root and contains one valid inspection next step per aggregate conflict row.
- **SC-003**: Existing forced-resolution, JSON, exit-category, selection, and `grip diff` regression tests retain their expected behavior.
- **SC-004**: Default human output for technical and non-force-resolvable blockers contains zero invented winner commands.

## Assumptions

- The existing exact-entry forced-resolution boundary remains authoritative; this feature corrects guidance rather than expanding force authority.
- `SOURCE` in aggregate-conflict guidance is the same invocation-directory-relative source display used by status and is accepted by `grip diff`.
- This feature supersedes Feature 017 only where its force-guidance requirement would display a command that the existing force contract rejects.
