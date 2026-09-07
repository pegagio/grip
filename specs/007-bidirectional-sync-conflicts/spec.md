# Feature Specification: Bidirectional Synchronization and Conflict Resolution

**Feature Branch**: `007-bidirectional-sync-conflicts`

**Created**: 2026-09-07

**Status**: Complete

**Input**: User description: "Specify roadmap entry 007, Bidirectional Synchronization and Conflict Resolution, using the roadmap and wiki-backed governing decisions and constraints."

## Clarifications

### Session 2026-09-07

- Q: How should users choose the winning side when resolving one conflicting entry? → A: Require explicit `--source` or `--destination` winner flags.
- Q: Should conflict resolution support `-n` and `--dry-run` to preview the chosen replacement without changing either side? → A: Support both `-n` and `--dry-run` for non-mutating resolution previews.
- Q: In `grip resolve PATH --source|--destination`, which path space should `PATH` use? → A: `PATH` always identifies the entry in source-path space.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Preview a Unified Synchronization (Priority: P1)

As a local user, I can preview every eligible source-to-destination and destination-to-source action in one selected scope and see every conflict or unsafe condition before any payload or Grip-owned state changes.

**Why this priority**: A complete non-mutating preview is the trust boundary for combining both mutation directions.

**Independent Test**: Build an isolated selected scope containing source-only changes, destination-only changes, synchronized entries, converged edits, divergent conflicts, and unsafe evidence; run `grip sync --dry-run`; verify the complete deterministic plan, all blockers, and zero mutations.

**Acceptance Scenarios**:

1. **Given** unambiguous source-only and destination-only changes in one selected scope, **When** the user previews sync, **Then** Grip reports every directional action in deterministic order without changing payloads or Grip-owned state.
2. **Given** one or more conflicts or known unsafe conditions in scope, **When** preview completes, **Then** Grip reports every discovered blocker and states that the plan cannot execute.
3. **Given** unchanged starting evidence, **When** preview is repeated or invoked with `-n`, **Then** each result describes an equivalent ordered plan.
4. **Given** both sides independently reached the same complete supported state, **When** preview completes, **Then** Grip reports convergence rather than a conflict or replacement action.

---

### User Story 2 - Synchronize Unambiguous Changes in Both Directions (Priority: P2)

As a local user, I can apply all unambiguous one-sided changes in scope through one command while retaining the safety, recovery, verification, and baseline guarantees of directional synchronization.

**Why this priority**: This delivers the roadmap outcome of practical bidirectional synchronization without making either mapping side permanently authoritative.

**Independent Test**: Create isolated accepted entries changed only at the source, changed only at the destination, and unchanged; run `grip sync`; verify both directional replacements, recovery evidence for each losing state, final equivalence, and one accepted baseline publication.

**Acceptance Scenarios**:

1. **Given** source-only and destination-only changes with no blockers, **When** sync succeeds, **Then** each change propagates to its unchanged peer, every replacement is verified, and the resulting accepted baseline describes the complete selected scope.
2. **Given** all selected entries are synchronized, **When** sync runs, **Then** it succeeds as a no-op without creating recovery evidence, an operation record, or a baseline generation.
3. **Given** relevant evidence changes after planning but before an action, **When** execution revalidates the action, **Then** Grip stops before that action and publishes no new baseline.
4. **Given** destination-only content outside established managed membership, **When** sync runs, **Then** Grip leaves it unmanaged and never imports or mutates it.

---

### User Story 3 - Resolve a Whole-Entry Conflict Explicitly (Priority: P3)

As a local user, I can resolve one exact divergent managed entry by explicitly choosing the complete source or complete destination state as the winner, after fresh inspection and with recovery preservation for the losing side.

**Why this priority**: Conflicts must be recoverable user decisions, not automatic merging or implicit direction choices.

**Independent Test**: Create an accepted entry with incompatible changes on both sides; resolve it once with the source as winner and once with the destination as winner; verify losing-side recovery, complete-state replacement, drift detection, and baseline publication.

**Acceptance Scenarios**:

1. **Given** one exact divergent managed entry and unchanged conflict evidence, **When** the user resolves it with the source as winner, **Then** Grip preserves the destination state, makes the destination equivalent to the complete supported source state, verifies the result, and publishes the accepted baseline.
2. **Given** the same conflict shape, **When** the user resolves it with the destination as winner, **Then** Grip preserves the source state and applies the complete supported destination state symmetrically.
3. **Given** either side or the accepted baseline changed after the conflict was reported, **When** resolution re-inspects the entry, **Then** Grip refuses the stale decision before mutation and requires a fresh user choice.
4. **Given** content differs on one side while supported metadata differs on the other, **When** Grip classifies the entry, **Then** it treats the whole entry as conflicting and never combines attributes from opposing sides.

