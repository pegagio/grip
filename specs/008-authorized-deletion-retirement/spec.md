# Feature Specification: Authorized Deletion and Retirement

**Feature Branch**: `008-authorized-deletion-retirement`

**Created**: 2026-09-07

**Status**: Complete

**Input**: User description: "Specify roadmap entry 008, Authorized Deletion and Retirement, using the roadmap and wiki-backed governing decisions and constraints."

## Clarifications

### Session 2026-09-07

- Q: What should Grip do when an authorized directory deletion would also remove unmanaged descendants? → A: Block the complete deletion before mutation.
- Q: After recovery content is explicitly removed, what historical record should Grip retain? → A: Keep immutable metadata marked with a cleaned tombstone.
- Q: How should users explicitly select every pending-retirement record for one operation? → A: Require either a path selector or an explicit `--all` flag.
- Q: When the current registry or baseline is missing or corrupt, when may Grip restore a saved copy? → A: Restore an exact verified copy after complete compatibility checks; the broken target need not validate.
- Q: When a pending-retirement entry still has differing surviving copies, should Grip allow its baseline to be discarded? → A: Require `--force` after reporting the differences.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Preview and Authorize a Directional Deletion (Priority: P1)

As a local user, I can inspect a previously managed entry missing from one side, preview the consequence of accepting that absence, and explicitly authorize removal of the remaining peer without ordinary synchronization inferring destructive intent.

**Why this priority**: Directional deletion is the core roadmap outcome and carries the greatest user-data risk.

**Independent Test**: Create isolated accepted entries with source-side and destination-side deletions; preview and execute each direction; verify explicit authorization, deterministic plans, recovery preservation, peer removal, baseline retirement, and dry-run non-mutation.

**Acceptance Scenarios**:

1. **Given** an accepted entry is absent only at the source and unchanged at the destination, **When** the user previews deletion with the source absence selected as authoritative, **Then** Grip reports removal of the destination and retirement of the accepted record without changing payloads or Grip-owned state.
2. **Given** the same unchanged evidence, **When** the user explicitly executes that deletion, **Then** Grip preserves the destination state for recovery, removes it, verifies both sides are absent, and retires the accepted record.
3. **Given** an accepted entry is absent only at the destination, **When** the user explicitly accepts destination absence, **Then** Grip performs the symmetric source removal under the same recovery and verification guarantees.
4. **Given** a deletion classification exists in scope, **When** the user runs ordinary push, pull, or sync without the deletion command, **Then** deletion remains a blocker and no payload mutation begins.
5. **Given** an authorized managed-directory deletion would also remove unmanaged descendants, **When** Grip completes preflight, **Then** it blocks the complete deletion before mutation and identifies the unmanaged descendants that must be handled independently.

---

### User Story 2 - Retire No-Longer-Managed Entries Deliberately (Priority: P2)

As a local user, I can deliberately retire accepted evidence for entries that became ignored, untracked, or absent on both sides without silently deleting either existing payload copy.

**Why this priority**: Membership policy changes must not be mistaken for permission to delete user files or discard synchronization history.

**Independent Test**: Create newly ignored, untracked, and converged-deletion records; preview and execute retirement over exact entries and selected subtrees; verify only eligible baseline records are removed and all payloads remain unchanged.

**Acceptance Scenarios**:

1. **Given** a previously accepted tree entry becomes newly ignored while one or both payload copies remain, **When** the user retires it, **Then** Grip removes its accepted membership evidence while leaving every payload copy unchanged.
2. **Given** a mapping was removed and its accepted entries remain as untracked pending-retirement evidence, **When** the user retires the retained scope, **Then** Grip removes only the selected accepted records without recreating the mapping or mutating payloads.
3. **Given** both sides of an accepted entry are absent, **When** the user retires the converged deletion, **Then** Grip removes the accepted record without creating payload recovery data.
4. **Given** a selected entry is still actively managed and is not pending retirement or converged deletion, **When** retirement is requested, **Then** Grip rejects it before changing state.
5. **Given** pending-retirement records exist, **When** the user runs bare `grip retire`, **Then** Grip rejects the request and requires either an explicit path selector or `--all`.
6. **Given** a pending-retirement entry has differing surviving copies, **When** retirement is requested without `--force`, **Then** Grip reports the differences and preserves the baseline; **when** the user repeats the reviewed request with `--force`, **Then** Grip may retire the selected record without changing either payload copy.

---

### User Story 3 - Inspect and Restore Recovery Evidence (Priority: P3)

As a local user, I can list and inspect retained recovery evidence, preview a safe restoration, and restore an exact preserved state when current evidence still matches the recovery record's assumptions.

