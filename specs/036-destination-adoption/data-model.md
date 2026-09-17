# Data Model: Destination Adoption

## Adoption request

| Field | Meaning | Validation |
| --- | --- | --- |
| `operation` | Typed mutation operation, `adopt` | Distinct from ordinary `pull` and conflict resolution. |
| `selector` | One destination-space path | Required; must canonicalize as a durable exact path. |
| `dry_run` | Preview-only execution mode | Never writes endpoints or accepted state. |
| `force_ignored_path` | Exact ignore-policy override | True only for `--adopt --force`; does not alter other safety decisions. |

`--source` and adoption are mutually exclusive because adoption always interprets its selector in destination space.

## Adoption set

| Member | Inclusion rule | Required state |
| --- | --- | --- |
| Target file | The exact selected destination leaf | Destination exists and is a supported regular file; paired source leaf is absent. |
| Structural ancestor directory | A paired source directory missing between the source target's nearest existing ancestor and parent | Corresponding destination directory exists, is supported, and can be copied/verified. |
| Existing source ancestor | Nearest existing source-side ancestor directory | Must be below or equal to the mapped source root, be a safe directory, and already be source-defined. |

The set is ordered ancestor-first for construction and directory-finalization. It never includes a sibling or descends from the selected target.

## Mapping and identity

The operation derives an `EntryIdentity` from the declared tree mapping plus the selected raw relative path. The identity is valid only when:

1. The selector is strictly below that mapping's destination root.
2. No exact file mapping or reserved leaf has stronger ownership of the path.
3. The source counterpart remains strictly below the matching source root.
4. The selector does not cross a symlink, unsupported node, project fence, or mapping topology boundary.

## Evidence and state transitions

```text
destination-only regular file
        |
        | exact tree ownership + source absence + policy/metadata/topology checks
        v
planned adoption set
        |
        | dry run ------------------------------> reported preview (no mutation)
        |
        | revalidate + stage/copy + verify
        v
verified paired source/destination members
        |
        | atomic accepted-state publication
        v
accepted tree members (target + newly-created ancestor chain)
```

Any observation, mapping, source, destination, ignore-policy, or accepted-state drift before action stops the operation before publication. A failed copy or verification leaves destination unchanged and publishes no unverified member.

## Ignore-policy warning

For a forced ignored target, the result carries:

| Field | Meaning |
| --- | --- |
| `ignored_path_override` | The operation admitted this exact ignored path only. |
| `recommended_gripignore_rules` | The exact ordered negation rules that would retain normal tree membership, including needed ancestor exemptions. |
| `gripignore_path` | The existing or proposed policy-file location where the rule set must be added. |
| `retention_notice` | A later ordinary discovery can ignore/prune the member until the operator adds that rule. |

These fields are advisory result data; they do not change `.gripignore` or alter the semantics of other tree members.
