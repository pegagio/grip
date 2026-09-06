# Feature Specification: Safe Push and Recovery

**Feature Branch**: `005-safe-push-recovery`

**Created**: 2026-09-06

**Status**: Complete

**Input**: User description: "Specify roadmap entry 005, Safe Push and Recovery, using the roadmap and wiki-backed governing decisions and constraints."

## Clarifications

### Session 2026-09-06

- Q: If final output delivery fails after every payload action and baseline publication have succeeded, should Grip retain the accepted baseline or treat the operation as unaccepted? → A: Retain the published baseline, report an operational error, and preserve the durable operation record.
- Q: During a mutating push, when should Grip hold its per-user mutation lock? → A: Acquire it after preflight, revalidate under it, and hold it through payload execution and baseline publication.
- Q: Which push invocations should create a durable operation record? → A: Create one immediately before the first payload mutation and update it through the terminal outcome; previews, blocked requests, and no-ops create none.
- Q: How should Grip order push actions when parent directories must exist before descendant entries can be published? → A: Order by dependency first, then by canonical mapping and source-relative path identity.
- Q: If a process crash leaves a nonterminal operation record, should a later push be allowed to proceed after a fresh complete inspection? → A: Allow a separately planned push after fresh complete inspection and normal preflight; preserve the interrupted record and recovery entries unchanged.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Preview a Safe Push (Priority: P1)

As a local user, I can preview every eligible source-to-destination action in a selected scope and see all reasons the push would be blocked without changing payloads or Grip-owned state.

**Why this priority**: A trustworthy preview is the user's primary safety boundary before Grip begins changing managed files.

**Independent Test**: Build an isolated scope containing additions, source-only changes, synchronized entries, conflicts, unsupported nodes, and unmanaged destination entries; run `grip push --dry-run`; verify the complete deterministic plan and zero mutations.

**Acceptance Scenarios**:

1. **Given** eligible source additions and source-only changes, **When** the user runs `grip push --dry-run`, **Then** Grip reports the ordered actions it would perform and changes no payload, baseline, backup, registry, or lock state.
2. **Given** a conflict or known unsafe condition anywhere in scope, **When** preview completes, **Then** Grip reports every discovered blocker and states that no action is executable.
3. **Given** unchanged evidence, **When** preview is repeated or invoked with `-n`, **Then** each result describes an equivalent plan in deterministic order.

---

### User Story 2 - Push Source Changes Safely (Priority: P2)

As a local user, I can push eligible source additions and changes to their managed destinations with stale-evidence detection, recovery preservation, result verification, and truthful baseline publication.

**Why this priority**: This is Grip's first payload-mutating capability and establishes the safety contract reused by later mutation directions.

**Independent Test**: Create isolated mappings with source additions and source-only changes, run `grip push`, and verify the destination, recovery evidence for replacements, action results, and accepted baseline all describe the same successfully verified outcome.

**Acceptance Scenarios**:

1. **Given** a complete unblocked plan containing a new eligible source entry, **When** the user runs `grip push`, **Then** Grip creates and verifies the destination entry and publishes the accepted resulting baseline.
2. **Given** a source-only change to an existing destination entry, **When** push succeeds, **Then** Grip preserves the prior destination in the operation recovery namespace, installs and verifies the source state, and publishes the accepted resulting baseline.
3. **Given** relevant evidence changes after planning but before an action is applied, **When** execution revalidates that action, **Then** Grip stops without applying that action and publishes no new baseline.
4. **Given** every selected entry is already synchronized, **When** the user runs push, **Then** Grip succeeds as a reported no-op without creating recovery evidence or publishing a new baseline generation.

---

### User Story 3 - Understand and Recover From Failure (Priority: P3)

As a local user, I receive a precise account of completed, failed, and unattempted actions when a push fails, while Grip preserves prior and replacement evidence and does not falsely accept a partial result.

**Why this priority**: Partial mutation is unavoidable on ordinary filesystems; trustworthy evidence and an unmodified baseline let the user inspect and recover without Grip hiding what happened.

**Independent Test**: Inject a failure after at least one successful action in an isolated multi-entry push and verify that execution stops immediately, completed filesystem changes remain verifiable, recovery references remain intact, later actions are unattempted, and the prior baseline stays authoritative.

**Acceptance Scenarios**:

