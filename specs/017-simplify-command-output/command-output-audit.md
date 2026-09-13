# Command output audit

This report captures the default human-readable output of every Grip subcommand as built from commit `b67be25`. It is an audit artifact only; it makes no product or contract changes.

## Method

The transcript used a disposable project and destination beneath `<audit-root>` (a unique directory created under `/tmp`). Five file mappings established clean baselines, then the fixture introduced one source-side change, one destination-side change, one divergent conflict, and two independent changes for `sync`. Paths, operation IDs, and baseline generations below are normalized so they are stable and contain no machine-specific user paths. Exit codes are shown in brackets.

`--output json` is intentionally excluded from the human-output findings. It is a machine contract and, as expected, contains all observations, capability profiles, hashes, and classification evidence.

## Transcript

### Version and initialization

`grip version`

```text
grip 0.1.0
[exit 0]
```

`grip init <audit-root>/project`

```text
Grip project initialized: <audit-root>/project
[exit 0]
```

### Mapping commands

`grip add app/push.txt <audit-root>/destination/push.txt`

```text
Mapping recorded
file app/push.txt -> <audit-root>/destination/push.txt
  resolved <audit-root>/project/app/push.txt -> <audit-root>/destination/push.txt
[exit 0]
```

`grip list`

```text
5 mapping(s)
file app/push.txt -> <audit-root>/destination/push.txt
  resolved <audit-root>/project/app/push.txt -> <audit-root>/destination/push.txt
file conflict.txt -> <audit-root>/destination/conflict.txt
  resolved <audit-root>/project/conflict.txt -> <audit-root>/destination/conflict.txt
file docs/pull.txt -> <audit-root>/destination/pull.txt
  resolved <audit-root>/project/docs/pull.txt -> <audit-root>/destination/pull.txt
file sync-pull.txt -> <audit-root>/destination/sync-pull.txt
  resolved <audit-root>/project/sync-pull.txt -> <audit-root>/destination/sync-pull.txt
file sync-push.txt -> <audit-root>/destination/sync-push.txt
  resolved <audit-root>/project/sync-push.txt -> <audit-root>/destination/sync-push.txt
[exit 0]
```

`grip list app/push.txt`

```text
1 mapping(s)
file app/push.txt -> <audit-root>/destination/push.txt
  resolved <audit-root>/project/app/push.txt -> <audit-root>/destination/push.txt
[exit 0]
```

`grip remove app/push.txt`

```text
Mapping removed
file app/push.txt -> <audit-root>/destination/push.txt
  resolved <audit-root>/project/app/push.txt -> <audit-root>/destination/push.txt
[exit 0]
```

### Status

`grip status` with a clean baseline

```text
Status: 5 entries checked; 5 current; no action needed.
[exit 0]
```

`grip status` with a source change, a destination change, and a conflict

```text
Status: 5 entries checked; 0 current; 2 to push; 2 to pull; 1 conflict.

Conflicts:
  conflict.txt <-> <audit-root>/destination/conflict.txt
    Blocked: Grip cannot safely proceed because both endpoints changed.

Changes to push:
  app/push.txt -> <audit-root>/destination/push.txt
  sync-push.txt -> <audit-root>/destination/sync-push.txt

Changes to pull:
  docs/pull.txt <- <audit-root>/destination/pull.txt
  sync-pull.txt <- <audit-root>/destination/sync-pull.txt
[exit 0]
```

`grip status --exit-code` produces the same human transcript and exits `1` when attention is required.

`grip status` from `<audit-root>/project/app`

```text
Status: 4 entries checked; 3 current; 1 to pull.

Changes to pull:
  ../docs/pull.txt <- <audit-root>/destination/pull.txt
[exit 0]
```

`grip status push.txt` from that same nested directory

```text
Status: no managed entries found.
[exit 10]
```

The latter is expected under the current contract: `status` selectors remain project-relative, while only a relative `push` selector is resolved from the current directory.

### Diff

`grip diff sync-push.txt` after a source-side change

