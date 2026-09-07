# CLI Contract: Sync and Resolve

## Grammar

```text
grip [--output human|json] [-v...] sync [-n|--dry-run] [--destination] [--] [PATH]
grip [--output human|json] [-v...] resolve [-n|--dry-run] (--source|--destination) [--] PATH
```

- Execute mode is the default; `-n` and `--dry-run` are equivalent previews.
- Sync accepts at most one optional selector and preserves the established source/destination path-space rules.
- Resolve requires exactly one path in source space and exactly one winner flag.
- Resolve's `--destination` selects the destination as winner; it never changes path interpretation.
- Resolve rejects omitted or multiple paths, omitted or conflicting winner flags, and destination-space inference as invalid usage with exit `2`.
- Winner flags must appear before `--`; `--` then ends option parsing for dash-prefixed source paths.

## Outcomes

| Condition | Result | Exit | Mutation |
|---|---|---:|---|
| Complete plan has blockers | `blocked` | inherited stable category | none |
| Selected evidence is synchronized | `no_op` | 0 | none |
| Preview has payload or acceptance work | `planned` | 0 | none |
| Execute verifies and accepts complete result | `applied` | 0 | payload as planned plus one baseline |
| Lock-held plan differs | stale failure | 20 unless more specific | none |
| First execution action fails | `failed` or `partial` | 20 unless more specific | completed effects remain; no new baseline |
| Baseline visibility changed without durability confirmation | failed with explicit authority | 20 | visible generation remains authoritative |
| Result delivery fails after acceptance | operational failure | 20 | accepted payload and baseline remain |

## Selection

Sync uses the established omitted, mapping, entry, and component-boundary subtree selector contract. Complete registry validation always applies; payload blockers outside the selected scope do not block a scoped sync.

Resolve canonicalizes `PATH` in source space and requires it to select exactly one established managed identity. Mapping roots, multi-entry subtrees, unmanaged content, destination-only paths, deletions, retirement states, and non-conflicting classifications are not valid resolution targets.

## Output

Both commands use Result Envelope V1. `operation` identifies `sync` or `resolve`; every payload action includes `direction: push|pull`; resolve also includes `winner: source|destination`. Mapping-role paths never swap. Human and JSON output communicate equivalent plan, blockers, action states, recovery, visibility, verification, baseline authority, and failure category. Diagnostics remain on stderr.
