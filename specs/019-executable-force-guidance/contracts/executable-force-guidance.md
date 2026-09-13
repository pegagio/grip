# Contract: Executable Force-Resolution Guidance

This contract applies only to default human output. JSON result documents, classification values, selector grammar, diff behavior, and force mutation semantics remain unchanged.

## Exact-entry conflict

When the shown source selector resolves to exactly one managed entry under the current accepted state, status offers both existing resolution choices.

```text
Status: 2 entries checked; 1 current; 1 conflict.

Conflicts:
  README.md <-> /tmp/grip-destination/README.md
    Keep source: grip push --force README.md
    Keep destination: grip pull --force --destination /tmp/grip-destination/README.md
```

Both printed commands must pass the existing exact-entry selector validation when run from the same invocation directory. They may still be blocked by a later, real safety condition, but not because the selector is an aggregate mapping or subtree.

## Aggregate conflict

When the shown conflict represents an aggregate mapping or otherwise cannot be force-resolved as one exact entry, status gives the read-only inspection step and does not print force commands.

```text
Status: 5 entries checked; 4 current; 1 conflict.

Conflicts:
  tests/fixtures <-> /tmp/grip-destination/tests/fixtures
    Run: grip diff tests/fixtures
```

The source selector is the same invocation-directory-relative value displayed on the row.

## Blocked mutation

Blocked `push`, `pull`, and `sync` output applies the same per-conflict guidance. For an aggregate conflict it must not repeat an invalid force command.

```text
Error: Pull blocked: 1 selected; 0 action(s); 1 blocker(s)
  tests/fixtures <-> /tmp/grip-destination/tests/fixtures
    Run: grip diff tests/fixtures
```

Technical and compatibility blockers retain their existing explanation and must not receive a speculative force or diff command.
