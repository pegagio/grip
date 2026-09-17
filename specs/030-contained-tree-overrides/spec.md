# Feature Specification: Contained Tree Mapping Overrides

**Feature Branch**: `030-contained-tree-overrides`

**Created**: 2026-09-16

**Status**: Implemented

**Input**: User description: "Allow `grip add home/ ~/` when an existing disjoint exact mapping owns a destination leaf such as `git/ignore -> ~/.gitignore`, but reject it when `home/.gitignore` exists because two mappings would then manage the same destination."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Add a contained home tree alongside an exact override (Priority: P1)

An operator can add a contained `home/ → ~/` tree after an exact file mapping to a destination leaf inside `~/`, when the tree source has no corresponding source member.

**Why this priority**: Exact mappings are useful overrides within a broad home deployment tree.

**Independent Test**: Add `git/ignore -> ~/.gitignore`, then add `home/ -> ~/` with no `home/.gitignore`; both declarations remain listed.

**Acceptance Scenarios**:

1. **Given** an exact file mapping whose destination is beneath a contained tree destination and whose source is outside the tree source, **When** the tree source has no corresponding member, **Then** `grip add home/ ~/` succeeds without replacing the exact mapping.
2. **Given** both mappings are present, **When** inspection or synchronization runs, **Then** the exact mapping remains the sole owner of its destination leaf.

### User Story 2 - Reject duplicate destination ownership (Priority: P2)

An operator receives a precise rejection rather than a registry with two mappings that manage the same destination leaf.

**Why this priority**: Exact-over-tree precedence must never hide duplicate managed content.

**Independent Test**: Create `home/.gitignore`, add `git/ignore -> ~/.gitignore`, then verify `grip add home/ ~/` fails without publishing mapping or state changes.

**Acceptance Scenarios**:

1. **Given** the tree source contains a member mapping to an existing exact mapping's destination leaf, **When** the operator adds the tree, **Then** Grip rejects the candidate with both conflicting source and destination paths.
2. **Given** an admitted registry later gains that tree source member, **When** Grip loads the registry for inspection or mutation, **Then** it blocks rather than silently assigning that destination to two mappings.

### User Story 3 - Explain failed add commands accurately (Priority: P3)

An operator sees the add failure rather than a successful-looking empty `Mapped:` section.

**Why this priority**: A failed configuration change must never look like a successful mapping publication.

**Independent Test**: Trigger a mapping ownership rejection through human `grip add` and verify it begins with `Error:` and does not contain `Mapped:`.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST allow a disjoint exact file mapping to reserve a destination leaf beneath a contained tree mapping destination when the tree has no source member at that leaf.
- **FR-002**: Grip MUST reject a candidate or loaded registry when a contained tree source contains a current member that would map to an exact file mapping's reserved destination leaf.
- **FR-003**: The exception MUST apply only to a tree whose source is a strict descendant of its destination, an exact file destination strictly beneath that tree destination, and disjoint source namespaces.
- **FR-004**: All other duplicate-source, destination-overlap, and cross-mapping-recursion checks MUST remain in force.
- **FR-005**: A human failed `add` result MUST render its error and conflict evidence, and MUST NOT render successful-add output.
- **FR-006**: Candidate rejection MUST publish no descriptor, baseline, state fence, or payload mutation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The valid exact-override fixture records exactly two mappings and leaves all payloads unchanged during add.
- **SC-002**: In 100% of duplicate-leaf fixtures, candidate and later-load rejection prevent tree ownership of the exact mapping destination.
- **SC-003**: In 100% of human rejected-add fixtures, output contains an error and no `Mapped:` heading.

## Assumptions

- The exact file mapping remains the leaf owner; the tree does not adopt that leaf through precedence or implicit ignore policy.
- Existing contained-tree, no-follow, source-defined membership, and mutation safety rules remain authoritative.