**Why this priority**: Preserving backups is useful only when users can understand and safely act on them after a deletion, replacement, or partial failure.

**Independent Test**: Produce payload and Grip-state recovery records through successful and failed operations; list and inspect them; restore each supported kind into safe and drifted targets; verify exact restoration, collision blocking, truthful authority reporting, and no automatic baseline acceptance.

**Acceptance Scenarios**:

1. **Given** retained recovery entries from one or more operations, **When** the user lists or inspects recovery evidence, **Then** Grip identifies its kind, origin, operation, managed identity, creation time, verification status, current availability, and whether restoration is presently safe without changing anything.
2. **Given** a verified payload recovery entry and an unchanged or absent original target, **When** the user previews restoration, **Then** Grip reports the exact target and expected effect without modifying payloads, baselines, or recovery evidence.
3. **Given** the same evidence, **When** restoration executes successfully, **Then** Grip stages and verifies the preserved state at its original side while retaining the recovery entry and leaving baseline acceptance to a later explicit synchronization decision.
4. **Given** the target, mapping, membership, accepted state, or recovery artifact no longer matches the recovery record's assumptions, **When** restoration is requested, **Then** Grip blocks before mutation and reports the changed evidence rather than overwriting current data.
5. **Given** the current registry or baseline is missing or corrupt and an exact integrity-verified recovery document exists, **When** complete live ownership, path, payload, and remaining-authority checks prove the saved document compatible, **Then** Grip may restore it without requiring the broken target to validate first.

---

### User Story 4 - Clean Recovery Material Explicitly (Priority: P4)

As a local user, I can preview and explicitly remove selected recovery material after deciding it is no longer needed, without an automatic retention policy deleting evidence behind my back.

**Why this priority**: Recovery data consumes space and may contain sensitive prior content, but cleanup must remain deliberate and bounded.

**Independent Test**: Create recovery entries of every supported kind, preview and execute cleanup by exact recovery reference, and verify selection, authorization, immutable survivors, partial-failure reporting, and non-interference with accepted registry or baseline state.

**Acceptance Scenarios**:

1. **Given** a verified recovery entry, **When** cleanup is previewed, **Then** Grip reports the exact material and byte count that would be removed without changing it.
2. **Given** an exact recovery reference and explicit cleanup confirmation, **When** cleanup succeeds, **Then** only that entry's recoverable bytes are removed, its immutable metadata records a cleaned tombstone, and accepted payload, registry, and baseline state remain unchanged.
3. **Given** cleanup lacks confirmation, uses an ambiguous selector, or includes recovery evidence required by an active mutation, **When** execution is requested, **Then** Grip rejects the request before removing anything.
4. **Given** removal fails after some selected recovery entries were removed, **When** the command ends, **Then** Grip identifies removed, failed, and unattempted entries without claiming atomic cleanup.

### Edge Cases

