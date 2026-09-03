---
title: Command-line and path selection
type: component
sources: [S001]
updated: 2026-09-03
---

# Command-line and path selection

The prospective executable name is `grip`, but exact command syntax for creating file and tree mappings remains open. Human-readable output and stable machine-readable output are separate interfaces, and automation needs a defined exit-code contract. (S001)

`status`, `check`, and `diff` are always read-only. `push`, `pull`, and `sync` mutate by default; `-n` and `--dry-run` preview their deterministic plans. Conflict resolution requires an explicit complete-side choice, provisionally expressed as `resolve PATH --source` or `resolve PATH --destination`. (S001)

The initial command contract permits one optional path selector. Selectors use the source path space by default, while `--destination` selects destination-path interpretation; `--` terminates option parsing so paths beginning with a hyphen can be selected safely. Complete registry validation still applies even when an action is scoped to one mapping or subtree. (S001)

Multiple path selectors, interactive two-way text reconciliation, and metadata-only remapping are potential future expansions rather than initial command commitments. A future remap would update registry and state only after separately performed filesystem movement and verified baseline continuity. (S001)

## Related pages

- [Grip product model](./grip-product-model.md)
