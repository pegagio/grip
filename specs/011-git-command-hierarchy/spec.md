# Feature Specification: Git-Inspired Command Hierarchy

**Feature Branch**: `011-git-command-hierarchy`

**Created**: 2026-09-09

**Status**: Complete

**Input**: User description: "Adopt the approved Git-inspired Grip command hierarchy."

## Clarifications

### Session 2026-09-09

- Q: When one endpoint is absent and its managed peer still exists, how should ordinary `push`, `pull`, or `sync` behave? → A: Block one-sided absence during ordinary operations; require `push -f <PATH>` or `pull -f <PATH>` to propagate it.
- Q: When `grip add` registers endpoints that are not already equivalent, what should happen next? → A: Record the mapping without a baseline; `status` reports the initial state and a later `push` or `pull` establishes it.
- Q: When an operator removes a mapping, how should Grip handle its internal baseline evidence? → A: Discard the mapping’s baseline evidence; re-adding it starts as a new mapping.
- Q: Must `grip add` allow an existing source to map to a destination that does not yet exist? → A: Require at least one existing endpoint; infer the mapping kind from an existing endpoint and allow the other endpoint to be absent.
- Q: How should `grip sync` handle a newly added mapping whose endpoints differ and have no baseline? → A: Block the mapping and require a directional `push` or `pull` to establish its first baseline.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect and synchronize a mapped project (Priority: P1)

An operator working in a Grip project can see whether mapped content is synchronized, inspect exact differences, and safely move unambiguous changes in either direction with a small, memorable set of commands.

**Why this priority**: The core value of Grip is reliable local deployment and synchronization without requiring the operator to understand internal baseline or recovery concepts.

**Independent Test**: In a project with file and tree mappings, an operator can use `status`, `diff`, `push`, `pull`, and `sync` to inspect and synchronize each supported change classification without invoking a removed command.

**Acceptance Scenarios**:

1. **Given** an established mapping with a source-only change, **When** the operator runs `grip sync`, **Then** Grip copies the source state to the destination and records the resulting synchronized state.
2. **Given** an established mapping with a destination-only change, **When** the operator runs `grip sync`, **Then** Grip copies the destination state to the source and records the resulting synchronized state.
3. **Given** a valid inspection that finds attention-required entries, **When** the operator runs `grip status -e`, **Then** Grip reports the same status information as `grip status` and returns a nonzero attention result.

---

### User Story 2 - Declare and manage mappings with flat commands (Priority: P1)

An operator can add, list, and remove mapping declarations without navigating a nested command family or learning a separate singular-display command.

**Why this priority**: Mapping setup is the entry point to every Grip workflow, and the command surface must make configuration actions unmistakable.

**Independent Test**: An operator can add a file mapping and a tree mapping, retrieve all mappings or one selected mapping with `list`, and remove a mapping while confirming that neither endpoint payload changes.

**Acceptance Scenarios**:

1. **Given** valid source and destination endpoints, **When** the operator runs `grip add <SOURCE> <DESTINATION>`, **Then** Grip creates a mapping whose file-or-directory kind matches the validated source endpoint.
2. **Given** multiple mappings, **When** the operator runs `grip list <SOURCE>`, **Then** Grip displays only the mapping identified by that source path.
3. **Given** an existing mapping, **When** the operator runs `grip remove <SOURCE>`, **Then** Grip removes the declaration and its internal baseline evidence without changing source or destination payloads.
4. **Given** newly mapped endpoints that differ, **When** the operator runs `grip add <SOURCE> <DESTINATION>`, **Then** Grip records no baseline and changes neither endpoint until the operator chooses a directional synchronization.
5. **Given** exactly one existing endpoint, **When** the operator runs `grip add <SOURCE> <DESTINATION>`, **Then** Grip infers the mapping kind from that endpoint, records the mapping without creating its absent peer, and a later directional synchronization can create that peer.

---

### User Story 3 - Resolve a conflict through an explicit direction (Priority: P2)

An operator can deliberately make source or destination state win a single conflict without a separate resolution command or a hidden merge.

**Why this priority**: A directional force action is familiar to users of Git while preserving a clear, narrow acknowledgement of destructive intent.

