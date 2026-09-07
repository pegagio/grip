# Feature Specification: Reverse Synchronization

**Feature Branch**: `006-reverse-synchronization`

**Created**: 2026-09-06

**Status**: Complete

**Input**: User description: "Specify roadmap entry 006, Reverse Synchronization, using the roadmap and wiki-backed governing decisions and constraints."

## Clarifications

### Session 2026-09-06

- Q: When `pull` encounters destination-only content outside established managed membership, should the pull result report it as a non-action record or omit it? → A: Report each item as unmanaged and non-actionable.
- Q: Should `pull` reuse the existing versioned push result schema with an explicit operation-direction value, or define a separate pull-specific schema? → A: Reuse the shared schema with `direction: pull`.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Preview Destination Changes (Priority: P1)

As a local user, I can preview every eligible destination-to-source action in a selected scope and see all reasons the pull would be blocked without changing payloads or Grip-owned state.

**Why this priority**: A complete, non-mutating preview lets the user verify that reverse synchronization will affect only established managed entries before source files change.

**Independent Test**: Build an isolated scope containing destination-only changes to managed entries, synchronized entries, conflicts, unsupported nodes, and unmanaged destination-only entries; run `grip pull --dry-run`; verify the complete deterministic plan and zero mutations.

**Acceptance Scenarios**:

1. **Given** eligible destination-only changes to established managed entries, **When** the user runs `grip pull --dry-run`, **Then** Grip reports the ordered source changes it would perform and changes no payload, baseline, recovery, registry, operation, or coordination state.
2. **Given** a conflict or known unsafe condition anywhere in scope, **When** preview completes, **Then** Grip reports every discovered blocker and states that no action is executable.
3. **Given** unchanged evidence, **When** preview is repeated or invoked with `-n`, **Then** each result describes an equivalent plan in deterministic order.
4. **Given** destination-only content that has never entered the managed namespace, **When** preview completes, **Then** Grip reports each item as unmanaged and non-actionable and never proposes importing it.

---

### User Story 2 - Pull Managed Changes Safely (Priority: P2)

As a local user, I can pull eligible changes from established managed destinations back to their sources with stale-evidence detection, recovery preservation, result verification, and truthful baseline publication.

**Why this priority**: This completes the second directional mutation capability while preserving the safety guarantees already established for push.

**Independent Test**: Create isolated mappings with accepted baseline entries and destination-only changes, run `grip pull`, and verify that each source, its recovery evidence, the action results, and the accepted baseline describe the same successfully verified outcome.

**Acceptance Scenarios**:

1. **Given** an established managed entry whose destination changed while its source still matches the accepted baseline, **When** pull succeeds, **Then** Grip preserves the prior source, installs and verifies the complete supported destination state at the source, and publishes the accepted resulting baseline.
2. **Given** every selected managed entry is already synchronized, **When** the user runs pull, **Then** Grip succeeds as a reported no-op without creating recovery evidence, an operation record, or a new baseline generation.
3. **Given** relevant evidence changes after planning but before an action is applied, **When** execution revalidates that action, **Then** Grip stops before the affected action and publishes no new baseline.
4. **Given** a selected tree mapping with an established managed member changed only at the destination, **When** pull succeeds, **Then** the corresponding source member is updated without importing adjacent destination-only content.

---

### User Story 3 - Understand and Recover From Pull Failure (Priority: P3)

As a local user, I receive a precise account of completed, failed, and unattempted source changes when a pull fails, while Grip preserves prior and replacement evidence and does not falsely accept a partial result.

**Why this priority**: Reverse mutation carries the same partial-failure risk as push; trustworthy evidence must survive even when the operation cannot finish.

**Independent Test**: Inject a failure after at least one successful action in an isolated multi-entry pull and verify that execution stops immediately, completed source changes remain verifiable, recovery references remain intact, later actions are unattempted, and the prior baseline stays authoritative.

**Acceptance Scenarios**:

