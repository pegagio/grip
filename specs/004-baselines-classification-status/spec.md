# Feature Specification: Baselines, Classification, and Status

**Feature Branch**: `004-baselines-classification-status`

**Created**: 2026-09-05

**Status**: Complete

**Input**: User description: "Specify roadmap entry 004, Baselines, Classification, and Status, using the roadmap and wiki-backed governing decisions and constraints."

## Clarifications

### Session 2026-09-05

- Q: Should `grip check` report an equivalent source/destination pair with no accepted baseline as requiring attention? → A: Yes. It exits `1` with `attention_required` until the equivalent pair is explicitly accepted as a baseline.
- Q: What should `grip baseline accept` do when every selected entry already equals its accepted baseline? → A: Succeed as a no-op, report that the baseline is already current, and publish no new state generation.
- Q: When a baseline exists, which comparisons should `grip diff` report for each entry? → A: Report source-to-baseline, destination-to-baseline, and current source-to-destination changed dimensions.
- Q: What should happen to accepted baseline evidence when its mapping is removed before Feature 008 adds explicit retirement? → A: Retain it as untracked pending-retirement evidence and report it through `status` and `check` until explicit retirement.
- Q: When `grip baseline accept PATH` accepts only part of the managed namespace, what should happen to accepted baselines outside that selected path? → A: Update only selected records and preserve all out-of-scope baseline and pending-retirement evidence unchanged.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Understand Current Synchronization State (Priority: P1)

As a local user, I can inspect all mappings or one selected path and receive a complete, deterministic classification of source, destination, and accepted-baseline evidence without changing payloads or accepted state.

**Why this priority**: Trustworthy classification is the read-only foundation for every later push, pull, sync, conflict-resolution, deletion, and retirement workflow.

**Independent Test**: Build isolated mappings covering every source/destination/baseline combination, run `grip status`, and verify the exact ordered classifications, summary counts, and zero filesystem or Grip-state mutations.

**Acceptance Scenarios**:

1. **Given** equivalent source and destination entries with an accepted equivalent baseline, **When** the user runs `grip status`, **Then** Grip reports the entries as synchronized and makes no changes.
2. **Given** only the source differs from the accepted baseline, **When** status completes, **Then** Grip reports a source-only change and the prospective source-to-destination direction.
3. **Given** both sides differ from the baseline in different ways, **When** status completes, **Then** Grip reports a conflict without selecting a winner.
4. **Given** entries spanning every supported classification, **When** status is repeated over unchanged evidence, **Then** both runs return equivalent records in deterministic order.
5. **Given** a mapping is removed after its entries have accepted baselines, **When** status covers all state, **Then** Grip reports those retained entries as untracked and pending retirement rather than deleting the evidence or treating it as corrupt.

---

### User Story 2 - Establish and Refresh Accepted Evidence (Priority: P2)

As a user, I can explicitly accept a baseline for managed entries whose source and destination already have equivalent supported state, allowing later inspections to distinguish one-sided drift without copying payloads.

**Why this priority**: Three-way classification cannot distinguish drift from initial state until the user has deliberately accepted equivalent paired evidence.

**Independent Test**: Create equivalent source and destination entries with no baseline, accept them in an isolated Grip home, then change each side independently and verify that status identifies the changing side while payloads remain untouched.

**Acceptance Scenarios**:

1. **Given** an eligible source entry and equivalent paired destination entry with no baseline, **When** the user runs `grip baseline accept`, **Then** Grip publishes their supported state as the accepted baseline without changing either payload.
2. **Given** source and destination have converged independently to the same supported state, **When** the user accepts the baseline, **Then** Grip refreshes the accepted evidence to that state.
3. **Given** absent, different, unsupported, unsafe, ignored, or pending-retirement evidence in the selected scope, **When** baseline acceptance is requested, **Then** Grip rejects the complete request before publishing any baseline change.
4. **Given** evidence changes between inspection and publication, **When** baseline acceptance revalidates it, **Then** Grip reports stale evidence and preserves the prior accepted state.
5. **Given** every selected entry already equals its accepted baseline, **When** the user accepts the baseline again, **Then** Grip succeeds, reports that the baseline is current, and publishes no new state generation or recovery evidence.
6. **Given** a selected entry or subtree is accepted while other baseline or pending-retirement records exist, **When** the updated state is published, **Then** only the selected baseline records change and every out-of-scope record remains unchanged.