---

### User Story 4 - Understand Failures and Automate Safely (Priority: P4)

As a user or script author, I receive a precise, stable account of blocked, completed, failed, and unattempted work for sync and resolution, including recovery and baseline authority.

**Why this priority**: Mixed-direction partial failure must never be mistaken for accepted convergence.

**Independent Test**: Exercise every terminal sync and resolution outcome in isolated fixtures, including injected failure after one completed action, and verify action direction, recovery references, visibility, verification, baseline outcome, exit behavior, and human/machine parity.

**Acceptance Scenarios**:

1. **Given** a conflict blocks sync, **When** machine-readable output is requested, **Then** the result identifies all blockers, confirms zero actions started, and distinguishes the blocked plan from an operational failure.
2. **Given** a mixed-direction sync fails after an earlier action completed, **When** the result is produced, **Then** every action is identified as completed, failed, or unattempted with its direction, recovery status, visibility, verification, and baseline non-publication.
3. **Given** a successful resolution, **When** human and machine-readable results are requested over equivalent starting evidence, **Then** both communicate the same winner, losing-side recovery, replacement, verification, and acceptance outcome while diagnostics remain separate.
4. **Given** all payload work and baseline publication succeeded but final result delivery fails, **When** the command terminates, **Then** Grip retains the accepted baseline, reports an operational error, and preserves durable operation evidence.

### Edge Cases

