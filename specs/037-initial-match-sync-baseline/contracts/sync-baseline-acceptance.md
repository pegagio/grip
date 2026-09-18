# Selected Sync Baseline-Acceptance Contract

`grip sync [SOURCE]` retains its existing source-path selection semantics. When the supplied source selector resolves to exactly one complete, equivalent, active managed entry with no accepted baseline, sync may establish accepted baseline evidence without a payload action. No-selector, tree-root, and multi-entry subtree scopes do not gain initial-match acceptance.

## Eligible outcome

```text
$ grip sync source-tree/nested/file
Established a baseline for 1 file(s).
```

The exact final human wording follows the existing baseline-only result form. It must not describe a copy, replacement, or direction.

## Dry run

```text
$ grip sync --dry-run source-tree/nested/file
Would establish a baseline for 1 file(s).
```

The dry run must not change endpoints, mapping intent, state, locks, or operation evidence.

## Exclusions

This contract does not accept unequal initial pairs, absent peers, destination-only entries, ignored entries, unsupported nodes, unsafe conditions, or any entry outside the selected scope. It does not add a `baseline` command or change `push`, `pull`, force, or status behavior.
