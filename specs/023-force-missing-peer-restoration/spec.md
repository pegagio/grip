# Feature Specification: Forced Missing-Peer Restoration

**Feature Branch**: `023-force-missing-peer-restoration`

**Created**: 2026-09-14

**Status**: Complete

**Input**: User description: "Repair the reproduced failure where `grip push --force` rejects an exact tree entry whose destination was removed, even though the present source was explicitly selected as authoritative. Preserve exact-entry force boundaries and existing absent-winner deletion behavior."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Restore a missing destination from the source (Priority: P1)

An operator who deliberately selects an existing source entry can force it to replace a missing destination peer for one exact managed file or tree member.

**Why this priority**: A forced source choice is the documented safe resolution for this one-sided absence, but it currently fails after the operator makes that explicit choice.

**Independent Test**: In isolated file and tree mappings with accepted baselines, remove one exact destination entry, preview an exact source-selected forced push, then execute it and verify the destination is restored and accepted.

**Acceptance Scenarios**:

1. **Given** an accepted tree member whose source remains unchanged and whose destination file has been removed, **When** an operator runs `grip push --force` with that exact source path, **Then** Grip restores only the missing destination file from the source and refreshes the entry's accepted state.
2. **Given** the same missing-destination entry, **When** an operator runs `grip push --dry-run --force` with the exact source path, **Then** Grip reports the single restoration action and changes neither endpoint nor accepted state.
3. **Given** a source entry whose destination is absent but the source has changed since acceptance, **When** the operator runs exact `grip push --force`, **Then** Grip uses that explicitly selected present source as the winner and restores the destination from its complete current state.

---

### User Story 2 - Restore a missing source from the destination (Priority: P2)

An operator who deliberately selects an existing destination entry can force it to replace a missing source peer for one exact managed file or tree member.

**Why this priority**: Forced direction must remain symmetric: the source-winning repair must not leave the corresponding destination-winning workflow broken.

**Independent Test**: In isolated file and tree mappings with accepted baselines, remove one exact source entry, preview an exact destination-selected forced pull, then execute it and verify the source is restored and accepted.

**Acceptance Scenarios**:

1. **Given** an accepted tree member whose destination remains unchanged and whose source file has been removed, **When** an operator runs `grip pull --force --destination` with that exact destination path, **Then** Grip restores only the missing source file from the destination and refreshes the entry's accepted state.
2. **Given** the same missing-source entry, **When** an operator runs `grip pull --dry-run --force --destination` with the exact destination path, **Then** Grip reports the single restoration action and changes neither endpoint nor accepted state.

---

### User Story 3 - Retain deliberate force boundaries (Priority: P3)

An operator receives accurate one-sided-absence guidance without gaining any broader force authority or changing the existing intentional-deletion behavior.

**Why this priority**: This correction must preserve the safety boundary that requires an exact selected winner rather than turning a missing peer into mapping-wide authority.

**Independent Test**: Exercise present-winner restoration, absent-winner deletion, aggregate tree selection, and unselected entries; verify only the selected entry can change and rejected broad selections remain unchanged.

**Acceptance Scenarios**:

1. **Given** a present selected winner and missing peer, **When** status or a blocked mutation describes the condition, **Then** the human explanation distinguishes missing-peer restoration from a changed peer and gives an executable next step when the entry is exactly selectable.
2. **Given** a selected winner that is itself absent and the present peer remains at its accepted state, **When** the operator uses the existing exact forced direction, **Then** Grip retains its current behavior of propagating that intentional absence to the peer.
3. **Given** a tree root, subtree, omitted selector, or selector covering more than one managed entry, **When** an operator uses forced push or pull, **Then** Grip rejects the request before endpoint mutation.

### Edge Cases

- The surviving winner may equal its accepted state or may contain current changes; an exact force always uses the selected winner's complete current state.
- A missing peer may be a file beneath a mapped tree whose other managed entries remain current; only the selected entry may be restored.
- The selected entry may have metadata or filesystem-safety blockers unrelated to the missing peer; those blockers continue to prevent mutation.
- A dry run must report the same selected restoration as execution while leaving payloads, baselines, locks, and operation evidence unchanged.
- A missing selected winner and an unchanged present peer remains an existing deletion request, not a restoration request. Deletion/change conflicts remain blocked.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST let an exact `push --force` use a present source winner to restore its missing managed destination peer.
- **FR-002**: Grip MUST let an exact `pull --force` use a present destination winner to restore its missing managed source peer.
- **FR-003**: The restoration behavior MUST apply to one exact managed file entry and one exact managed tree member, including a winner whose current state differs from its accepted state.
- **FR-004**: Forced restoration MUST preserve exact-entry selection, complete-scope validation, pre-action revalidation, verified publication, and accepted-state refresh.
- **FR-005**: A forced dry run for missing-peer restoration MUST report the same selected action as execution without changing endpoint payloads, mapping declarations, accepted state, locks, or operation evidence.
- **FR-006**: Grip MUST preserve the existing exact forced-deletion behavior when the explicitly selected winner is absent and the present peer is unchanged from its accepted state.
- **FR-007**: Grip MUST continue to reject omitted, mapping-wide, tree-wide, subtree, ambiguous, multi-entry, ignored, unmanaged, unsupported, or otherwise blocked forced selections before endpoint mutation.
- **FR-008**: Human status and blocked-mutation output for an exact one-sided absence MUST accurately describe the missing-peer condition and present an executable forced next step only when the entry meets the existing exact-entry contract.
- **FR-009**: User-facing documentation MUST state that an exact forced direction can restore a missing losing peer from a present winner or, for an existing simple one-sided deletion, propagate a selected absent winner as deletion.
- **FR-010**: The feature MUST preserve command syntax, selector interpretation, mapping membership, ordinary synchronization behavior, JSON schemas, and unrelated conflict-resolution behavior.

### Key Entities

- **Present forced winner**: The complete current source or destination state explicitly selected to restore its missing managed peer.
- **Missing peer**: The corresponding managed entry absent from the opposite endpoint after an accepted relationship existed.
- **Exact forced restoration**: A one-entry forced directional action that recreates the missing peer from the selected present winner.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In isolated file and tree fixtures, 100% of exact source-selected forced restorations recreate the missing destination and leave unselected managed entries unchanged.
- **SC-002**: In isolated file and tree fixtures, 100% of exact destination-selected forced restorations recreate the missing source and leave unselected managed entries unchanged.
- **SC-003**: In all affected dry-run fixtures, endpoints and accepted state remain byte-for-byte unchanged while the reported restoration action matches execution's selected entry and direction.
- **SC-004**: Existing exact-force rejection and absent-winner deletion regressions retain their expected behavior.
- **SC-005**: Default human output contains no message that characterizes a one-sided missing peer as a changed peer.

## Assumptions

- Feature 011's forced directional contract remains authoritative: the operator must choose one exact managed winner, and a chosen absent winner intentionally propagates deletion.
- Feature 019's executable-guidance rule remains authoritative: force guidance is shown only when its displayed selector resolves to one exact established managed entry.
- This feature corrects the behavior required by those earlier contracts; it does not expand force to aggregate mappings, subtrees, or multiple entries.
- The existing mutation safety model remains sufficient; no new persistence, recovery, locking, caching, or background behavior is required.
