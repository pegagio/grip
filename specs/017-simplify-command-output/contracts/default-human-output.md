# Default Human Output Contract

This contract defines only the changed default human output. `<SOURCE>`, `<DESTINATION>`, `<KIND>`, and `N` stand for values already determined by the existing command result. JSON output, `grip diff`, help, and verbose diagnostics are unchanged.

## Mapping commands

```text
Mapped:
 <SOURCE> -> <DESTINATION>
```

```text
N mapping(s):
 <SOURCE> -> <DESTINATION>
```

```text
1 mapping(s):
 <KIND> <SOURCE> -> <DESTINATION>
```

```text
Mapping removed:
 <KIND> <SOURCE> -> <DESTINATION>
```

Mapping rows in these forms never include a `resolved` line.

## Status

When nonempty, sections appear in this order:

1. `Changes to push`
2. `Changes to pull`
3. `Conflicts`
4. `Needs baseline`

An initial collision or ordinary divergent conflict identifies the two conflicting endpoints and the exact force command for each resolution choice:

```text
  <SOURCE> <-> <DESTINATION>
    Keep source: grip push --force <SOURCE>
    Keep destination: grip pull --force --destination <DESTINATION>
```

Technical compatibility and unsupported-state blockers retain their existing safety-significant details.

## Terminal mutation results

When a push, pull, or sync has no work, it identifies the public direction without rendering plan evidence:

```text
Nothing to push.
```

```text
Nothing to pull.
```

```text
Nothing to synchronize.
```

When a mutation has no copy action but establishes an accepted initial baseline, it uses a similarly concise result:

```text
Would establish a baseline for N file(s).
```

```text
Established a baseline for N file(s).
```

When a public mutation is blocked solely by initial collisions or ordinary divergent conflicts, it uses the public operation name and lists each resolvable conflict with the same two concrete resolution choices:

```text
Error: <Push|Pull|Sync> blocked: N selected; N action(s); N blocker(s)
  <SOURCE> <-> <DESTINATION>
    Keep source: grip push --force <SOURCE>
    Keep destination: grip pull --force --destination <DESTINATION>
```

When a public mutation is blocked by a technical or mixed safety condition, it provides the next safe inspection step without presenting raw planner identifiers:

```text
Error: <Push|Pull|Sync> blocked: N selected; N action(s); N blocker(s)
  Grip cannot safely continue. Run: grip status
```

When execution stops after it begins, Grip retains the existing completion count and sends the operator to current status before retrying:

```text
Error: <Push|Pull|Sync> failed after N of N actions. Run: grip status before retrying.
```

The result remains blocked until the operator explicitly chooses a side. Default terminal mutation output omits raw blocker identifiers, winner fields, action milestones, recovery fields, baseline outcome and generation statements, and operation-record identifiers. These values remain available in JSON and diagnostics.

## Mutation previews and completions

```text
Would push N file(s):
  <SOURCE> -> <DESTINATION>
```

```text
Pushed N file(s):
  <SOURCE> -> <DESTINATION>
```

```text
Would pull N file(s):
  <SOURCE> <- <DESTINATION>
```

```text
Pulled N file(s):
  <SOURCE> <- <DESTINATION>
```

```text
Would synchronize N file(s):
  <SOURCE> <- <DESTINATION>
  <SOURCE> -> <DESTINATION>
```

```text
Synchronized N file(s):
  <SOURCE> <- <DESTINATION>
  <SOURCE> -> <DESTINATION>
```

The concise action-bearing forms omit internal action names, recovery fields, verification fields, baseline statements, and operation-record identifiers. A forced source-winning conflict uses the ordinary `Pushed` form and does not add a winner line.

## Errors

The first line of a default human parser or domain error begins:

```text
Error: <existing actionable message>
```

Existing remaining usage text and exit categories remain unchanged.
