# Feature Specification: Exact Destination Symlink Replacement

**Feature Branch**: `026-exact-destination-symlink-replacement`

**Created**: 2026-09-15

**Status**: Complete

**Input**: User description: "Allow mapping intent to be recorded when an exact managed destination leaf is a symbolic link, report the link precisely, and permit explicit exact-entry forced push to replace the link object with the mapped source state without following its target."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Record a Mapping with a Destination Link (Priority: P1)

An operator can add a source-defined mapping even when an exact destination member is a symbolic link, so that a dotfiles tree can be managed without modifying existing payloads during admission.

**Why this priority**: Mapping admission is the prerequisite for every later safe action and preserves Grip's nonmutating `add` contract.

**Independent Test**: Add a tree containing a regular source member whose exact paired destination is a symbolic link, then verify the mapping is recorded, neither endpoint nor the link target changes, and the member is reported as unresolved.

**Acceptance Scenarios**:

1. **Given** a source-defined tree has a regular member and its exact paired destination is a symbolic link, **When** the operator adds the mapping, **Then** Grip records mapping intent without changing the source, link object, or link target and reports that member as an unresolved destination link.
2. **Given** an added mapping has an unresolved exact destination link, **When** the operator inspects its status or requests a dry run, **Then** Grip identifies the managed member and link path and explains that exact source-winning force is required before replacement.

---

### User Story 2 - Explicitly Replace One Destination Link (Priority: P2)

An operator can deliberately make one selected regular-file or directory source state authoritative over its exact destination link without affecting other links or any link target.

**Why this priority**: It provides a deliberate, narrowly bounded path from an unresolved obstacle to an accepted managed entry.

**Independent Test**: Select one unresolved member and force a source-winning push; verify that only its checked destination link object is replaced, the target remains unchanged, the replacement matches the selected source, and accepted state is recorded only after verification.

**Acceptance Scenarios**:

1. **Given** an unresolved exact destination link and a selected regular-file source member, **When** the operator runs an exact source-winning forced push, **Then** Grip replaces only the destination link object with the source state, does not follow or modify the former target, and records accepted state after verifying the replacement.
2. **Given** an unresolved exact destination link and a selected directory source member, **When** the operator runs an exact source-winning forced push, **Then** Grip replaces only that link object with the selected directory state and leaves unrelated managed and unmanaged links untouched.
3. **Given** the selected entry is absent, ambiguous, aggregate, or a destination-winning operation, **When** the operator requests force, **Then** Grip refuses before changing any payload.

---

### User Story 3 - Preserve Link Safety Boundaries (Priority: P3)

An operator receives a precise blocker instead of an unsafe action when a source link, a required destination ancestor link, or an ordinary synchronization operation would require link support beyond this feature.

**Why this priority**: The feature must make one exception without weakening Grip's allowlisted payload boundary or ownership protections.

**Independent Test**: Exercise each unsupported link placement and each non-force operation, then verify it is blocked before mutation with an actionable path-specific diagnostic.

**Acceptance Scenarios**:

1. **Given** a required destination ancestor is a symbolic link, **When** Grip adds, inspects, plans, or acts on the affected managed member, **Then** it reports the ancestor as a hard blocker and does not follow, replace, or modify it or its target.
2. **Given** a source member is a symbolic link or an ordinary push, pull, sync, or delete would encounter an unresolved destination link, **When** the operator runs the command, **Then** Grip blocks the affected action without changing either endpoint.

### Edge Cases

