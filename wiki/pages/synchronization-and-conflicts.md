---
title: Synchronization and conflicts
type: component
sources: [S001]
updated: 2026-09-03
---

# Synchronization and conflicts

Grip compares the current source, current destination, and last accepted baseline for each managed entry. That comparison distinguishes synchronized entries, initial additions or collisions, one-sided changes, converged identical changes, divergent changes, deletions, delete/change conflicts, and entries pending retirement after becoming ignored. (S001)

`push` propagates eligible source-side changes, `pull` propagates eligible destination-side changes, and `sync` plans all unambiguous one-sided changes in both directions. An accepted operation updates the baseline only after its changes have been applied and verified. (S001)

A divergent change on both sides is a whole-entry conflict, including cases where content changed on one side and supported metadata changed on the other. The plan of record requires an explicit source-wins or destination-wins decision; it does not combine the two current states. Grip must re-inspect if either side changes between conflict reporting and resolution. (S001)

Mutating operations perform a complete preflight and block before the first mutation when any conflict or known unsafe condition exists in scope. A plan is deterministic evidence from one validated inspection, and execution revalidates the relevant filesystem evidence before applying it. (S001)

## Related pages

- [Grip product model](./grip-product-model.md)
