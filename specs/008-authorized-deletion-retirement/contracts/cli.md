# CLI Contract: Delete, Retire, and Recovery

## Grammar

```text
grip [--output human|json] [-v...] delete [-n|--dry-run] (--source|--destination) [--] PATH
grip [--output human|json] [-v...] retire [-n|--dry-run] [--destination] [--force] (--all|[--] PATH)
grip [--output human|json] [-v...] recovery list
grip [--output human|json] [-v...] recovery show RECOVERY_REF
grip [--output human|json] [-v...] recovery restore [-n|--dry-run] [--] RECOVERY_REF
grip [--output human|json] [-v...] recovery remove [-n|--dry-run] --confirm RECOVERY_REF...
```

- Delete and retirement mutate by default; `-n` and `--dry-run` are equivalent previews.
- Delete requires exactly one authority flag and one source-space path. `--source` accepts source absence and removes the destination; `--destination` accepts destination absence and removes the source.
- Retirement requires exactly one path or `--all`. A path uses source space unless `--destination` is present. `--force` applies only to pending-retirement records with differing surviving copies.
- Recovery references use the closed opaque grammar in [recovery.md](recovery.md); filesystem paths are never accepted as recovery references or alternate restore destinations.
- Recovery cleanup requires one or more unique exact references and `--confirm` in execute mode. Preview accepts `--confirm` but changes nothing, allowing the same reviewed invocation to be repeated without `--dry-run`.
- Flags must appear before `--`; `--` then ends option parsing for a dash-prefixed delete or retirement path or recovery reference.

## Usage rejections

Exit `2` with no mutation for omitted or conflicting delete authority, missing or extra delete paths, bare retirement, simultaneous retirement path and `--all`, retirement `--destination` with `--all`, duplicate cleanup references, missing cleanup confirmation in execute mode, invalid recovery-reference grammar, extra list/show/restore arguments, or unknown subcommands.

## Result boundary

All commands preserve Result Envelope V1 and existing exit categories:

| Condition | Result | Exit | Mutation |
|---|---|---:|---|
| Valid read-only list/show | `complete` | 0 | none |
| Valid preview | `planned` or `blocked` | existing success/blocking category | none |
| Semantic no-op | `no_op` | 0 | none |
| Fully verified execution | `applied` | 0 | exact planned effect |
| Known blocker | `blocked` | 10 | none |
| Unsupported stored schema | failure | 11 | none |
| Corrupt private evidence | failure | 12 | none or prior completed cleanup effects |
| Active writer | failure | 13 | none |
| Stale or operational failure | `failed` or `partial` | 20 | completed effects retained and reported |

Human and JSON output communicate the same operation, scope, authority or force state, references, entries, actions, blockers, recovery availability, visibility, verification, durability, operation record, and accepted-state authority. Diagnostics remain on stderr.
