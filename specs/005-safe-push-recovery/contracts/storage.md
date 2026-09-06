# Storage Contract: Mutation Coordination and Operation Evidence

This contract adds writer coordination and push operation evidence without changing State Envelope V1 or V2.

## Layout

```text
<grip-home>/
├── config.toml
├── .mutation.lock
├── .registry.lock
└── state/
    ├── state.json
    ├── state.lock
    ├── recovery/
    │   └── generation-<N>/
    │       └── state.json
    └── operations/
        └── <operation-id>/
            ├── plan.json
            ├── operation.json
            ├── .operation.tmp-<attempt-id>
            ├── actions/
            │   ├── <zero-padded-action-index>.json
            │   └── .action.tmp-<attempt-id>
            └── recovery/
                └── <zero-padded-action-index>/
                    ├── metadata.json
                    └── payload
```

The existing `state/recovery/generation-<N>/state.json` contains prior accepted-state bytes. It is distinct from payload recovery and remains unchanged.

Grip-owned directories are current-user-owned, non-symlink directories with mode `0700`. Lock, journal, metadata, and recovery payload files are current-user-owned non-symlink regular files with mode `0600`.

## Mutation lock

`.mutation.lock` is a stable file protected by a nonblocking kernel advisory lock. Creation or opening uses no-follow semantics and validates owner, type, and mode. Its contents are strict diagnostic owner metadata and are replaced and synced only after successful advisory acquisition.

| Condition | Result |
|---|---|
| advisory lock acquired, old metadata present | overwrite metadata and proceed |
| advisory lock held | `state_contention`/13 with available owner details |
| unlocked file with old PID | stale diagnostic only; proceed after acquisition |
| unsafe node, owner, or mode | `corrupt_state`/12 |
| malformed old metadata after acquisition | overwrite with current valid metadata |

Grip never unlinks the stable lock file, breaks a lock based on PID, or signals another process.

## Global writer order

All writers acquire locks in this order when applicable:

```text
.mutation.lock -> .registry.lock -> state/state.lock
```

- Actionful push: mutation for execution; registry and state only for accepted-state publication.
- Mapping add/remove: mutation, then registry publication.
- Changed baseline acceptance: mutation, registry, then state publication.
- Dry runs, blocked plans, and semantic no-ops: no mutation lock or operation artifacts.
- Read-only commands: no writer locks.

## Operation directory allocation

An operation ID is display-safe and opaque. Grip proposes a name from UTC time, process ID, and process-local counter, then creates its directory exclusively. A collision increments the counter up to a bounded limit. Exclusive directory creation establishes uniqueness; callers never derive authority from the ID text.

The directory is created and synced only after lock-held plan equivalence and immediately before the first payload mutation. If initialization cannot publish a valid immutable plan and initial operation summary, execution stops before payload mutation.

## Partitioned Operation Record V1

The complete wire fields are defined in [the data model](../data-model.md#operation-record). The logical record consists of an immutable complete plan envelope, a bounded operation summary envelope, and sparse bounded action checkpoint envelopes. The decoder for each requested component:

1. validates safe directory and file ancestry;
2. decodes UTF-8 JSON;
3. rejects unknown or duplicate fields;
4. requires schema version 1;
5. verifies SHA-256 integrity over canonical `{schema_version,payload}` bytes;
6. validates the operation and plan identity, component-specific transitions, relative recovery references, baseline generations, and result-delivery state;
7. rejects an action checkpoint whose index or immutable intent is inconsistent with `plan.json`.

An unsupported schema exits `11`. Unsafe, malformed, integrity-invalid, escaping, duplicate, or semantically inconsistent evidence exits `12`.

## Record publication

Initialization atomically publishes `plan.json` once and then the bounded initial `operation.json`. Each later milestone atomically replaces only `operation.json` or the current action's bounded checkpoint; immutable plan data and unrelated action evidence are never reserialized. Each component publication:

1. encode canonical bytes and integrity;
2. exclusively create an attempt-owned sibling staging file with mode `0600`;
3. write and sync bytes;
4. reread and strictly decode the staging file;
5. compare its domain value with the intended record;
6. verify descriptor/path identity;
7. rename over the targeted summary or action-checkpoint file;
8. sync the operation directory.

Cleanup removes only a staging pathname still bound to the attempt-owned opened file. Publication failure after rename reports the candidate visible with durability unconfirmed.

The immutable plan is never replaced. Action checkpoints are created only when their action starts, so a missing checkpoint means `unattempted`; no files are precreated for unattempted actions. Operation records are immutable after another invocation begins. A later invocation may read a specifically selected prior nonterminal record but never checkpoint it.

## Recovery publication

Each replacement uses its zero-padded action index as the only recovery directory key. `metadata.json` binds the operation ID, action index, Entry Identity, prior Supported State, and relative payload reference. `payload` contains exact prior file bytes at private mode `0600`.

Recovery publication is exclusive and immutable. A byte- and metadata-equivalent existing entry may be recognized only within the same live operation retry before payload publication; any conflicting target is corrupt state. The operation record exposes an opaque recovery state, not a public filesystem path.

## Accepted-state publication

State Envelope V2 remains the only accepted baseline authority. Push calls the existing locked publisher once, after every payload action and final complete observation verify. It updates actioned identities only and preserves out-of-scope and pending-retirement records.

- Failure before state rename: prior accepted state remains authoritative.
- Rename succeeded and directory sync succeeded: new generation is authoritative and durable.
- Rename succeeded and directory sync failed: new generation is visible, durability is unconfirmed, and the result exits `20`.
- Output delivery failure after successful publication: published generation remains authoritative.

## Read behavior and interruptions

Ordinary push preflight does not enumerate or validate retained operation directories. A valid nonterminal record means the prior process may have stopped; it is not an active lock and is preserved unchanged. A new push relies on fresh inspection, the advisory mutation lock, and exclusive allocation of a new operation directory. A later read-only operation-inspection feature may validate one specifically selected record without making all history a prerequisite.

Read-only `status`, `check`, and `diff` remain governed by Feature 004 and do not create, finalize, repair, or delete operation evidence. Feature 008 owns public backup inspection, cleanup, rollback, and bounded state-recovery workflows.

## Result delivery

Before rendering an actionful push result, journal state records the terminal payload/baseline outcome and `result_delivery: "prepared"`. Rendering failure triggers a best-effort checkpoint to `failed` and exit `20`. Successful rendering may checkpoint `delivered`; failure of that post-output checkpoint is diagnostic only because emitted success cannot be recalled.

This explicitly acknowledges the unavoidable crash window between stdout delivery and its journal acknowledgement.
