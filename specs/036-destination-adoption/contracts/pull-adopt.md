# `grip pull --adopt` CLI Contract

## Synopsis

```text
grip pull [--dry-run] [--force] [--use-modification-time] --adopt DESTINATION
grip pull [--dry-run] [--force] [--use-modification-time] -a DESTINATION
```

`DESTINATION` is always interpreted in destination space. `--adopt` cannot be combined with `--source`. It requires exactly one path.

## Eligibility

The command succeeds only when all of these are true:

1. The selected path is an existing supported regular file strictly beneath an existing tree mapping's destination root.
2. The paired source file does not exist.
3. A safe existing source directory exists somewhere in the target's ancestor chain within the mapping source root.
4. Each required missing source ancestor has an eligible destination directory counterpart.
5. The selected path has no stronger or conflicting mapping ownership and has no unsupported node, metadata, or topology evidence.
6. The paired source-relative path is not ignored, unless `--force` is supplied.

## Behavior

| Request | Result |
| --- | --- |
| `pull --adopt DESTINATION` | Creates the source target and needed source ancestors, verifies complete managed equivalence, then publishes accepted baselines for the adoption set. |
| `pull -a --dry-run DESTINATION` | Reports the same deterministic adoption plan without endpoint or accepted-state mutation. |
| `pull -a --force DESTINATION` for an ignored target | Performs the same exact adoption with only the ignore-policy eligibility override and reports an advisory retention warning. |
| Ordinary `pull`, `status`, or `diff` | Does not discover or import destination-only members. |

The destination target is never modified. `--force` does not convert adoption into conflict resolution and cannot override source existence, unsupported metadata, mapping ownership, or stale-evidence failures.

## Human result contract

Successful output identifies the selected destination and paired source, reports whether the result was a preview or applied adoption, and counts target plus structural ancestors separately or unambiguously.

When `--force` adopted an ignored target, output identifies the `.gripignore` file where the recommendation belongs and includes a complete ordered rule set, for example:

```text
Adopted an ignored path for this operation only.
To retain it in ordinary tree discovery, add to PATH/.gripignore:
!relative/
!relative/path/
!relative/path/to/file
Without these rules, a later ordinary operation will ignore this member.
```

The precise rule set uses the selected policy file's relative-path grammar, contains only the exemptions required for the target and pruned ancestors, and is checked against effective policy evaluation before it is reported.

## JSON result contract

JSON retains the standard operation, mode, completion, result, scope, action, blocker, and baseline fields. An adoption result identifies `operation: "adopt"` and includes exact source and destination paths for the target in its selected-entry data.

Forced ignored adoption adds an advisory warning object equivalent to:

```json
{
  "kind": "ignored_path_adopted",
  "gripignore_path": "PATH/.gripignore",
  "recommended_gripignore_rules": ["!relative/", "!relative/path/", "!relative/path/to/file"],
  "message": "A later ordinary operation will ignore this member unless these rules are added to the reported .gripignore file."
}
```

The warning is absent for unignored adoption. Neither result shape authorizes an ignore-policy mutation.

## Failure contract

Failures name the selected path and concrete reason, including one of: no eligible tree mapping, non-regular destination node, existing source path, no existing source ancestor, ignored paired source path, ownership conflict, unsupported metadata, incompatible flags, stale evidence, copy failure, verification failure, or accepted-state publication failure. A failure never changes the destination and never publishes an unverified baseline.
