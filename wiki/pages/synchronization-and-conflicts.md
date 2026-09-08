---
title: Synchronization and conflicts
type: component
sources: [S001, S004, S007, S008, S009, S010, S011, S012]
updated: 2026-09-08
---

# Synchronization and conflicts

Grip compares the current source, current destination, and last accepted baseline for each managed entry. That comparison distinguishes synchronized entries, initial additions or collisions, one-sided changes, converged identical changes, divergent changes, deletions, delete/change conflicts, and entries pending retirement after becoming ignored. (S001)

Feature 004 implements this model as deterministic typed records shared by `status`, `check`, and `diff`. Explicit baseline acceptance is allowed only when all selected source and destination evidence is complete and equivalent; an already-current acceptance is an exact no-op, and acceptance-relevant drift detected before publication preserves the prior baseline. (S004)

`push` propagates eligible source-side changes, `pull` propagates eligible destination-side changes, and `sync` plans all unambiguous one-sided changes in both directions. An accepted operation updates the baseline only after its changes have been applied and verified. (S001)

Feature 005 established only the `push` portion: source additions and source-only changes become actions, synchronized and other non-action classifications remain reported, and any blocker prevents all mutation. It deliberately deferred pull, bidirectional planning, conflict winners, deletion, retirement, and automatic recovery to their owning features. (S008)

Feature 006 implements `pull` for established accepted entries classified as destination-only changes. It reports unmanaged destination-only content without importing it, blocks conflicts and unsafe or incomplete evidence before mutation, and leaves source-side changes, bidirectional planning, conflict winners, deletion, and retirement to their owning workflows. (S004) (S009)

Feature 007 implements one deterministic `sync` plan containing eligible push and pull actions in canonical managed-identity order. Converged identical changes need no payload replacement but enter the accepted baseline only after the complete selected operation succeeds; a synchronized-only selection creates no operation record or generation. Any conflict, deletion, retirement, unsupported node, unsafe path, or incomplete observation blocks the whole selected plan before mutation. (S004) (S010)

A divergent change on both sides is a whole-entry conflict, including cases where content changed on one side and supported metadata changed on the other. The plan of record requires an explicit source-wins or destination-wins decision; it does not combine the two current states. Grip must re-inspect if either side changes between conflict reporting and resolution. (S001)

`resolve` accepts one exact established entry identified in source-path space and exactly one `--source` or `--destination` winner. It preserves the complete losing state, applies the winner through the corresponding directional pipeline, verifies equality, and publishes one accepted generation; preview aliases perform no mutation. (S004) (S010)

Mutating operations perform a complete preflight and block before the first mutation when any conflict or known unsafe condition exists in scope. A plan is deterministic evidence from one validated inspection, and execution revalidates the relevant filesystem evidence before applying it. (S001)

Feature 008 executes a one-sided deletion only through `delete` with an explicit authoritative side. A changed remaining peer creates a delete/change blocker, ordinary push, pull, and sync continue to delete nothing, and successful deletion preserves recovery evidence before removing the peer and retiring the accepted record. Converged deletions and policy-driven membership changes are handled separately by explicit retirement. (S011)

Feature 009 applies every direction and conflict rule to the complete metadata state. Metadata-only changes are ordinary actions, directory metadata finalizes after descendants, and resolution always transfers one complete winner rather than combining content or metadata fields. (S012)

## Related pages

- [Grip product model](./grip-product-model.md)
- [Baseline classification and status](./baseline-classification-and-status.md)
- [Metadata and filesystem contract](./metadata-and-filesystem-contract.md)
