---
title: Mappings and managed membership
type: concept
sources: [S001, S004, S015, S017, S020, S025, S026]
updated: 2026-09-15
---

# Mappings and managed membership

`grip add SOURCE DESTINATION` declares a file or tree mapping without copying endpoints. At least one endpoint must exist and existing endpoints must have compatible kinds. Equivalent endpoints establish a baseline; an unequal source-defined member records the destination as its initial comparison reference and is immediately eligible for an ordinary push, while a source-only member retains its existing pending-push behavior. Destination-only content remains unmanaged. (S001) (S020)

A destination declaration may be absolute, exactly `~`, `~/`-prefixed, or relative to the selected project root. Grip preserves every accepted declaration spelling and resolves it separately before publishing a mapping. (S001) (S017)

Ownership and topology validation apply after destination resolution, so distinct declarations that resolve to overlapping namespaces remain invalid. [Destination path forms](./destination-path-forms.md) records this declaration-versus-endpoint distinction. (S015) (S017)

`grip list [SOURCE]` reads declarations. `grip remove SOURCE` removes the declaration and its baseline evidence without changing either endpoint. Re-adding a mapping begins new membership and never restores a previous baseline. (S001)

Tree membership comes from source-side discovery subject to `.gripignore`. Operators edit `.gripignore` directly; Grip provides no `ignore` command. (S001)

A tree source may be strictly beneath its own destination root, enabling `grip add home/ ~/` when the project is stored under that home. Grip examines only current non-ignored source members and retained accepted identities at their exact paired destination paths; unrelated destination content is outside inventory and ownership. A managed member that equals, contains, or falls beneath the destination-relative source location blocks as `recursive_member_topology`; an intentionally unmanaged unsafe subtree must be excluded explicitly through `.gripignore`. Equal roots, destinations beneath their source, contained file mappings, and cross-mapping ownership overlap remain invalid. (S001) (S004) (S025)

When an otherwise valid file mapping or exact source-defined tree member meets a symbolic link at its paired destination leaf, `grip add` records intent without changing either endpoint or the link target. The entry remains unresolved without an accepted baseline until an exact source-winning forced push completes replacement. (S001) (S004) (S026)

See [Mapping addition and initial baselines](./mapping-addition-and-initial-baselines.md) for the unequal-endpoint and fenced-publication contract.

See [Contained-source tree mappings](./contained-source-tree-mappings.md) for the managed-identity inspection boundary, member-topology rule, and revalidation contract.

See [Exact destination symlink replacement](./exact-destination-symlink-replacement.md) for the unresolved-leaf and explicit replacement boundary.