- A destination link changes identity, is removed, or an ancestor changes after inspection but before forced replacement; the operation stops before mutation and records no accepted state.
- A selected source is not a supported regular file or directory; force does not convert it into a replaceable payload.
- A mapping contains both replaceable exact destination links and safe members; admitting the mapping never replaces any link, and forcing one selected member cannot authorize any other member.
- The former link target is outside the managed namespace, inside it, missing, or itself a link; it remains untouched in every case.
- Existing ownership, recursive-topology, metadata, selection, baseline-publication, and project-containment blockers continue to reject the operation before mutation.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST admit a file mapping or source-defined tree mapping when an exact paired destination leaf is a symbolic link and all non-link ownership, topology, containment, and policy validation succeeds.
- **FR-002**: `grip add` MUST remain payload-nonmutating when it admits an exact destination link: it MUST NOT replace, remove, open as payload, follow, or modify the link object or its target.
- **FR-003**: Grip MUST represent each admitted exact destination link as unresolved managed-entry evidence without treating the link as supported payload or accepted baseline evidence.
- **FR-004**: Status, diagnostics, and dry-run results MUST identify the affected managed entry and exact destination-link path, state that ordinary synchronization is blocked, and give only an executable exact source-winning force action when that selector is valid.
- **FR-005**: Grip MUST allow replacement only for one exact source-space selector through explicit source-winning forced push when the selected source is a supported regular file or directory and all ordinary force preconditions pass.
- **FR-006**: Before replacing a destination link, Grip MUST revalidate the selected source, destination-link identity, required destination ancestry, ownership, topology, and relevant accepted-state evidence; detected drift MUST stop before any payload change.
- **FR-007**: A successful forced replacement MUST remove only the revalidated destination link object, publish the selected source state at that exact path, verify the resulting supported state, and record accepted state only after verification succeeds.
- **FR-008**: Grip MUST never follow, read as payload, modify, replace, or otherwise act on the former link target while admitting, inspecting, planning, replacing, verifying, or reporting an exact destination link.
- **FR-009**: A symbolic link in any required destination ancestor MUST remain a path-specific hard blocker for all affected operations; this feature MUST NOT provide an override for it.
- **FR-010**: Source symbolic links, destination-winning pull replacement, ordinary push, pull, sync, delete, aggregate force, mapping-wide force, multiple selectors, implicit link replacement, and link-target synchronization MUST remain unsupported and nonmutating for unresolved destination links.
- **FR-011**: Grip MUST preserve existing cross-mapping ownership, recursive-member topology, metadata, project-containment, selection, dry-run, partial-failure, verification, and publication safeguards for every admitted or replaced entry.
- **FR-012**: Grip MUST provide isolated filesystem coverage for admission, exact regular-file and directory replacement, no-follow target preservation, ancestor and source-link blockers, invalid selector rejection, concurrent drift, verification failure, and accepted-state publication.
- **FR-013**: The feature's inspection and planning work MUST remain bounded to managed identities and their required ancestors; it MUST NOT enumerate unrelated destination content to discover links.

### Key Entities

- **Unresolved destination link**: An exact managed destination leaf observed as a symbolic link, retained as an obstacle with path-specific evidence but without accepted payload state.
- **Exact replacement candidate**: One selected source member, its paired destination-link object, and the current safety evidence required to authorize a source-winning forced replacement.
- **Accepted replacement state**: The verified supported state recorded only after the destination link object has been replaced successfully.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of isolated admission fixtures containing an exact destination-leaf symlink, Grip records the mapping without changing the source, link object, or link target.
- **SC-002**: In 100% of valid exact forced-replacement fixtures for regular files and directories, Grip replaces only the selected destination link object, preserves the former target unchanged, verifies the resulting source-equivalent state, and records accepted state.
- **SC-003**: In 100% of fixtures for source links, destination-link ancestors, ordinary operations, destination-winning force, aggregate selection, and invalid selectors, Grip performs no payload mutation and reports a path-specific actionable blocker.
- **SC-004**: In 100% of injected pre-action drift and verification-failure fixtures, Grip stops or reports failure without publishing accepted state for the unresolved or partially replaced entry.
- **SC-005**: On the supported local platform, status and dry-run over 100 managed identities plus 10,000 unrelated destination entries complete within one second at the 95th percentile across 100 warm samples while retaining exact link diagnostics.

## Assumptions

- The feature changes only exact destination leaves already paired with managed source identities; unrelated destination links remain unmanaged and uninspected.
- Existing exact-selector and source-winning force syntax remains the operator interface; no new command or option is required.
- Source links and destination ancestor links remain outside the supported payload boundary and require a later separately approved feature if support is needed.
- The supported local filesystem provides a way to inspect and replace the verified link object without following its target; inability to preserve this boundary is a blocker.
- Features 009, 011, 019, 023, and 025 remain authoritative for filesystem safety, command semantics, executable force guidance, missing-peer restoration, and contained-source managed-identity boundaries.
