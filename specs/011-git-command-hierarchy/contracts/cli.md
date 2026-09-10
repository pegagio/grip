# CLI Contract: Git-Inspired Command Hierarchy

## Invocation grammar

```text
grip [-p PATH|--project PATH] [-o json|--output json] [-v...]
     <command> [command-options] [arguments]

grip init [PATH]
grip version

grip add <SOURCE> <DESTINATION>
grip list [SOURCE]
grip remove <SOURCE>

grip status [-e|--exit-code] [-d PATH|--destination PATH] [PATH]
grip diff [-d PATH|--destination PATH] [PATH]

grip push [-n|--dry-run] [-f|--force] [-d PATH|--destination PATH] [PATH]
grip pull [-n|--dry-run] [-f|--force] [-d PATH|--destination PATH] [PATH]
grip sync [-n|--dry-run] [-d PATH|--destination PATH] [PATH]
```

`-p` and `-o` are global options. `init` and `version` do not accept project selection. Human-readable output is the default. `json` is the only accepted output value in this release; `human` is not a value.

## Command contracts

| Command | Contract |
|---|---|
| `init [PATH]` | Initialize Grip at `PATH`, or the current directory if omitted. |
| `version` | Print the Grip version without requiring a project. |
| `add SOURCE DESTINATION` | Declare one mapping. At least one endpoint must exist. Infer one supported compatible kind from existing endpoints. Do not copy or create endpoint data. Establish a baseline only when both endpoints match. |
| `list [SOURCE]` | List declared mappings, optionally filtered by a source-space path. |
| `remove SOURCE` | Remove the matching mapping declaration and its internal baseline evidence. Do not alter either endpoint payload. |
| `status [PATH]` | Report mapping health and differences. `-e` returns nonzero for ordinary attention. `-d` interprets an optional path in destination space. |
| `diff [PATH]` | Show content differences without mutating mappings, endpoint payloads, or baselines. `-d` selects destination path space. |
| `push [PATH]` | Publish a clear source-side winner to destination. Ordinary mode blocks ambiguity and one-sided absence. `-f` permits source authority for exactly one entry. |
| `pull [PATH]` | Publish a clear destination-side winner to source. Ordinary mode blocks ambiguity and one-sided absence. `-f` permits destination authority for exactly one entry. |
| `sync [PATH]` | Compose ordinary clear-direction updates for selected mappings. Block ambiguity, one-sided absence, and differing unbaselined entries. |

`-n` is a dry run: it reports the candidate action and performs no endpoint, mapping, or baseline mutation. `-f` is not accepted by `sync`.

## Selection and force

- Without `-d`, optional `PATH` selectors are evaluated in source space.
- With `-d`, optional `PATH` selectors are evaluated in destination space.
- `-d` does not reverse `push` or `pull` and does not choose an authoritative endpoint.
- `push -f` and `pull -f` require a selector resolving to exactly one managed entry.
- The forced winner may be absent; therefore force can intentionally propagate deletion in its named direction.
- `remove` always changes declaration membership only. It is never a payload deletion operation.

## Output and exits

Human mode presents concise operation results and diagnostics. JSON mode emits the same semantic result in machine-readable form.

| Condition | Exit outcome |
|---|---|
| Successful command, including ordinary status attention without `-e` | Zero |
| `status -e` with ordinary attention | Nonzero status-attention result |
| Invalid arguments, malformed project metadata, unsupported endpoint, lock failure, failed publication, or post-publication verification failure | Error/nonzero |
| Conflict, one-sided absence in ordinary synchronization, or unbaselined differing `sync` | Nonzero actionable result |

## Removed surface

The parser rejects every spelling and alias from the removed hierarchy, including `mapping`, `validate`, `fsck`, `check`, `show`, `resolve`, `delete`, `retire`, `accept`, recovery commands, and untracking commands. `.gripignore` is edited directly; no `ignore` command is provided.