```text
Diff complete: 1 entries; 1 attention; 0 blocking
endpoint_capability: mapping=<audit-root>/project/sync-push.txt endpoint=source filesystem="apfs" case_sensitive=false mtime_precision_ns=1
endpoint_capability: mapping=<audit-root>/project/sync-push.txt endpoint=destination filesystem="apfs" case_sensitive=false mtime_precision_ns=1
source_only_change <audit-root>/project/sync-push.txt direction=source_to_destination attention=yes blocking=no
  reasons source_changed
  source_to_baseline content,modification_time
  destination_to_baseline none
  source_to_destination content,modification_time
  compatibility endpoint=source field=extended_attribute reason=excluded_xattr blocking=no required=com.apple.provenance corrective_choice=no action is required; Grip leaves this attribute unmanaged
  compatibility endpoint=destination field=extended_attribute reason=excluded_xattr blocking=no required=com.apple.provenance corrective_choice=no action is required; Grip leaves this attribute unmanaged
[exit 0]
```

### Push

`grip push --dry-run push.txt` from `<audit-root>/project/app`

```text
Push preview complete: 1 selected; 1 action(s); 0 blockers
planned replace_file <audit-root>/project/app/push.txt -> <audit-root>/destination/push.txt recovery=not_required
Baseline not attempted; generation <generation> remains authoritative
[exit 0]
```

`grip push push.txt` from the same directory

```text
Push applied: 1 action(s); accepted generation <generation>
completed replace_file <audit-root>/project/app/push.txt -> <audit-root>/destination/push.txt visible=yes verified=yes durable=yes recovery=not_required
Baseline published; generation <generation> remains authoritative
Operation record push-<generated-id> preserved
[exit 0]
```

`grip push --force conflict.txt`

```text
Resolution applied: 1 action(s); accepted generation <generation>
Winner source
completed replace_file <audit-root>/project/conflict.txt -> <audit-root>/destination/conflict.txt visible=yes verified=yes durable=yes recovery=not_required
Baseline published; generation <generation> remains authoritative
Operation record resolve-<generated-id> preserved
[exit 0]
```

### Pull

`grip pull --dry-run docs/pull.txt`

```text
Pull preview complete: 1 selected; 1 action(s); 0 blockers
planned replace_file <audit-root>/destination/pull.txt -> <audit-root>/project/docs/pull.txt recovery=not_required
Baseline not attempted; generation <generation> remains authoritative
[exit 0]
```

`grip pull docs/pull.txt`

```text
Pull applied: 1 action(s); accepted generation <generation>
completed replace_file <audit-root>/destination/pull.txt -> <audit-root>/project/docs/pull.txt visible=yes verified=yes durable=yes recovery=not_required
Baseline published; generation <generation> remains authoritative
Operation record pull-<generated-id> preserved
[exit 0]
```

### Sync

`grip sync --dry-run`

```text
Sync preview complete: 5 selected; 2 action(s); 0 blockers
planned replace_file <audit-root>/destination/sync-pull.txt -> <audit-root>/project/sync-pull.txt recovery=not_required
planned replace_file <audit-root>/project/sync-push.txt -> <audit-root>/destination/sync-push.txt recovery=not_required
Baseline not attempted; generation <generation> remains authoritative
[exit 0]
```

`grip sync`

```text
Sync applied: 2 action(s); accepted generation <generation>
completed replace_file <audit-root>/destination/sync-pull.txt -> <audit-root>/project/sync-pull.txt visible=yes verified=yes durable=yes recovery=not_required
completed replace_file <audit-root>/project/sync-push.txt -> <audit-root>/destination/sync-push.txt visible=yes verified=yes durable=yes recovery=not_required
Baseline published; generation <generation> remains authoritative
Operation record sync-<generated-id> preserved
[exit 0]
```

### Help and errors

`grip --help`

