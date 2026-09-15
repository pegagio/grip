---
title: Contained-source tree mappings
type: component
sources: [S025]
updated: 2026-09-15
---

# Contained-source tree mappings

A contained-source tree mapping is a tree whose resolved source root is a strict descendant of its resolved destination root. The primary layout is a repository-owned `home/` directory mapped to `~/` while the repository itself lives under that home. Grip admits only this source-beneath-destination tree relationship; equal roots, destinations beneath their source, and contained file mappings remain `recursive_topology` errors. (S025)

## Managed identity boundary

The destination root is a container, not an owned tree. Grip builds an ephemeral managed identity set from current non-ignored source members plus retained accepted identities. It probes only each identity's exact paired destination and the ancestors needed to reach it safely, without enumerating unrelated destination siblings. Never-managed destination content therefore creates no discovery record, classification, baseline, selector candidate, drift evidence, or action. (S025)

Retained accepted identities remain observable when their source disappears, preserving deletion, destination-change, conflict, converged-deletion, and forced-direction behavior. If current `.gripignore` policy prunes a retained identity, Grip records ignored-retirement coverage without opening or inventing descendants. (S025)

## Member topology

Let `P` be the non-empty path from the destination root to the source root and `R` a managed identity's relative destination. A member is safe only when `R` and `P` are component-disjoint. An equal, ancestor, or descendant relationship blocks with `recursive_member_topology`, including the mapping, relative member, resolved source and destination, and relation. Grip creates no implicit exclusion; an intentionally unmanaged unsafe subtree must be excluded through ordinary source-side `.gripignore` policy. (S025)

Recursive-member safety applies to the entire selected mapping. Exact selectors and force cannot bypass an unsafe sibling, while an unsafe member in an unrelated unselected mapping does not block the selected mapping. (S025)

## Revalidation and compatibility

Addition validates source policy, managed identities, member topology, and paired destination evidence before publishing mapping intent or accepted state, and `grip add` remains payload-nonmutating. Push, pull, sync, force, and deletion revalidate the complete selected mapping before the first payload action, then retain their existing per-action evidence checks, verification, and atomic accepted-state publication. (S025)

During project or home rebinding, retained identities are topology-checked before payload fingerprints are probed. A newly unsafe identity blocks without discarding accepted evidence. Project descriptor V2, accepted state V4, command syntax, selector interpretation, force authority, deletion authorization, and machine-output envelope versions remain unchanged. (S025)

The representative performance acceptance uses 100 managed identities, 10,000 unrelated destination entries, and 100 warm status samples with a one-second p95 threshold on the supported macOS Apple Silicon release platform. The completed debrief found no roadmap discrepancy and recommended verification. (S025)

## Related pages

- [Mappings and managed membership](./mappings-and-managed-membership.md)
- [Mapping addition and initial baselines](./mapping-addition-and-initial-baselines.md)
- [Safety and recovery model](./safety-and-recovery-model.md)
- [Filesystem support boundaries](./filesystem-support-boundaries.md)
