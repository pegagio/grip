# Feature Specification: Source-Authoritative Mapping Addition

**Feature Branch**: `018-source-authoritative-add`

**Created**: 2026-09-12

**Status**: Draft

**Input**: User description: "`grip add` remains non-mutating. When its source differs from its destination, source is authoritative: record the destination as the initial baseline reference so `grip status` offers a push, not a pull."

## Clarifications

### Session 2026-09-12

- Q: For a newly added tree member that exists only at the source, should Grip retain its existing source-addition behavior or record initial source-authoritative state for it? → A: Retain existing source-addition behavior.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Add a source that is ready to push (Priority: P1)

An operator adds an existing source-to-destination mapping whose endpoints differ and can immediately see that the source should be pushed, without Grip copying either payload during `add`.

**Why this priority**: The command’s source-to-destination declaration must establish an unsurprising first synchronization direction rather than present the destination as authoritative.

**Independent Test**: In an isolated project, add a file mapping whose source and destination differ; verify that `add` changes neither payload and that `status`, `push --dry-run`, and `sync --dry-run` identify a source-to-destination action while `pull --dry-run` does not overwrite the source.

**Acceptance Scenarios**:

1. **Given** an existing compatible source and destination with different supported state, **When** an operator runs `grip add SOURCE DESTINATION`, **Then** Grip declares the mapping, changes neither endpoint payload, and records enough initial state that `grip status` lists `SOURCE -> DESTINATION` under `Changes to push`.
2. **Given** such a newly added differing file mapping, **When** an operator previews `grip push` or `grip sync`, **Then** the preview proposes copying source to destination; **When** the operator previews `grip pull`, **Then** it does not propose copying destination to source.
3. **Given** such a mapping, **When** the operator completes `grip push`, **Then** the destination matches the source and a subsequent status is current.

---

### User Story 2 - Preserve source authority for managed trees (Priority: P2)

An operator adds a tree mapping with source-managed members that differ from existing destination counterparts and receives one safe source-to-destination plan for those members, while destination-only content remains unmanaged.

**Why this priority**: Source-defined tree membership must not have a different initial-authority rule from a file mapping.

**Independent Test**: In an isolated tree fixture with matching, differing, source-only, and destination-only entries, add the tree mapping and verify the subsequent status and dry-run plan for every managed source member.

**Acceptance Scenarios**:

1. **Given** a tree mapping with admitted source members whose destination counterparts differ, **When** the operator adds the mapping, **Then** each differing managed member is offered as a push and neither payload tree is changed by `add`.
2. **Given** a source member without a destination counterpart, **When** the tree mapping is added, **Then** it remains eligible for the existing source-to-destination addition behavior.
3. **Given** destination-only content outside source-defined membership, **When** the tree mapping is added, **Then** Grip neither baselines, reports as a source action, nor changes that content.

---

### User Story 3 - Retain conservative behavior outside the new initial state (Priority: P3)

An operator retains existing safety behavior once either endpoint changes after mapping addition, and existing equivalent additions remain current.

**Why this priority**: Source authority at mapping creation must not turn later two-sided edits into a silent overwrite.

**Independent Test**: Add equivalent and differing mappings, alter one or both endpoints after addition, and verify status classification, blocked conflicts, JSON results, and payload effects remain safe.

**Acceptance Scenarios**:

1. **Given** equivalent compatible endpoints, **When** the operator adds their mapping, **Then** Grip preserves the current equivalent-baseline behavior and status is current.
2. **Given** a differing mapping added under this feature and then a changed destination before the source is pushed, **When** the operator runs status, **Then** Grip reports a conflict rather than silently selecting a winner.
3. **Given** source or destination inspection, state publication, or registry publication cannot complete safely during addition, **When** `grip add` returns, **Then** it changes neither payload and exposes no usable partially initialized mapping state.
4. **Given** Grip begins adding an unequal source/destination pair, **When** it cannot create and verify its publication fence, **Then** it reports the failure without changing the mapping descriptor or managed state.
5. **Given** Grip has created a publication fence for an unequal source/destination pair, **When** descriptor or state publication does not complete and verify, **Then** Grip keeps that exact mapping fenced; a later `grip status` identifies it as incomplete rather than treating it as an ordinary collision or silently choosing a winner, while unrelated mappings remain available. Retrying `grip add` for that mapping revalidates the fenced transition and either completes it or restores the prior descriptor before clearing the fence.

### Edge Cases

