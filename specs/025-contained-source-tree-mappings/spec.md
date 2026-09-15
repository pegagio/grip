# Feature Specification: Contained-Source Tree Mappings

**Feature Branch**: `specs/025-contained-source-tree-mappings`

**Created**: 2026-09-15

**Status**: Complete

**Input**: User description: "Support the core dotfiles use case where a project source directory such as `home/` maps to `~/` even though the destination transitively contains the source. Preserve Grip's ownership safety instead of generally disabling recursive-topology validation."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Map a repository home tree to the user's home (Priority: P1)

An operator keeps dotfiles beneath a project directory named `home/` and maps that source tree to `~/`, even when the project itself is stored somewhere beneath the same home directory.

**Why this priority**: Deploying a repository-owned home overlay is a core Grip use case, but the current root-level topology rule rejects it before Grip can distinguish safe managed members from the containing destination.

**Independent Test**: In an isolated home containing a Grip project with a `home/` source directory, add `home/` to `~/`, then inspect and synchronize representative files and directories while verifying the project and unrelated home content remain unchanged.

**Acceptance Scenarios**:

1. **Given** a project beneath the user's home whose `home/` tree contains `.bashrc`, `.gitconfig`, and `.local/bin/tool`, **When** the operator adds `home/` with destination `~/`, **Then** Grip accepts one tree mapping and identifies those safe source-defined members at their matching home-relative destinations.
2. **Given** the accepted mapping and a changed source member, **When** the operator previews and executes an ordinary push, **Then** Grip changes only the corresponding destination member and publishes accepted state only after successful verification.
3. **Given** the accepted mapping and an unambiguous destination-side change to an established member, **When** the operator previews and executes an ordinary pull, **Then** Grip changes only the corresponding source member under the existing synchronization rules.
4. **Given** the source and destination roots are equal, the destination is beneath the source, or the mapping is a file mapping with containment, **When** the operator attempts to add it, **Then** Grip rejects it without changing mapping intent, accepted state, or payloads.

---

### User Story 2 - Leave the rest of the home directory alone (Priority: P2)

An operator can use the contained-source mapping without Grip recursively inspecting, reporting, or reacting to unrelated files elsewhere in the destination home.

**Why this priority**: The mapping is useful only if ordinary inspection remains bounded to Grip's managed namespace rather than turning the user's entire home directory into an implicit inventory.

**Independent Test**: Populate the isolated destination home with a large unrelated tree, inaccessible directories, unsupported nodes, and the Grip project itself; verify ordinary add, status, diff, preview, and synchronization inspect only current source members and previously accepted identities.

**Acceptance Scenarios**:

1. **Given** unrelated destination-only files that have never been managed, **When** the operator runs add, status, diff, push, pull, or sync for the contained-source mapping, **Then** those files are not enumerated, reported, classified, baselined, selected, or changed.
2. **Given** an unrelated destination subtree changes between two inspections, **When** no current or previously accepted managed identity is affected, **Then** the change does not make the operation stale or blocked.
3. **Given** a destination-only path corresponds to a previously accepted member whose source is now absent, **When** Grip inspects the mapping, **Then** Grip still observes that exact identity and preserves the existing deletion and conflict classifications.
4. **Given** a paired destination for a current or previously accepted member is unsafe, unsupported, inaccessible, or incompatible, **When** Grip inspects the selected scope, **Then** the existing safety blocker remains effective for that member.

---

### User Story 3 - Block only genuinely recursive members (Priority: P3)

An operator receives a precise blocker if a current source member would map to a destination that equals, contains, or falls within the source tree, while unrelated safe members remain understandable and no mutation begins.

**Why this priority**: Permitting the root relationship must not let a later source addition redirect synchronization into the project that supplies the source.

**Independent Test**: Create safe and unsafe source-relative members around the source's location beneath the destination, exercise inspection and mutation commands, then exclude the unsafe member through ordinary source policy and verify the mapping becomes operable without hidden exclusions.

**Acceptance Scenarios**:

1. **Given** the source is located at `~/dotfiles/home/`, **When** a source member's relative path is equal to, an ancestor of, or a descendant of `dotfiles/home`, **Then** Grip reports that member as recursively unsafe and begins no payload mutation in the selected scope.
2. **Given** the same mapping contains only source members whose relative destinations are disjoint from the source tree, **When** Grip validates the mapping, **Then** the root containment alone does not block those members.
3. **Given** an unsafe member is explicitly excluded by applicable `.gripignore` policy before it becomes managed, **When** Grip reinspects the mapping, **Then** the excluded member does not own a destination and does not produce a recursive-member blocker.
4. **Given** a previously accepted identity becomes recursively unsafe after the project moves or the home binding changes, **When** Grip attempts to rebind or operate, **Then** it blocks before mutation and does not silently discard the accepted evidence.

