# Push Contract: Planning and Execution

This contract defines the typed planner and execution state machine. Filesystem mechanics are specified separately in [the filesystem contract](filesystem.md), and durable records in [the storage contract](storage.md).

## Classification disposition matrix

| Classification | Entry disposition | Payload action | Push effect |
|---|---|---|---|
| `source_addition` | action | create directory or add file | Copy supported source state to absent destination |
| `source_only_change` | action | replace file | Preserve prior destination, then publish supported source state |
| `synchronized` | no_action | none | Preserve accepted baseline |
| `destination_only_unmanaged` | outside managed entries | none | Leave untouched |
| `divergent_conflict` | blocked | none | Block complete plan |
| `delete_change_conflict` | blocked | none | Block complete plan |
| `change_delete_conflict` | blocked | none | Block complete plan |
| `unsupported_managed` | blocked | none | Block complete plan |
| `unsafe_collision` | blocked | none | Block complete plan |
| every other classification | no_action | none | Report reason; do not accept or mutate it |

Any inherited record with `blocking: true` blocks regardless of the default row above. Every selected managed entry appears exactly once in `entries`.

## Synthetic parent actions

For each managed action whose destination parent is absent, the planner creates one synthetic action for each missing directory component within the accepted destination ancestry. Equivalent synthetic parents required by multiple managed actions are deduplicated and reference every dependent identity.

Synthetic parents are not managed baseline entries. Their creation and terminal state appear in the action list and partitioned operation record so partial outcomes remain auditable.

## Plan identity

`plan_id` is the lowercase SHA-256 digest of deterministic compact serialization containing:

1. resolved scope;
2. exact accepted registry evidence identity;
3. exact accepted-state evidence identity and generation;
4. canonically ordered entry dispositions;
5. dependency-ordered actions and expected supported states;
6. complete blockers.

Request mode, message text, timestamps, operation ID, and mutable execution statuses are excluded. Equivalent dry-run and execution planning therefore produce the same plan identity.

## Ordering

Actions form a dependency graph. Planning rejects cycles as an internal invariant failure. The stable topological ordering chooses the smallest available action by:

1. canonical mapping source raw bytes;
2. source-relative raw bytes;
3. action kind rank: parent directory, managed directory, file addition, file replacement;
4. destination raw bytes.

Every dependency index is lower than its dependent action index.

## Execution sequence

1. Load and validate the complete registry, accepted state, selection, observation, and classifications without scanning unrelated retained operation history.
2. Build the initial pure plan.
3. Return immediately for blocked, no-op, or dry-run results without acquiring writer coordination or writing state.
4. Acquire the per-user mutation lock.
5. Reload registry, state, selection, observations, classifications, and plan. Require semantic equality with the initial plan.
6. Create and sync the private operation directory, immutable complete plan, and bounded initial operation summary. An action without a checkpoint is `unattempted`.
7. For each action in order:
   1. checkpoint `in_progress`;
   2. revalidate its current source, destination, ancestry, policy, mapping, baseline, and plan assumptions;
   3. perform each applicable recovery, staging, publication, durability, and verification milestone with a durable checkpoint;
   4. checkpoint `completed`.
8. Stop at the first failure, checkpoint it when possible, leave later actions `unattempted`, preserve all visible changes and recovery evidence, and do not invoke baseline publication.
9. After every action completes, repeat complete selected observation under the mutation lock. Require every actioned identity to be complete and equivalent to its source plan state.
10. Acquire registry then state publication guards, revalidate expected snapshots, and publish one copy-on-write State V2 generation updating only actioned identities.
11. Checkpoint the terminal payload and baseline outcome, then prepare result delivery.
12. Release locks after baseline and operation-record publication are complete; render from the typed result.

## Global lock order

Every writer follows:

```text
mutation lock -> registry lock -> state lock
```

Push holds the mutation lock across steps 5 through 11 and takes inner locks only for final metadata publication. Mapping add/remove and changed baseline acceptance participate in the same outer order. Read-only commands and semantic no-ops take none of these locks unless their existing narrow publication behavior requires a write.

## Failure semantics

- Preflight blockers accumulate completely and start zero actions.
- Lock-held plan drift creates no operation record or payload change.
- A checkpoint failure before its associated side effect stops without that side effect.
- A checkpoint failure after publication visibility leaves the last durable state, visible payload, and recovery evidence unchanged; later inspection establishes current filesystem truth.
- The first action failure stops execution and baseline publication.
- Completed actions are never automatically rolled back.
- Baseline failure before rename leaves the prior generation authoritative.
- Baseline visibility with unconfirmed directory durability reports the candidate visible and never claims the prior generation is visible.
- Output failure changes only result-delivery evidence and process exit; it does not change payload or baseline authority.

## Interrupted records

A valid nonterminal operation record is preserved as immutable historical evidence. It does not prove active ownership and does not block a later push by itself. A later push must acquire the advisory mutation lock, perform fresh complete inspection, build a separate plan, and create a new operation ID. It never scans, resumes, or edits unrelated prior records during ordinary planning or execution.

Corrupt or unsafe operation/recovery state remains a fail-closed state error when the current operation or a specifically requested historical record encounters it; unrelated retained history is not an ordinary push prerequisite.
