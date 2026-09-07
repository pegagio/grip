# Data Model: Reverse Synchronization

This model generalizes Feature 005's push types into a direction-aware mutation model while preserving Feature 004's `MappingSnapshot`, `EntryIdentity`, `SupportedState`, `ClassificationRecord`, `ClassificationScope`, and State Envelope V2.

## Table of Contents

- [Mutation direction](#mutation-direction)
- [Mutation request](#mutation-request)
- [Mutation plan](#mutation-plan)
- [Entry disposition](#entry-disposition)
- [Mutation action](#mutation-action)
- [Action evidence](#action-evidence)
- [Recovery entry](#recovery-entry)
- [Operation record](#operation-record)
- [Mutation result](#mutation-result)
- [State transitions](#state-transitions)
- [Identity ordering and integrity](#identity-ordering-and-integrity)

## Mutation direction

| Value | Origin mapping role | Target mapping role | Actionable classification |
|---|---|---|---|
| `push` | source | destination | `source_addition`, `source_only_change` |
| `pull` | destination | source | `destination_only_change` |

Direction is explicit plan identity and result evidence. It never changes mapping identity, source-defined membership, selector semantics, or baseline record shape.

## Mutation request

| Field | Type | Rules |
|---|---|---|
| `direction` | `push` or `pull` | Fixed by the invoked command |
| `mode` | `dry_run` or `execute` | Execute is the default |
| `path_space` | `source` or `destination` | Changes selector interpretation only |
| `selector` | optional safe path | At most one; uses established resolution |

`pull --destination PATH` selects in destination path space but does not change pull direction.

## Mutation plan

| Field | Type | Rules |
|---|---|---|
| `direction` | Mutation Direction | Required and included in plan identity |
| `plan_id` | SHA-256 identity | Digest of canonical semantic plan; excludes mode and execution state |
| `scope` | Classification Scope | Resolved through Feature 004 selection |
| `expected_registry` | Registry Snapshot identity | Exact accepted bytes and safe file identity |
| `expected_state` | State Snapshot identity | Exact accepted bytes, identity, and generation |
| `entries` | ordered Entry Dispositions | One per selected observed record, including unmanaged destination non-actions |
| `actions` | ordered Mutation Actions | Push may include parent/add actions; pull includes replacement actions only |
| `blockers` | ordered Plan Blockers | Complete discovered blocker set |
| `counts` | Mutation Counts | Selected, actionable, no-action, blocker, completed, failed, and unattempted counts |

The plan is executable only when blockers are empty and actions are non-empty. Dry run and execution over equivalent evidence have the same direction, identity, entries, actions, and blockers.

## Entry disposition

| Field | Type | Rules |
|---|---|---|
| `identity` | Entry Identity | Canonical mapping snapshot plus raw relative bytes |
| `classification` | inherited classification | One of Feature 004's 18 values |
| `source_path` | Safe Path | Mapping-role source, never swapped by direction |
| `destination_path` | Safe Path | Mapping-role destination, never swapped by direction |
| `disposition` | `action`, `no_action`, or `blocked` | Derived from direction plus inherited blocking evidence |
| `action_indexes` | ordered integer array | Empty for pull no-actions and blockers; one index for an eligible pull replacement |
| `reasons` | ordered stable strings | Explains no-action or blocking disposition |

### Pull disposition table

| Classification | Nonblocking disposition | Pull meaning |
|---|---|---|
| `destination_only_change` | `action` | Replace established source from destination |
| `destination_only_unmanaged` | `no_action` | Report `unmanaged_destination`; never import or block by presence alone |
| `synchronized` | `no_action` | Already current |
| `source_addition`, `source_only_change` | `no_action` | Wrong direction |
| `initial_match`, `initial_collision` | `no_action` | No accepted managed identity for pull |
| `converged_two_sided_change` | `no_action` | Requires explicit baseline acceptance |
| `divergent_conflict` | `no_action` | Inherited blocking flag makes the actual record blocked |
| directional deletion or delete/change conflict | `no_action` | Feature 008 owns execution; inherited blockers remain blocked |
| pending retirement | `no_action` | Feature 008 owns retirement |
| unsupported or unsafe | `no_action` | Inherited blocking flag makes the actual record blocked |

Any record with `blocking: true` is `blocked` regardless of the nonblocking default.

## Mutation action

| Field | Type | Rules |
|---|---|---|
| `index` | unsigned integer | Dense, zero-based execution order |
| `kind` | action kind | Pull permits only `replace_file`; existing push kinds remain valid for push |
| `identity` | Entry Identity | Required for every pull action |
| `source_path` | Safe Path | Stable mapping-role source |
| `destination_path` | Safe Path | Stable mapping-role destination |
| `expected_source` | Supported State | Existing accepted source state that must still match before pull replacement |
| `expected_destination` | Supported State | Destination-only changed state used as pull origin and expected final state |
| `target_path` | internal absolute path | Derived as source for pull; never serialized as authority |
| `dependencies` | ordered action indexes | Empty for pull replacement actions; retained for push compatibility |
| `status` | action status | Mutable only in operation evidence |
| `milestones` | Action Evidence | Mutable only in operation evidence |
| `failure` | optional stable reason | Present only with failed status |

Pull action status transitions remain:

```text
unattempted -> in_progress -> completed
                         -> failed
```

When one action fails, every later action remains `unattempted`.

## Action evidence

| Field | Values | Pull rule |
|---|---|---|
| `revalidation` | `not_attempted`, `passed`, `failed` | Destination origin and source target evidence must pass |
| `recovery` | `planned`, `preserved`, `failed` | Every pull replacement requires verified prior-source recovery |
| `recovery_ref` | optional operation-local reference | Present only after preservation |
| `staging` | `not_attempted`, `verified`, `failed` | Staged beside source and verified against expected destination |
| `publication` | `not_attempted`, `not_visible`, `visible` | Records source replacement visibility |
| `verification` | `not_attempted`, `verified`, `failed` | Compares final source with expected destination state |
| `durability_confirmed` | boolean | True only after source parent synchronization |

Visibility, verification, and durability remain independent.

## Recovery entry

The existing Recovery Metadata V1 remains structurally sufficient.

| Field | Type | Pull rule |
|---|---|---|
| `action_index` | unsigned integer | Unique within operation |
| `identity` | Entry Identity | Exact established managed entry |
| `prior_state` | Supported State | Prior source state for pull |
| `payload_ref` | operation-local relative reference | Derived from action index |
| `payload_present` | boolean | True only after exclusive publication |
| `verified` | boolean | True only after descriptor-bound reread matches prior source state |

Direction is bound by the parent operation plan and action index, so the recovery envelope does not duplicate a side field. Payloads remain private mode `0600`; the prior supported mode remains in metadata.

## Operation record

Partitioned Operation Record V1 keeps its existing layout and integrity rules:

```text
state/operations/<operation-id>/
├── plan.json
├── operation.json
├── actions/<zero-padded-index>.json
└── recovery/<zero-padded-index>/
    ├── metadata.json
    └── payload
```

The `operation` field admits exactly `push` or `pull`. A pull operation ID is opaque, exclusively allocated, and may use a `pull-` diagnostic prefix without deriving authority from that text. The immutable plan contains `direction: pull`; the summary and plan must agree. Checkpoint transitions and result-delivery states remain unchanged.

```json
{
  "schema_version": 1,
  "payload": {
    "operation_id": "pull-<opaque-suffix>",
    "operation": "pull",
    "state": "executing",
    "plan_id": "<digest>",
    "plan_ref": "plan.json",
    "baseline": {"outcome": "not_attempted"},
    "result_delivery": "not_attempted"
  },
  "integrity": {"algorithm": "sha256", "digest": "<digest>"}
}
```

## Mutation result

The existing Result Envelope V1 remains the outer wire contract. Shared mutation details are:

| Field | Type | Rules |
|---|---|---|
| `operation` | `push` or `pull` | Invoked command |
| `direction` | `push` or `pull` | Required for every push and pull plan or execution result |
| `mode` | `dry_run` or `execute` | Required |
| `completion` | `complete`, `blocked`, `partial`, or `failed` | Derived from plan and actions |
| `result` | `planned`, `no_op`, `applied`, `blocked`, or `failed` | Required |
| `scope` | Classification Scope | Required |
| `plan_id` | SHA-256 identity | Required after complete planning |
| `counts` | Mutation Counts | Shared shape for both directions |
| `entries` | ordered Entry Dispositions | Includes unmanaged destination non-actions |
| `actions` | ordered Mutation Actions | Mapping-role paths plus execution evidence |
| `blockers` | ordered Plan Blockers | Complete preflight set |
| `operation_record` | optional opaque availability | Present only after mutation initialization |
| `baseline` | Baseline Outcome | Required |

## State transitions

### Pull orchestration

```text
request
  -> complete preflight -> blocked result, no lock or operation record
  -> complete no-action plan -> no-op result, no lock or operation record
  -> dry run with actions -> planned result, no lock or operation record
  -> execute with actions -> acquire mutation lock
       -> lock-held plan differs -> stale failure, no record or payload mutation
       -> equivalent plan -> initialize pull record
            -> preserve source -> stage destination -> replace and verify source
            -> all actions verify -> final complete observation
                 -> baseline publishes -> completed/applied
                 -> baseline fails -> failed with precise authority evidence
            -> first action failure -> failed/partial, later actions unattempted, no baseline publication
```

### Result delivery

```text
terminal payload/baseline outcome -> delivery prepared
  -> render succeeds -> delivery delivered
  -> render fails -> delivery failed, exit 20, payload/baseline authority unchanged
```

### Later pull after interruption

```text
valid prior nonterminal pull record
  -> preserve it unchanged
  -> fresh inspection and new mutation-lock acquisition
  -> separate plan and operation ID
```

## Identity ordering and integrity

- Entry identity remains the Feature 004 `EntryIdentity`; mapping roles never swap.
- Plan identity includes direction, scope, accepted registry/state evidence, ordered entries, actions, expected mapping-side states, and blockers.
- Mode, messages, timestamps, operation ID, and mutable statuses remain excluded from plan identity.
- Pull entries and actions sort by canonical mapping source bytes and source-relative raw bytes; there are no synthetic pull parent actions.
- Operation envelopes, accepted state, and recovery metadata keep independent integrity domains.
- Recovery references remain operation-local relative components.