1. **Given** a multi-entry plan whose next action fails after earlier actions completed, **When** the failure occurs, **Then** Grip stops before later actions, reports each action as completed, failed, or unattempted, and publishes no new baseline.
2. **Given** an existing destination is about to be replaced, **When** staging or publication fails, **Then** Grip reports whether the authoritative destination changed and preserves any recovery evidence already created.
3. **Given** a prior operation failed partially, **When** the user runs read-only inspection, **Then** the unchanged accepted baseline enables the resulting filesystem state to be classified rather than represented as accepted convergence.
4. **Given** a process crash left a nonterminal operation record, **When** a later push performs a fresh complete inspection and passes ordinary preflight, **Then** Grip may execute the new plan while preserving the interrupted record and its recovery entries unchanged.

---

### User Story 4 - Use Push Results in Automation (Priority: P4)

As a script author, I can consume stable machine-readable preview and execution results that distinguish blocked plans, no-op success, complete success, stale evidence, contention, partial failure, and output failure.

**Why this priority**: Automation must never mistake a rendered plan, partial mutation, or failed result delivery for an accepted push.

**Independent Test**: Exercise every terminal push outcome in isolated fixtures and verify stable result categories, action records, exit behavior, and parity with human-readable results.

**Acceptance Scenarios**:

1. **Given** a blocked preview or execution request, **When** machine-readable output is requested, **Then** it identifies every preflight blocker and confirms that zero actions started.
2. **Given** a partially failed push, **When** machine-readable output is produced, **Then** it identifies the failed action, completed actions, unattempted actions, recovery references, baseline non-publication, and whether any destination became visible.
3. **Given** a successful push, **When** human and machine-readable output are requested separately over equivalent starting evidence, **Then** both communicate equivalent actions and acceptance outcome while diagnostics remain separate.
4. **Given** every payload action and baseline publication succeeded but final output delivery fails, **When** the command terminates, **Then** Grip retains the accepted baseline, returns an operational error, and preserves a durable operation record for later inspection.

### Edge Cases

