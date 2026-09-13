## Revised Command Output

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
Mapped:
 app/push.txt -> <audit-root>/destination/push.txt
[exit 0]
```

`grip list`

```text
5 mapping(s):
 app/push.txt -> <audit-root>/destination/push.txt
 conflict.txt -> <audit-root>/destination/conflict.txt
 docs/pull.txt -> <audit-root>/destination/pull.txt
 sync-pull.txt -> <audit-root>/destination/sync-pull.txt
 sync-push.txt -> <audit-root>/destination/sync-push.txt
[exit 0]
```

`grip list app/push.txt`

```text
1 mapping(s):
 file app/push.txt -> <audit-root>/destination/push.txt
[exit 0]
```

`grip remove app/push.txt`

```text
Mapping removed:
 file app/push.txt -> <audit-root>/destination/push.txt
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

Changes to push:
  app/push.txt -> <audit-root>/destination/push.txt
  sync-push.txt -> <audit-root>/destination/sync-push.txt

Changes to pull:
  docs/pull.txt <- <audit-root>/destination/pull.txt
  sync-pull.txt <- <audit-root>/destination/sync-pull.txt

Conflicts:
  conflict.txt <-> <audit-root>/destination/conflict.txt
    Keep source: grip push --force conflict.txt
    Keep destination: grip pull --force --destination <audit-root>/destination/conflict.txt
[exit 0]
```

`grip status` with an initial collision uses the same resolution choices after its conflict row.

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
Would push 1 file(s):
  <audit-root>/project/app/push.txt -> <audit-root>/destination/push.txt
[exit 0]
```

`grip push push.txt` from the same directory

```text
Pushed 1 file(s):
  <audit-root>/project/app/push.txt -> <audit-root>/destination/push.txt
[exit 0]
```

`grip push --force conflict.txt`

```text
Pushed 1 file(s):
  <audit-root>/project/conflict.txt -> <audit-root>/destination/conflict.txt
[exit 0]
```

`grip push README.md` when an initial collision blocks the requested file

```text
Error: Push blocked: 1 selected; 0 action(s); 1 blocker(s)
  README.md <-> <audit-root>/destination/README.md
    Keep source: grip push --force README.md
    Keep destination: grip pull --force --destination <audit-root>/destination/README.md
[exit 10]
```

`grip push --force README.md` when a technical safety condition still blocks the requested file

```text
Error: Push blocked: 1 selected; 0 action(s); 1 blocker(s)
  Grip cannot safely continue. Run: grip status
[exit 10]
```

`grip push` when every selected entry is current

```text
Nothing to push.
[exit 0]
```

### Pull

`grip pull --dry-run docs/pull.txt`

```text
Would pull 1 file(s):
  <audit-root>/project/docs/pull.txt <- <audit-root>/destination/pull.txt 
[exit 0]
```

`grip pull docs/pull.txt`

```text
Pulled 1 file(s):
  <audit-root>/project/docs/pull.txt <- <audit-root>/destination/pull.txt 
[exit 0]
```

### Sync

`grip sync --dry-run`

```text
Would synchronize 4 file(s):
  <audit-root>/project/sync-pull.txt <- <audit-root>/destination/sync-pull.txt
  <audit-root>/project/sync-pull-2.txt <- <audit-root>/destination/sync-pull-2.txt
  <audit-root>/project/sync-push.txt -> <audit-root>/destination/sync-push.txt
  <audit-root>/project/sync-push-2.txt -> <audit-root>/destination/sync-push-2.txt
[exit 0]
```

`grip sync`

```text
Synchronized 4 file(s):
  <audit-root>/project/sync-pull.txt <- <audit-root>/destination/sync-pull.txt
  <audit-root>/project/sync-pull-2.txt <- <audit-root>/destination/sync-pull-2.txt
  <audit-root>/project/sync-push.txt -> <audit-root>/destination/sync-push.txt
  <audit-root>/project/sync-push-2.txt -> <audit-root>/destination/sync-push-2.txt
[exit 0]
```

`grip sync` when an ordinary conflict blocks the selected file

```text
Error: Sync blocked: 1 selected; 0 action(s); 1 blocker(s)
  conflict.txt <-> <audit-root>/destination/conflict.txt
    Keep source: grip push --force conflict.txt
    Keep destination: grip pull --force --destination <audit-root>/destination/conflict.txt
[exit 10]
```

`grip sync` when every selected entry is current

```text
Nothing to synchronize.
[exit 0]
```

`grip sync --dry-run` when matching endpoints need their first accepted baseline

```text
Would establish a baseline for 1 file(s).
[exit 0]
```

If a mutation stops after it starts, its human output keeps the action-completion count and directs the operator to inspect current state before retrying:

```text
Error: Push failed after 1 of 2 actions. Run: grip status before retrying.
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
Error: source must be a relative path outside .grip
[exit 10]
```

`grip add app/push.txt`

```text
Error: the following required arguments were not provided:
  <DESTINATION>

Usage: grip add <SOURCE> <DESTINATION>

For more information, try '--help'.
[exit 2]
```
