# Storage Contract: Sync and Resolve Evidence

Feature 007 preserves State Envelope V2, Recovery Metadata V1, and the Partitioned Operation Record V1 layout.

## Operation vocabulary

New records admit `operation: sync` and `operation: resolve` in addition to `push` and `pull`. New plan JSON carries the matching operation, per-action direction, and optional resolve winner. Historical V1 records with top-level `direction: push|pull` remain valid and are never rewritten.

Semantic validation rejects unknown operations, operation/plan mismatch, missing or invalid action direction, resolve without exactly one winner, winner/direction mismatch, non-dense action indexes, escaping recovery references, and inconsistent plan identity.

## Writer coordination

All actionful sync and resolution operations preserve:

```text
.mutation.lock -> .registry.lock -> state/state.lock
```

The outer mutation lock spans lock-held plan reconstruction, operation publication, payload actions, final observation, and baseline publication. Preview, blocked, and synchronized-only plans remain lock-free. Converged-only acceptance takes the mutation lock because it changes accepted state.

## Baseline publication

After complete verification, publish one State V2 generation updating actioned identities and selected converged identities. Preserve synchronized selected records, out-of-scope records, and pending-retirement records. Publish nothing for preview, blocked, stale, contended, partial, verification-failed, or semantic no-op results.

## Recovery and interruption

Recovery Metadata V1 remains operation-local. The immutable plan binds operation, action index, Entry Identity, mapping-role paths, direction, and resolve winner. Prior nonterminal or failed records remain immutable evidence and do not themselves block a fresh invocation; later commands never silently resume or roll them back.
