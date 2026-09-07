# Storage Contract: Recovery Lifecycle and Operation Evidence

## Preserved authority formats

- Registry V1 remains the user-authored mapping authority.
- State Envelope V2 remains accepted synchronization authority.
- Partitioned Operation Record V1 remains immutable operation evidence.
- Existing Recovery Metadata V1 and historical recovery bytes remain byte-identical.

No migration or eager rewrite is required.

## Recovery Manifest V1

Every newly retained payload, registry, or accepted-state document receives an immutable `manifest.json` sidecar in its contained recovery directory. The strict envelope contains `schema_version: 1`, a typed payload, and SHA-256 integrity over canonical schema version plus payload. The payload records the opaque reference, kind, creation time, origin, prior evidence, expected post-transition evidence, byte count, and bound target where applicable. Unknown fields, invalid references, layout mismatch, digest mismatch, or semantic mismatch fail closed.

Existing recovery without a manifest remains legacy evidence. Readers synthesize only facts proven by its existing metadata and storage identity.

## Cleanup Tombstone V1

Successful cleanup publishes immutable `cleaned.json` beside the manifest only after recoverable byte absence and containing-directory synchronization are verified. Its integrity-protected payload binds recovery reference, cleanup operation ID, cleaned time, and removed byte count.

The manifest is never mutated. These states are distinct:

| Bytes | Tombstone | State |
|---|---|---|
| present and verified | absent | `available` |
| absent | valid | `cleaned` |
| absent | absent/invalid | `cleanup_incomplete` |
| present | present | corrupt/inconsistent |

## Operation records

Operation Record V1 admits `delete`, `retire`, `recovery_restore`, and `recovery_remove`. A generic typed initializer persists operation, plan ID, serialized immutable plan, and action count while command-specific validation preserves each plan's invariants. Existing push, pull, sync, and resolve records remain valid and are never rewritten.

Operation metadata needed for recovery provenance is retained in this feature. It is not removed by cleanup.

## Writer coordination

All actionful delete, retire, restore, and cleanup operations acquire `.mutation.lock` after lock-free preflight and before plan reconstruction. Registry restore then acquires `.registry.lock`; accepted-state publication or restore then acquires `state/state.lock`. The global order remains:

```text
.mutation.lock -> .registry.lock -> state/state.lock
```

Read-only inventory, previews, blockers, and semantic no-ops remain lock-free.

## Authority restoration

Registry and state restoration publish exact verified recovered bytes with their existing strict decoders and integrity domains. Publication uses sibling staging, attempt-owned identity verification, atomic rename where supported, and parent-directory synchronization. A missing/corrupt target may be repaired, but a different valid current authority is never overwritten. Visibility and durability remain independent result fields.
