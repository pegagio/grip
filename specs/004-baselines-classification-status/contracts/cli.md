# CLI Contract: Baselines, Classification, and Status

This contract extends the established Result Envelope V1 and leaves `mapping inspect` as the membership-only Feature 003 command.

## Grammar

```text
grip [--output human|json] [-v...] status [--destination] [--] [PATH]
grip [--output human|json] [-v...] check [--destination] [--] [PATH]
grip [--output human|json] [-v...] diff [--destination] [--] [PATH]
grip [--output human|json] [-v...] baseline accept [--destination] [--] [PATH]
```

All four commands share one selector grammar. Omitted `PATH` covers every current mapping plus retained baseline-only identities. Source path space is the default; `--destination` selects destination path space. `--` terminates option parsing. More than one path is invalid usage.

## Scope resolution

| Input | Scope |
|---|---|
| no `PATH` | all current mappings and retained baseline identities |
| exact file-mapping path | one file entry |
| exact tree-mapping root | complete tree mapping |
| path beneath tree mapping | one entry or component-boundary subtree |
| deleted managed path | baseline-backed entry or subtree |
| untracked retained path | baseline-backed pending-retirement entry or subtree; no removed payload path is reopened |
| ignored or destination-only unmanaged path | explicit unmanaged result |
| path outside mapping and baseline identity | invalid request |

Every command validates the complete registry and accepted state before narrowing scope. A destination selector resolves back to its complete Mapping Snapshot and raw relative identity.

## Machine-readable classification result

The outer envelope remains version 1:

```json
{
  "schema_version": 1,
  "status": "ok",
  "code": "attention_required",
  "message": "Check complete: 1 entry requires attention",
  "details": {
    "operation": "check",
    "completion": "complete",
    "state": "attention_required",
    "scope": {
      "kind": "entry",
      "path_space": "source",
      "selector": {"display": "/example/source/config"},
      "mapping_source": "/example/source/config"
    },
    "counts": {
      "source_addition": 0,
      "initial_match": 1,
      "initial_collision": 0,
      "destination_only_unmanaged": 0,
      "synchronized": 0,
      "source_only_change": 0,
      "destination_only_change": 0,
      "converged_two_sided_change": 0,
      "divergent_conflict": 0,
      "source_side_deletion": 0,
      "destination_side_deletion": 0,
      "delete_change_conflict": 0,
      "change_delete_conflict": 0,
      "converged_deletion": 0,
      "newly_ignored_pending_retirement": 0,
      "untracked_pending_retirement": 0,
      "unsupported_managed": 0,
      "unsafe_collision": 0
    },
    "attention_count": 1,
    "blocking_count": 0,
    "records": [
      {
        "classification": "initial_match",
        "mapping_kind": "file",
        "mapping_source": "/example/source/config",
        "relative_path": null,
        "source_path": {"display": "/example/source/config"},
        "destination_path": {"display": "/example/destination/config"},
        "source": {
          "node_kind": "file",
          "content_sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
          "length": 42,
          "permission_mode": "0644"
        },
        "destination": {
          "node_kind": "file",
          "content_sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
          "length": 42,
          "permission_mode": "0644"
        },
        "baseline": null,
        "prospective_direction": "none",
        "changed_dimensions": {
          "source_to_baseline": null,
          "destination_to_baseline": null,
          "source_to_destination": []
        },
        "attention": true,
        "blocking": false,
        "reasons": ["baseline_uninitialized"]
      }
    ]
  }
}
```

All 18 count keys are always present. Records use the canonical order from [the classification contract](classification.md). Safe paths add `raw_hex` only when required. `null` comparison means unavailable; `[]` means a valid comparison found no changed dimensions. File contents never appear.

`status` and `check` render the same classification data. `diff` uses the same record schema but emphasizes all three changed-dimension comparisons. The `operation` field is the invoked command.

## Machine-readable baseline result

Changed acceptance:

```json
{
  "schema_version": 1,
  "status": "ok",
  "code": "ok",
  "message": "Accepted 3 baseline entries; generation 2",
  "details": {
    "operation": "baseline_accept",
    "completion": "complete",
    "result": "accepted",
    "scope": {"kind": "mapping", "path_space": "source", "selector": {"display": "/example/source"}},
    "selected_count": 3,
    "changed_count": 3,
    "published": true,
    "generation": 2
  }
}
```

An already-current request returns `result: "already_current"`, `changed_count: 0`, `published: false`, and the unchanged generation. An uninitialized empty scope may omit generation and remains a no-op.

An ineligible baseline request returns all selected corrective classification records under reason `baseline_not_acceptable`; it never returns only the first offending entry.

## Human-readable contract

Human output derives from the same typed result. A classification begins with one stable summary and follows with one line per ordered record. Status and check show category, safe path identity, direction, attention, and blocking state. Diff additionally emits the three comparison labels and their ordered changed dimensions. Unavailable and no-change comparisons are visibly distinct.

Representative output:

```text
Check complete: 1 entry; 1 attention; 0 blocking
initial_match /example/source/config direction=none attention=yes blocking=no
  source_to_baseline unavailable
  destination_to_baseline unavailable
  source_to_destination none
```

Baseline acceptance prints either the accepted count and generation or `Baseline already current; no state published`. Diagnostics remain on stderr and never alter result semantics.

## Exit behavior

| Exit | Code | Use |
|---:|---|---|
| 0 | `ok` | Completed status or diff; clean check; accepted or already-current baseline |
| 1 | `attention_required` | Completed check with one or more attention records; top-level status remains `ok` |
| 2 | `invalid_usage` | Extra selector, malformed option, or invalid grammar |
| 10 | `invalid_configuration` | Invalid registry/selector/policy or baseline request containing ineligible evidence |
| 11 | `unsupported_schema` | Unsupported registry or state schema |
| 12 | `corrupt_state` | Unsafe, malformed, integrity-invalid, duplicate, unsorted, or inconsistent accepted state |
| 13 | `state_contention` | Another state publication holds the bounded state lock |
| 20 | `operational_failure` | Read/hash/enumeration/output/stale-evidence/publication failure or registry-publication contention |

`attention_required` means inspection completed; it is not an error. A post-rename directory-sync failure uses exit 20 with `publication_visible: true` and `durability_confirmed: false`.

## Non-mutation boundary

Status, check, and diff never create or modify registry, state, locks, recovery, policies, payloads, or Git state on success or failure. Baseline acceptance may create or replace only Grip-owned state publication artifacts after complete acceptance; it never changes mappings, policies, source, destination, or Git state.