1. **Given** a multi-entry plan whose next action fails after earlier actions completed, **When** the failure occurs, **Then** Grip stops before later actions, reports each action as completed, failed, or unattempted, and publishes no new baseline.
2. **Given** an existing source is about to be replaced, **When** recovery, staging, publication, or verification fails, **Then** Grip reports whether the authoritative source changed and preserves all recovery evidence already created.
3. **Given** a prior pull failed partially, **When** the user runs read-only inspection, **Then** the unchanged accepted baseline enables the resulting filesystem state to be classified rather than represented as accepted convergence.
4. **Given** a process interruption left a nonterminal pull record, **When** a later pull derives a fresh complete plan and passes ordinary preflight, **Then** it may execute as a separate operation while preserving the interrupted record and recovery entries unchanged.

---

### User Story 4 - Use Pull Results in Automation (Priority: P4)

As a script author, I can consume stable machine-readable preview and execution results that distinguish blocked plans, no-op success, complete success, stale evidence, contention, partial failure, and result-delivery failure.

**Why this priority**: Automation must not mistake destination-only unmanaged content, a preview, partial source mutation, or failed result delivery for an accepted pull.

**Independent Test**: Exercise every terminal pull outcome in isolated fixtures and verify stable result categories, action records, exit behavior, and parity with human-readable results.

**Acceptance Scenarios**:

1. **Given** a blocked preview or execution request, **When** machine-readable output is requested, **Then** it identifies every preflight blocker and confirms that zero actions started.
2. **Given** a partially failed pull, **When** machine-readable output is produced, **Then** it identifies the failed action, completed actions, unattempted actions, recovery references, baseline non-publication, and whether any source change became visible.
3. **Given** a successful pull, **When** human and machine-readable output are requested separately over equivalent starting evidence, **Then** both communicate equivalent actions and acceptance outcome while diagnostics remain separate, and machine-readable output uses the shared versioned mutation-result schema with `direction: pull`.
4. **Given** every source action and baseline publication succeeded but final output delivery fails, **When** the command terminates, **Then** Grip retains the accepted baseline, returns an operational error, and preserves durable operation evidence.

### Edge Cases