- A deletion target is absent on one side but the remaining side changed from its accepted baseline.
- Both sides are absent, both sides still exist, or the entry has no accepted baseline.
- A directory deletion contains mixed classifications, unsupported descendants, mount boundaries, or unmanaged descendants; each condition blocks the complete deletion before mutation.
- A selected parent and child would produce duplicate deletion or retirement actions.
- A bare retirement command omits both a path selector and `--all` and therefore selects nothing.
- A pending-retirement entry has surviving source and destination copies that differ from each other or from the accepted baseline.
- An entry becomes ignored or a mapping is removed after planning but before state publication.
- A deletion target is recreated, replaced by another node kind, made inaccessible, or substituted through an ancestor between inspection and removal.
- Recovery preservation succeeds but deletion, verification, state publication, or final result delivery fails.
- A recovery artifact is missing, corrupt, unverifiable, already cleaned, or belongs to a different operation or managed identity.
- A restore target contains new user data, differs from the post-operation evidence, or is no longer within a valid mapping.
- Registry or baseline state is missing, corrupt, unreadable, internally inconsistent, or newer than the candidate recovery generation.
- Cleanup selects payload, registry, baseline, and operation evidence from different operations.
- Multiple Grip processes attempt deletion, retirement, restore, cleanup, or another mutation concurrently.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST expose `grip delete (--source|--destination) [-n|--dry-run] [--] PATH` for one exact established managed entry or one component-boundary managed subtree identified in source-path space.
- **FR-002**: `--source` MUST mean that current source absence is authoritative and the remaining destination state is proposed for removal; `--destination` MUST mean that current destination absence is authoritative and the remaining source state is proposed for removal. Exactly one authority flag MUST be supplied.
- **FR-003**: Deletion MUST mutate only through the dedicated delete command. Push, pull, sync, resolve, baseline acceptance, mapping removal, and ignore-policy changes MUST NOT execute deletion or infer deletion authorization.
- **FR-004**: `-n` and `--dry-run` MUST build and render the same complete deterministic deletion plan as an equivalent mutating invocation over unchanged evidence and MUST NOT change payloads, registry data, baselines, recovery data, operation records, or coordination state.
- **FR-005**: A directional deletion action MUST be eligible only when an established accepted entry is absent on the authoritative side and the remaining peer is complete and equivalent to its accepted baseline.
- **FR-006**: A delete/change conflict, changed remaining peer, missing or corrupt required state, unsupported node, unsafe path, unmanaged descendant that would be removed with a managed directory, incomplete observation, invalid ownership, ambiguous selection, or unavailable required transition MUST block the entire selected deletion plan before its first mutation.
- **FR-007**: Deletion preflight MUST validate the complete mapping registry and completely inspect and classify the selected scope, accumulate every known blocker, and order eligible child entries before their parent directories using canonical managed identity.
- **FR-008**: Immediately before each deletion action, Grip MUST revalidate mapping, accepted membership, authoritative absence, remaining-side evidence, target ancestry, accepted baseline, and plan assumptions whose change could make removal unsafe or different.
- **FR-009**: Before removing a remaining entry, Grip MUST preserve and verify its complete supported state in operation-bound recovery evidence. The first preservation, removal, verification, coordination, or state-publication failure MUST stop execution, retain completed effects and recovery evidence, and leave later actions unattempted.
- **FR-010**: Grip MUST remove entries without following symbolic links or crossing a discovered managed directory boundary, MUST verify the intended entry is absent afterward, and MUST report durability separately when the filesystem cannot confirm it.
- **FR-011**: A fully successful directional deletion MUST retire the corresponding accepted baseline records only after every removal verifies and final complete observation confirms the selected result. A scoped deletion MUST preserve every out-of-scope accepted record.
- **FR-012**: A blocked, stale, contended, partially failed, verification-failed, or recovery-failed deletion MUST NOT publish a new baseline. A state-publication or result-delivery failure MUST report whether a retired generation became visible and whether durability was confirmed.
- **FR-013**: Grip MUST expose `grip retire [-n|--dry-run] [--destination] [--force] (--all|[--] PATH)` using the established path-selector contract: source space by default and destination space when requested. Exactly one path selector or `--all` MUST be supplied; a bare command MUST select nothing and fail without mutation.
- **FR-014**: Retirement MUST be eligible only for accepted records classified as newly ignored pending retirement, untracked pending retirement, or converged deletion. It MUST NOT retire actively managed records, unsupported or unsafe entries, or entries with incomplete observations. When a pending-retirement entry has surviving copies that differ from each other or the accepted baseline, ordinary retirement MUST block and report the differences; `--force` MAY authorize retirement of that reviewed record without mutating either copy.
- **FR-015**: Retirement MUST remove only selected accepted membership and baseline records. It MUST NOT delete, copy, replace, restore, or otherwise mutate source or destination payloads; recreate mappings; alter ignore policy; or remove recovery material.
- **FR-016**: Retirement dry run MUST render the exact deterministic record-removal plan, including whether differing surviving copies require `--force`, without mutation. Mutating retirement MUST revalidate registry, policy, membership, current observations, selected classifications, accepted state, and any force-required differences immediately before atomic state publication.
- **FR-017**: A successful retirement MUST publish one new accepted-state generation when at least one record is removed, preserve every unselected record, and report each retired identity and reason. An empty or already-retired selection MUST succeed as a no-op without publishing a generation.
- **FR-018**: Grip MUST expose read-only `grip recovery list` and `grip recovery show RECOVERY_REF` operations that cover payload, registry, accepted-state, and durable operation recovery evidence without opening unsupported payload nodes or printing recovered file contents by default.
- **FR-019**: Recovery inspection MUST report stable opaque reference, recovery kind, originating operation or state transition, original side and path where applicable, managed identity, creation time, verification and integrity status, current availability or cleaned status, byte count where knowable, and restoration eligibility or blocking reason.
- **FR-020**: Grip MUST expose `grip recovery restore [-n|--dry-run] [--] RECOVERY_REF` for one exact verified payload, registry, or accepted-state recovery entry. Restore MUST target only the original bound location and MUST NOT accept an arbitrary destination path.
- **FR-021**: Restore MUST validate recovery integrity, current ownership and path safety, every available authority not being repaired, and the current payload target against evidence recorded by the originating operation. A missing or corrupt registry or accepted-state target being repaired need not validate, but missing recovery provenance, stale live evidence, ambiguity, unsafe paths, or incompatibility MUST block before mutation.
- **FR-022**: Payload restore MUST refuse to overwrite an occupied target unless that target exactly matches the originating operation's verified post-action state. A successful restore MUST preserve the displaced post-action state as new recovery evidence before replacement or removal.
- **FR-023**: Registry or accepted-state restore MUST publish only an exact previously verified document after complete live registry, path, ownership, payload, and remaining-authority checks prove it compatible. The missing or corrupt target being repaired MUST NOT be required to validate first. Grip MUST NOT reconstruct mappings, baselines, or authority from guesses or partial payload observations.
- **FR-024**: Restore MUST use the established mutation coordination, revalidation, staging, strongest-supported atomic publication, verification, recovery, and truthful partial-failure contracts. It MUST retain the source recovery entry after success and MUST NOT automatically accept or claim synchronized payload state.
- **FR-025**: Grip MUST expose `grip recovery remove --confirm RECOVERY_REF...` with `-n` and `--dry-run` preview aliases. Cleanup MUST require one or more exact recovery references plus explicit confirmation and MUST NOT support implicit age-, count-, or space-based deletion in this feature.
- **FR-026**: Cleanup MUST reject ambiguous or missing recovery references, corrupt or inconsistent selected recovery evidence, and recovery evidence required by an active operation before its first removal. It MUST never remove accepted registry or baseline state, current operation coordination, or payload outside Grip's private recovery namespace.
- **FR-027**: Cleanup MUST remove selected recoverable bytes in deterministic order, verify each removal, retain immutable recovery metadata marked with a cleaned tombstone, stop at the first operational failure, and identify removed, failed, and unattempted entries. Cleanup MUST NOT claim atomicity across multiple references.
- **FR-028**: Deletion, retirement, restore, and cleanup mutations MUST participate in the established short-lived per-user mutation-coordination boundary. Read-only inspection, previews, and semantic no-ops MUST remain lock-free, and Grip MUST NOT lock whole payload trees.
- **FR-029**: Immediately before its first mutation, each actionful deletion, retirement, restore, or cleanup MUST durably initialize an operation record and checkpoint planned, completed, failed, and unattempted actions plus recovery, visibility, verification, and state-authority outcomes through the terminal result.
- **FR-030**: A nonterminal operation record left by interruption MUST remain immutable evidence and MUST NOT be silently resumed, completed, rolled back, or treated as proof that current filesystem state is safe. Later operations require fresh complete inspection.
- **FR-031**: Machine-readable results MUST reuse the existing versioned result boundary while identifying `operation: delete|retire|recovery_restore|recovery_remove`, authority side or retirement reason where applicable, exact recovery references, action outcomes, visibility, verification, and accepted-state authority. Human output MUST communicate equivalent facts while diagnostics remain separate.
- **FR-032**: Grip MUST NOT prune destination-only unmanaged content, infer deletion from absence or ignore changes, automatically clean recovery evidence, restore to arbitrary paths, perform automatic whole-operation rollback, elevate privileges, invoke version-control operations, follow symbolic links, cross mount boundaries, or coerce unsupported payload or metadata.
- **FR-033**: Automated tests using isolated temporary roots MUST cover both directional deletions, converged deletion, newly ignored and untracked retirement, directory ordering, dry-run parity, complete-scope blocking, stale evidence, contention, every injected recovery and publication failure, recovery inspection, safe and blocked restores, cleanup authorization and partial failure, output parity, and all non-mutation guarantees.
- **FR-034**: Representative measurement MUST cover preview planning and recovery inventory for at least 10,000 managed entries and 10,000 recovery entries. Any cache, index, parallel execution, automatic retention policy, or additional coordination mechanism MUST be justified by those measurements before introduction.