- The selected scope is empty, already synchronized, names a deleted source, names an unmanaged or ignored path, or begins with `-`.
- A parent destination directory is absent, replaced, non-directory, inaccessible, or outside the accepted mapping ancestry.
- Source content, supported metadata, mapping identity, membership, ignore policy, destination ancestry, destination state, or accepted baseline changes between inspection and action.
- A staged candidate cannot be created on the destination filesystem, fully prepared, verified, or installed with the strongest supported atomic replacement.
- The destination is absent for an addition, disappears before replacement, or appears concurrently where absence was required.
- An existing destination is a symbolic link, hard-linked file, sparse file, special node, nested mount, wrong node kind, or otherwise unsupported.
- Recovery material cannot be created or verified before replacement, or the recovery namespace is unsafe, unavailable, or contended.
- An action succeeds but result verification fails; publication visibility changes but durability cannot be confirmed; or baseline publication fails after every payload action verifies.
- Human or machine-readable output fails after zero, some, or all payload actions have completed.
- Multiple Grip processes attempt payload or baseline mutation concurrently.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST expose `grip push [PATH]` as a source-to-destination action that mutates by default, while `-n` and `--dry-run` MUST build and render the same plan without mutation.
- **FR-002**: Push MUST reuse the established single-selector contract: source-space interpretation by default, destination-space interpretation with `--destination`, and `--` as option termination.
- **FR-003**: Before any action begins, Grip MUST validate the complete mapping registry and completely inspect and classify the selected scope against current source, current destination, and the accepted baseline.
- **FR-004**: A push plan MUST be a complete deterministic ordered set of proposed actions derived from one validated inspection, with an explicit outcome for every selected managed entry. An action that creates a required parent directory MUST precede actions for its descendants; all other ordering ties MUST follow canonical mapping source and source-relative raw path identity.
- **FR-005**: Push MUST plan only eligible source additions and source-only changes. Synchronized entries MUST require no action.
- **FR-006**: Initial matches, initial collisions, destination-only changes, divergent changes, converged two-sided changes awaiting acceptance, directional deletions, delete/change conflicts, pending-retirement entries, and unsupported or unsafe managed evidence MUST NOT be mutated by this feature.
- **FR-007**: Destination-only entries outside the managed namespace MUST remain unmanaged and untouched, including when their names are adjacent to planned destinations.
- **FR-008**: Any conflict, unsafe path, unsupported managed entry, invalid ownership, incomplete inspection, unavailable required metadata transition, corrupt required state, or other known blocking condition in the selected scope MUST block the entire push before its first mutation.
- **FR-009**: Dry run MUST report the same ordered actions, blockers, and relevant evidence as an equivalent mutating invocation over unchanged state, and MUST NOT change payloads, registry data, baselines, recovery data, or coordination state.
- **FR-010**: Immediately before each action, Grip MUST revalidate the mapping, membership and ignore policy, source evidence, destination evidence, destination ancestry, accepted baseline, and plan assumptions whose change could make that action unsafe or different.
- **FR-011**: Any failed revalidation MUST stop execution before the affected action, mark every later action unattempted, and prevent publication of a new baseline.
- **FR-012**: Grip MUST create absent destination parent directories only within the accepted mapped destination ancestry and only when the complete plan proves those directories eligible and safe.
- **FR-013**: Replacement content and supported metadata MUST be fully staged and verified on the destination filesystem before publication wherever the supported filesystem permits.
- **FR-014**: Grip MUST use the strongest supported atomic publication operation for installing a staged entry and MUST report when visibility changed but durability could not be confirmed.
- **FR-015**: Before replacing an existing managed destination, Grip MUST preserve its prior supported state in a per-operation recovery namespace and verify the recovery copy before publishing the replacement.
- **FR-016**: A new destination addition with no prior destination payload MUST NOT create a fabricated backup; its action record MUST state that no prior entry existed.
- **FR-017**: Recovery evidence MUST bind the operation, managed entry identity, prior-state fingerprint, and resulting action record sufficiently for later inspection and cleanup, without making backup inspection, removal, or automatic rollback part of this feature.
- **FR-018**: Push MUST verify every published destination against the planned complete supported source state before treating that action as completed.
- **FR-019**: After execution begins, the first staging, recovery, publication, verification, coordination, or state-publication failure MUST stop the operation. Completed actions MUST remain in place; automatic rollback MUST NOT be attempted.
- **FR-020**: A failed push MUST preserve all recovery material already created, identify the first failed action and reason, distinguish completed, failed, and unattempted actions, and report whether each attempted destination became visible and verified.
- **FR-021**: Grip MUST publish a new accepted baseline only after every planned payload action completes and verifies successfully and final acceptance-relevant revalidation confirms the complete selected result.
- **FR-022**: Baseline publication after a scoped successful push MUST update accepted records only for identities whose planned payload actions completed and verified successfully. It MUST preserve selected no-action records, all out-of-scope baseline records, and all pending-retirement records.
- **FR-023**: A blocked, stale, contended, partially failed, or verification-failed push MUST NOT invoke baseline publication. If baseline publication fails before a new generation becomes visible, Grip MUST report that payloads changed but the prior baseline remains authoritative. If the new generation becomes visible but its durability cannot be confirmed, Grip MUST report that visible generation as authoritative with durability unconfirmed rather than claim that the prior generation remains authoritative. If final output delivery fails after payload verification and baseline publication succeeded, the published baseline MUST remain authoritative and the command MUST return an operational error.
- **FR-024**: An unblocked plan with no actions MUST succeed as a no-op without creating recovery evidence or publishing a new state generation.
- **FR-025**: A mutating push MUST acquire one short-lived per-user mutation lock after completing read-only preflight, revalidate all relevant evidence while holding the lock, and retain the lock through payload execution and baseline publication. Existing mapping and baseline commands that would publish a change MUST participate in the same outer coordination boundary so they cannot interleave with an executing push; their semantic no-ops and every read-only command MUST remain lock-free. Dry runs and read-only push preflight MUST NOT acquire this lock. Contention MUST fail safely with actionable owner and stale-lock information, and the lock MUST NOT become a payload-tree filesystem lock.
- **FR-026**: Push MUST NOT elevate privileges, invoke version-control operations, follow symbolic links, open unsupported nodes as payload, weaken required metadata silently, or claim filesystem snapshot isolation.
- **FR-027**: Machine-readable results MUST use stable versioned fields for operation mode, selected scope, plan identity, completion state, ordered action records, blockers, stale evidence, recovery references, publication and verification state, baseline outcome, and error category.
- **FR-028**: Human-readable results MUST communicate the same planned or attempted actions, blockers, recovery availability, partial-failure boundary, and baseline outcome as machine-readable results. Diagnostics MUST remain separate.
- **FR-029**: Immediately before the first payload mutation, Grip MUST durably initialize an operation record, checkpoint the relevant last-known state before each side effect, and checkpoint each successfully recorded milestone through the terminal payload and baseline outcome. If a checkpoint itself fails, Grip MUST stop and retain the last durable state; after final output delivery fails, it MUST make a best-effort delivery-failure checkpoint without changing payload or baseline authority. The record MUST be sufficient for a later read-only command to identify what was attempted, each action's last known status, whether baseline publication occurred, and whether the published baseline remains authoritative. Dry runs, blocked requests, and no-op pushes MUST NOT create an operation record. A nonterminal record left by process interruption MUST remain immutable evidence and MUST NOT by itself permanently block a later push; the later push MUST derive a separate plan from a fresh complete inspection, pass ordinary preflight, and preserve the interrupted record and its recovery entries unchanged. Ordinary push planning and execution MUST NOT require scanning or validating unrelated retained operation records or recovery entries. Grip MUST NOT silently resume, complete, or roll back the interrupted operation.
- **FR-030**: This feature MUST NOT perform destination-to-source propagation, bidirectional synchronization, conflict resolution, deletion, retirement, backup cleanup, automatic rollback, or repair/reconstruction of missing, unreadable, corrupt, or inconsistent registry or baseline state.
- **FR-031**: Push behavior MUST be covered by automated tests using isolated temporary roots, including complete preflight blocking, deterministic dry-run parity, safe addition and replacement, parent creation, stale evidence, contention, recovery failure, injected partial failure, verification failure, baseline non-publication, output failure, and non-mutation guarantees.
- **FR-032**: A documented representative workload of 10,000 eligible entries with a mixture of no-op, addition, and replacement actions MUST measure preview and execution planning, and any cache, index, parallel execution, or additional coordination mechanism MUST be justified by those measurements before introduction.