- The selected scope is empty, already synchronized, names one mapping or managed subtree, points at unmanaged content, or begins with `-`.
- A scope contains actions in both directions plus one conflict, unsupported node, unsafe ancestry, unavailable metadata transition, or incomplete observation.
- A source or destination disappears, changes kind, becomes inaccessible, or is replaced between inspection and its planned action.
- Two sides changed differently but happen to share content while supported metadata differs, or share metadata while content differs.
- Both sides independently converge to an identical supported state after the accepted baseline.
- A conflict target is not exact, selects multiple entries, is no longer conflicting, has no accepted baseline, or is pending deletion or retirement.
- A winner option is omitted, duplicated, or names both source and destination.
- Recovery, staging, publication, verification, coordination, baseline publication, or final result delivery fails.
- A mixed-direction plan requires alternating target filesystems or encounters different filesystem capabilities.
- Multiple Grip processes attempt sync, resolution, or another payload mutation concurrently.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST expose `grip sync [PATH]` as a bidirectional action that mutates by default, while `-n` and `--dry-run` MUST build and render the same plan without mutation.
- **FR-002**: Sync MUST reuse the established single-selector contract: source-space interpretation by default, destination-space interpretation with `--destination`, and `--` as option termination.
- **FR-003**: Before any action begins, Grip MUST validate the complete mapping registry and completely inspect and classify the selected scope against current source, current destination, and the accepted baseline.
- **FR-004**: A sync plan MUST contain a deterministic ordered outcome for every selected managed entry, including action direction, no-action disposition, or blocker. Ordering ties MUST follow canonical mapping source and source-relative raw path identity rather than action direction.
- **FR-005**: Sync MUST plan source-to-destination replacement for eligible source-only changes and destination-to-source replacement for eligible destination-only changes using the established push and pull eligibility contracts.
- **FR-006**: Synchronized entries and converged identical changes MUST require no payload replacement. A successful sync MUST include converged complete supported evidence in the accepted result without requiring the user to select a winner.
- **FR-007**: Destination-only content outside established managed membership MUST remain unmanaged and non-actionable and MUST NOT be imported, mutated, or treated as a blocker unless it independently creates an unsafe collision at a managed path.
- **FR-008**: Divergent changes, initial collisions, delete/change conflicts, directional deletions, converged deletions, pending-retirement entries, unsupported managed entries, unsafe paths, incomplete inspection, invalid ownership, corrupt required state, and unavailable required transitions MUST block the entire selected sync before its first mutation.
- **FR-009**: Complete preflight MUST accumulate and report every known blocker in the selected scope rather than stop after the first discovered blocker.
- **FR-010**: Dry run MUST report the same ordered actions, no-action records, blockers, and relevant evidence as an equivalent mutating invocation over unchanged state, and MUST NOT change payloads, registry data, baselines, recovery data, operation records, or coordination state.
- **FR-011**: Immediately before each action, Grip MUST revalidate the mapping, accepted membership, source and destination evidence, target ancestry, accepted baseline, and plan assumptions whose change could make that action unsafe or different.
- **FR-012**: Any failed revalidation MUST stop execution before the affected action, mark every later action unattempted, and prevent publication of a new baseline.
- **FR-013**: Every replacement MUST reuse the established directional staging, losing-side recovery, strongest-supported atomic publication, durability reporting, and result verification contracts from push or pull according to its direction.
- **FR-014**: Before replacing an existing managed entry, Grip MUST preserve and verify its complete prior supported state in a recovery namespace bound to the operation, managed identity, and action direction.
- **FR-015**: After execution begins, the first staging, recovery, publication, verification, coordination, or state-publication failure MUST stop the operation. Completed actions MUST remain in place and automatic rollback MUST NOT be attempted.
- **FR-016**: A failed sync MUST preserve all recovery material already created, identify the first failed action and reason, distinguish completed, failed, and unattempted actions, and report whether each attempted change became visible and verified.
- **FR-017**: Grip MUST publish a new accepted baseline only after every planned action completes and verifies successfully and final acceptance-relevant revalidation confirms the complete selected result, including converged no-replacement entries.
- **FR-018**: A scoped successful sync MUST update accepted records for selected verified actions and converged changes, preserve selected synchronized records plus all out-of-scope and pending-retirement records, and publish no generation for a semantic no-op.
- **FR-019**: A blocked, stale, contended, partially failed, or verification-failed sync MUST NOT invoke baseline publication. Baseline-publication and result-delivery failures MUST preserve and report whether an accepted generation became visible.
- **FR-020**: Mutating sync and conflict resolution MUST participate in the established short-lived per-user mutation-coordination boundary from post-preflight revalidation through payload execution and baseline publication. They MUST NOT lock payload trees; previews, read-only preflight, and semantic no-ops MUST remain lock-free.
- **FR-021**: Immediately before its first payload mutation or, for a converged-only sync, before its first accepted-state mutation, each sync or resolution MUST durably initialize one distinct operation record and checkpoint action, direction, recovery, visibility, verification, and baseline outcomes through the terminal result. A converged-only sync MUST create an operation record because it publishes accepted state; blocked, preview, and synchronized-only semantic no-op operations MUST create no operation record.
- **FR-022**: A nonterminal operation record left by interruption MUST remain immutable evidence and MUST NOT alone block a later operation that passes fresh complete inspection. Grip MUST NOT silently resume, complete, or roll back the interrupted operation.
- **FR-023**: Grip MUST expose `grip resolve PATH --source` and `grip resolve PATH --destination` for one exact established managed entry identified in source-path space. Exactly one winner flag MUST be supplied, and the selected flag MUST choose the complete source or destination state without changing path interpretation or inferring the winner from the path. Both forms MUST accept `-n` and `--dry-run` to render the equivalent resolution plan without changing payloads or Grip-owned state.
- **FR-024**: Resolution MUST freshly inspect the exact entry, validate complete registry and ownership safety, confirm a divergent conflict against the current accepted baseline, and reject a stale, absent, ambiguous, non-conflicting, deletion, or retirement target before mutation.
- **FR-025**: Resolution MUST treat content and all supported metadata as one complete entry state. It MUST NOT merge, combine, or preserve selected attributes from the losing side.
- **FR-026**: Resolution MUST preserve and verify the losing side, replace it from the chosen winner using the established directional mutation contract, verify complete equivalence, and publish the resulting accepted baseline only after success.
- **FR-027**: Resolution MUST NOT mutate entries outside the exact selected identity; complete-registry validation remains mandatory, but unrelated payload drift or conflicts outside the selected entry MUST NOT expand the resolution scope.
- **FR-028**: Sync and resolution MUST NOT execute deletion, retirement, initial-collision adoption, destination-only import, backup cleanup, automatic rollback, conflict merging, repair or reconstruction of missing or corrupt state, privilege elevation, version-control operations, symbolic-link following, or unsupported payload coercion.
- **FR-029**: Machine-readable sync and resolution results MUST reuse the existing versioned mutation-result contract while identifying the operation and each action's direction. Human and machine-readable results MUST communicate equivalent scope, plan, blockers, winner where applicable, action outcomes, recovery references, visibility, verification, baseline authority, and stable error category while diagnostics remain separate.
- **FR-030**: Automated tests using isolated temporary roots MUST cover mixed-direction planning and execution, deterministic dry-run parity, complete-scope blocking, converged changes, both conflict winners, stale conflict decisions, whole-entry supported-state selection, unmanaged destination content, contention, every injected failure boundary, interruption evidence, baseline authority, output parity, and all non-mutation guarantees.
- **FR-031**: A documented representative workload of 10,000 eligible managed entries with mixed no-op, source-only, destination-only, converged, and conflicting classifications MUST measure preview planning, and any cache, index, parallel execution, or additional coordination mechanism MUST be justified by those measurements before introduction.