---

### User Story 3 - Use Classification in Automation (Priority: P3)

As a user or script author, I can run `grip check` and consume stable machine-readable classifications and exit behavior that distinguish a clean scope, attention-worthy state, invalid input or configuration, corrupt state, and operational failure.

**Why this priority**: Automation needs to react to drift without confusing a successfully completed inspection with a runtime failure.

**Independent Test**: Run `check` with human and machine output across clean, drifted, conflicting, unsupported, invalid, corrupt, and failed fixtures and verify the documented result categories and exit codes.

**Acceptance Scenarios**:

1. **Given** a completely inspected scope containing only synchronized entries and non-attention informational records, **When** `grip check` completes, **Then** it exits `0` with a clean result.
2. **Given** a completely inspected scope containing drift, conflict, collision, deletion, unsupported evidence, or pending retirement, **When** check completes, **Then** it returns the full classification and exits `1` for attention required.
3. **Given** equivalent source and destination entries without an accepted baseline, **When** check completes, **Then** it reports the initial match and exits `1` until that evidence is explicitly accepted.
4. **Given** inspection cannot complete because input, configuration, state, or an operation is invalid, **When** check terminates, **Then** it uses the established error exit category rather than exit `1`.
5. **Given** unchanged evidence, **When** human and machine-readable output are requested separately, **Then** both forms express equivalent classifications while diagnostics remain separate.

---

### User Story 4 - Explain Differences Safely (Priority: P4)

As a user, I can run `grip diff` to see which supported content or metadata dimensions differ for each classified entry without exposing file contents or mutating anything.

**Why this priority**: Classification tells users what state exists; bounded difference evidence helps them decide what to do next without turning this feature into a mutation or content-merge tool.

**Independent Test**: Create content-only, permission-only, content-and-permission, kind-collision, deletion, and unsupported fixtures, then verify that diff reports only the applicable dimensions and safe fingerprint evidence.

**Acceptance Scenarios**:

1. **Given** regular files whose bytes differ but permission modes match, **When** diff completes, **Then** it identifies a content difference without printing file contents.
2. **Given** regular files whose permission modes differ but bytes match, **When** diff completes, **Then** it identifies a permission-mode difference.
3. **Given** an accepted baseline and safely inspected current entries, **When** diff completes, **Then** it reports the changed supported dimensions from source to baseline, destination to baseline, and current source to current destination.
4. **Given** an entry is missing, has an unsafe node-kind collision, or cannot be fingerprinted safely, **When** diff completes, **Then** it explains the available classification evidence without opening unsupported payloads or inventing a byte-level comparison.

### Edge Cases

