# Data Model: Baselines, Classification, and Status

This model separates ephemeral filesystem evidence, version-neutral accepted state, strict wire envelopes, and pure classification results. Persisted identity never depends on inode values, and persisted baselines never contain payload bytes.

## Table of Contents

- [Selection request](#selection-request)
- [Mapping snapshot](#mapping-snapshot)
- [Entry identity](#entry-identity)
- [Supported state](#supported-state)
- [Observed entry](#observed-entry)
- [Accepted state](#accepted-state)
- [State Envelope V2](#state-envelope-v2)
- [Classification record](#classification-record)
- [Classification result](#classification-result)
- [Baseline acceptance](#baseline-acceptance)
- [State transitions](#state-transitions)
- [Validation order](#validation-order)

## Selection request

| Field | Type | Rules |
|---|---|---|
| `operation` | `status`, `check`, `diff`, or `baseline_accept` | Required |
| `path_space` | `source` or `destination` | Defaults to `source`; `destination` requires `--destination` |
| `selector` | optional Safe Path | At most one; absent means all current mappings plus all retained baseline identities |
| `scope` | resolved all, mapping, entry, or subtree | Derived after complete registry and accepted-state validation |

A selector resolves against current canonical mapping paths first and retained baseline mapping snapshots when current ownership is absent. Tree selection uses component boundaries over exact relative bytes, never string-prefix matching. A path outside both current mappings and retained baseline identity is invalid.

## Mapping snapshot

The durable mapping identity captured with each baseline record.

| Field | Type | Rules |
|---|---|---|
| `kind` | `file` or `tree` | Required |
| `source` | canonical absolute UTF-8 path | Required and equal to accepted mapping identity at acceptance time |
| `destination` | canonical absolute UTF-8 path | Required and paired with `source` at acceptance time |

The full tuple is the identity. A mapping removed and later recreated with a different kind or destination does not adopt the former baseline. An exact recreation may match retained evidence but does not retire or rewrite it implicitly.

## Entry identity

| Field | Type | Rules |
|---|---|---|
| `mapping` | Mapping Snapshot | Required |
| `relative_path_hex` | lowercase even-length hexadecimal | Empty for a file mapping; for a tree mapping, losslessly encodes a safe non-absolute relative path with no empty, dot, or parent components |

Ordering is canonical mapping source, mapping kind, canonical destination, then decoded raw relative bytes. The persisted array and all public classification records use this ordering. Duplicate identities are corrupt state.

## Supported state

A complete comparable state for one supported node.

| Field | File | Directory | Equality role |
|---|---|---|---|
| `node_kind` | `file` | `directory` | Required |
| `content.algorithm` | `sha256` | forbidden | Required for file equality |
| `content.digest` | 64 lowercase hexadecimal characters | forbidden | Required for file equality |
| `content.length` | unsigned byte count | forbidden | Supporting evidence; equality still requires digest |
| `permission_mode` | four-digit octal string for `st_mode & 0o7777` | forbidden | Required for file equality |

Current observation may additionally carry modification time in seconds and nanoseconds as diagnostic evidence. Mtime is never copied into `SupportedState`, persisted, or emitted as a changed equality dimension.

Changed dimensions are stable ordered values: `node_kind`, `content`, and `permission_mode`. Node-kind mismatch prevents content or mode equivalence claims for the incompatible pair.

## Observed entry

One normalized current identity produced by a stable two-pass observation.

| Field | Type | Rules |
|---|---|---|
| `identity` | Entry Identity | Current mapping snapshot plus raw relative identity |
| `membership` | `active`, `ignored`, or `destination_only` | Derived from current source discovery and baseline presence |
| `source` | optional Supported State | Present only for safely inspected eligible source evidence |
| `destination` | optional Supported State | Present only for safely inspected eligible paired destination evidence |
| `source_diagnostic` | optional current metadata | May include mtime; never equality evidence |
| `destination_diagnostic` | optional current metadata | May include mtime; never equality evidence |
| `unsupported` | ordered stable reasons | Reasons for evidence that cannot become Supported State |
| `blocking` | boolean | True for unsupported managed source or unsafe paired destination evidence |
| `ephemeral_evidence` | registry, node, directory, policy, descriptor, and fingerprint evidence | Command-local; discarded after stability or publication checks |

The observation join merges source discovery, paired destination inspection, destination-only inventory, and policy results by Entry Identity. It does not infer equality from separate presentation records.

## Accepted state

The version-neutral domain representation loaded from any supported state envelope.

| Field | Type | Rules |
|---|---|---|
| `generation` | unsigned integer | V1 value retained; absent state has no generation until first publication |
| `baselines` | ordered map of Entry Identity to Supported State | Empty for V1; unique and canonical for V2 |
| `accepted_bytes` | absent or exact bytes | Used only for expected-snapshot comparison and recovery |
| `file_identity` | absent or ephemeral safe file evidence | Used only for revalidation; never serialized |

Missing state yields `Uninitialized`. A valid V1 document yields an empty-baseline Accepted State. Unsupported versions fail with `unsupported_schema`; malformed, unsafe, integrity-invalid, unsorted, duplicate, or semantically invalid state fails with `corrupt_state`.

## State Envelope V2

```json
{
  "schema_version": 2,
  "payload": {
    "generation": 4,
    "baselines": [
      {
        "mapping": {
          "kind": "file",
          "source": "/example/source/config",
          "destination": "/example/destination/config"
        },
        "relative_path_hex": "",
        "state": {
          "node_kind": "file",
          "content": {
            "algorithm": "sha256",
            "digest": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "length": 42
          },
          "permission_mode": "0644"
        }
      }
    ]
  },
  "integrity": {
    "algorithm": "sha256",
    "digest": "<digest-of-canonical-version-and-payload>"
  }
}
```

Every named object rejects unknown fields. The V2 integrity input is the deterministic compact serialization of the typed object containing `schema_version` followed by `payload`, excluding `integrity`. Baselines are already sorted; no generic map ordering participates. The digest is recomputed and compared before conversion to Accepted State.

## Classification record

| Field | Type | Rules |
|---|---|---|
| `identity` | Entry Identity | Required |
| `classification` | stable classification | Required |
| `source` | optional current fingerprint evidence | `null` when absent or unsupported |
| `destination` | optional current fingerprint evidence | `null` when absent or unsupported |
| `baseline` | optional Supported State | Present only for accepted evidence |
| `prospective_direction` | `source_to_destination`, `destination_to_source`, or `none` | Informational; never executes in this feature |
| `changed_dimensions.source_to_baseline` | optional ordered dimensions | `null` when comparison unavailable; empty when valid and equal |
| `changed_dimensions.destination_to_baseline` | optional ordered dimensions | Same semantics |
| `changed_dimensions.source_to_destination` | optional ordered dimensions | Same semantics |
| `attention` | boolean | Drives `check` result |
| `blocking` | boolean | Known unsafe evidence for later mutation |
| `reasons` | ordered stable reason values | Empty only when classification needs no explanation |

Stable classifications are:

1. `source_addition`
2. `initial_match`
3. `initial_collision`
4. `destination_only_unmanaged`
5. `synchronized`
6. `source_only_change`
7. `destination_only_change`
8. `converged_two_sided_change`
9. `divergent_conflict`
10. `source_side_deletion`
11. `destination_side_deletion`
12. `delete_change_conflict`
13. `change_delete_conflict`
14. `converged_deletion`
15. `newly_ignored_pending_retirement`
16. `untracked_pending_retirement`
17. `unsupported_managed`
18. `unsafe_collision`

`initial_match` is attention-worthy until accepted. `destination_only_unmanaged` is informational and nonblocking. `synchronized` is clean. Unsupported and unsafe classifications are blocking. Every other non-clean managed classification requires attention.

## Classification result

| Field | Type | Rules |
|---|---|---|
| `operation` | `status`, `check`, or `diff` | Required |
| `completion` | `complete` | Present only after stable complete inspection |
| `state` | `clean` or `attention_required` | Derived from records |
| `scope` | resolved scope | Includes path space and safe selector when present |
| `counts` | all 18 classification counts | Every key present, including zero |
| `attention_count` | unsigned integer | Exact count where `attention=true` |
| `blocking_count` | unsigned integer | Exact count where `blocking=true` |
| `records` | ordered Classification Records | Complete selected union, canonically ordered |

Status and diff return result code `ok` after completion. Check returns `ok` for `clean` and `attention_required` for attention while top-level completion status remains `ok`.

## Baseline acceptance

### Candidate

| Field | Type | Rules |
|---|---|---|
| `expected_registry` | exact accepted Registry Snapshot | Required |
| `expected_state` | Accepted State snapshot | Required |
| `scope` | resolved selection | Required |
| `selected_records` | ordered Classification Records | Complete for selection, including baseline-only records |
| `next_baselines` | full ordered baseline map | Clone expected map; replace only acceptable selected identities |
| `changed_count` | unsigned integer | Number of selected baseline records whose semantic value changes |

Acceptance is permitted only when every selected active entry has complete supported equivalent source and destination state and no selected deletion, unsupported, collision, ignored, or pending-retirement evidence exists. Empty tree scope and a candidate semantically equal to expected baselines are valid no-ops.

### Result

| Field | Type | Rules |
|---|---|---|
| `result` | `accepted` or `already_current` | Required |
| `selected_count` | unsigned integer | Number of selected active entries considered |
| `changed_count` | unsigned integer | Zero for `already_current` |
| `published` | boolean | True only after accepted rename |
| `generation` | unsigned integer or absent | New or unchanged generation; absent only when uninitialized empty scope remains a no-op |

## State transitions

```text
Absent
  -> read-only inspection -> Uninitialized, no write
  -> accepted non-empty candidate -> V2 generation 0
  -> accepted empty candidate -> Uninitialized no-op

Valid V1 generation N
  -> read-only inspection -> empty-baseline Accepted State at N, no write
  -> accepted changed candidate -> V2 generation N+1
  -> accepted empty/already-current candidate -> V1 N no-op

Valid V2 generation N
  -> read-only inspection -> V2 N unchanged
  -> accepted changed candidate -> recover V2 N, publish V2 N+1
  -> already-current candidate -> V2 N no-op

Any accepted state
  -> mapping removed -> baseline bytes unchanged; records classify untracked pending retirement
  -> policy newly ignores baseline identity -> baseline bytes unchanged; record classifies newly ignored pending retirement
  -> publication failure before rename -> prior accepted state authoritative
  -> directory-sync failure after rename -> candidate visible, durability unconfirmed, operational failure
```

## Validation order

### Read-only classification

1. Resolve Grip Home; load and validate exact registry bytes, durable canonical ownership, and topology without requiring current source presence.
2. Load absent, V1, or V2 accepted state through strict version dispatch and semantic validation.
3. Resolve the optional source/destination selector against current mappings and retained baseline identities.
4. Produce the first complete source/destination observation and stream supported file fingerprints.
5. Produce a second complete observation from fresh descriptors.
6. Revalidate accepted registry and state snapshots; require equivalent observations and fingerprints.
7. Join current and baseline identities, classify through the pure matrix, sort, derive counts, and render without mutation.

### Baseline acceptance

1. Complete the read-only sequence and build a full copy-on-write candidate.
2. Acquire registry then state publication locks without blocking.
3. Reload and exactly compare expected registry and state bytes plus safe file identities.
4. Repeat the selected observation and fingerprint process from fresh descriptors and rebuild the candidate.
5. Reject the complete request if any selected record is unacceptable.
6. Revalidate registry immediately before publication.
7. If candidate baselines equal accepted baselines, report `already_current` without recovery, staging, generation, or state write.
8. Otherwise assign the exact next generation, preserve and verify prior accepted bytes when present, stage and decode the V2 candidate, verify semantic equality and path identity, atomically rename, sync, and report the truthful result.
