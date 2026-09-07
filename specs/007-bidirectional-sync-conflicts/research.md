# Research: Bidirectional Synchronization and Conflict Resolution

This research resolves Feature 007's implementation choices against the verified push/pull mutation core. No technical clarification remains open.

## Table of Contents

- [Operation and transfer direction](#operation-and-transfer-direction)
- [Unified sync planning](#unified-sync-planning)
- [Converged baseline acceptance](#converged-baseline-acceptance)
- [Exact conflict resolution](#exact-conflict-resolution)
- [Execution and failure behavior](#execution-and-failure-behavior)
- [Operation-record compatibility](#operation-record-compatibility)
- [Public results](#public-results)
- [Testing and performance](#testing-and-performance)

## Operation and transfer direction

**Decision**: Add a closed invoked-operation value (`push`, `pull`, `sync`, `resolve`) while retaining the two-value transfer direction (`push`, `pull`). Every action carries its transfer direction; plans and results carry the invoked operation. Directional plans retain their single direction for compatibility, while mixed sync plans derive behavior from each action.

**Rationale**: `sync` is an operation, not a third direction. Resolution is also an operation whose transfer direction depends on the selected winner. Separating these concepts prevents mapping-side source/destination fields from being overloaded and lets the shared executor choose the correct origin, target, recovery side, and diagnostics per action.

**Alternatives considered**:

- Add `sync` and `resolve` to `MutationDirection`: conflates command identity with byte-transfer direction.
- Split one sync request into independent push and pull operations: violates complete-scope conflict preflight and one accepted baseline publication.
- Create command-specific executors: duplicates the safety-critical transaction.

## Unified sync planning

**Decision**: Build one plan from one sorted classification set. Nonblocking source additions and source-only changes produce push actions; nonblocking destination-only changes produce pull actions; synchronized and destination-only unmanaged entries are reported no-actions; converged changes are marked for acceptance without payload action; every inherited blocker blocks the entire plan. Preserve existing push parent-directory action construction and attach dependencies before canonical entry actions.

**Rationale**: A single policy pass guarantees every selected record appears once and all blockers are known before any action starts. Canonical entry identity, not transfer direction, determines mixed action ordering, so input order and directional grouping cannot change the plan.

**Alternatives considered**:

- Concatenate separately built push and pull plans: duplicates entries, produces competing plan identities, and can miss cross-plan blocking.
- Execute all push actions before pulls: makes partial outcomes depend on direction rather than canonical identity.
- Continue around conflicts: explicitly excluded by the roadmap and spec.

## Converged baseline acceptance

**Decision**: Track an ordered acceptance set in the plan containing actioned identities and selected `converged_two_sided_change` identities. After every payload action verifies, re-observe the complete scope and build the next baseline from that acceptance set. A plan containing only converged changes is actionful for state acceptance: it acquires coordination, creates an operation record, performs no payload action, and publishes one baseline generation. A scope containing only synchronized entries remains a semantic no-op.

**Rationale**: Converged changes differ from the accepted baseline and must be recorded to satisfy the spec, but they require no payload replacement. Treating acceptance work explicitly avoids hiding state mutation behind a zero-action result and keeps blocked or partially failed syncs from accepting convergence.

**Alternatives considered**:

- Leave converged entries pending explicit `baseline accept`: contradicts Feature 007's accepted sync result.
- Publish converged entries before payload actions: can falsely accept a scope whose later action fails.
- Treat converged-only sync as no-op: leaves status reporting unresolved drift.

## Exact conflict resolution

**Decision**: Parse exactly one source-space path and exactly one of `--source` or `--destination`. Freshly resolve it to one established Entry Identity, inspect only that entry after complete registry validation, and require `divergent_conflict`. Source winner creates a push replacement of the destination; destination winner creates a pull replacement of the source. Both preview aliases build the identical one-action plan without coordination or state changes.

**Rationale**: Canonical source path is already mapping identity. Explicit winner flags keep selection independent from direction and allow stale or no-longer-conflicting requests to fail before mutation. The existing transfer pipeline already preserves and replaces the correct losing target.

**Alternatives considered**:

- Infer winner or path space: rejected during clarification and conflicts with deterministic selection.
- Permit mapping or subtree resolution: could overwrite multiple conflicts from one decision.
- Store a prior conflict token: unnecessary because each invocation performs fresh inspection.

## Execution and failure behavior

**Decision**: Generalize the executor around invoked operation and per-action direction. Rebuild the same operation-specific plan under the mutation lock, require semantic equality, initialize one operation record immediately before the first payload or converged-state mutation, and execute actions in canonical order. Revalidation, staging, recovery, publication, verification, stop-after-first-failure, final observation, and baseline publication remain unchanged.

**Rationale**: The current executor already encodes the correct safety transaction. Selecting direction per action is the smallest extension that supports mixed plans and resolution without weakening recovery or failure evidence.

**Alternatives considered**:

- Hold separate locks for each direction: permits interleaving and stale baseline publication.
- Roll back earlier actions after failure: rollback may fail and obscure evidence.
- Add filesystem-wide locking: constitutionally disproportionate.

## Operation-record compatibility

**Decision**: Preserve Partitioned Operation Record V1 and its layout. Expand the allowed operation vocabulary to `sync` and `resolve`; validate new plans by their explicit operation and action directions; continue accepting historical push/pull plans whose top-level identity is represented by `direction`. Do not rewrite retained records or add a migration.

**Rationale**: V1 stores the plan as integrity-protected JSON, so its envelope can safely admit additive operation values and fields while retaining strict semantic validation. A new envelope version or history tree would add complexity without changing persistence guarantees.

**Alternatives considered**:

- Operation Record V2: unnecessary schema churn for additive command identity.
- Rewrite old records: violates immutable evidence.
- Separate sync/resolve directories: fragments inspection and recovery binding.

## Public results

**Decision**: Keep Result Envelope V1. Every mutation result exposes `operation`; push/pull retain their plan-level `direction`; sync exposes per-action direction and a bidirectional operation; resolve exposes `winner` plus its one action direction. Plans distinguish payload actions, converged acceptance entries, blockers, and semantic no-actions. Stable categories and exit codes remain inherited.

**Rationale**: Automation needs to distinguish the invoked workflow from each transfer without a new outer envelope. Existing mapping-role paths remain stable and recovery/baseline fields already express authority.

**Alternatives considered**:

- Use `direction: sync`: semantically false for individual transfers.
- Define separate result envelopes: forces unnecessary consumer branching.
- Encode direction only in messages: violates structured-output governance.

## Testing and performance

**Decision**: Add exhaustive policy tables for all 18 classifications, deterministic mixed-plan tests, both winner paths, converged-only acceptance, legacy/new record validation, and isolated CLI/filesystem/recovery/contention/failure/output tests. Extend the representative harness to 10,000 entries mixed across synchronized, source-only, destination-only, converged, and conflicting classifications; measure 100 warm preview and pure planning samples.

**Rationale**: The new risks are policy composition, per-action direction, and baseline acceptance, not new filesystem primitives. Existing push/pull regression suites protect shared behavior while focused mixed tests prove the new contracts.

**Alternatives considered**:

- Duplicate all directional suites for sync: high maintenance with limited new evidence.
- Add caching or parallelism before measurement: unsupported complexity.
- Test resolution only through unit planners: misses recovery and stale filesystem behavior.