- No accepted baseline exists for a new source entry, an equivalent initial pair, a differing initial pair, or a destination-only entry.
- An accepted baseline exists while one side, both sides, or a previously managed entry has disappeared.
- Both sides change to the same supported state, or content changes on one side while permission mode changes on the other.
- A newly ignored path has prior baseline evidence and must remain visible as pending retirement rather than disappearing from classification.
- A mapping is removed after one or more entries have accepted baseline evidence; those entries remain reportable as untracked pending retirement.
- An ignored path has no baseline evidence and remains outside the managed namespace.
- A selector names a deleted managed source, an ignored path, an unmanaged destination-only path, a path outside all mappings, or a path beginning with `-`.
- Registry or baseline state is missing, unreadable, corrupt, incompatible, replaced during inspection, or inconsistent with current mapping identity.
- A file changes while being fingerprinted, or relevant mapping, membership, metadata, or baseline evidence changes before baseline publication.
- A supported path is replaced by a symbolic link, hard-linked file, sparse file, special node, non-UTF-8 name, or nested mount boundary.
- Two entries have equal modification times but different supported state, or different modification times but otherwise equal supported state.
- Human output cannot be written after a complete inspection, or machine-readable output is interrupted.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST expose `grip status [PATH]`, `grip check [PATH]`, and `grip diff [PATH]` as intrinsically read-only operations.
- **FR-002**: Each inspection command MUST validate the complete accepted mapping registry before classifying the requested scope, even when one mapping, entry, or subtree is selected.
- **FR-003**: Omitting `PATH` MUST select every current accepted mapping plus every retained baseline-only identity. One optional selector MUST use source-path interpretation by default, and `--destination` MUST instead interpret the selector in destination-path space.
- **FR-004**: Selector resolution MUST support an exact file-mapping source, a complete tree-mapping source, or one managed entry or subtree beneath a tree source. `--` MUST terminate option parsing.
- **FR-005**: A selector for a deleted previously managed entry or an untracked pending-retirement entry MUST resolve through accepted mapping or baseline evidence. An ignored or unmanaged selector MUST be reported explicitly, and a selector outside both current mapping ownership and retained baseline identity MUST be an invalid request rather than an empty successful result.
- **FR-006**: Grip MUST compare current source state, current destination state, and the last accepted baseline independently for every entry in scope.
- **FR-007**: The first supported equality contract MUST include filesystem node kind; regular-file byte content; and regular-file permission mode. File content MUST be represented by a deterministic collision-resistant fingerprint rather than retained baseline content.
- **FR-008**: Regular-file length MAY be retained and displayed as supporting evidence but MUST NOT substitute for content equality. Modification time MUST be diagnostic evidence only and MUST NOT determine equality, direction, conflict, or accepted state in this feature.
- **FR-009**: Independent directory metadata, user and group ownership, extended attributes, access-control lists, BSD flags, symbolic-link objects, hard-link identity, sparse representation, and other metadata not named in FR-007 MUST remain outside the equality contract and MUST NOT be silently treated as synchronized.
- **FR-010**: For eligible ordinary directories, this feature MUST classify membership and presence by directory node kind while deferring independent directory metadata equality to Feature 009.
- **FR-011**: With no accepted baseline, Grip MUST distinguish source addition, initial match, initial collision, and destination-only unmanaged evidence.
- **FR-012**: With an accepted baseline, Grip MUST distinguish synchronized, source-only change, destination-only change, converged two-sided change, divergent conflict, source-side deletion, destination-side deletion, delete/change conflict, change/delete conflict, converged deletion, newly ignored pending retirement, and untracked pending retirement.
- **FR-013**: A difference in any supported equality dimension MUST count as a change. If opposing sides changed different supported dimensions from the baseline, the entry MUST be a conflict unless their complete supported states are equivalent.
- **FR-014**: A current source and destination that independently equal the same complete supported state MUST be classified as initial match when no baseline exists or converged two-sided change when both differ from an accepted baseline.
- **FR-015**: Destination-only entries without baseline evidence MUST remain unmanaged and non-blocking. Grip MUST NOT admit them to managed membership or apply source ignore policy to them.
- **FR-016**: A previously managed entry excluded by current source policy or detached by mapping removal MUST remain visible as pending retirement. Mapping removal MUST retain accepted baseline evidence, and `status` and `check` MUST report it as untracked pending retirement. This feature MUST NOT infer retirement, deletion, or baseline removal from an ignore-policy change or mapping removal.
- **FR-017**: Unsupported source entries and unsafe nodes at paired managed destination paths MUST remain blocking classification evidence. Unsupported destination-only entries MUST remain unmanaged and non-blocking.
- **FR-018**: Classification MUST reuse the established non-following filesystem and `.gripignore` boundaries. It MUST NOT follow or open symbolic links or unsupported nodes to obtain content or metadata evidence.
- **FR-019**: Every successful inspection MUST return a complete deterministic result for its selected scope, including per-classification counts, attention and blocking counts, and records ordered by canonical mapping source and source-relative raw path identity.
- **FR-020**: `grip status` MUST exit `0` whenever classification completes, including when results contain drift, conflicts, unsupported evidence, or blocking findings. Inspection failures MUST use the established error category.
- **FR-021**: `grip check` MUST exit `0` when classification completes with no attention-worthy entries and `1` when classification completes with any source addition, unaccepted initial match, initial collision, change, conflict, deletion, unsupported managed evidence, unsafe collision, or pending retirement. An initial match MUST remain attention-worthy until explicitly accepted as a baseline.
- **FR-022**: Exit `1` MUST mean only `attention_required` after a complete check. Invalid usage (`2`), invalid configuration (`10`), unsupported schema (`11`), corrupt state (`12`), and operational failure (`20`) MUST retain their established meanings. Contention on baseline state publication MUST activate the reserved `state_contention` result at exit `13`; registry contention and other operational failures MUST remain exit `20`.
- **FR-023**: `grip diff` MUST exit `0` whenever difference inspection completes, regardless of whether differences exist, and MUST use established error categories when it cannot complete.
- **FR-024**: Machine-readable results MUST use stable, versioned fields that distinguish operation, selected scope, completion status, clean or attention state, summary counts, ordered entry records, classification, supported fingerprints, changed dimensions, blocking reasons, and safe path identities.
- **FR-025**: Human output MUST communicate the same classifications, directions, changed dimensions, and blocking reasons as machine-readable output. Diagnostics MUST remain separate from both result forms.
- **FR-026**: When an accepted baseline and safely inspected current entries exist, difference output MUST report source-to-baseline, destination-to-baseline, and current source-to-destination changed supported dimensions. It MUST use supported dimensions and fingerprints without printing file contents and MUST NOT claim a comparison for absent or unsupported evidence that was not safely inspected.
- **FR-027**: Grip MUST expose `grip baseline accept [PATH]` as an explicit state-only operation using the same selector rules as inspection commands.
- **FR-028**: Baseline acceptance MUST require every selected eligible entry to have equivalent, complete, supported source and destination state. It MUST reject the whole request for additions with an absent destination, initial collisions, divergent state, deletions, unsupported evidence, unsafe collisions, ignored entries, newly ignored pending retirement, or untracked pending retirement.
- **FR-029**: Baseline acceptance MUST build a complete deterministic candidate, acquire only bounded coordination for Grip-owned state publication, reread the accepted state, and revalidate all mapping, membership, source, destination, and fingerprint evidence whose change could invalidate acceptance.
- **FR-030**: Baseline acceptance MUST publish one versioned, integrity-checked state generation only after successful revalidation and only when the accepted baseline would change. A scoped acceptance MUST change only selected baseline records and preserve every out-of-scope baseline and pending-retirement record unchanged. If every selected entry already equals its accepted baseline, the operation MUST succeed as a reported no-op without publishing a new generation or recovery evidence. Any publication MUST preserve recoverable prior state and MUST leave the prior generation authoritative on failure before replacement.
- **FR-031**: A baseline record MUST bind each managed entry to its mapping identity, lossless source-relative identity, node kind, supported fingerprint, and supported metadata evidence. It MUST NOT retain file contents.
- **FR-032**: Missing state with no prior accepted baseline MUST be treated as uninitialized, not corrupt. Retained baseline evidence whose mapping was removed MUST be treated as untracked pending retirement, not corrupt. Malformed, incompatible, unsafe, or otherwise internally inconsistent accepted state MUST block classification or acceptance with its precise established error category rather than being guessed or silently rebuilt.
- **FR-033**: `status`, `check`, and `diff` MUST NOT create or modify registry data, accepted state, baselines, locks, recovery evidence, source or destination payloads, ignore policy, or version-control state. Failed baseline acceptance MUST NOT change accepted state or payloads.
- **FR-034**: This feature MUST NOT copy, replace, delete, retire, or resolve payload entries; select a conflict winner; accept unequal sides as equivalent; introduce multiple selectors; add automatic state recovery; or promise metadata outside FR-007.
- **FR-035**: Classification and baseline publication behavior MUST be covered by automated tests using isolated temporary roots, including every classification, content-only and metadata-only changes, selector boundaries, unsupported nodes, stale evidence, state corruption, failed publication, and non-mutation guarantees.
- **FR-036**: A documented representative workload of 10,000 eligible entries MUST measure full classification and justify any cache, persistent index, parallel traversal, or new concurrency mechanism before such complexity is introduced.