- The selected scope is empty, already synchronized, identifies a mapping or established managed entry, names unmanaged destination-only content, or begins with `-`.
- A destination path exists without accepted managed-entry evidence and therefore cannot be imported by pull.
- A source parent directory is absent, replaced, non-directory, inaccessible, or outside the accepted mapping ancestry.
- Destination content, supported metadata, mapping identity, managed membership, ignore policy, source ancestry, source state, or accepted baseline changes between inspection and action.
- A staged candidate cannot be created on the source filesystem, fully prepared, verified, or installed with the strongest supported atomic replacement.
- The source disappears before replacement or is concurrently replaced where the plan expected its prior accepted state.
- An existing source is a symbolic link, hard-linked file, sparse file, special node, nested mount, wrong node kind, or otherwise unsupported.
- Recovery material cannot be created or verified before source replacement, or the recovery namespace is unsafe, unavailable, or contended.
- An action succeeds but result verification fails; publication visibility changes but durability cannot be confirmed; or baseline publication fails after every source action verifies.
- Multiple Grip processes attempt payload or baseline mutation concurrently.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST expose `grip pull [PATH]` as a destination-to-source action that mutates by default, while `-n` and `--dry-run` MUST build and render the same plan without mutation.
- **FR-002**: Pull MUST reuse the established single-selector contract: source-space interpretation by default, destination-space interpretation with `--destination`, and `--` as option termination.
- **FR-003**: Before any action begins, Grip MUST validate the complete mapping registry and completely inspect and classify the selected scope against current source, current destination, and the accepted baseline.
- **FR-004**: A pull plan MUST be a complete deterministic ordered set of proposed actions derived from one validated inspection, with an explicit outcome for every selected managed entry. Ordering ties among eligible replacements MUST follow canonical mapping source and source-relative raw path identity.
- **FR-005**: Pull MUST plan replacement only for established managed entries classified as destination-only changes against accepted baseline evidence.
- **FR-006**: Pull MUST report each destination-only item outside established managed membership as an unmanaged, non-actionable record. Such an item MUST NOT become an addition, import candidate, source-creation action, or blocker unless it independently creates an unsafe collision at a managed path.
- **FR-007**: Synchronized entries MUST require no action. Source additions, source-only changes, initial matches, initial collisions, divergent or delete/change conflicts, converged changes awaiting acceptance, directional deletions, converged deletions, pending-retirement entries, and unsupported or unsafe evidence MUST NOT be mutated by this feature.
- **FR-008**: Any conflict, unsafe path, unsupported managed entry, invalid ownership, incomplete inspection, unavailable required metadata transition, corrupt required state, or other known blocking condition in the selected scope MUST block the entire pull before its first mutation.
- **FR-009**: Dry run MUST report the same ordered actions, blockers, and relevant evidence as an equivalent mutating invocation over unchanged state, and MUST NOT change payloads, registry data, baselines, recovery data, operation records, or coordination state.
- **FR-010**: Immediately before each action, Grip MUST revalidate the mapping, accepted membership, destination evidence, source evidence, source ancestry, accepted baseline, and plan assumptions whose change could make that action unsafe or different.
- **FR-011**: Any failed revalidation MUST stop execution before the affected action, mark every later action unattempted, and prevent publication of a new baseline.
- **FR-012**: Pull MUST require the established managed source and its accepted parent ancestry to remain present, supported, and safe. It MUST NOT recreate a missing source or parent as an implicit reversal of source-side deletion.
- **FR-013**: Replacement content and supported metadata MUST be fully staged and verified on the source filesystem before publication wherever the supported filesystem permits.
- **FR-014**: Before replacing an existing managed source, Grip MUST preserve and verify its prior supported state in a per-operation recovery namespace bound to the operation and managed entry.
- **FR-015**: Grip MUST use the strongest supported atomic publication operation for installing a staged source entry and MUST report when visibility changed but durability could not be confirmed.
- **FR-016**: Pull MUST verify every published source against the planned complete supported destination state before treating that action as completed.
- **FR-017**: After execution begins, the first staging, recovery, publication, verification, coordination, or state-publication failure MUST stop the operation. Completed actions MUST remain in place; automatic rollback MUST NOT be attempted.
- **FR-018**: A failed pull MUST preserve all recovery material already created, identify the first failed action and reason, distinguish completed, failed, and unattempted actions, and report whether each attempted source change became visible and verified.
- **FR-019**: Grip MUST publish a new accepted baseline only after every planned source action completes and verifies successfully and final acceptance-relevant revalidation confirms the complete selected result.
- **FR-020**: A scoped successful pull MUST update accepted records for identities whose actions completed and verified, preserve selected no-action records and all out-of-scope and pending-retirement records, and publish no generation for an unblocked no-action plan.
- **FR-021**: A blocked, stale, contended, partially failed, or verification-failed pull MUST NOT invoke baseline publication. Baseline-publication and result-delivery failures MUST preserve and report the same authority distinctions established for push, including retention of an already visible accepted generation.
- **FR-022**: A mutating pull MUST participate in the established short-lived per-user mutation-coordination boundary from post-preflight revalidation through payload execution and baseline publication. It MUST NOT lock payload trees; previews, read-only preflight, and semantic no-ops MUST remain lock-free.
- **FR-023**: Immediately before its first payload mutation, pull MUST durably initialize a distinct operation record and checkpoint action, recovery, visibility, verification, and baseline outcomes through the terminal result. Blocked, preview, and no-op pulls MUST create no operation record.
- **FR-024**: A nonterminal pull record left by interruption MUST remain immutable evidence and MUST NOT alone block a later operation that passes a fresh complete inspection. Grip MUST NOT silently resume, complete, or roll back the interrupted pull.
- **FR-025**: Pull MUST NOT elevate privileges, invoke version-control operations, follow symbolic links, open unsupported nodes as payload, weaken required metadata silently, or claim filesystem snapshot isolation.
- **FR-026**: Machine-readable pull results MUST reuse the existing versioned mutation-result schema established for push and identify the operation with `direction: pull`. Machine-readable and human-readable results MUST communicate equivalent selected scope, ordered plan, blockers, completion state, action outcomes, recovery references, publication and verification state, baseline outcome, and stable error category while diagnostics remain separate.
- **FR-027**: This feature MUST NOT perform source-to-destination propagation, bidirectional execution, conflict resolution, deletion, retirement, backup inspection or cleanup, automatic rollback, destination-only import, or repair or reconstruction of missing, unreadable, corrupt, or inconsistent registry or baseline state.
- **FR-028**: Pull behavior MUST be covered by automated tests using isolated temporary roots, including complete preflight blocking, deterministic dry-run parity, managed replacement, non-import of destination-only content, parent handling, stale evidence, contention, recovery failure, injected partial failure, verification failure, baseline non-publication, output failure, and non-mutation guarantees.
- **FR-029**: A documented representative workload of 10,000 eligible managed entries with a mixture of no-op and destination-only changed entries MUST measure preview and execution planning, and any cache, index, parallel execution, or additional coordination mechanism MUST be justified by those measurements before introduction.

