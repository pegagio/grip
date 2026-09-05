# CLI Contract: Mapping Discovery

This contract extends the Feature 001 result envelope and Feature 002 mapping command family without adding baseline-informed status semantics.

## Grammar

```text
grip [--output human|json] [-v...] mapping inspect [SOURCE]
```

`SOURCE` is optional and accepts one mapping source identity. `--` terminates option parsing. No selector covers a nested member or destination path in this feature.

## Operation

| Form | Scope | Mutation |
|---|---|---|
| `mapping inspect` | Every accepted mapping in canonical source order | None |
| `mapping inspect SOURCE` | Exactly one mapping selected through canonical source identity | None |

Every invocation validates the complete accepted registry before selecting or inspecting mappings. File mappings yield one exact eligible record. Tree mappings yield dynamic member records under [the discovery contract](discovery.md) and [Gripignore contract](gripignore.md).

## Machine-readable success

A complete inspection returns exit `0`, including when blocking records exist:

```json
{
  "schema_version": 1,
  "status": "ok",
  "code": "ok",
  "message": "Discovery completed with 3 records and 1 blocking finding",
  "details": {
    "operation": "mapping_inspect",
    "scope": {
      "kind": "mapping",
      "source": "/example/source/editor"
    },
    "counts": {
      "eligible": 1,
      "ignored": 1,
      "destination_only": 0,
      "unsupported_source": 1,
      "unsafe_destination_collision": 0
    },
    "blocking_count": 1,
    "records": [
      {
        "category": "eligible",
        "mapping_kind": "tree",
        "mapping_source": "/example/source/editor",
        "relative_path": {"display": "config.toml"},
        "source_path": {"display": "/example/source/editor/config.toml"},
        "destination_path": {"display": "/example/destination/editor/config.toml"},
        "node_kind": "file",
        "blocking": false
      }
    ]
  }
}
```

For all mappings, `scope.kind` is `all` and `source` is absent. Counts include all five keys even when zero. Records are ordered by canonical mapping source, relative raw bytes, and category. Path objects add `raw_hex` only when needed to retain exact non-UTF-8 identity.

## Human-readable success

Human output begins with one count summary and then one stable line per record. Each line includes category, mapping source, relative or exact path, paired destination, detected kind, and reason when present. Control and non-UTF-8 bytes are escaped; diagnostics never share stdout.

An ignored directory line describes the exclusion root only. Grip does not claim to have inspected or enumerated its pruned descendants.

## Machine-readable failure

Failures retain the existing envelope and include stable details:

```json
{
  "schema_version": 1,
  "status": "error",
  "code": "operational_failure",
  "message": "Discovery evidence changed before the inventory was complete",
  "details": {
    "operation": "mapping_inspect",
    "reason": "stale_discovery_evidence",
    "paths": [
      {"display": "/example/source/editor/themes"}
    ]
  }
}
```

Stable discovery failure reasons are:

- `mapping_not_found`
- `invalid_registry`
- `invalid_policy`
- `non_utf8_policy`
- `unreadable_policy`
- `path_unavailable`
- `directory_unreadable`
- `stale_discovery_evidence`
- `discovery_failure`

Unsupported payload nodes are records, not command failures, when Grip completes both passes.

## Exit behavior

| Exit | Code | Discovery use |
|---:|---|---|
| 0 | `ok` | Complete inventory, including inventories with blocking records |
| 2 | `invalid_usage` | Extra selector, malformed option, or invalid command grammar |
| 10 | `invalid_configuration` | Invalid mapping selector, registry, malformed/non-UTF-8 policy, or structurally invalid mapped path |
| 11 | `unsupported_schema` | Unsupported registry schema |
| 12 | `corrupt_state` | Reserved by the existing envelope; discovery does not inspect or publish machine state or coordination nodes |
| 20 | `operational_failure` | Read/stat/enumeration/output failure, unreadable policy, or stale discovery evidence |

No public state-contention or attention exit is added. Feature 004 owns synchronization-state automation semantics.

## Non-mutation contract

Success, blockers, invalid input, stale evidence, operational failure, and output failure do not create or modify registry data, state, locks, staging paths, recovery evidence, baselines, policy files, source/destination payloads, or version-control state.
