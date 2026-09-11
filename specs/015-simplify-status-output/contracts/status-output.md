# Human `grip status` Contract

This contract applies only to the default human output of `grip status`. It supersedes earlier detailed human-status presentation where they conflict. It does not change `grip diff`, JSON output, selector interpretation, exit codes, classifications, or mutation behavior.

## Summary

The first line reports mutually exclusive counts for entries checked, current entries, changes to push, changes to pull, conflicts, and entries needing a baseline. When every actionable category is zero, it states that no action is needed. An empty selected scope reports that no managed entries were found.

```text
Status: 2 entries checked; 2 current; no action needed.
```

```text
Status: no managed entries found.
```

## Actionable entries

Only actionable entries appear after the summary. Render only nonempty sections. Each row has stable source and destination paths and a status-specific symbol; the default view does not print command suggestions.

```text
Status: 5 entries checked; 1 current; 1 to push; 1 to pull; 1 conflict; 1 needs baseline.

Conflicts:
  README.md <-> ~/workspace/README.md

Changes to push:
  app/main.py -> ~/workspace/app/main.py

Changes to pull:
  docs/config.yml <- ~/workspace/docs/config.yml

Needs baseline:
  CHANGELOG.md >-< ~/workspace/CHANGELOG.md
```

The symbols mean `->` source-to-destination, `<-` destination-to-source, `<->` conflicting endpoints with no selected winner, and `>-<` a non-directional state with baseline or reconciliation work pending. `>-<` does not assert that endpoint payloads match. A safety blocker that is not an endpoint-content conflict adds a concise plain-language explanation but must not contain serialized classification names, raw compatibility reasons, endpoint capability profiles, or baseline comparison dimensions.

## Safety and metadata

A blocking unsupported or compatibility finding remains visible. Its entry must identify the affected path, state that Grip cannot safely proceed, and use the existing safe explanatory or corrective text where applicable. A non-blocking informational finding is omitted from the default human status result.

No row may make `--force` appear to be the ordinary solution. Conflict notation does not choose a winner, and baseline notation does not imply a payload-copy direction.

## Compatibility

`grip status -o json` and `grip status --output json` retain the existing result envelope, record order, classifications, compatibility findings, endpoint capabilities, counts, and exit behavior. `grip status -e` retains its existing valid-attention exit behavior.
