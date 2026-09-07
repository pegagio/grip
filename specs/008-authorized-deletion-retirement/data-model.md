# Data Model: Authorized Deletion and Retirement

This model extends Feature 007's classification, accepted-state, mutation coordination, result, and partitioned operation-record contracts. It preserves State Envelope V2, Registry V1, Operation Record V1, and historical recovery bytes while adding typed deletion, retirement, and recovery lifecycle models.

## Table of Contents

- [Deletion authority and request](#deletion-authority-and-request)
- [Deletion plan and action](#deletion-plan-and-action)
- [Retirement request and plan](#retirement-request-and-plan)
- [Recovery reference](#recovery-reference)
- [Recovery manifest and tombstone](#recovery-manifest-and-tombstone)
- [Recovery inventory entry](#recovery-inventory-entry)
- [Restore and cleanup plans](#restore-and-cleanup-plans)
- [Operation record extension](#operation-record-extension)
- [Public results](#public-results)
- [State transitions](#state-transitions)
- [Identity ordering and integrity](#identity-ordering-and-integrity)

## Deletion authority and request

### Deletion Authority

| Value | Authoritative evidence | Target removed |
|---|---|---|
| `source` | Accepted source absence | Remaining destination peer |
| `destination` | Accepted destination absence | Remaining source peer |

### Delete Request

| Field | Type | Rules |
|---|---|---|
| `mode` | `dry_run` or `execute` | Execute default; both preview aliases map to dry run |
| `authority` | Deletion Authority | Exactly one value required |
| `selector` | Safe path | Exactly one source-space exact entry or component-boundary subtree |

The selector remains source-space identity even when destination absence is authoritative.

## Deletion plan and action

### Deletion Plan

| Field | Type | Rules |
|---|---|---|
| `operation` | `delete` | Fixed |
| `authority` | Deletion Authority | Required |
| `plan_id` | SHA-256 identity | Covers semantic evidence; excludes mode and mutable outcomes |
| `scope` | Classification Scope | One exact identity or subtree |
| `entries` | ordered Deletion Dispositions | Exactly one per selected observation |
| `actions` | ordered Deletion Actions | Dense, child-first, dependency valid |
| `baseline_retirements` | ordered Entry Identities | Exactly the accepted records removed after successful deletion |
| `blockers` | ordered Plan Blockers | Complete discovered set |
| `counts` | Deletion Counts | Selected, actionable, blockers, completed, failed, unattempted |

### Deletion Disposition

| Value | Meaning |
|---|---|
| `action` | Eligible directional deletion with unchanged remaining peer |
| `no_action` | Selected evidence requires no removal |
| `blocked` | Classification, descendant ownership, or evidence prevents all mutation |

### Deletion Action

| Field | Type | Rules |
|---|---|---|
| `index` | unsigned integer | Dense execution order |
| `kind` | `remove_file` or `remove_directory` | Derived from verified remaining peer |
| `identity` | Entry Identity | Accepted managed identity |
| `authority` | Deletion Authority | Matches plan |
| `target_side` | `source` or `destination` | Opposite authority |
| `target_path` | Safe Path | Mapping-role path; never caller-composed |
| `expected_target` | Supported State | Must equal accepted baseline |
| `expected_absent_peer` | mapping-role side | Must remain absent |
| `expected_children` | ordered raw child identities | Required for directory; no unmanaged children |
| `dependencies` | lower action indexes | Child actions precede parent |
| `status` | action status | `unattempted -> in_progress -> completed|failed` |
| `milestones` | Deletion Evidence | Revalidation, recovery, removal, absence verification, durability |

Directory actions are ordered by mapping identity, relative-path depth descending, then raw relative bytes. A directory action may run only after every planned child completes and a fresh child enumeration is empty.

## Retirement request and plan

### Retirement Request

| Field | Type | Rules |
|---|---|---|
| `mode` | `dry_run` or `execute` | Execute default |
| `path_space` | `source` or `destination` | Applies only to path selection |
| `selector` | path or `all` | Exactly one required |
| `force` | boolean | Authorizes only comparison-history discard for differing survivors |

### Retirement Disposition

| Value | Meaning |
|---|---|
| `retire` | Eligible without force |
| `force_required` | Pending retirement has differing surviving evidence |
| `no_action` | No retained eligible record selected |
| `blocked` | Active, unsupported, unsafe, incomplete, or invalid evidence |

### Retirement Plan

| Field | Type | Rules |
|---|---|---|
| `operation` | `retire` | Fixed |
| `plan_id` | SHA-256 identity | Includes force decision and evidence |
| `scope` | Classification Scope | Exact selector or explicit all |
| `force` | boolean | Included in plan identity |
| `entries` | ordered Retirement Dispositions | Includes surviving differences |
| `retired_identities` | ordered Entry Identities | Removed from next accepted state |
| `blockers` | ordered blockers | Includes force-required entries when force is false |

Publication clones `AcceptedState`, removes only `retired_identities`, preserves every other record, and advances State V2 exactly once when the result differs.

## Recovery reference

`RecoveryRef` is a tagged opaque public identity. It is parsed into validated components before private storage access.

| Kind | Public grammar | Private identity source |
|---|---|---|
| Payload | `payload:<operation-id>:<action-index>` | Operation ID plus dense action index |
| Registry | `registry:sha256:<64-lower-hex>` | Content digest |
| Accepted state | `state:generation:<u64>:sha256:<64-lower-hex>` | Generation plus document digest |
| Operation | `operation:<operation-id>` | Opaque operation ID |

Operation references are inspectable provenance, not restore or cleanup targets. User input never supplies a filesystem path.

## Recovery manifest and tombstone

### Recovery Manifest V1

The manifest is a strict integrity-protected envelope. It is immutable after publication.

| Field | Type | Rules |
|---|---|---|
| `schema_version` | integer | Exactly `1` |
| `reference` | Recovery Ref | Matches containing recovery identity |
| `kind` | `payload`, `registry`, or `accepted_state` | Closed vocabulary |
| `created_at` | UTC timestamp | Diagnostic ordering tie-breaker only |
| `origin_operation` | optional operation ID and operation | Required when operation-derived |
| `origin_transition` | stable string | Replacement, deletion, registry publication, or state publication |
| `managed_identity` | optional Recovery Identity | Required for payload |
| `bound_side` | optional `source` or `destination` | Required for payload |
| `bound_target` | optional Safe Path | Required for payload; never arbitrary |
| `prior_evidence` | Supported State or document digest/generation | Exact recovered content identity |
| `expected_post_evidence` | absent, Supported State, or document digest/generation | Required to protect occupied/current target |
| `byte_count` | unsigned integer | Recoverable bytes at creation |
| `payload_ref` | validated private relative component | Internal only; never rendered as authority |
| `integrity` | SHA-256 | Covers canonical schema version and payload |

### Cleanup Tombstone V1

| Field | Type | Rules |
|---|---|---|
| `schema_version` | integer | Exactly `1` |
| `reference` | Recovery Ref | Matches immutable manifest |
| `cleaned_at` | UTC timestamp | When absence was verified |
| `removed_byte_count` | unsigned integer | Must match planned available bytes |
| `cleanup_operation_id` | operation ID | Binds explicit cleanup |
| `integrity` | SHA-256 | Covers canonical payload |

The manifest is never rewritten. Recoverable byte absence plus valid tombstone means `cleaned`; absence without a tombstone means `cleanup_incomplete` unless the entry was a validated legacy metadata-only record.

## Recovery inventory entry

| Field | Type | Rules |
|---|---|---|
| `reference` | Recovery Ref | Stable public identity |
| `kind` | Recovery kind | Required |
| `origin` | typed origin | Operation or authority transition |
| `managed_identity` | optional | Payload only |
| `bound_side` / `bound_path` | optional | Payload only |
| `created_at` | optional UTC timestamp | Missing for some legacy entries |
| `integrity` | `verified`, `failed`, or `unavailable` | Explicit |
| `availability` | `available`, `cleaned`, `cleanup_incomplete`, or `missing` | Explicit |
| `byte_count` | optional unsigned integer | No payload bytes are rendered |
| `provenance` | `complete` or `legacy_incomplete` | Controls restore eligibility |
| `restore_eligibility` | `eligible` or blocked reason | Computed from manifest and current evidence for show/restore |

Legacy storage is projected without mutation. Unknown schema, unsafe nodes, digest mismatch, or missing required components fail the selected inspection; unrelated valid entries may still be listed with a precise invalid-entry record only when the inventory contract can do so without trusting corrupt content.

## Restore and cleanup plans

### Restore Plan

| Field | Type | Rules |
|---|---|---|
| `operation` | `recovery_restore` | Fixed |
| `mode` | preview or execute | Mode excluded from semantic identity |
| `reference` | one payload, registry, or state Recovery Ref | Exact and required |
| `kind` | recovery kind | Determines target publisher |
| `bound_target` | mapping-role path or authority document | Derived only from manifest |
| `expected_current` | absent or exact post-transition evidence | Required |
| `displacement_recovery` | planned or not required | Planned for occupied exact post-state |
| `blockers` | ordered blockers | Complete compatibility result |
| `plan_id` | SHA-256 | Covers candidate, target, authorities, and current evidence |

### Cleanup Plan

| Field | Type | Rules |
|---|---|---|
| `operation` | `recovery_remove` | Fixed |
| `mode` | preview or execute | Required |
| `confirmed` | boolean | Execute requires true |
| `references` | canonical unique list | One or more payload, registry, or state refs |
| `actions` | ordered Cleanup Actions | Canonical reference order |
| `blockers` | ordered blockers | Missing, ambiguous, corrupt, cleaned, or active evidence |
| `plan_id` | SHA-256 | Covers exact availability and tombstone absence |

Duplicate references are invalid usage rather than silently deduplicated. Each cleanup action progresses `unattempted -> in_progress -> bytes_removed -> tombstone_published -> completed`, with failure retaining the last truthful milestone.

## Operation record extension

Partitioned Operation Record V1 retains `plan.json`, `operation.json`, and sparse `actions/` checkpoints. The closed operation vocabulary adds:

- `delete`
- `retire`
- `recovery_restore`
- `recovery_remove`

Operation publication accepts a typed immutable plan projection containing operation, plan ID, serialized plan, action count, and command-specific validator. It does not coerce these plans into transfer `MutationPlan`. Historical records remain byte-identical and valid.

## Public results

Result Envelope V1 remains the outer contract. New typed details include:

| Field | Applies to | Meaning |
|---|---|---|
| `operation` | all | Closed operation string |
| `authority` | delete | Authoritative absent side |
| `force` / `force_required` | retire | Explicit history-discard state |
| `references` / `reference` | recovery | Opaque identities only |
| `entries`, `actions`, `blockers`, `counts` | action workflows | Complete deterministic plan/outcome |
| `availability`, `cleaned`, `integrity`, `eligibility` | inventory | Recovery lifecycle |
| `operation_record` | actionful execution | Opaque operation ID |
| `baseline` | delete/retire/restore | Actual accepted-state authority |
| `visibility`, `verification`, `durability` | destructive effects | Independent result evidence |

Existing usage, invalid-configuration, unsupported-schema, corrupt-state, contention, and operational-failure categories remain unchanged.

## State transitions

### Directional deletion

```text
request -> complete plan
  -> blockers -> blocked, no lock or record
  -> preview -> planned, no lock or record
  -> execute -> mutation lock -> rebuild equal plan -> initialize record
       -> for each child-first action:
            revalidate -> preserve target -> remove -> verify absence
       -> final complete observation -> retire selected baselines once
       -> applied
  -> first failure -> completed effects retained, later actions unattempted,
                      recovery retained, prior baseline authoritative
```

### Retirement

```text
path or --all -> complete plan
  -> force-required without --force or other blocker -> blocked
  -> empty/already retired -> no-op
  -> preview -> planned
  -> execute -> mutation lock -> rebuild equal plan -> initialize record
       -> publish one State V2 generation without payload mutation -> applied
```

### Restore

```text
exact reference -> verify manifest/provenance/bytes/current compatibility
  -> blocked or legacy provenance incomplete -> no mutation
  -> preview -> planned
  -> execute -> mutation lock -> rebuild equal plan -> initialize record
       -> preserve exact occupied post-state when present
       -> stage/publish recovered bytes or authority document -> verify
       -> retain source recovery and do not accept payload baseline -> applied
```

### Cleanup

```text
exact refs + confirmation -> complete plan
  -> preview -> planned
  -> execute -> mutation lock -> rebuild equal plan -> initialize record
       -> unlink bytes -> sync parent -> verify absent -> publish tombstone
       -> stop after first failure; later entries unattempted
```

## Identity ordering and integrity

- Entry identity remains canonical mapping source plus raw relative bytes.
- Deletion actions sort by mapping, descending relative depth, then raw relative bytes; dependencies independently enforce child before parent.
- Retirement entries sort by Entry Identity; cleanup entries sort by Recovery Ref kind and canonical components.
- Every plan identity includes current registry/state snapshots and all evidence that can change eligibility or effect.
- Recovery references and private relative components use closed grammars and containment validation; display paths never become authority.
- Manifests, tombstones, accepted-state documents, registry documents, and operation components retain separate integrity domains.
- Timestamps are reported but never determine safety, uniqueness, or authority.
