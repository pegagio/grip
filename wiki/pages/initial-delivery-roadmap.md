---
title: Initial delivery roadmap
type: reference
sources: [S003]
updated: 2026-09-08
---

# Initial delivery roadmap

Roadmap version 1.0.19 records all nine specifications as verified and therefore completes the approved initial Grip product roadmap. The sequence moved from read-only foundations to increasingly capable mutation so namespace, state, and classification contracts were established before user-data risk increased. (S003)

1. **001 — CLI, Configuration, and State Foundation (`verified`):** establish the Rust application, per-user root, versioned configuration and state envelopes, and distinct human, machine, and diagnostic interfaces without payload mutation. (S003)
2. **002 — Mapping Registry and Ownership Validation (`verified`):** define file and tree mapping intent, canonical source identity, lifecycle commands, and ambiguous or recursive topology rejection. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
3. **003 — Source Discovery and Gripignore (`verified`):** discover source-defined tree membership with nested `.gripignore`, deterministic inventory, destination-only exclusion, and unsupported-entry detection. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
4. **004 — Baselines, Classification, and Status (`verified`):** persist accepted evidence, implement the complete three-way classification model, and expose deterministic read-only inspection. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
5. **005 — Safe Push and Recovery (`verified`):** add source-to-destination mutation with dry runs, revalidation, staging, recovery, verification, truthful failure reporting, and baseline publication. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
6. **006 — Reverse Synchronization (`verified`):** reuse the proven mutation pipeline for established destination-to-source changes without importing destination-only entries. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
7. **007 — Bidirectional Synchronization and Conflict Resolution (`verified`):** unify both directions, block on complete-scope conflicts, and require an explicit complete-side winner. Its attributable implementation debrief recorded no Must-Address findings. (S003)
8. **008 — Authorized Deletion and Retirement (`verified`):** introduce separately authorized deletion, converged deletion, newly ignored retirement, and recovery-material lifecycle. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
9. **009 — Metadata and Filesystem Contract Completion (`verified`):** finalizes the promised macOS/APFS metadata contract, filesystem edge cases, integration coverage, and representative performance acceptance. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)

Specs 001 through 009 now provide the verified foundation, mutation, conflict, deletion, retirement, recovery, metadata, filesystem-boundary, and qualification chain. Two-way interactive merge, remap, multiple selectors, remote synchronization, daemons, privileged services, and multi-user coordination remain outside this initial roadmap. (S003)

## Related pages

- [Implementation and delivery direction](./implementation-and-delivery-direction.md)
- [Roadmap open questions](./roadmap-open-questions.md)