- A source or destination that is absent, unsupported, unsafe, or has an incompatible node kind retains its existing validation or classification behavior; this feature does not treat an unusable endpoint as a source-authoritative differing pair.
- Existing destination metadata differences are included in the initial comparison under the same supported-state contract as later synchronization.
- Re-adding a removed mapping remains new membership and applies this feature’s initial-state rule; it does not revive historical baseline evidence.
- `.gripignore` continues to determine admitted tree members before initial source-authority state is recorded.
- A descriptor/state transition whose compensation cannot be confirmed retains its already-created incomplete-add publication fence and blocks ordinary and forced synchronization for that mapping; it is neither an initial collision nor a request to choose a winner. Commands that select only unfenced mappings remain available.
- The feature does not change destination-only ownership, selection syntax, force semantics, `diff`, JSON schemas, or normal behavior after a successful source-to-destination synchronization.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: When adding a compatible file mapping with both endpoints present but unequal, Grip MUST keep both endpoint payloads unchanged and initialize its managed state so the current source is classified as a source-to-destination change relative to the current destination.
- **FR-002**: After the addition described by FR-001, default human `grip status` MUST list the mapping under `Changes to push` using `SOURCE -> DESTINATION`; push and sync previews MUST select source-to-destination work, and pull preview MUST not select destination-to-source work for that unchanged initial state.
- **FR-003**: The first successful ordinary push of a mapping initialized by FR-001 MUST synchronize destination to source and leave the mapping current under the existing accepted-state rules.
- **FR-004**: Adding a compatible mapping whose source and destination already have equivalent supported state MUST retain the existing equivalent-baseline and current-status behavior.
- **FR-005**: For a tree mapping, Grip MUST apply FR-001 and FR-002 to every source-defined, non-ignored managed member with an existing unequal destination counterpart; source-only managed members retain their existing addition behavior, and destination-only content remains unmanaged.
- **FR-006**: `grip add` MUST remain non-mutating with respect to source and destination payloads, including file content, supported metadata, and filesystem node presence. It MAY publish only the mapping declaration and Grip-owned state needed to establish the initial classification.
- **FR-007**: For a newly added unequal source/destination pair, Grip MUST create and verify a durable, mapping-scoped incomplete-add fence before publishing either the mapping descriptor or matching destination comparison state. The fence MUST bind the exact mapping identity and the expected prior and candidate descriptor and state digests. If fence creation or verification fails, Grip MUST leave the descriptor and state unchanged. After the fence exists, Grip MUST inspect the destination endpoint and publish the descriptor and matching destination comparison state, verify that both match the fenced candidate, and only then clear the fence. If publication or verification fails, Grip MUST keep the fence and fail closed for that mapping until a retry of the same `grip add` either completes the verified transition or restores the prior descriptor and clears the fence.
- **FR-008**: After source-authoritative initialization, later drift on both endpoints MUST retain existing conflict behavior; the feature MUST NOT silently overwrite either side or select a winner without the existing explicit force authority.
- **FR-009**: This feature MUST preserve existing command syntax, destination-only ownership, selector behavior, `grip diff` detail, JSON schemas and values for unaffected operations, force semantics, filesystem safety validation, and error categories. The incomplete-add fence uses an existing configuration or state-integrity category rather than creating a new public error category. A command that explicitly selects only unfenced mappings MAY proceed; a command whose selected set includes a fenced mapping MUST report that mapping as blocked.
- **FR-010**: User-facing documentation MUST explain that `grip add` leaves payloads unchanged and that a differing destination becomes the initial comparison reference, causing the source to appear as a pending push.

### Key Entities

- **Source-authoritative initial state**: Managed state established at mapping addition for an unequal source/destination pair. It records the destination as the comparison reference so the source is immediately eligible to be pushed without changing either payload.
- **Initial comparison reference**: The destination’s complete supported state at successful mapping addition, used only to establish the initial source-to-destination classification.
- **Managed source member**: A file or directory entry admitted by a source-defined file or tree mapping and its ignore policy.
- **Incomplete-add publication fence**: Grip-owned, bounded coordination state, created before publication, that records the exact mapping identity plus prior and candidate descriptor/state digests. It blocks synchronization decisions only for that mapping until a verified retry completes the transition or restores the prior descriptor and clears the fence.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of isolated unequal-file-add fixtures, `add` changes zero source or destination payloads and the next status reports exactly one source-to-destination action.
- **SC-002**: In 100% of isolated unequal-file-add fixtures, push and sync previews select source-to-destination work and pull preview selects zero destination-to-source actions before either payload changes.
- **SC-003**: In a representative tree fixture containing matching, unequal, source-only, ignored, and destination-only entries, 100% of managed unequal members are offered as pushes and zero destination-only or ignored entries are selected or changed.
- **SC-004**: In 100% of failure-injection fixtures, fence creation failure leaves the descriptor and state unchanged, and a fenced add either becomes a fully coherent descriptor/state pair or remains blocked only for its mapping. Tests cover fence creation, registry/state publication, verification, fence clearing, retry completion, and verified restoration; no subsequent command observes a usable declaration without its required initial state.
- **SC-005**: Existing equivalent-addition, later-conflict, `diff`, JSON, force, selection, and filesystem-safety regression tests retain their expected behavior, including complete supported metadata and node presence across `add`.

## Assumptions

- “Non-mutating” refers to source and destination payloads; publishing the mapping declaration and Grip-owned state is permitted when it completes safely.
- Source authority is expressed by making the destination the initial comparison reference, not by copying source content or metadata during `add`.
- The incomplete-add publication fence is short-lived coordination state, not mapping history, a payload copy, or a user-facing recovery system. It is scoped by the exact mapping identity and expected descriptor/state digests; retrying the same `grip add` is the only needed operator interaction.
- This feature supersedes the unequal-endpoint baseline behavior described by Feature 004 only for newly added, source-defined managed members. Historical specifications remain unchanged records.
- The source-defined membership and ignore-policy rules established by prior features remain authoritative for tree mappings.
