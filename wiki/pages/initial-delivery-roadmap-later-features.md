---
title: Initial delivery roadmap later features
type: reference
sources: [S003]
updated: 2026-09-14
---

# Initial delivery roadmap later features

This continuation records Features 013 through 022 from the current roadmap ledger. (S003)

13. **013 — Source Path Input Normalization (`implemented`):** accepts ordinary project-contained relative source spellings while retaining strict canonical source declarations. (S003)
14. **014 — Relative Destination Paths (`verified`):** extends destination declarations with paths relative to the selected project root while retaining exact declaration text and resolving it separately for operations. (S003)
15. **015 — Simplify Status Output (`verified`):** replaces verbose default human status detail with concise, path-centered conflicts, push, pull, and baseline-attention groups while preserving detailed JSON, `diff`, selection, and exit contracts. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
16. **016 — Current-Directory Status Paths (`implemented`):** makes default-human source labels relative to the invocation directory and lets an ordinary displayed label select the same entry in a following `push`. (S003)
17. **017 — Simplify Default Command Output (`implemented`):** makes mapping, status, mutation, and error output concise while preserving JSON and detailed `diff` contracts. (S003)
18. **018 — Source-Authoritative Mapping Addition (`implemented`):** records the destination as the initial comparison reference for an unequal newly added source-defined member, so ordinary synchronization offers a push without payload mutation during `add`. (S003)
19. **019 — Executable Force-Resolution Guidance (`implemented`):** emits force guidance only for exact resolvable entries and directs aggregate conflicts to inspect `grip diff SOURCE`. (S003)
20. **020 — Binary Size Investigation (`abandoned`):** was not pursued because the reported approximately 4.5 MB release artifact already met its less-than-10 MiB stretch target. (S003)
21. **021 — Large-File Operation Performance (`verified`):** defines isolated local macOS ARM64 release workloads with differing 19 MiB regular endpoints for `grip add` and JSON `grip status`, then removes demonstrated same-pass duplicate observation while retaining full content and metadata evidence, independent stable-observation passes, and fenced add reinspection. A 100-sample p95 gate requires both operations to complete within one second; the recorded final p95 values were 573.669375 ms for `grip add` and 184.788375 ms for `grip status`. (S003)
22. **022 — Force Mapping Replacement (`verified`):** permits `grip add --force` to replace exactly one active file mapping at the same resolved destination, preserves other ownership and topology rejections, and keeps add payload-nonmutating. Exact removal retires the corresponding ownership and accepted comparison evidence so a later ordinary add may reuse the destination. Its no-finding debrief verified the implementation against the roadmap. (S003)

Verified Features 001 through 012, 014, 015, 021, and 022 provide the foundation, mutation, conflict, metadata, filesystem-boundary, qualification, portable project-authority, concise public-command, destination-declaration, initial status-presentation, measured large-file responsiveness, and exact mapping-replacement chain. Implemented Features 013 and 016 through 019 extend source input, status-path, concise-output, initial-add, and force-guidance behavior pending debrief. Two-way interactive merge, remap, multiple selectors, remote synchronization, daemons, privileged services, and multi-user coordination remain outside this initial roadmap. (S003)

## Related pages

- [Initial delivery roadmap](./initial-delivery-roadmap.md)
- [Implementation and delivery direction](./implementation-and-delivery-direction.md)