### Key Entities

- **Deletion Plan**: Complete deterministic evidence for one authority direction and selected scope, containing ordered removals, baseline retirements, no-action records, blockers, and revalidation expectations.
- **Deletion Action**: One explicitly authorized removal of the remaining peer after the opposite side's absence is proven authoritative, including recovery, verification, and accepted-record retirement outcomes.
- **Pending-Retirement Record**: Accepted membership and baseline evidence retained after an entry becomes newly ignored or its mapping is removed, awaiting explicit state-only retirement.
- **Retirement Plan**: A deterministic selection of eligible accepted records to remove without payload mutation.
- **Recovery Entry**: Immutable metadata and, until explicit cleanup, verified recoverable bytes for a prior payload, registry, or accepted-state document, bound to its origin, identity, integrity, and permitted restoration target; cleanup replaces byte availability with a permanent cleaned tombstone.
- **Recovery Restore Plan**: A single-entry restoration decision containing exact source evidence, bound destination, current-target expectations, displacement recovery, blockers, and verification requirements.
- **Recovery Cleanup Plan**: Explicit exact recovery references selected for permanent removal, with byte counts, eligibility, and terminal outcomes.
- **Destructive Operation Record**: Durable evidence for deletion, retirement, restore, or cleanup, including authorization, ordered actions, recovery references, visibility, verification, and accepted-state authority.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of deletion fixtures, ordinary push, pull, sync, mapping removal, and ignore-policy changes execute zero deletions; only an explicitly directed delete invocation can remove a remaining peer.
- **SC-002**: In 100% of source-authoritative and destination-authoritative deletion fixtures, dry run and mutation over equivalent unchanged evidence produce equivalent ordered plans, while dry run produces zero payload or Grip-state changes.
- **SC-003**: Every successful directional deletion fixture preserves and verifies the remaining peer before removal, leaves both sides absent, retires exactly the selected accepted records, and reports the new accepted-state authority.
- **SC-004**: Every fixture containing a delete/change conflict, unsafe path, unsupported entry, incomplete observation, invalid state, or changed relevant evidence starts zero deletion actions and preserves the prior baseline.
- **SC-005**: In 100% of newly ignored, untracked, and converged-deletion fixtures, retirement removes exactly the selected eligible accepted records and produces zero source or destination payload changes; differing surviving copies block without `--force` and retire only when the reviewed request includes `--force`.
- **SC-006**: Across injected failures at every preservation, removal, verification, and state-publication phase, results identify completed, failed, and unattempted work, retain all created recovery evidence, and never represent partial deletion as accepted retirement.
- **SC-007**: Recovery list and show identify kind, origin, bound location, integrity, availability, byte count, and restoration eligibility for 100% of valid recovery fixtures without printing recovered payload content by default.
- **SC-008**: In 100% of successful restore fixtures, Grip reproduces the exact verified preserved state at its bound location, preserves any displaced post-action state, retains the source recovery entry, and does not claim automatic synchronization acceptance.
- **SC-009**: In 100% of stale, occupied-with-unexpected-data, corrupt, ambiguous, or unsafe restore fixtures, Grip detects the blocker before mutation and preserves current payload and accepted state.
- **SC-010**: Cleanup removes only exact confirmed recovery references in 100% of success fixtures; missing confirmation or any non-recovery target results in zero removals.
- **SC-011**: Human and machine-readable results communicate the same authority direction, retirement reasons, ordered actions, recovery references, failures, visibility, verification, and accepted-state outcome in 100% of paired output tests.
- **SC-012**: At least 95 of 100 deletion-preview and recovery-list invocations over documented representative sets of 10,000 entries complete within two seconds on a documented representative local workstation without requiring a persistent cache, background service, broad payload lock, or automatic cleanup policy.
- **SC-013**: After every tested successful or partial outcome, a user can determine current payload effects, authoritative accepted state, and available recovery actions from command results plus read-only inspection without directly examining Grip's internal files.