### Key Entities

- **Pull Plan**: Complete deterministic evidence for one selected scope, containing ordered destination-to-source actions, no-action records, blockers, and the observations each action must revalidate.
- **Pull Action**: One planned replacement of an established managed source from its destination state, including expected destination and source evidence, staging and recovery requirements, and terminal execution status.
- **Source Recovery Entry**: Verified prior source state retained for one replacement and bound to the operation and managed entry; it is evidence for later user-directed recovery rather than an automatic rollback instruction.
- **Pull Operation Record**: Durable outcome evidence for one mutating pull, including completed, failed, and unattempted actions, recovery references, visibility and verification results, and baseline-publication outcome.
- **Accepted Pull Result**: A fully applied and verified selected scope whose new baseline has been successfully published.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of conformance fixtures, dry run and mutating pull over equivalent unchanged starting evidence produce equivalent ordered plans, while dry run produces zero payload or Grip-state changes.
- **SC-002**: Every fixture containing a known conflict, unsafe condition, unsupported managed entry, or incomplete observation starts zero actions. In 100% of fixtures containing destination-only content outside managed membership, every such item is reported as unmanaged and non-actionable, none is proposed for import, and its presence alone does not block otherwise eligible actions.
- **SC-003**: Across 100% of eligible managed replacement fixtures, a successful pull leaves each source equivalent to the complete supported destination state, retains verified prior source state, and publishes matching accepted baseline evidence.
- **SC-004**: Across injected drift points before every action phase, Grip detects changed relevant evidence before the affected publication, stops, and publishes no new baseline in 100% of cases.
- **SC-005**: Across injected first failures at every staging, recovery, publication, verification, and baseline-publication phase, the result identifies every action as completed, failed, or unattempted, preserves all created recovery evidence, and never represents partial work as accepted convergence.
- **SC-006**: In 100% of no-action fixtures, pull reports successful no-op completion and creates no recovery entry, operation record, or baseline generation.
- **SC-007**: Human and machine-readable results identify the same actions, blockers, recovery availability, failure boundary, and baseline outcome in 100% of paired output tests.
- **SC-008**: For a documented representative scope of 10,000 eligible managed entries, at least 95 of 100 dry-run planning invocations complete within two seconds on a documented representative local workstation without requiring a persistent cache, background service, or broad payload lock.
- **SC-009**: From the pull result plus a subsequent read-only status, a user can identify every changed source, the still-authoritative baseline, and available recovery evidence after every tested partial-failure scenario without inspecting Grip's internal files directly.

## Assumptions

- Features 001 through 005 are verified, and their mapping, discovery, classification, selector, output, recovery, operation-record, coordination, supported-equality, and baseline-publication contracts remain authoritative.
- Reverse synchronization applies only to established managed entries with accepted baseline evidence. Destination-only entries that have never entered source-defined membership remain unmanaged.
- Destination-side additions without accepted managed identity are excluded because importing them would weaken source-defined membership and expand scope beyond the roadmap.
- Feature 006 reuses the accepted push safety and failure semantics in the reverse direction; it does not reopen Feature 005 decisions about coordination, recovery evidence, stop-after-first-failure behavior, interruption records, or result-delivery failure.
- The supported payload contract remains ordinary regular files and directories with regular-file byte content and Unix permission mode; broader metadata and filesystem behavior remain Feature 009 work.
- Directional deletion remains Feature 008 work, so a missing or deleted source or destination is classified and reported but not executed by pull.
- This checkout has no registered `before_specify` hook, so creating the feature artifacts does not create or switch a Git branch.
