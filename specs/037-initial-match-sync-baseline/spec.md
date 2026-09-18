# Feature Specification: Initial-Match Baseline Synchronization

**Feature Branch**: `037-initial-match-sync-baseline`

**Created**: 2026-09-18

**Status**: Complete

**Input**: User description: "Allow `grip sync SOURCE` to establish a baseline for an equal, unbaselined child beneath a tree mapping, without adding a separate baseline command."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Accept an equal tree member (Priority: P1)

An operator can select one source-defined child beneath a declared tree mapping and use the ordinary sync command to record the child’s already-equivalent source and destination state as accepted.

**Why this priority**: An equal managed child currently remains in `Needs baseline` with no executable ordinary path to make it current.

**Independent Test**: Create a tree mapping with an equivalent unbaselined child, run `grip sync` with that child’s source selector, and confirm the endpoints are unchanged while a later status identifies the child as current.

**Acceptance Scenarios**:

1. **Given** an active tree mapping contains one source-defined regular-file child whose destination peer is equivalent and has no accepted baseline, **When** the operator runs `grip sync` with that child’s source selector, **Then** Grip accepts that child as the baseline without copying, replacing, deleting, or otherwise changing either endpoint.
2. **Given** the same eligible child, **When** the operator runs `grip sync --dry-run` with its source selector, **Then** Grip reports that it would establish a baseline and leaves endpoints and accepted state unchanged.
3. **Given** the operator successfully synchronizes the eligible child, **When** they run `grip status` for the same child, **Then** Grip reports it as current rather than under `Needs baseline`.

---

### User Story 2 - Preserve non-eligible safeguards (Priority: P2)

An operator receives the existing conservative behavior for any selected entry that is not a complete, equivalent, source-defined managed pair.

**Why this priority**: Baseline acceptance must not turn an ordinary synchronization command into a conflict-resolution or ownership-bypass mechanism.

**Independent Test**: Exercise differing initial endpoints, absent peers, ignored members, destination-only entries, and unsupported or unsafe entries through selected sync; confirm no new accepted baseline is published for any of them.

**Acceptance Scenarios**:

1. **Given** an unbaselined selected entry whose endpoints differ, **When** the operator runs `grip sync`, **Then** Grip retains its current conflict or blocked behavior and publishes no baseline.
2. **Given** an entry with an absent, ignored, destination-only, unsupported, or unsafe endpoint condition, **When** the operator runs `grip sync`, **Then** Grip retains the applicable current safety result and publishes no baseline.
3. **Given** a selected entry that is already current, **When** the operator runs `grip sync`, **Then** Grip retains the existing no-action result and does not publish a new baseline generation.

---

### User Story 3 - Understand acceptance-only synchronization (Priority: P3)

An operator can distinguish a successful baseline-only synchronization from a payload-copy synchronization in both preview and completion output.

**Why this priority**: The operation must not suggest that data was copied when it only accepted equivalent evidence.

**Independent Test**: Run dry-run and actual sync for an eligible equal unbaselined child and verify each result identifies baseline establishment without listing payload-copy actions.

**Acceptance Scenarios**:

1. **Given** an eligible equal unbaselined child, **When** the operator previews sync, **Then** the result states that a baseline would be established and does not present a source-to-destination or destination-to-source copy.
2. **Given** the same child, **When** the operator completes sync, **Then** the result states that a baseline was established and does not claim payload changes.

### Edge Cases

- A selected file-mapping root with equivalent unbaselined endpoints follows the same acceptance-only behavior as an exact child beneath a tree mapping.
- An unbaselined tree root selected as a mapping-wide scope does not implicitly accept unrelated members outside the requested exact child scope.
- A concurrent change to the selected endpoint, managed membership, relevant policy, or accepted state before publication prevents baseline acceptance and leaves the prior accepted state authoritative.
- A successful acceptance-only operation does not modify endpoint content, permissions, ownership, timestamps, mappings, ignore policy, or destination-only neighbors.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `grip sync SOURCE` MUST accept a complete equivalent active managed entry with no accepted baseline only when the requested source selector resolves to exactly that one managed entry.
- **FR-002**: The acceptance-only behavior MUST support an exact source selector for a discovered child under a declared tree mapping as well as an exact file mapping.
- **FR-003**: Before publishing accepted evidence, Grip MUST validate the selected scope and revalidate mutable acceptance-relevant evidence so a changed, unsafe, unsupported, ignored, or no-longer-managed entry is not accepted.
- **FR-004**: Acceptance-only sync MUST not copy, replace, delete, create, or modify endpoint payloads, mappings, ignore policy, or unrelated accepted records.
- **FR-005**: `grip sync --dry-run SOURCE` MUST report the same eligible acceptance-only outcome without changing accepted state or endpoints.
- **FR-006**: A successful acceptance-only sync MUST make a subsequent status for the selected entry current.
- **FR-007**: Entries with unequal, absent, destination-only, ignored, unsupported, unsafe, or otherwise ineligible evidence MUST retain their existing sync outcome and MUST NOT receive accepted baseline evidence through this feature.
- **FR-008**: Existing behavior for unselected or broad tree scopes, synchronized entries, directional source or destination changes, conflicts, force selection, mapping declarations, selector spaces, and machine-readable result compatibility MUST remain unchanged.
- **FR-009**: Human preview and completion output for acceptance-only sync MUST distinguish baseline establishment from payload-copy actions.
- **FR-010**: Documentation MUST explain that ordinary selected sync can establish a baseline only for a complete equivalent managed pair and does not replace a separate conflict-resolution or baseline command.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In isolated file-mapping and tree-child scenarios, 100% of eligible equivalent unbaselined selected entries become current after one successful `grip sync` invocation without endpoint payload changes.
- **SC-002**: In dry-run scenarios for the same entries, 100% leave endpoint and accepted-state snapshots unchanged while reporting baseline establishment.
- **SC-003**: In isolated ineligible scenarios covering unequal, absent, ignored, destination-only, unsupported, and unsafe evidence, 100% publish no new accepted baseline through ordinary sync.
- **SC-004**: Existing synchronization, force-resolution, and selector regression suites retain their expected outcomes without behavior changes outside the acceptance-only case.

## Assumptions

- A complete equivalent source and destination pair is already the project’s accepted-evidence eligibility boundary.
- The existing selected-sync command and result forms are the appropriate interface for this narrowly scoped non-copy state transition.

## Implementation Traceability

| Requirement | Implementation and verification |
| --- | --- |
| FR-001, FR-002, FR-004, FR-006 | `src/lib.rs`, `src/mutation/plan.rs`, and `tests/sync_filesystem_integration.rs` prove exact file and tree-child acceptance without payload actions. |
| FR-003 | `src/mutation/execution.rs` rebuilds the selected sync plan under the existing mutation lock; the sync contention and failure integration suites retain revalidation coverage. |
| FR-005, FR-009 | `tests/sync_cli_contract.rs` and `tests/sync_filesystem_integration.rs` verify baseline-only preview and completion output plus JSON plan counts. |
| FR-007, FR-008 | Scope, collision, ignored, destination-only, missing-peer, and destination-link regressions retain no-acceptance or existing directional behavior. |
| FR-010 | `README.md`, `docs/product-definition.md`, and `contracts/sync-baseline-acceptance.md` document the exact complete-pair boundary and exclusions. |
| SC-001 through SC-004 | Focused sync suites and `mise run validate` verify selected acceptance, dry-run non-mutation, excluded-state preservation, and unchanged regression behavior. |
- This feature flows forward from prior baseline classification and synchronization behavior and does not alter historical feature artifacts.
