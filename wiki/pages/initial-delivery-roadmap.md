---
title: Initial delivery roadmap
type: reference
sources: [S003]
updated: 2026-09-11
---

# Initial delivery roadmap

Roadmap version 1.3.3 records all twelve specifications as verified and completes the approved initial Grip product roadmap, including the project-scoped portability correction, the flat command-hierarchy replacement, and the destination-path form correction. The sequence moved from read-only foundations to increasingly capable mutation, then replaced the pre-release global authority model with portable project intent and the superseded public command surface after the underlying safety contracts were established. (S003)

1. **001 — CLI, Configuration, and State Foundation (`verified`):** establish the Rust application, per-user root, versioned configuration and state envelopes, and distinct human, machine, and diagnostic interfaces without payload mutation. (S003)
2. **002 — Mapping Registry and Ownership Validation (`verified`):** define file and tree mapping intent, canonical source identity, lifecycle commands, and ambiguous or recursive topology rejection. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
3. **003 — Source Discovery and Gripignore (`verified`):** discover source-defined tree membership with nested `.gripignore`, deterministic inventory, destination-only exclusion, and unsupported-entry detection. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
4. **004 — Baselines, Classification, and Status (`verified`):** persist accepted evidence, implement the complete three-way classification model, and expose deterministic read-only inspection. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
5. **005 — Safe Push and Recovery (`verified`):** add source-to-destination mutation with dry runs, revalidation, staging, recovery, verification, truthful failure reporting, and baseline publication. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
6. **006 — Reverse Synchronization (`verified`):** reuse the proven mutation pipeline for established destination-to-source changes without importing destination-only entries. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
7. **007 — Bidirectional Synchronization and Conflict Resolution (`verified`):** unify both directions, block on complete-scope conflicts, and require an explicit complete-side winner. Its attributable implementation debrief recorded no Must-Address findings. (S003)
8. **008 — Authorized Deletion and Retirement (`verified`):** introduce separately authorized deletion, converged deletion, newly ignored retirement, and recovery-material lifecycle. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
9. **009 — Metadata and Filesystem Contract Completion (`verified`):** finalizes the promised macOS/APFS metadata contract, filesystem edge cases, integration coverage, and representative performance acceptance. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
10. **010 — Project-Scoped Initialization and Portable Mappings (`verified`):** replaces the global mapping model with initialized projects, committed portable declarations, exact explicit or ancestor-based selection, and isolated machine-local state. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
11. **011 — Git-Inspired Command Hierarchy (`verified`):** replaces the broad nested public surface with `init`, `version`, `add`, `list`, `remove`, `status`, `diff`, `push`, `pull`, and `sync`; retains direct `.gripignore` policy and makes Git or another selected system responsible for history and recovery. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
12. **012 — Destination Path Forms (`verified`):** broadens destination declarations to absolute paths, `~`, and any `~/` spelling while retaining exact declaration text and project-relative source declarations. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)

Specs 001 through 012 now provide the verified foundation, mutation, conflict, metadata, filesystem-boundary, qualification, portable project-authority, concise public-command, and explicit destination-declaration chain. Two-way interactive merge, remap, multiple selectors, remote synchronization, daemons, privileged services, and multi-user coordination remain outside this initial roadmap. (S003)

## Related pages

- [Implementation and delivery direction](./implementation-and-delivery-direction.md)
- [Roadmap open questions](./roadmap-open-questions.md)
