# Data Model: Safe Push and Recovery

This model extends Feature 004's `MappingSnapshot`, `EntryIdentity`, `SupportedState`, `ClassificationRecord`, `ClassificationScope`, `StateSnapshot`, and `AcceptedState`. It preserves State Envelope V2 and adds transient planning types plus durable operation evidence.

## Table of Contents

- [Push request](#push-request)
- [Push plan](#push-plan)
- [Entry disposition](#entry-disposition)
- [Push action](#push-action)
- [Action evidence](#action-evidence)
- [Recovery entry](#recovery-entry)
- [Operation record](#operation-record)
- [Mutation lock owner](#mutation-lock-owner)
- [Push result](#push-result)
- [State transitions](#state-transitions)
- [Identity, ordering, and integrity](#identity-ordering-and-integrity)

## Push request

| Field | Type | Rules |
|---|---|---|
| `mode` | `dry_run` or `execute` | `execute` is the default |
| `path_space` | `source` or `destination` | Changes selector interpretation only |
| `selector` | optional safe path | At most one; uses established resolution |

The request never selects push direction. Push always applies eligible source state to the managed destination.

## Push plan

| Field | Type | Rules |
|---|---|---|
| `plan_id` | SHA-256 identity | Digest of canonical semantic plan; excludes mode and execution state |
| `scope` | Classification Scope | Resolved through Feature 004 selection |
| `expected_registry` | Registry Snapshot identity | Exact bytes and safe file identity |
| `expected_state` | State Snapshot identity | Exact accepted bytes, identity, and generation |
| `entries` | ordered Entry Dispositions | Exactly one per selected managed entry |
| `actions` | dependency-ordered Push Actions | Includes synthetic parent actions |
| `blockers` | ordered Plan Blockers | Complete discovered blocker set |
| `counts` | Plan Counts | Selected, actionable, no-action, and blocked counts |

The plan is executable only when `blockers` is empty and `actions` is non-empty. Dry run and execution over equivalent evidence have the same `plan_id`, entries, actions, and blockers.

## Entry disposition

| Field | Type | Rules |
|---|---|---|
| `identity` | Entry Identity | Canonical mapping snapshot plus raw relative bytes |
| `classification` | inherited classification | One of Feature 004's 18 values |
| `disposition` | `action`, `no_action`, or `blocked` | Derived without I/O |
| `action_indexes` | ordered integer array | Empty unless one or more actions serve the entry |
| `reasons` | ordered stable strings | Explains no-action or blocking result |

`source_addition` and `source_only_change` use `action`; `synchronized` uses `no_action`. Destination-only unmanaged evidence is not a managed entry. Other classifications remain no-action only when their inherited evidence is explicitly nonblocking; known conflicts, unsafe collisions, and unsupported managed evidence are blockers.

## Push action

| Field | Type | Rules |
|---|---|---|
| `index` | unsigned integer | Dense, zero-based execution order |
| `kind` | action kind | `create_parent_directory`, `create_directory`, `add_file`, or `replace_file` |
| `identity` | optional Entry Identity | Required for managed actions; synthetic parents list dependent identities instead |
| `dependent_identities` | ordered Entry Identity array | Non-empty for synthetic parents |
| `source_path` | optional Safe Path | Required for managed payload actions |
| `destination_path` | Safe Path | Required |
| `expected_source` | optional Supported State | Required for managed payload actions |
| `expected_destination` | optional Supported State | Present for replacement; absent for additions/parents |
| `dependencies` | ordered action indexes | Every index is lower than this action's index |
| `status` | action status | Mutable only in an operation record |
| `milestones` | Action Evidence | Mutable only in an operation record |
| `failure` | optional Action Failure | Present only with `failed` status |

Action kinds rank `create_parent_directory`, `create_directory`, `add_file`, then `replace_file` after dependency constraints are satisfied. Remaining ties use canonical mapping source bytes, source-relative bytes, and destination bytes.

Action status transitions are:

```text
unattempted -> in_progress -> completed
                         -> failed
```

When one action fails, every later action remains `unattempted`.

## Action evidence

| Field | Type | Rules |
|---|---|---|
| `revalidation` | `not_attempted`, `passed`, or `failed` | Must pass before mutation |
| `recovery` | `not_required`, `planned`, `preserved`, or `failed` | `replace_file` requires `preserved` before publication |
| `recovery_ref` | optional relative reference | Never an absolute or user-supplied path |
| `staging` | `not_attempted`, `verified`, or `failed` | Managed file actions require `verified` before publication |
| `publication` | `not_attempted`, `not_visible`, or `visible` | Records rename visibility |
| `verification` | `not_attempted`, `verified`, or `failed` | Complete destination supported state |
| `durability_confirmed` | boolean | True only after containing-directory sync |

Visibility, verification, and durability are independent. A renamed destination can be visible and verified while durability remains unconfirmed.

## Recovery entry

| Field | Type | Rules |
|---|---|---|
| `action_index` | unsigned integer | Unique within operation |
| `identity` | Entry Identity | Exact managed entry being replaced |
| `prior_state` | Supported State | File content fingerprint, length, and permission mode |
| `payload_ref` | relative operation-local reference | Derived from action index |
| `payload_present` | boolean | True only after exclusive recovery publication |
| `verified` | boolean | True only after descriptor-bound reread matches `prior_state` |

Recovery payload files are stored with private mode `0600`; `prior_state.permission_mode` preserves the user-visible mode for later Feature 008 recovery. Additions have no Recovery Entry.

## Operation record

### Partitioned Operation Record V1

One logical Operation Record is reconstructed from three strict envelope kinds:

- immutable `plan.json`, containing the complete lock-held Push Plan exactly once;
- bounded `operation.json`, containing operation-level state, baseline outcome, failure, and result-delivery state;
- sparse `actions/<zero-padded-index>.json` checkpoints, each containing the latest state and evidence for one started action.

An action listed in `plan.json` with no checkpoint file is `unattempted`. Readers reject duplicate indexes, checkpoints not present in the plan, and immutable intent that differs from the plan.

#### Operation summary envelope

```json
{
  "schema_version": 1,
  "payload": {
    "operation_id": "<opaque-id>",
    "operation": "push",
    "state": "executing",
    "plan_id": "<digest>",
    "plan_ref": "plan.json",
    "baseline": {},
    "result_delivery": "not_attempted"
  },
  "integrity": {
    "algorithm": "sha256",
    "digest": "<digest>"
  }
}
```

Each envelope kind has its own schema discriminator and integrity digest. All named objects reject unknown fields. Integrity covers deterministic compact serialization of `schema_version` and `payload` in declared order and excludes `integrity`.

| Field | Type | Rules |
|---|---|---|
| `operation_id` | opaque string | Bound to exclusively created operation directory |
| `operation` | `push` | Required |
| `state` | `executing`, `completed`, or `failed` | Nonterminal only while execution may be incomplete |
| `plan_id` | SHA-256 identity | Must match immutable `plan.json` |
| `plan_ref` | `plan.json` | Fixed operation-local reference |
| `baseline` | Baseline Outcome | Tracks prior, published, and authoritative generation |
| `result_delivery` | `not_attempted`, `prepared`, `delivered`, or `failed` | Separate from payload/baseline success |
| `failure` | optional Operation Failure | First terminal execution failure |

The immutable plan envelope contains `operation_id` and the complete Push Plan projection. Each action checkpoint envelope contains `operation_id`, `plan_id`, `action_index`, the action's current status and milestones, and any action failure. Its immutable intent is obtained from `plan.json`, not duplicated in every checkpoint.

Operation state transitions are:

```text
absent -> executing -> completed
                   -> failed
                   -> executing after process interruption
```

A valid nonterminal record remains `executing` permanently unless its original process checkpoints it. Later invocations never mutate it.

## Mutation lock owner

| Field | Type | Rules |
|---|---|---|
| `schema_version` | integer | Exactly `1` |
| `pid` | positive integer | Diagnostic only |
| `operation` | stable command name | `push`, `mapping_add`, `mapping_remove`, or `baseline_accept` |
| `acquired_at` | UTC timestamp | Diagnostic only |

The file's advisory lock determines active ownership. Metadata can outlive the owner and is overwritten after successful acquisition.

## Push result

| Field | Type | Rules |
|---|---|---|
| `mode` | `dry_run` or `execute` | Required |
| `completion` | `complete`, `blocked`, `partial`, or `failed` | Derived from plan and action states |
| `result` | `planned`, `no_op`, `applied`, `blocked`, or `failed` | Required |
| `scope` | Classification Scope | Required |
| `plan_id` | SHA-256 identity | Required after complete planning |
| `counts` | Push Counts | Selected, actions, no-action, blockers, completed, failed, unattempted |
| `entries` | ordered Entry Dispositions | Complete selected managed scope |
| `actions` | ordered Push Actions | Execution order and terminal states |
| `blockers` | ordered Plan Blockers | Complete preflight set |
| `operation_record` | optional opaque availability | Present only after mutation was about to begin |
| `baseline` | Baseline Outcome | Required |

## State transitions

### Push orchestration

```text
Request
  -> complete preflight -> blocked result, no lock or journal
  -> complete no-action plan -> no-op result, no lock or journal
  -> dry run with actions -> planned result, no lock or journal
  -> execute with actions -> acquire mutation lock
       -> lock-held plan differs -> stale failure, no journal or payload mutation
       -> equivalent plan -> initialize journal
            -> actions complete and verify -> final complete observation
                 -> baseline publishes -> completed/applied
                 -> baseline fails before visibility -> failed, prior baseline authoritative
                 -> baseline visible but sync fails -> failed, candidate visible, durability unconfirmed
            -> first action failure -> failed/partial, later actions unattempted, no baseline publication
```

### Result delivery

```text
terminal payload/baseline outcome -> delivery prepared
  -> render succeeds -> delivery delivered
  -> render fails -> delivery failed, exit 20, payload/baseline authority unchanged
```

### Later push after interruption

```text
valid prior nonterminal record
  -> read-only discovery preserves it
  -> fresh preflight and new mutation-lock acquisition
  -> separate plan and operation ID
  -> prior record and recovery remain unchanged
```

## Identity, ordering, and integrity

- Entry identity remains the complete Feature 004 `EntryIdentity` and never uses display text as authority.
- `entries` sort by `EntryIdentity`; `actions` sort by dependency then canonical action key.
- `plan_id` is SHA-256 over canonical scope, expected registry/state evidence, entries, actions, and expected supported states. It excludes request mode, messages, timestamps, operation ID, and mutable statuses.
- Operation IDs are opaque and unique by exclusive directory creation; their text is never trusted as a path.
- Operation envelopes and accepted-state envelopes have independent schema versions and integrity domains.
- Recovery references are validated as operation-local relative components and cannot escape their operation directory.