### Key Entities

- **Sync Plan**: Complete deterministic evidence for one selected scope, containing ordered actions in both directions, no-action records, blockers, and the observations each action must revalidate.
- **Directional Sync Action**: One planned replacement with an explicit transfer direction, expected source and destination evidence, recovery and staging requirements, and terminal status.
- **Conflict Decision**: A fresh, explicit choice of the complete source or destination state for one exact divergent managed entry, bound to the evidence inspected for that invocation.
- **Losing-Side Recovery Entry**: Verified prior state retained before conflict resolution or synchronization replacement and bound to the operation, managed identity, and direction.
- **Mutation Operation Record**: Durable evidence for one sync or resolution, including ordered actions, direction or winner, completed, failed, and unattempted outcomes, recovery references, visibility, verification, and baseline-publication status.
- **Accepted Bidirectional Result**: A fully verified selected scope, including applied one-sided changes and accepted converged changes, whose baseline publication succeeded.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of conformance fixtures, dry run and mutating sync over equivalent unchanged starting evidence produce equivalent ordered plans, while dry run produces zero payload or Grip-state changes.
- **SC-002**: Every fixture containing a known conflict, unsafe condition, unsupported managed entry, or incomplete observation reports all known blockers and starts zero actions, including when otherwise eligible actions exist in both directions.
- **SC-003**: Across 100% of eligible mixed-direction fixtures, a successful sync leaves every selected entry equivalent on both sides, retains verified prior state for every replacement, and publishes matching accepted baseline evidence exactly once.
- **SC-004**: In 100% of converged-change fixtures, Grip recognizes identical complete supported states without a conflict or payload replacement and accepts them only as part of a fully successful selected result.
- **SC-005**: Across source-wins and destination-wins conflict fixtures, 100% of successful resolutions preserve the losing complete supported state, make both sides equivalent to the selected winner, verify the result, and publish matching baseline evidence.
- **SC-006**: Across injected drift points between conflict reporting and resolution, Grip rejects the stale decision before mutation in 100% of cases.
- **SC-007**: Across injected first failures at every staging, recovery, publication, verification, and baseline-publication phase, the result identifies every action as completed, failed, or unattempted, preserves all created recovery evidence, and never represents partial work as accepted convergence.
- **SC-008**: Human and machine-readable results identify the same actions, directions, blockers, conflict winner, recovery availability, failure boundary, and baseline outcome in 100% of paired output tests.
- **SC-009**: For a documented representative scope of 10,000 eligible managed entries, at least 95 of 100 dry-run planning invocations complete within two seconds on a documented representative local workstation without requiring a persistent cache, background service, or broad payload lock.
- **SC-010**: From a sync or resolution result plus subsequent read-only status, a user can identify every visible change, the authoritative baseline, and available recovery evidence after every tested complete or partial outcome without inspecting Grip's internal files directly.
- **SC-011**: In 100% of conflict fixtures, `-n` and `--dry-run` produce an equivalent resolution plan to a mutating invocation over unchanged evidence while producing zero payload or Grip-state changes.

## Assumptions

- Features 001 through 006 are verified, and their mapping, discovery, classification, selector, output, staging, recovery, operation-record, coordination, supported-equality, and baseline-publication contracts remain authoritative.
- `sync` composes the already accepted push and pull semantics within one complete plan; it does not reopen their directional eligibility, failure, or recovery decisions.
- Conflict resolution is one exact-entry action with explicit source or destination selection. It does not accept a broad scope or remembered decision because fresh evidence is required at mutation time.
- Converged identical changes may be incorporated into a successful accepted sync result without payload replacement; a scope containing only already-synchronized entries remains a no-op with no new baseline generation.
- Directional deletion, converged deletion, and retirement remain Feature 008 work and block mutation when encountered in selected scope.
- The supported payload contract remains ordinary regular files and directories with regular-file byte content and Unix permission mode; broader metadata and filesystem behavior remain Feature 009 work.
- The wiki query provides partial coverage: it governs behavior and safety boundaries but does not yet contain Feature 007's final command and result contracts. This specification supplies those feature-owned contracts from the roadmap and existing verified interfaces.
- This checkout has no registered `before_specify` hook, so creating the feature artifacts does not create or switch a Git branch.
