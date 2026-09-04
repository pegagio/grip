---
title: Initial delivery roadmap
type: reference
sources: [S003]
updated: 2026-09-04
---

# Initial delivery roadmap

Roadmap version 1.0.2 defines nine specifications that culminate in the complete initial Grip product. Features 001 and 002 are verified; Feature 003 is the next dependency-eligible entry and remains planned until separately started. The sequence moves from read-only foundations to increasingly capable mutation so namespace, state, and classification contracts are established before user-data risk increases. (S003)

1. **001 — CLI, Configuration, and State Foundation (`verified`):** establish the Rust application, per-user root, versioned configuration and state envelopes, and distinct human, machine, and diagnostic interfaces without payload mutation. (S003)
2. **002 — Mapping Registry and Ownership Validation (`verified`):** define file and tree mapping intent, canonical source identity, lifecycle commands, and ambiguous or recursive topology rejection. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
3. **003 — Source Discovery and Gripignore:** discover source-defined tree membership with nested `.gripignore`, deterministic inventory, destination-only exclusion, and unsupported-entry detection. (S003)
4. **004 — Baselines, Classification, and Status:** persist accepted evidence, implement the complete three-way classification model, and expose deterministic read-only inspection. (S003)
5. **005 — Safe Push and Recovery:** add source-to-destination mutation with dry runs, revalidation, staging, recovery, verification, truthful failure reporting, and baseline publication. (S003)
6. **006 — Reverse Synchronization:** reuse the proven mutation pipeline for established destination-to-source changes without importing destination-only entries. (S003)
7. **007 — Bidirectional Synchronization and Conflict Resolution:** unify both directions, block on complete-scope conflicts, and require an explicit complete-side winner. (S003)
8. **008 — Authorized Deletion and Retirement:** introduce separately authorized deletion, converged deletion, newly ignored retirement, and recovery-material lifecycle. (S003)
9. **009 — Metadata and Filesystem Contract Completion:** finalize promised macOS and Unix metadata, filesystem edge cases, integration coverage, and representative performance acceptance. (S003)

Specs 001 through 006 form a dependency chain; 007 depends on both mutation directions, 008 depends on bidirectional conflict handling, and 009 closes the product after baseline classification and deletion are established. Two-way interactive merge, remap, multiple selectors, remote synchronization, daemons, privileged services, and multi-user coordination remain outside this initial roadmap. (S003)

## Related pages

- [Implementation and delivery direction](./implementation-and-delivery-direction.md)
- [Roadmap open questions](./roadmap-open-questions.md)
