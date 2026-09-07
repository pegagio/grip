# Data Model: Bidirectional Synchronization and Conflict Resolution

This model extends Feature 006's direction-aware mutation types while preserving mapping identity, State Envelope V2, Recovery Metadata V1, and Partitioned Operation Record V1.

## Table of Contents

- [Mutation operation](#mutation-operation)
- [Transfer direction and conflict winner](#transfer-direction-and-conflict-winner)
- [Mutation request](#mutation-request)
- [Mutation plan](#mutation-plan)
- [Entry disposition](#entry-disposition)
- [Mutation action](#mutation-action)
- [Operation record](#operation-record)
- [Mutation result](#mutation-result)
- [State transitions](#state-transitions)
- [Identity and integrity](#identity-and-integrity)

## Mutation operation

| Value | Scope | Plan policy |
|---|---|---|
| `push` | Existing selector contract | Source-to-destination eligibility |
| `pull` | Existing selector contract | Destination-to-source eligibility |
| `sync` | Existing selector contract | Both directional eligibility tables in one plan |
| `resolve` | One exact source-space Entry Identity | One divergent conflict plus explicit winner |

Operation identifies the invoked workflow, operation-record prefix, result wording, and lock owner. It does not identify the transfer direction of every action.

## Transfer direction and conflict winner

| Direction | Origin | Target | Resolution winner |
|---|---|---|---|
| `push` | mapping source | mapping destination | `source` |
| `pull` | mapping destination | mapping source | `destination` |

`ConflictWinner` is `source` or `destination` and is present only for resolve plans/results. Mapping-role `source_path` and `destination_path` never swap.

## Mutation request

| Field | Type | Rules |
|---|---|---|
| `operation` | Mutation Operation | Fixed by command |
| `mode` | `dry_run` or `execute` | Execute default; both preview aliases map to dry run |
| `path_space` | `source` or `destination` | Sync supports both; resolve is always source |
| `selector` | optional safe path | Sync accepts at most one; resolve requires exactly one |
| `winner` | optional Conflict Winner | Required exactly once for resolve; absent otherwise |

## Mutation plan

| Field | Type | Rules |
|---|---|---|
| `operation` | Mutation Operation | Included in plan identity |
| `direction` | optional Transfer Direction | Required for push/pull compatibility; absent for sync/resolve |
| `winner` | optional Conflict Winner | Required only for resolve |
| `plan_id` | SHA-256 identity | Digest of canonical semantic plan; excludes mode and execution state |
| `scope` | Classification Scope | Sync may be broad; resolve is one exact source-space identity |
| `entries` | ordered Entry Dispositions | Every selected record exactly once |
| `acceptance_identities` | ordered Entry Identity array | Actioned plus converged identities eligible only after complete success |
| `actions` | ordered Mutation Actions | Each payload action has one transfer direction |
| `blockers` | ordered Plan Blockers | Complete before execution |
| `counts` | Mutation Counts | Includes selected, actions, converged acceptance, no-action, blockers, and execution states |

A plan is mutating when it has payload actions or acceptance identities whose supported state differs from the accepted baseline. A synchronized-only plan is a semantic no-op.

## Entry disposition

| Classification | Sync disposition | Acceptance behavior |
|---|---|---|
| `source_addition`, `source_only_change` | push action | Accept after verified publication |
| `destination_only_change` | pull action | Accept after verified publication |
| `converged_two_sided_change` | no payload action | Accept only with complete successful sync |
| `synchronized` | no action | Preserve baseline |
| `destination_only_unmanaged` | no action | Report and never import |
| inherited conflict, deletion, retirement, unsupported, or unsafe blocker | blocked | Accept nothing |
| other nonblocking initial state | no action | Preserve existing authority |

Resolve admits exactly one `divergent_conflict` record and converts it to one action according to the explicit winner. Every other classification is a resolution blocker.

## Mutation action

| Field | Type | Rules |
|---|---|---|
| `index` | unsigned integer | Dense execution order |
| `direction` | Transfer Direction | Required for every payload action |
| `kind` | existing action kind | Sync reuses push additions/directories and directional file replacement; resolve is replacement only |
| `identity` | optional Entry Identity | Required except existing synthetic push parent actions |
| `source_path` / `destination_path` | Safe Path | Stable mapping roles |
| `expected_source` / `expected_destination` | optional Supported State | Preserve complete classification evidence |
| `target_path` | internal absolute path | Destination for push, source for pull; never serialized as authority |
| `dependencies` | action index array | Existing parent-before-child rules |
| `status` | action status | `unattempted`, `in_progress`, `completed`, or `failed` |
| `milestones` | Action Evidence | Existing revalidation/recovery/staging/publication/verification contract |

State transitions remain:

```text
unattempted -> in_progress -> completed
                         -> failed
```

The first failure leaves all later actions unattempted regardless of direction.

## Operation record

Partitioned Operation Record V1 retains its layout:

```text
state/operations/<operation-id>/
├── plan.json
├── operation.json
├── actions/<zero-padded-index>.json
└── recovery/<zero-padded-index>/
    ├── metadata.json
    └── payload
```

The closed `operation` field admits `push`, `pull`, `sync`, and `resolve`. New immutable plans contain matching `operation`; each action contains direction; resolve plans also contain winner. Historical push/pull plans with matching top-level `direction` remain valid. Unknown operations, mismatched direction/winner, non-dense actions, invalid recovery references, or inconsistent plan identities are corrupt state.

## Mutation result

Result Envelope V1 remains the outer contract. Mutation details contain:

| Field | Type | Rules |
|---|---|---|
| `operation` | Mutation Operation | Required |
| `direction` | optional Transfer Direction | Retained for push/pull only |
| `winner` | optional Conflict Winner | Present for resolve |
| `mode` | Mutation Mode | Required |
| `completion` / `result` | stable strings | Distinguish planned, blocked, no-op, applied, partial, and failed |
| `scope` / `plan_id` | structured identity | Required after complete planning |
| `counts` | Mutation Counts | Includes converged acceptance count |
| `entries` / `actions` / `blockers` | ordered arrays | Action direction required |
| `operation_record` | optional opaque reference | Present only after mutation initialization |
| `baseline` | Baseline Outcome | Explicit authority and durability |

## State transitions

### Sync

```text
request -> complete plan
  -> blockers -> blocked, no lock or record
  -> synchronized-only -> no-op, no lock or record
  -> preview -> planned, no lock or record
  -> execute -> lock -> rebuild equal plan
       -> initialize record
       -> execute mixed actions in canonical order
       -> final complete observation
       -> accept actioned and converged identities once
       -> completed/applied
```

### Resolve

```text
exact source-space path + winner
  -> fresh divergent conflict -> one directional plan
  -> preview -> planned, no lock or record
  -> execute -> lock -> rebuild same conflict/winner plan
       -> preserve loser -> stage winner -> replace loser -> verify
       -> publish accepted baseline -> completed/applied
  -> stale or non-conflicting evidence -> blocked/failure before mutation
```

### Failure

```text
first action failure -> completed effects retained
                     -> recovery retained
                     -> later actions unattempted
                     -> prior baseline authoritative
                     -> no acceptance of converged entries
```

## Identity and integrity

- Canonical source path remains mapping and resolution identity.
- Entry/action ordering uses canonical mapping source bytes and source-relative raw bytes; direction is never the primary sort key.
- Plan identity includes operation, optional winner, scope, accepted registry/state evidence, ordered entries, acceptance identities, actions with direction, and blockers.
- Mode, messages, timestamps, operation ID, and mutable checkpoints are excluded from plan identity.
- Old operation records remain immutable; compatibility is read validation, never rewrite.
- Recovery references remain operation-local and are bound through operation, plan, action index, identity, and direction.
