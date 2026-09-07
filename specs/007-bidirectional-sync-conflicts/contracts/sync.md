# Domain Contract: Bidirectional Sync

## Disposition policy

Any inherited blocking record is blocked. Otherwise:

| Classification | Disposition | Meaning |
|---|---|---|
| `source_addition` | push action | Create eligible destination state |
| `source_only_change` | push action | Replace destination from source |
| `destination_only_change` | pull action | Replace source from destination |
| `converged_two_sided_change` | accept-only | Publish no payload; accept after complete success |
| `synchronized` | no action | Preserve accepted state |
| `destination_only_unmanaged` | no action | Report; never import |
| other nonblocking initial states | no action | Not eligible for sync |
| conflict, deletion, retirement, unsupported, unsafe | blocked | Separate workflow or correction required |

## Plan invariants

- Every selected classification appears exactly once.
- All blockers are collected before execution.
- Every payload action has exactly one transfer direction.
- Mixed actions sort by canonical Entry Identity, with existing parent-before-child dependencies preserved.
- Direction participates in each action and plan identity but does not alter mapping-role paths.
- Converged acceptance identities are explicit and cannot be published separately from the complete successful plan.
- Blocked, preview, and synchronized-only plans acquire no mutation lock and create no operation record.

## Execution sequence

1. Load and validate registry and State V2, resolve scope, inspect, classify, and build one complete plan.
2. Return blocked, no-op, or preview results without mutation when applicable.
3. Acquire the mutation lock as `sync`, reload evidence, rebuild the same plan, and require equality.
4. Initialize one operation record immediately before payload or converged-state mutation.
5. Execute actions in canonical order, selecting origin, target, staging, recovery, and verification from each action direction.
6. Stop after the first failure; retain completed effects and recovery; leave later actions unattempted.
7. Re-observe the complete selected scope and verify every actioned and converged acceptance identity.
8. Acquire registry then state guards and publish one accepted State V2 generation.
9. Checkpoint baseline and result-delivery evidence and render the typed result.

## Exclusions

Sync never resolves a conflict automatically, continues around known blockers, imports destination-only content, executes deletion or retirement, adopts an initial collision, merges content or metadata, cleans recovery, rolls back completed work, repairs state, elevates privileges, invokes Git, follows links, or claims snapshot isolation.