**Independent Test**: For a divergent managed entry, an operator can preview and execute `push -f` or `pull -f` against the exact entry and verify that the chosen complete state becomes the state at both endpoints.

**Acceptance Scenarios**:

1. **Given** one divergent managed entry, **When** the operator runs `grip push -f <PATH>`, **Then** Grip makes the complete source state the winner at the destination and updates its synchronization baseline.
2. **Given** one divergent managed entry, **When** the operator runs `grip pull -f <PATH>`, **Then** Grip makes the complete destination state the winner at the source and updates its synchronization baseline.
3. **Given** a force request without an exact single-entry selection, **When** the operator runs `push -f` or `pull -f`, **Then** Grip rejects the request without changing either endpoint.

### Edge Cases

- A one-sided absent entry blocks ordinary `push`, `pull`, and `sync`. A `push -f` or `pull -f` winner may be absent; Grip applies that complete absence only to the exact selected peer and does not infer a winner for an unselected entry.
- A newly added mapping whose endpoints differ has no baseline until a successful directional synchronization establishes one.
- An absent source or destination is valid when adding a mapping if the other endpoint exists and can establish the mapping kind. A mapping with both endpoints absent is invalid.
- A newly added mapping with two differing endpoints blocks `sync` until an explicit `push` or `pull` establishes its first baseline.
- Re-adding a removed mapping never reuses its former baseline evidence.
- A normal `push`, `pull`, or `sync` that encounters a conflict or unsafe condition makes no partial mutation within the selected operation.
- A selected path is interpreted in source space by default and in destination space only when `-d` or `--destination` is supplied; this changes selection, never synchronization direction.
- An entry newly excluded by `.gripignore` stops participating in managed synchronization. If an exclusion is later removed, the entry is treated as newly discovered rather than silently regaining an obsolete baseline.
- Configuration or internal baseline corruption is always reported as an error. It is not converted into an ordinary attention result by `status -e`.
- A legacy command, option, explicit `human` output format, compatibility alias, or recovery command is rejected rather than silently redirected to new behavior.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST expose exactly these public commands: `init`, `version`, `add`, `list`, `remove`, `status`, `diff`, `push`, `pull`, and `sync`.
- **FR-002**: Grip MUST support `-p` and `--project <PATH>` for every project-dependent command, selecting an existing Grip project. `init` and `version` MUST reject this option.
- **FR-003**: `grip init [PATH]` MUST initialize the supplied directory or the current directory when omitted, and `grip version` MUST print the Grip version without requiring project selection.
- **FR-004**: Grip MUST use human-readable text by default and support `-o` and `--output json` as the only explicit output-format selection in this feature. An explicit `human` format value MUST not be accepted.
- **FR-005**: Grip MUST support repeatable `-v` and `--verbose` for diagnostic detail without mixing diagnostics into the selected standard output format.
- **FR-006**: `grip add <SOURCE> <DESTINATION>` MUST require and validate at least one existing endpoint under Grip's ownership and path rules, infer whether it is a file or directory mapping from an existing endpoint, and permit its peer to be absent. It MUST reject a mapping with both endpoints absent. It MUST not copy or create either endpoint; if either endpoint is absent or the endpoints differ, Grip MUST record no baseline until a later directional synchronization succeeds.
- **FR-007**: `grip list [SOURCE]` MUST list all mappings when `SOURCE` is omitted and exactly the selected mapping when it is supplied. Grip MUST not provide a separate `show` command.
- **FR-008**: `grip remove <SOURCE>` MUST remove the selected mapping declaration and its internal baseline evidence without modifying either endpoint payload. A later re-addition of the same mapping MUST be treated as new.
- **FR-009**: `.gripignore` MUST remain the direct, user-edited policy for excluding entries within tree mappings. Grip MUST not provide a command for ignoring, retiring, or separately untracking individual entries.
- **FR-010**: `grip status [PATH]` MUST validate Grip-owned configuration and synchronization state before reporting the selected scope. Valid attention findings return the ordinary status result; `-e` and `--exit-code` MUST instead return a nonzero attention result without changing the reported findings.
- **FR-011**: `grip diff [PATH]` MUST report detailed source and destination differences for the selected scope without changing mappings, endpoint payloads, or baseline state.
- **FR-012**: `-d` and `--destination` on `status`, `diff`, `push`, `pull`, and `sync` MUST interpret the supplied selector in destination-path space. They MUST not reverse an operation's source-to-destination or destination-to-source direction.
- **FR-013**: `push`, `pull`, and `sync` MUST mutate by default, and `-n` and `--dry-run` MUST render the same complete operation plan without changing endpoints, mapping declarations, or synchronization baselines.
- **FR-014**: Normal `push` and `pull` MUST apply only eligible non-absent changes in their declared direction and MUST block on divergent, one-sided absent, or otherwise unsafe selected entries before mutation.
- **FR-015**: `sync` MUST apply every unambiguous non-absent source-side change as a push and every unambiguous non-absent destination-side change as a pull within the selected scope. It MUST block one-sided absence, a newly added mapping with differing endpoints and no baseline, and every other conflict. It MUST not select a winner for a conflict.
- **FR-016**: `-f` and `--force` MUST be available only on `push` and `pull`. A forced action MUST require a selector that resolves to exactly one managed entry and MUST use the source state for `push` or destination state for `pull` as the complete winner, including absence.
- **FR-017**: `add` MUST establish an internal synchronization baseline only when the newly mapped endpoints are already equivalent. Successful `push`, `pull`, and `sync` operations MUST establish or update the baseline when their resulting endpoint state is eligible. Grip MUST not expose a separate baseline-acceptance command.
- **FR-018**: Grip MUST not expose `mapping`, `validate`, `fsck`, `check`, `show`, `resolve`, `delete`, `retire`, `accept`, recovery, or per-entry tracking-removal commands, nor preserve them as command aliases.
- **FR-019**: Grip MUST not provide a user-facing history, recovery, restore, or recovery-evidence cleanup interface. Operators remain responsible for history and recovery through Git or another chosen system.
- **FR-020**: Grip MUST provide command-specific help that documents each command's purpose, positional arguments, and supported options, including the exact force-selection restriction.

