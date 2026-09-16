# Data Model: Aggregate Forced Push

## Aggregate Force Request

| Field | Meaning | Validation |
|---|---|---|
| `operation` | `aggregate_force_push` | Created only for `push --force` without a selector. |
| `project` | Selected managed project | Must resolve under existing selected-project rules. |
| `winning_side` | `source` | Fixed by the command; no destination-winning aggregate exists. |
| `selector` | Absent | A supplied selector uses existing exact-force behavior. |
| `dry_run` | Whether execution is non-mutating | Uses the same planned scope and result view. |

## Aggregate Entry Disposition

Each managed identity in deterministic project order has one disposition.

| Field | Meaning |
|---|---|
| `identity` | Stable managed-entry identity and displayed relative path. |
| `classification` | Existing inspection classification for source and destination state. |
| `source_complete` | The source-present payload or authorized source absence to make authoritative. |
| `disposition` | `unchanged`, `planned`, or `blocked`. |
| `action_indexes` | All mutation-plan actions required to complete this entry, including finalizers. |
| `blockers` | Safety or validation reasons; populated before any mutation. |

An aggregate scope may include unchanged entries for complete reporting, but it creates work only for entries requiring supported source-winning propagation.

## Entry Completion Record

| Field | Meaning |
|---|---|
| `identity` | Entry being processed. |
| `required_actions` | Ordered actions associated with that entry. |
| `revalidated` | Evidence was current immediately before each applicable action. |
| `verified` | The completed destination state matched the planned source-complete state. |
| `publication` | `not_required`, `published`, or a typed publication failure. |
| `status` | `unchanged`, `completed`, `failed`, or `unattempted`. |

An entry becomes `completed` only after every required action is revalidated and verified. It must not publish baseline evidence while a required action or finalizer remains pending.

## Accepted Publication Ledger

The aggregate operation result accounts for each State V4 publication independently.

| Field | Meaning |
|---|---|
| `starting_generation` | Accepted-state generation read before execution. |
| `publications` | Ordered completed-entry publications with their resulting generation or equivalent state evidence. |
| `current_snapshot` | Reloaded state used by subsequent revalidation. |
| `aggregate_outcome` | `succeeded`, `failed`, `blocked`, or `dry_run`. |
| `unaccepted_identities` | Failed and later unattempted entries. |

The ledger must not collapse per-entry publications into a single all-or-nothing baseline result. A failed aggregate can therefore contain valid earlier publications while remaining failed.

## State Transitions

```text
requested
  -> inspected
  -> blocked                         (any aggregate preflight blocker; no mutations)
  -> planned
  -> dry_run_complete                (dry run; no mutations or publications)
  -> executing(entry n)
  -> verified(entry n)
  -> published(entry n)
  -> reload_state
  -> executing(entry n+1)
  -> succeeded                       (all planned entries complete)

executing / verified / published
  -> failed(entry n)                 (stop; later entries are unattempted)
```

The implementation may use existing internal result types, but it must preserve these observable states and distinctions.