### Key Entities

- **Push Plan**: Complete deterministic evidence for one selected scope, containing ordered source-to-destination actions, no-action records, blockers, and the observations each action must revalidate.
- **Push Action**: One planned addition or replacement for a managed entry, including expected source state, expected destination state, staging requirements, and terminal execution status.
- **Recovery Entry**: Verified prior destination state retained for one replacement and bound to the operation and managed entry; it is evidence for later user-directed recovery rather than an automatic rollback instruction.
- **Operation Record**: Durable outcome evidence for one mutating invocation, including completed, failed, and unattempted actions; recovery references; visibility and verification results; and baseline-publication outcome.
- **Accepted Push Result**: A fully applied and verified selected scope whose new baseline has been successfully published.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of conformance fixtures, dry run and mutating push over equivalent unchanged starting evidence produce equivalent ordered plans, while dry run produces zero payload or Grip-state changes.
- **SC-002**: Every fixture containing a known conflict, unsafe condition, unsupported managed entry, or incomplete required observation reports all known blockers and starts zero actions.
- **SC-003**: Across 100% of eligible addition and replacement fixtures, a successful push leaves each destination equivalent to the complete supported source state, retains verified prior state for every replacement, and publishes matching accepted baseline evidence.
- **SC-004**: Across injected drift points before every action phase, Grip detects changed relevant evidence before the affected publication, stops, and publishes no new baseline in 100% of cases.
- **SC-005**: Across injected first failures at every staging, recovery, publication, verification, and baseline-publication phase, the result identifies every action as completed, failed, or unattempted; preserves all created recovery evidence; and never represents partial work as an accepted baseline.
- **SC-006**: In 100% of no-action fixtures, push reports successful no-op completion and creates no recovery entry or baseline generation.
- **SC-007**: Human and machine-readable results identify the same actions, blockers, recovery availability, failure boundary, and baseline outcome in 100% of paired output tests.
- **SC-008**: For a documented representative scope of 10,000 eligible entries, at least 95 of 100 dry-run planning invocations complete within two seconds on a documented representative local workstation without requiring a persistent cache, background service, or broad payload lock.
- **SC-009**: From the push result plus a subsequent read-only status, a user can identify every changed destination, the still-authoritative baseline, and available recovery evidence after every tested partial-failure scenario without inspecting Grip's internal files directly.

## Assumptions

- Features 001 through 004 are verified, and their command, mapping, discovery, classification, selector, output, baseline, supported-equality, and state-publication contracts remain authoritative.
- Feature 005 resolves Q-10 with stop-after-first-operational-failure behavior once execution begins. Complete preflight still accumulates and reports all known blockers before mutation.
- Feature 005 resolves only the creation, preservation, binding, and reporting portion of Q-11. Backup inspection and removal, automatic or user-directed recovery commands, and repair of missing or corrupt registry or baseline state remain Feature 008 work.
- Automatic rollback is excluded because rollback can fail and obscure the authoritative evidence needed for recovery.
- The first payload contract remains ordinary regular files and directories with regular-file byte content and Unix permission mode; broader metadata and filesystem behavior remain Feature 009 work.
- This checkout has no registered `before_specify` hook, so creating the feature artifacts does not create or switch a Git branch.