### Key Entities

- **Mapping declaration**: The portable association between a project-relative source endpoint and a user-relative destination endpoint, with a validated file or directory kind.
- **Synchronization baseline**: Internal evidence of the last accepted equivalent endpoint state, used to classify later changes; it is not a user-managed history or recovery record.
- **Selection**: The one mapping, entry, or mapped subtree addressed by an optional path argument in source space by default or destination space with `-d`.
- **Complete winner**: The selected source or destination state, including absence, explicitly chosen by a forced directional operation for one managed entry.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every supported user workflow in this specification is reachable through one of the ten top-level commands, with no nested command required.
- **SC-002**: 100% of documented command and option combinations produce either the specified result or a clear invalid-usage error; no retired command or alias silently performs work.
- **SC-003**: In representative source-only, destination-only, synchronized, and divergent-entry scenarios, `sync` selects the same unambiguous direction as separate `push` or `pull` would, and changes zero entries when any selected conflict remains unresolved.
- **SC-004**: 100% of forced directional operations reject omitted, mapping-wide, tree-wide, ambiguous, or multi-entry selections before mutating endpoints.
- **SC-005**: An operator can identify whether a valid project is synchronized, needs attention, is conflicted, or has invalid Grip metadata from one `status` invocation and its exit result.
- **SC-006**: Human output requires no output option, while every machine-readable invocation uses `-o json` or `--output json` and produces no human-only framing on standard output.

## Assumptions

- This is a pre-release interface replacement. Existing public commands and options may be removed without compatibility aliases or migration behavior.
- Mapping sources remain project-relative, destinations remain user-relative, and file-versus-directory mapping kind is determined from either existing endpoint when a mapping is added.
- Git or another operator-selected system is responsible for historical recovery before an operator uses a forced directional deployment action.
- Temporary staging necessary to publish a single operation safely is not a user-facing history or recovery facility.
- The feature changes public behavior and its documentation, but does not add automatic content merging, a new output format beyond JSON, or a new recovery mechanism.