---

### User Story 4 - Preserve established ownership boundaries (Priority: P4)

An operator gains the contained-source use case without gaining a general topology override, broader force authority, or cross-mapping overlap.

**Why this priority**: The exception must remain narrow enough that existing mapping and synchronization guarantees retain their meaning.

**Independent Test**: Run the existing equal, nested, duplicate, source-overlap, destination-overlap, cross-mapping recursion, selector, force, and dry-run scenarios alongside contained-source fixtures and verify only the intended same-mapping tree relationship changes.

**Acceptance Scenarios**:

1. **Given** two mappings whose source or destination namespaces overlap under existing rules, **When** one of them is added or operated, **Then** the existing complete-registry conflict remains rejected.
2. **Given** a contained-source mapping with a conflict, absence, or unsupported managed member, **When** the operator uses ordinary or forced synchronization, **Then** existing exact-entry authority and blocker rules remain unchanged.
3. **Given** an inspection or dry run of a contained-source mapping, **When** it completes or fails, **Then** it changes no payload, mapping intent, accepted state, or operation evidence.

### Edge Cases

- The source may be an immediate child or a deeply nested descendant of the destination; containment comparisons use complete path components rather than textual prefixes.
- A safe source member may be a sibling of the destination-relative path that locates the source, while an equal, ancestor, or descendant member is recursively unsafe.
- An ignored directory may contain paths that would otherwise be recursively unsafe; its unseen descendants remain outside managed membership rather than being fabricated as safe entries.
- A previously accepted source member may be deleted while its destination remains; Grip must inspect that retained identity without walking unrelated destination siblings.
- The project may move between machines so a previously disjoint mapping becomes contained, a contained mapping becomes disjoint, or the containment-relative path changes.
- Unrelated destination content may include unreadable directories, links, special nodes, nested mounts, or very large trees; it must not affect operations unless it is the paired path of a current or previously accepted identity.
- A current or retained managed identity may resolve through a path whose node kind, name comparison, metadata capability, or ancestry is unsafe; existing fail-closed behavior remains authoritative.
- A missing destination root, incompatible endpoint kinds, path traversal, unsafe ancestry, or project metadata overlap retains its existing validation behavior.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST accept a tree mapping whose resolved source root is a strict descendant of its resolved destination root when every currently managed member satisfies the contained-source safety rules.
- **FR-002**: Grip MUST continue to reject equal source and destination roots, a destination root equal to or beneath its source root, and any contained file mapping.
- **FR-003**: Grip MUST determine current tree membership from source-side discovery and applicable `.gripignore` policy before evaluating member-level recursive topology.
- **FR-004**: For a source root located at destination-relative path `P`, Grip MUST treat a managed member as recursively unsafe when the member's relative destination is equal to `P`, contains `P`, or is contained by `P`.
- **FR-005**: A recursively unsafe member MUST block mapping addition or any later selected mutation before the first payload action, with deterministic human and machine-readable detail identifying the mapping, member, source path, destination path, and unsafe relationship.
- **FR-006**: Grip MUST NOT create implicit exclusions for contained-source mappings. An operator MAY use ordinary explicit `.gripignore` policy to keep an otherwise unsafe source member outside managed membership.
- **FR-007**: Ordinary mapping inspection MUST examine destination payload only for current non-ignored source members and retained accepted identities belonging to active mappings; it MUST NOT recursively enumerate arbitrary never-managed destination-only content.
- **FR-008**: Never-managed destination-only content MUST remain unreported, unclassified, unbaselined, unselected, and untouched by ordinary add, list, status, diff, push, pull, and sync workflows.
- **FR-009**: Grip MUST continue to inspect a retained accepted identity when its source member is absent so existing source-deletion, destination-change, conflict, converged-deletion, and forced-direction behavior remains available.
- **FR-010**: Current ignored members and retained identities beneath an effective ignored exclusion MUST retain the existing ignore and accepted-state retirement behavior rather than being interpreted as ordinary source deletions.
- **FR-011**: Paired destinations for current or retained managed identities MUST retain existing endpoint-kind, ancestry, name-compatibility, metadata-capability, unsupported-node, conflict, drift, and revalidation protections.
- **FR-012**: Adding a contained-source mapping MUST retain existing non-mutation and initial comparison behavior: `grip add` changes no endpoint payload, equivalent peers may be accepted, differing peers follow the current source-authoritative addition contract, and source-only members retain their current addition behavior.
- **FR-013**: Grip MUST derive the containment relationship from the selected machine's resolved project, source, and destination context without persisting machine-specific paths in portable mapping intent.
- **FR-014**: If rebinding changes the containment relationship or makes any retained accepted identity recursively unsafe, Grip MUST block acceptance and mutation until the contradiction is resolved; it MUST NOT silently discard accepted evidence or weaken the member-safety rule.
- **FR-015**: Grip MUST revalidate the contained-source relationship and relevant current or retained member evidence before mutation so a newly introduced unsafe member or concurrent topology change cannot be acted upon from stale inspection.
- **FR-016**: Existing source/source overlap, destination/destination overlap, duplicate identity, cross-mapping recursion, project-metadata exclusion, and complete-registry validation MUST remain authoritative for all mappings.
- **FR-017**: The feature MUST preserve command syntax, selector interpretation, ordinary and forced synchronization authority, dry-run behavior, supported payload types, conflict resolution, and deletion authorization.
- **FR-018**: User-facing documentation MUST explain the supported `home/` to `~/` layout, the bounded managed-member inspection model, the recursive-member blocker, explicit `.gripignore` remediation, and the unchanged unsafe topology cases.