## Assumptions

- Features 001 through 007 are verified, and their mapping, discovery, classification, selector, output, coordination, staging, recovery, operation-record, supported-equality, and baseline-publication contracts remain authoritative.
- The dedicated delete command treats `--source` or `--destination` as the side whose current absence the user accepts as authoritative, mirroring the explicit whole-side choice used by conflict resolution while keeping deletion separate from ordinary synchronization.
- Retirement is a state-only transition. Existing payload copies remain ordinary user data after accepted membership evidence is retired; if policy later admits the path again, Grip classifies it from fresh evidence rather than reviving retired history implicitly.
- Recovery evidence remains retained until exact user-confirmed cleanup. This feature does not introduce automatic age, quota, count, or disk-pressure retention policies.
- Bounded registry or accepted-state recovery is allowed only from an exact verified prior document that is still compatible with current evidence; missing or corrupt authority is never reconstructed from guesses.
- The supported payload contract remains ordinary regular files and directories with regular-file byte content and Unix permission mode. Broader metadata, symbolic-link, filesystem capability, case, Unicode, and portability behavior remain Feature 009 work.
- The wiki query provides Partial coverage: it establishes deletion authorization, recovery preservation, pending-retirement visibility, and safety boundaries, while leaving exact retirement and broader recovery contracts to this feature. Its roadmap page still describes version 1.0.14 and therefore does not evidence the separately approved version 1.0.15 status transition.
- This checkout has no registered `before_specify` or `after_specify` hook, so creating the feature artifacts does not create or switch a Git branch or dispatch another command.
