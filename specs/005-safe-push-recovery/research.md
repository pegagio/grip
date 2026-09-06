# Research: Safe Push and Recovery

This research resolves Feature 005 implementation choices without expanding into pull, sync, deletion, conflict resolution, backup cleanup, automatic rollback, state reconstruction, or final metadata fidelity.

## Table of Contents

- [Planning boundary](#planning-boundary)
- [Writer coordination](#writer-coordination)
- [Destination ancestry and parent creation](#destination-ancestry-and-parent-creation)
- [File staging and publication](#file-staging-and-publication)
- [Recovery preservation](#recovery-preservation)
- [Operation journal](#operation-journal)
- [Interrupted-operation coexistence](#interrupted-operation-coexistence)
- [Baseline publication](#baseline-publication)
- [Output failure](#output-failure)
- [Public result and error contract](#public-result-and-error-contract)
- [Testing and fault injection](#testing-and-fault-injection)
- [Performance](#performance)

## Planning boundary

**Decision**: Build a pure, mode-neutral `PushPlan` from Feature 004's complete observation and classification records. Only `source_addition` and `source_only_change` produce managed payload actions. `synchronized` produces a no-action disposition. Every other managed classification is retained as a non-action or blocker according to its existing blocking evidence; destination-only unmanaged entries remain outside ownership. The plan includes every selected entry and a separate dependency-ordered action sequence.

**Rationale**: Reusing the established observation and classifier preserves one definition of identity, equality, selection, and blockers. Separating entry dispositions from executable actions lets results explain the complete selected scope while synthetic parent actions remain auditable.

**Alternatives considered**: Reimplementing push-specific traversal would duplicate policy and race behavior. Treating rendered status output as planner input would discard typed identity and evidence. Planning only actionable entries would hide skipped or blocking evidence.

## Writer coordination

**Decision**: Add one stable owner-only advisory lock at `<grip-home>/.mutation.lock`. All Grip writers participate in the global order `mutation → registry → state`: actionful push holds mutation across locked revalidation, operation-journal initialization, payload execution, final observation, and baseline publication; mapping add/remove acquire mutation before registry publication; changed baseline acceptance acquires mutation before registry and state publication. Read-only commands, dry runs, blocked plans, and no-ops acquire no mutation lock.

The lock file persists. Kernel advisory-lock ownership, not file existence or recorded PID, determines contention. After acquisition, Grip replaces and syncs diagnostic owner metadata containing a schema version, process ID, operation, and acquisition time. Contention reports available owner metadata but never unlinks the lock, kills a process, or treats an unlocked file as held.

**Rationale**: A push-only lock cannot prevent mapping or baseline writers from invalidating a push while it mutates payloads. A distinct outer lock preserves the current narrow registry and state publication guards and gives every writer one deadlock-free acquisition order.

**Alternatives considered**: Reusing `state/state.lock` as the outer lock conflates long mutation ownership with accepted-state publication and requires mapping-only changes to create state directories. Per-mapping or per-entry locks add ordering complexity without a demonstrated concurrency need. Holding the lock during initial preflight serializes expensive read-only work unnecessarily.

## Destination ancestry and parent creation

**Decision**: Resolve and open destination ancestry component by component with descriptor-relative, no-follow operations. Missing parents become explicit synthetic actions linked to every dependent managed action. A parent action expects absence, creates exactly one directory with mode `0700`, reopens it as a no-follow directory, syncs its parent, and precedes descendants. Among dependency-ready actions, use canonical mapping source bytes, source-relative bytes, action-kind rank, and destination bytes as tie-breakers.

**Rationale**: Explicit parent actions make ownership and partial outcomes visible. Mode `0700` is a conservative usable default while independent directory metadata remains Feature 009 work. Exact component creation detects concurrent appearance rather than silently adopting changed evidence.

**Alternatives considered**: Recursive `create_dir_all` obscures individual mutations and can reopen changed ancestry. Copying source directory mode would pull independent directory metadata into this feature. A directories-first partition does not express dependencies shared across actions.

## File staging and publication

**Decision**: For each addition or replacement, create an attempt-owned hidden sibling staging file in the final destination parent with exclusive, no-follow, close-on-exec creation and initial mode `0600`. Stream the descriptor-bound source into staging while calculating SHA-256 and length, revalidate source metadata, apply the planned Unix mode, sync, reread, and verify the complete staged `SupportedState` plus descriptor/path identity.

Publish an addition with no-replace rename. Publish a replacement only after recovery preservation and immediate destination revalidation, using same-directory rename for atomic visibility. Sync the destination parent afterward. Report publication visibility, verification, and durability independently; a successful rename followed by directory-sync failure means the new entry is visible but durability is unconfirmed.

**Rationale**: Sibling staging guarantees the rename is on one filesystem. Descriptor-bound source and destination access preserves the repository's no-follow safety model while avoiding in-place partial writes.

**Alternatives considered**: `std::fs::copy`, composed-path reopening, and in-place truncation weaken ancestry or atomicity guarantees. Staging under Grip Home may cross filesystems. Rename is not compare-and-swap, so narrow final revalidation remains the supported boundary rather than a snapshot claim.

## Recovery preservation

**Decision**: Before replacement, stream the current destination through a no-follow descriptor into an exclusive private recovery staging file under `state/operations/<operation-id>/recovery/<action-index>/`. Store payload bytes as mode `0600` beneath mode `0700` directories, retain the original supported mode in typed metadata, sync and atomically publish the recovery payload, reopen it, and verify its SHA-256, length, and bound prior state before destination replacement.

Recovery references are repository-internal relative references derived from action indexes, never user paths. An addition records `prior_entry=false` and creates no fabricated backup. A collision with non-identical recovery evidence is a fail-closed operational or corrupt-state condition.

**Rationale**: Recovery is complete evidence before destructive replacement begins. Numeric action keys avoid unsafe path-derived storage. Cross-filesystem copying into private state is expected; only the later destination staging rename requires same-filesystem placement.

**Alternatives considered**: Moving the destination into recovery would cross filesystems and temporarily remove the authoritative path. Best-effort backups would violate the recovery gate. Automatic rollback remains excluded.

## Operation journal

**Decision**: Keep operation evidence separate from accepted State Envelope V2. Create a strict, integrity-checked partitioned Operation Record V1 under `state/operations/<operation-id>/`: immutable `plan.json` stores the complete lock-held plan once, bounded `operation.json` stores operation-level state, baseline outcome, failure, and result delivery, and `actions/<zero-padded-index>.json` stores the latest bounded checkpoint only for an action that has started. Absence of an action checkpoint means the immutable plan action remains `unattempted`. Use exclusive mode `0700` directories, mode `0600` files, same-directory staging, descriptor/path verification, atomic rename, and directory sync. Generate opaque IDs from time, process ID, and a process-local counter, but rely on exclusive directory creation and bounded collision retry for uniqueness.

After lock-held revalidation confirms the same actionful plan, publish the immutable plan and initial bounded summary immediately before the first payload mutation. Publish the current action's `in_progress` checkpoint before each side effect and replace only that action checkpoint after recovery, publication, verification, and terminal milestones. The first checkpoint failure stops execution; after a visible side effect, the last durable `in_progress` state remains truthful evidence. Terminal payload, baseline, and delivery changes replace only the bounded summary. Dry runs, blocked requests, and no-ops create no operation record.

**Rationale**: Separate operation evidence avoids changing the accepted-baseline schema and allows interrupted history to coexist with a later operation. Partitioning preserves strict atomic snapshots while serializing the complete plan once and only bounded evidence at each milestone, avoiding redundant quadratic serialization for large plans.

**Alternatives considered**: State Envelope V3 would couple accepted truth to operational history. A JSON-lines journal adds torn-tail framing and recovery rules. Rewriting one complete plan-and-action envelope at every milestone is simple but violates the constitutional prohibition on redundant serialization and becomes quadratic as action count grows. Precreating one file for every unattempted action adds unnecessary file-system fan-out; sparse action checkpoints retain linear work and derive `unattempted` from the immutable plan. Failure-only records cannot explain a crash before failure finalization.

## Interrupted-operation coexistence

**Decision**: Ordinary push does not enumerate or validate prior operation directories. A valid nonterminal operation is immutable evidence, not an active owner signal and not a blocker by itself. After fresh complete inspection and ordinary preflight, a later push allocates a new operation directory exclusively and never edits, resumes, completes, rolls back, or deletes any prior operation or recovery entry. Strict validation is limited to the newly allocated live record and to a specifically selected record when a later read-only inspection feature requests one.

**Rationale**: Advisory-lock ownership proves whether another process is active. Exclusive new-directory allocation preserves prior history without making common planning traverse an unbounded retained log. Treating record state as ownership would permanently disable mutation until Feature 008 adds recovery commands, while automatic resume would execute stale evidence.

**Alternatives considered**: Mandatory recovery before any future push expands Feature 008 into this feature. Automatic resume and rollback contradict the clarified contract. Scanning every retained operation on every push makes planning cost grow with unrelated history. A specifically requested record still fails closed if its operation or recovery evidence is corrupt.

## Baseline publication

**Decision**: After every action verifies, repeat the complete selected observation under the mutation lock and require the actioned identities to have complete equivalent supported source and destination state. Acquire registry then state guards, revalidate the expected registry and accepted-state snapshots, build a copy-on-write accepted baseline that updates only successfully actioned identities, preserves synchronized and out-of-scope records, and reuse `state::publication::publish_accepted_locked`.

Failure before accepted-state rename leaves the prior baseline authoritative. Rename success followed by state-directory sync failure reports the candidate visible with durability unconfirmed; it must not falsely claim the prior baseline remains visible. A no-op or partial operation never invokes baseline publication.

**Rationale**: Reusing State V2 publication preserves generation, integrity, recovery, and retry semantics. Updating only actioned identities avoids silently accepting initial matches, converged edits, or other classifications that Feature 005 does not own.

**Alternatives considered**: Accepting every equivalent entry in scope would make push an implicit baseline-accept command. Publishing after each action would falsely accept partial operations. Adding operation records to State V2 conflates distinct authority.

## Output failure

**Decision**: Extend process orchestration so an actionful push outcome carries an internal operation receipt. Before rendering, checkpoint the payload and baseline terminal outcome with result delivery `prepared`. If rendering fails, best-effort checkpoint result delivery `failed`, retain any published baseline as authoritative, write the existing concise stderr diagnostic, and exit `20`. A successful render may checkpoint `delivered`; inability to record that post-output fact cannot retroactively turn already-emitted success into failure.

**Rationale**: Filesystem publication and stdout cannot be atomic. Recording the durable operation outcome before output and treating delivery separately preserves truth across the unavoidable crash window.

**Alternatives considered**: Publishing the baseline after output would emit success before acceptance is durable. Rolling back a baseline after output failure contradicts the clarified contract. Ignoring render failure would leave automation with a false success exit.

## Public result and error contract

**Decision**: Preserve Result Envelope V1 and add typed push details with stable mode, completion, result, scope, plan identity, counts, entry dispositions, ordered actions, blockers, operation-record availability, baseline outcome, publication visibility, verification, durability, and recovery state. Expose only an opaque operation ID, never private storage paths. Human output is derived from the same type.

Use exit `0` for completed preview, no-op, and fully accepted execution; `2` for grammar errors; `10` for invalid configuration or a completely discovered blocked plan; `11` for unsupported schemas; `12` for corrupt private state; `13` for mutation contention; and `20` for stale evidence or operational, publication, verification, journal, or output failure. Exit `1` remains exclusive to a completed `check` requiring attention.

**Rationale**: Stable typed results keep automation separate from diagnostics and avoid adding an unnecessary exit category.

**Alternatives considered**: Free-form result maps repeat the current mapping-specific error limitations. Returning exit `1` for blockers conflicts with Feature 004. Exposing recovery paths would create a public storage-layout contract.

## Testing and fault injection

**Decision**: Add pure planning tests and isolated CLI, filesystem, recovery, failure, contention, and operation-journal suites. Extend test support with strict operation/recovery decoders and complete snapshots. Use typed `#[doc(hidden)]` fault hooks at preflight, lock, revalidation, journal, parent creation, source streaming, staging, recovery, destination publication, directory sync, final verification, baseline publication, and result delivery boundaries. Reuse existing state-publication fault cases.

**Rationale**: Typed deterministic seams reproduce narrow race and failure boundaries without sleeps, environment-only backdoors, or real user files.

**Alternatives considered**: Timing-dependent race tests are flaky. Environment variables risk production activation. A trait hierarchy is unnecessary unless the focused fault enum becomes unmanageable.

## Performance

**Decision**: Extend the ignored release harness with exactly 10,000 eligible entries: 100 directories, 3,300 source additions, 3,300 source-only replacements, and 3,300 no-action files. Warm and measure 100 JSON dry-run planning invocations and 100 internal execute-mode planning invocations stopped before coordination or mutation. Require equivalent semantic plans, byte-identical repeated dry-run output, unchanged state snapshots, and a sorted dry-run p95 of at most two seconds; record the execution-planning measurements without imposing a second threshold. Report OS, architecture, Rust version, profile, mix, and both planning distributions. Do not execute payload mutations in the timed loop.

**Rationale**: Dry run exercises the same mode-neutral inspection and planner without fixture-reset noise. The mix measures the new action and dependency model while keeping the existing representative scale.

**Alternatives considered**: Timing repeated real mutation measures reset strategy more than planning. Adding cache, indexing, parallel traversal, or another dependency before a failing profile would violate proportional rigor.
