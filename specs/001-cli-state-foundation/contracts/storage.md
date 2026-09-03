# Storage Contract: Feature 001

This contract defines the externally inspectable Feature 001 registry and machine-state layout beneath the selected Grip Home.

## Layout

```text
<GRIP_HOME>/
├── config.toml
└── state/
    ├── state.json
    ├── state.lock
    ├── recovery/
    │   └── generation-<N>/state.json
    └── .state.tmp-<unique-token>
```

Only `config.toml` is user-authored. Everything beneath `state/` is Grip-owned. Accepted, recovery, and staging state are never portable configuration, and staging files are never accepted state.

## Registry V1

A valid empty registry is:

```toml
schema_version = 1
mappings = []
```

Both fields are required. Unknown fields, unsupported versions, malformed values, and a non-empty mapping collection are rejected. Feature 001 validates but does not rewrite the registry.

## State V1

A valid initial state envelope is:

```json
{
  "schema_version": 1,
  "payload": {
    "generation": 0
  },
  "integrity": {
    "algorithm": "sha256",
    "digest": "14438570ac616b5d5d0e0c085a43709b2ef43b346f3b7d923782447368da0850"
  }
}
```

The digest is SHA-256 over these exact canonical UTF-8 bytes:

```text
{"schema_version":1,"payload":{"generation":0}}
```

For generation `N`, replace only the unsigned decimal generation value. Unknown fields are rejected in the envelope, payload, and integrity object. An absent `state.json` means uninitialized; an absent file is not synthesized during validation.

## Publication protocol

A state writer performs the following bounded operation:

1. Validate the selected root and machine-state descendants without following final-component symbolic links.
2. Open the stable `state.lock` regular file and acquire a nonblocking exclusive advisory lock.
3. Revalidate the accepted state and paths while holding the lock.
4. When replacing generation `N`, stage, verify, atomically publish, and sync an immutable copy at `recovery/generation-N/state.json`; reuse an existing byte-identical copy on retry and reject a conflicting copy as corrupt state. Initial publication skips this step.
5. Create a unique staging regular file in `state/` with exclusive creation and mode `0600`.
6. Write the complete envelope, flush and sync it, reread it, and verify schema, semantics, and integrity.
7. Rename it over `state.json`; do not substitute copy-and-delete behavior if atomic rename is unavailable.
8. Sync `state/` where supported, report success only after all required operations succeed, and release the descriptor-backed lock.

The final accepted-state rename is the replacement acceptance point. Readers ignore staging names and never acquire the publication lock. A failed recovery-copy operation blocks replacement; any later pre-rename failure leaves the previous accepted file authoritative and its recovery copy retained. Lock-file existence alone is normal and never signals contention.

## Filesystem safety

Grip-created state and recovery directories use `0700`; state, lock, staging, and recovery files use `0600`. Creation requests the restrictive mode from the first operation. Existing roots and Grip-owned nodes are validated for expected type, ownership, symlink status, and accessibility; pre-existing user-authored roots are not silently chmodded.

The guarantee is atomic visibility of the old or new complete accepted document on supported local filesystems. Grip syncs file and directory data where supported but does not claim universal survival across all hardware caches, sudden power loss, network filesystems, or user-space filesystem implementations. Unsupported required locking, synchronization, or atomic replacement fails as `operational_failure` rather than silently weakening the protocol.
