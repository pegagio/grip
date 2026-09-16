# Aggregate Forced Push Contract

## Command Selection

| Invocation | Required behavior |
|---|---|
| `grip push --force` | Create one `aggregate_force_push` request for all managed entries in the selected project, with source complete state as the winner. |
| `grip push --force --dry-run` or `-n` | Inspect and report the same aggregate scope and planned states without payload or accepted-state mutation. |
| `grip push <selector> --force` | Preserve the existing exact-entry forced-push contract. |
| `grip pull --force` | Preserve existing exact-entry semantics; no aggregate pull is added. |
| Ordinary `push`, `pull`, or `sync` | Preserve current behavior; no implicit force or aggregate sync is added. |

## Scope and Safety

Before execution, Grip must validate the complete selected project scope. Unsupported nodes, unsafe symbolic-link ancestry, ownership failures, topology failures, and invalid project selection block the aggregate and leave destinations and accepted state untouched.

Source-present divergent entries and source-present missing destination peers are eligible where existing node contracts support them. Source absence is eligible only as part of this explicit source-complete aggregate authority and must use typed, deterministic planning; it does not authorize implicit deletion in ordinary operations.

An exact managed destination-leaf symbolic link remains an exact-force-only replacement case. Aggregate force must block it rather than follow or replace it.

## Execution and Publication

1. Order entries and actions deterministically.
2. Immediately before each action, revalidate evidence relevant to that action.
3. Apply and verify every action required for one entry.
4. Publish accepted baseline evidence for that completed entry only.
5. Reload the current State V4 snapshot before later revalidation.
6. Stop at the first revalidation, execution, verification, or publication failure. Do not attempt later entries.

If a failure occurs after earlier entries were completed and published, those entries remain accepted. The aggregate result is nevertheless failed, and the failed plus later entries remain unaccepted and are identified as failed or unattempted.

## Result Requirements

Human and machine-readable results must include:

- Operation kind and selected project scope.
- Deterministically ordered completed, unchanged, blocked, failed, and unattempted entries.
- Per-entry verification and accepted-publication status.
- The first failure or drift reason, when applicable.
- A final aggregate outcome that never calls a partial failure successful convergence.

Illustrative result shape:

```json
{
  "operation": "aggregate_force_push",
  "outcome": "failed",
  "entries": [
    {"identity": "alpha", "status": "completed", "accepted_state": "published"},
    {"identity": "beta", "status": "failed", "reason": "destination drift detected"},
    {"identity": "gamma", "status": "unattempted", "accepted_state": "unaccepted"}
  ]
}
```

Field names may follow existing operation-record conventions, but the distinctions above are contractually required.

## Compatibility

This feature changes only omitted-selector `push --force`. Existing selected-entry force guidance, safety constraints, ordinary directional commands, state format compatibility, and no-follow behavior remain in force unless a later feature explicitly changes them.
