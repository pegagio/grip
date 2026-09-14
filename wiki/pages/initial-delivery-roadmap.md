---
title: Initial delivery roadmap
type: reference
sources: [S003]
updated: 2026-09-14
---

# Initial delivery roadmap

Roadmap version 1.7.4 records verified Features 001 through 012, 014, 015, and 021. Features 013 and 016 through 019 remain implemented pending feature-specific debriefs; Feature 020 is abandoned because the release artifact was already below its size target. (S003)

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
The [later implemented features](./initial-delivery-roadmap-later-features.md) continue this ledger from Feature 013 through Feature 021. (S003)

## Related pages

- [Implementation and delivery direction](./implementation-and-delivery-direction.md)
- [Later implemented features](./initial-delivery-roadmap-later-features.md)
- [Roadmap open questions](./roadmap-open-questions.md)
