# CLI Contract: Mapping Lifecycle

This contract extends the Feature 001 application-command and result-envelope contract.

## Grammar

```text
grip [--output human|json] [-v...] mapping add file SOURCE DESTINATION
grip [--output human|json] [-v...] mapping add tree SOURCE DESTINATION
grip [--output human|json] [-v...] mapping list
grip [--output human|json] [-v...] mapping show SOURCE
grip [--output human|json] [-v...] mapping remove SOURCE
```

`SOURCE` and `DESTINATION` are one path argument each. `--` terminates option parsing. Feature 002 accepts absolute paths only; stored and returned paths are canonical absolute UTF-8 paths.

## Operations

| Operation | Registry effect | Payload or synchronization-state effect |
|---|---|---|
| `mapping add file` | Retain prior registry recovery, then add one validated file tuple | No payload or synchronization-state change |
| `mapping add tree` | Retain prior registry recovery, then add one validated tree-root tuple | No payload or synchronization-state change; no recursive discovery |
| `mapping list` | None | None |
| `mapping show` | None | None |
| `mapping remove` | Retain prior registry recovery, then remove one tuple by canonical source | No payload or synchronization-state change |

Every operation validates the complete accepted registry. Add and remove then validate the complete candidate again under the registry publication lock.

## Machine-readable success

The Feature 001 envelope remains unchanged. A mapping has exactly these fields:

```json
{
  "kind": "file",
  "source": "/Users/pegagio/GripSource/git/.gitconfig",
  "destination": "/Users/pegagio/.gitconfig"
}
```

Add, show, and remove success use:

```json
{
  "schema_version": 1,
  "status": "ok",
  "code": "ok",
  "message": "Mapping recorded",
  "details": {
    "operation": "mapping_add",
    "mapping": {
      "kind": "file",
      "source": "/Users/pegagio/GripSource/git/.gitconfig",
      "destination": "/Users/pegagio/.gitconfig"
    }
  }
}
```

For remove, `operation` is `mapping_remove` and `mapping` is the removed tuple. Show uses `mapping_show`. List success uses `mapping_list` and a `mappings` array ordered by canonical source path; an empty list succeeds with an empty array.

## Machine-readable failure

Mapping failures retain the standard envelope and add stable details:

```json
{
  "schema_version": 1,
  "status": "error",
  "code": "invalid_configuration",
  "message": "Mapping ownership conflicts with the accepted registry",
  "details": {
    "operation": "mapping_add",
    "reason": "destination_overlap",
    "paths": [
      "/Users/pegagio/.config/editor",
      "/Users/pegagio/.config/editor/themes"
    ]
  }
}
```

Stable `reason` values are:

- `mapping_not_found`
- `invalid_registry`
- `duplicate_source`
- `duplicate_tuple`
- `source_overlap`
- `destination_overlap`
- `equal_endpoints`
- `recursive_topology`
- `cross_mapping_recursion`
- `relative_path`
- `parent_traversal`
- `non_utf8_path`
- `path_unavailable`
- `unsafe_ancestry`
- `symlink_endpoint`
- `wrong_node_kind`
- `unsupported_node`
- `stale_registry`
- `stale_path_evidence`
- `registry_contention`
- `publication_failure`
- `registry_recovery_failure`
- `unsafe_registry_mode`

Multiple ownership conflicts are returned in deterministic order under `details.conflicts`; the top-level `reason` is `ownership_conflicts`.

## Exit behavior

| Exit | Code | Mapping use |
|---:|---|---|
| 0 | `ok` | Successful add, list, show, or remove |
| 2 | `invalid_usage` | Missing/extra argument, invalid subcommand, or malformed option |
| 10 | `invalid_configuration` | Invalid mapping, path, topology, existing registry, not-found identity, or stale accepted evidence |
| 11 | `unsupported_schema` | Unsupported registry schema |
| 12 | `corrupt_state` | Unsafe Grip-owned registry lock or staging node |
| 20 | `operational_failure` | Lock contention, inaccessible I/O, output failure, sync failure, or rename failure |

No feature-specific exit code is added. Diagnostics remain on stderr and never alter the result envelope.
