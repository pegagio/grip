# Storage Contract: Accepted Baselines

This contract extends Feature 001's machine-owned state without changing State Envelope V1 or permitting read-only commands to migrate state.

## Layout

```text
<grip-home>/
├── config.toml
├── .registry.lock
└── state/
    ├── state.json
    ├── state.lock
    ├── .state.tmp-<attempt-id>
    └── recovery/
        └── generation-<N>/
            └── state.json
```

All Grip-owned state directories and files retain the owner/type/mode and no-symlink rules established for private state. Staging paths are attempt-owned, exclusively created, and never accepted state until final same-directory rename.

## Supported readers and writer

| Input | Read interpretation | Next changed publication |
|---|---|---|
| absent `state.json` | uninitialized, no baselines | V2 generation 0 |
| valid State Envelope V1 generation N | empty baselines at generation N | V2 generation N+1 |
| valid State Envelope V2 generation N | decoded accepted baselines | V2 generation N+1 |
| unsupported schema | fail with exit 11 | none |
| unsafe, malformed, integrity-invalid, or semantically invalid state | fail with exit 12 | none |

Status, check, and diff never rewrite V1 or create missing state. Baseline acceptance writes V2 only when its baseline map changes.

## State Envelope V2

The wire shape and field validation are defined in [the data model](../data-model.md#state-envelope-v2). All named objects reject unknown fields. Baseline arrays must be uniquely and canonically ordered. File states require SHA-256 digest, length, and permission mode; directory states forbid file-only fields. Mtime and payload bytes are never persisted.

V2 integrity uses SHA-256 over the deterministic compact UTF-8 serialization of the typed object containing `schema_version: 2` and `payload` in declared field order. The integrity member is excluded. Decoding validates syntax, version, integrity, then semantic invariants before exposing version-neutral Accepted State.

## Expected snapshots

A loaded state snapshot is `Absent` or `Valid` with exact bytes and safe non-following file identity. Publication accepts that snapshot as an expectation. After locks are acquired, the accepted path must still have the same absence or exact bytes and identity; otherwise publication fails as stale evidence.

The registry snapshot similarly includes exact bytes, accepted file identity, complete durable mapping validity, and topology evidence. Current endpoint presence is observation evidence rather than a prerequisite for loading durable mapping intent.

## Publication transaction

The global acquisition order is:

1. stable owner-only `.registry.lock`;
2. stable owner-only `state/state.lock`.

Locks are nonblocking and cover only Grip-owned registry/state consistency and state publication. Grip never locks payload trees.

While both locks are held, baseline acceptance:

1. reloads and compares expected registry and state snapshots;
2. repeats selected membership, path, policy, metadata, and content-fingerprint observation from fresh descriptors;
3. rebuilds the full copy-on-write candidate and validates all selected eligibility;
4. rereads the registry immediately before publication;
5. returns `already_current` without state artifacts when baseline maps are semantically equal;
6. otherwise assigns the exact next generation;
7. retains and verifies immutable prior accepted bytes under their generation when prior state exists;
8. exclusively stages V2 bytes in the state directory, syncs and rereads them, validates envelope integrity and domain equality, and verifies the staging pathname still identifies the opened file;
9. renames the staged file over `state.json`, then syncs the state directory.

Generation must be exactly zero from absent state or exactly prior generation plus one. Out-of-scope and pending-retirement records are copied unchanged into the candidate.

## Failure and retry

- Failure before accepted-state rename leaves prior state authoritative and removes only the attempt-owned staging file when safe.
- Recovery failure blocks accepted-state replacement.
- A byte-identical existing recovery generation is reused; a conflicting generation is corrupt state.
- Failure after rename reports `publication_visible: true` and `durability_confirmed: false`; it never claims the old state remains visible.
- Retry begins by loading current accepted bytes. If the intended baselines are now current, locked revalidation returns `already_current` without another generation.
- State-lock contention returns `state_contention`/13. Registry-lock contention and other I/O failures remain operational/20.

## Mapping removal and retirement

Mapping removal remains a registry-only operation and does not rewrite state. At the next classification, any baseline whose complete Mapping Snapshot is absent from the current registry becomes `untracked_pending_retirement`. Grip reports the retained evidence without reopening paths through removed ownership. Feature 008 owns explicit retirement and deletion.

## Non-goals

This contract does not restore or delete recovery generations, automatically rebuild corrupt state, migrate state during read-only commands, store payload contents, lock payload paths, publish registry and state as one cross-file atomic transaction, or implement Feature 008 retirement.