### Key Entities

- **Accepted Baseline**: Versioned, integrity-checked machine-owned evidence describing the last explicitly accepted complete supported state of managed entries without retaining their file contents.
- **Entry Fingerprint**: Deterministic evidence for one side of a managed entry, including node kind and the supported equality dimensions applicable to that kind.
- **Classification Record**: The three-way conclusion for one source-relative managed identity, including current evidence, baseline presence, classification, prospective direction, changed dimensions, and attention or blocking status.
- **Classification Result**: A complete, deterministic, non-persisted set of records and summary counts for one selected scope.
- **Baseline Acceptance Request**: An explicit state-only request to accept equivalent complete paired evidence for all entries in a selected scope.
- **Pending Retirement Entry**: An entry with accepted baseline evidence that current source policy no longer admits or whose mapping has been removed, requiring a later explicit retirement decision.
- **Destination-Only Entry**: Current destination evidence with no eligible source counterpart and no accepted baseline; it remains outside Grip ownership.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A fixed conformance matrix covering every classification in FR-011 and FR-012 produces the expected classification, direction, attention status, and blocking status in 100% of cases.
- **SC-002**: Across content-only, permission-only, combined, timestamp-only, deletion, ignored, unsupported, and unsafe-collision fixtures, Grip reports every changed supported dimension correctly and never uses modification time alone to classify a change.
- **SC-003**: In 100% of tests for status, check, diff, invalid input, invalid configuration, corrupt state, and operational failure, the observed exit code matches FR-020 through FR-023 and machine-readable output uses the corresponding stable result category.
- **SC-004**: Baseline acceptance succeeds for 100% of equivalent complete paired fixtures, preserves every out-of-scope record in 100% of scoped-acceptance fixtures, publishes no new generation for 100% of already-current fixtures, and rejects 100% of unequal, incomplete, unsupported, stale, or pending-retirement fixtures without changing payloads or publishing a partial candidate.
- **SC-005**: Repeating any inspection 100 times over unchanged evidence produces equivalent ordered classification records and summary counts in every run.
- **SC-006**: For a documented representative scope containing 10,000 eligible entries, at least 95 of 100 full status runs complete within two seconds on a documented representative local workstation without requiring a persistent cache or background service.
- **SC-007**: Automated mutation audits confirm zero registry, baseline, recovery, source, destination, ignore-policy, or version-control changes for every status, check, and diff success or failure fixture.
- **SC-008**: A user can identify the state, changed supported dimensions, prospective direction, and required attention for every entry in a representative mixed mapping from one status or diff result without consulting internal state files.

## Assumptions

- Features 001 through 003 are verified and their Grip-home, registry, mapping identity, discovery, `.gripignore`, safe-path, unsupported-node, result-envelope, and deterministic-ordering contracts remain authoritative.
- The roadmap's `in-progress` state records the user's explicit start of Feature 004; this checkout is detached and no before-specify branch hook is registered, so this workflow does not create or switch a Git branch.
- The first supported metadata set deliberately includes regular-file permission mode while treating modification time as diagnostic only. Ownership, extended attributes, access-control lists, BSD flags, independent directory metadata, and other filesystem metadata remain Feature 009 work.
- Explicit baseline acceptance is limited to already equivalent source and destination state. Initial copying belongs to Feature 005, reverse copying to Feature 006, conflict resolution to Feature 007, and deletion or retirement to Feature 008.
- Exit `1` is available for a completed `check` requiring attention because established command failures begin at exit `2` or `10`; this feature does not renumber existing error categories.
- Machine-owned state publication reuses the established versioning, integrity, owner-only storage, bounded-locking, atomic-publication, and recovery principles without making its serialization layout part of this stakeholder specification.