### Key Entities

- **Contained-source tree mapping**: A tree mapping whose resolved source root is a strict descendant of its resolved destination root and whose ownership is limited to safe source-defined members.
- **Containment-relative path**: The non-empty relative path from the destination root to the source root on the selected machine. It is operational evidence, not portable mapping intent.
- **Managed member**: A current non-ignored source entry or retained accepted identity paired with the same relative destination beneath a tree mapping.
- **Recursive member**: A candidate managed member whose paired destination equals, contains, or falls within the mapping's source root.
- **Never-managed destination content**: A destination entry that corresponds to neither current source membership nor a retained accepted identity and therefore remains outside Grip ownership and inspection.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In isolated fixtures where a project beneath the destination home contains a `home/` source tree, 100% of safe `home/` to `~/` additions succeed without changing source or destination payloads during `add`.
- **SC-002**: Across representative status, diff, preview, push, pull, and sync fixtures, zero never-managed destination-only entries are reported, baselined, selected, changed, or required to be readable.
- **SC-003**: In the complete equal, source-contained, destination-contained, and member-relative topology matrix, 100% of unsafe cases block before payload mutation and 100% of safe contained-source members remain operable.
- **SC-004**: In source-deletion fixtures, 100% of retained accepted identities preserve their existing deletion or conflict classification without requiring traversal of unrelated destination siblings.
- **SC-005**: In project-move and home-rebinding fixtures, every changed topology is either safely accepted with the same portable declaration or rejected before mutation with a deterministic actionable blocker; no accepted evidence is silently lost.
- **SC-006**: In a representative contained-source fixture with at least 100 managed members and 10,000 unrelated destination entries, at least 95% of warm read-only status runs complete within one second on the supported release platform.
- **SC-007**: Existing ownership, selector, synchronization, force, deletion, ignore-policy, filesystem-safety, dry-run, and publication regression suites retain their expected behavior outside the explicitly superseded same-mapping and destination-only-enumeration rules.

## Assumptions

- The primary layout is a Grip project stored beneath the user's home with a source directory such as `home/` mapped to `~/`; the same rules apply to any equivalent strict tree containment.
- Tree roots are mapping anchors. Managed payload ownership belongs to admitted relative members, not to every unrelated entry beneath the destination anchor.
- Never-managed destination-only content does not need to appear in ordinary status or diff output because Grip neither owns nor acts on it. This feature intentionally supersedes Feature 003's arbitrary destination-only enumeration while preserving observation of retained accepted identities.
- Existing `.gripignore` semantics are sufficient for explicit source-policy exclusions; the feature adds no automatic ignore pattern, destination-side ignore authority, or new public command.
- Existing baseline, classification, synchronization, force, deletion, metadata, and publication contracts remain sufficient once inspection is bounded to current and retained managed identities.
- The feature flows forward from Features 002 and 003; their merged artifacts remain historical records rather than being rewritten.