```text
Safely manage per-user file mappings

Usage: grip [OPTIONS] <COMMAND>

Commands:
  init     Initialize an existing directory as a Grip project
  version
  add      Record a source-to-destination mapping without copying either endpoint
  list     List mappings, optionally selecting one source path
  remove   Remove a mapping declaration without changing endpoint payloads
  status   Report synchronization state and validate Grip-owned metadata
  diff     Show detailed source and destination differences without mutation
  push     Copy an eligible source-side change to its mapped destination
  pull     Copy an eligible destination-side change back to its mapped source
  sync     Synchronize all unambiguous non-absent changes in both directions
  help     Print this message or the help of the given subcommand(s)

Options:
  -p, --project <PATH>
  -o, --output <FORMAT>  [possible values: json]
  -v, --verbose...
  -h, --help             Print help
  -V, --version          Print version
[exit 0]
```

`grip push <absolute-source-path>`

```text
source must be a relative path outside .grip
[exit 10]
```

`grip add app/push.txt`

```text
error: the following required arguments were not provided:
  <DESTINATION>

Usage: grip add <SOURCE> <DESTINATION>

For more information, try '--help'.
[exit 2]
```

## Findings for review

The commands fall into three output styles: concise state summaries (`version`, `init`, clean `status`), mapping declaration reports (`add`, `list`, `remove`), and mutation-engine transcripts (`push`, `pull`, `sync`, and forced resolution). The final group is where the default human output most consistently exposes details a typical operator does not need to decide the next action.

| Area | Evidence | Human-output concern | Suggested direction |
| --- | --- | --- | --- |
| `add`, `list`, `remove` | Every mapping prints both its declared and resolved form. | Large lists repeat a long absolute source path and a destination that is often identical to the declaration. | Default to the declared mapping row; reserve resolved endpoint details for `-v` or JSON. |
| `diff` | A one-file diff emits 11 detail lines, including capability probes and two excluded-xattr notices. | Its name promises detail, but the default does not distinguish decision-relevant differences from diagnostic implementation data. | Retain changed dimensions and conflict reason; move endpoint capabilities and unmanaged-xattr repetition behind `-v` or JSON. |
| `push`, `pull`, `sync` previews | Rows name `replace_file`, `recovery=not_required`, and a baseline generation. | A user primarily needs the direction, affected paths, action count, and blockers. | Use `Would push`, `Would pull`, or `Would synchronize` plus `SOURCE -> DESTINATION` rows; expose recovery and generation only when verbose. |
| Completed mutations | Rows add `visible=yes verified=yes durable=yes`, baseline publication, and a generated operation ID. | These confirm internal durability rather than user intent, dominate one-file operations, and include an opaque identifier with no documented follow-up use. | Default to a completion summary and affected paths; retain IDs and verification fields for verbose output or failure recovery. |
| Conflict resolution | `Winner source` is separate from the direction-bearing action row. | The operator can infer the choice only after parsing both lines. | Render the selected direction in the heading, for example `Pushed conflict.txt to <destination> (forced)`. |
| Nested selectors | `status` displays `../docs/pull.txt` from `app`, but `status push.txt` fails because it remains project-relative. | This differs from the recently added CWD-relative `push` behavior and is likely surprising to users who copy a displayed path or follow Git conventions. | Decide explicitly whether all read-only source selectors should become CWD-relative, or document the intentional asymmetry more prominently. |
| Help | `version` and global options have no descriptions. | Standard Clap layout is readable, but the blank entries reduce discoverability. | Add concise help text for `version`, `--project`, `--output`, and `--verbose`. |
| Errors | The absolute source rejection is a bare sentence with exit `10`. | It has no `Error:` marker, usage context, or clear distinction between invalid invocation and an invalid project configuration. | Normalize domain errors to a short actionable sentence; separately review whether a selector violation should use exit `2`. |

## Recommended next slice

Prioritize the mutation-engine transcript because it is the most common success path after `status` and has the highest density of internal detail. A narrow feature could define concise default rows for preview, applied, no-op, and forced-resolution outcomes while preserving the current complete evidence under `-v` and JSON. `diff`, mapping-list rendering, selector consistency, and help/error copy can then be handled as separate, reviewable follow-ups.
