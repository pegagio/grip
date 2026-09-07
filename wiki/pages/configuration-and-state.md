---
title: Configuration and state
type: component
sources: [S001, S004, S005, S007, S008, S009, S010, S011]
updated: 2026-09-07
---

# Configuration and state

Grip separates durable user intent from machine-owned operational evidence. The registry holds mappings, paired paths, kinds, and options; state holds discovered members, fingerprints, baselines, pending retirement information, and backup references. Users do not maintain the derived member inventory manually. (S001)

The default per-user root for both registry and state is `~/.grip/`. A non-empty absolute `GRIP_HOME` selects an alternate exact root; relative or empty values are invalid. `/etc/grip/` is not implicitly merged as a registry or state location, and any future system-wide policy would require a distinct precedence model. (S001)

Configuration must carry an explicit schema version and reject unknown fields, unsupported versions, malformed options, path traversal, and duplicate or overlapping ownership. Machine-owned state should be written atomically and include enough versioning and integrity information to detect incompatible or corrupt data. (S001)

The implemented v1 `config.toml` registry carries `schema_version = 1` and canonical file or tree mapping tuples; an empty `mappings` array remains valid. Grip-owned `state/state.json` is optional, and `validate` reports an absent state document as uninitialized without creating it. (S004)

The accepted registry must be a current-user-owned, non-symlink regular file without group or other write permission; mapping mutations also require owner write permission and preserve the exact accepted mode. Writers use a stable owner-only `.registry.lock`, retain exact prior bytes in content-addressed recovery generations, verify a complete staged candidate, and atomically replace `config.toml`. These artifacts contain no synchronization state and mapping commands do not create `state/state.json`. (S004)

Baseline records contain fingerprints and accepted supported metadata, not historical copies of file contents. Recovery copies belong to the operation backup namespace and are a separate concern from baseline comparison. (S001)

Feature 004 publishes integrity-checked State Envelope V2 generations atomically at `<GRIP_HOME>/state/state.json` and retains exact prior bytes as immutable recovery evidence. The state directory must be a current-user-owned, non-symlink directory with exact owner-only mode. State V1 remains readable as an empty-baseline predecessor, while corrupt, unsupported, or stale evidence produces stable non-success results rather than guessed recovery. (S004)

Feature 005 adds partitioned Operation Record V1 under `state/operations/<operation-id>/`: immutable plan intent, a bounded operation summary, one bounded checkpoint per started action, and private recovery payloads. These records are evidence rather than accepted synchronization state. A later push may proceed from a fresh complete plan after an interrupted record, but it never silently resumes, repairs, rolls back, or deletes that retained history. (S008)

Feature 006 extends the closed Operation Record V1 operation identity from `push` to `push | pull` without adding a second history tree or a new state schema. Direction-prefixed opaque operation IDs and direction-bound plans distinguish otherwise symmetric transfers, while interrupted and failed pull records remain immutable evidence and do not alone block a later independently planned operation. (S009)

Feature 007 preserves that V1 layout while admitting `sync` and `resolve` operations, per-action direction, and an optional resolution winner. Historical push and pull plans remain valid and immutable; strict validation rejects unknown or inconsistent operation, direction, winner, action-index, recovery-reference, and plan-identity evidence. (S010)

Feature 008 extends immutable operation evidence to `delete`, `retire`, `recovery-restore`, and `recovery-remove`. Recovery Manifest V1 binds preserved payload or authority documents to their origin, operation, integrity, and exact target; cleanup removes only confirmed recoverable bytes and publishes an immutable Cleanup Tombstone V1 while retaining provenance. Exact verified Registry V1 or State V2 copies may repair missing or corrupt authority only after complete compatibility checks, never by guessing. (S011)

## Related pages

- [Grip product model](./grip-product-model.md)
- [Baseline classification and status](./baseline-classification-and-status.md)
- [Command-line and path selection](./command-line-and-path-selection.md)
- [Mapping registry publication](./mapping-registry-publication.md)
