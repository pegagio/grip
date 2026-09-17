---
title: Initial delivery roadmap current features
type: reference
sources: [S003]
updated: 2026-09-17
---

# Initial delivery roadmap current features

This continuation records Features 029 through 036 from the current roadmap ledger. (S003)

29. **029 — Operational Output and State Rebinding (`implemented`):** makes tree-operation summaries count file payload actions, keeps selected external-diff standard output owned by the tool unless verbose inspection is requested, removes unmanaged metadata notes from that inspection, and prevents output-only diff-profile changes from invalidating payload authorization. (S003)
30. **030 — Contained Tree Mapping Overrides (`implemented`):** permits an exact file mapping to reserve one destination leaf inside a contained tree mapping only while the tree source has no corresponding member; duplicate ownership remains rejected. (S003)
31. **031 — Ambient Label Metadata and Add Diagnostics (`implemented`):** excludes only opaque macOS `com.apple.metadata:kMDLabel_*` labels from synchronization and identifies any other blocking extended attribute by path, endpoint, and name without exposing its value. (S003)
32. **032 — Current Status Entries (`implemented`):** adds a deterministic human `Current:` section that names every current managed entry with `=` while preserving machine output and action semantics. (S003)
33. **033 — Global Ignore Policy and Directory Timestamp Boundary (`implemented`):** applies the project-root `.gripignore` to every tree mapping with narrower policy overrides and treats directory modification times as unmanaged. (S003)
34. **034 — Optional Modification-Time Detection (`implemented`):** ignores timestamp-only drift by default and permits one-operation timestamp comparison and transfer through `-m` or `--use-modification-time`. (S003)
35. **035 — Pull Destination Selectors (`verified`):** makes `pull PATH` destination-oriented by default while retaining `-s` and `--source` for source-space selection and exact destination-winning force repair. Its follow-up debrief resolved the validation blocker with no findings. (S003)
36. **036 — Destination Adoption (`verified`):** permits `pull -a|--adopt DESTINATION` to import one exact destination-only regular file beneath a tree mapping, requiring a source-side ancestor and preserving ordinary source-defined membership. A forced ignored-path adoption changes no policy and reports the precise exemptions required for later ordinary discovery. Its debrief recorded no Must-Address findings. (S003)

## Related pages

- [Initial delivery roadmap future features](./initial-delivery-roadmap-future-features.md)
- [Command-line and path selection](./command-line-and-path-selection.md)
- [Mappings and managed membership](./mappings-and-managed-membership.md)
