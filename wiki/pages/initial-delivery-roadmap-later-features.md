---
title: Initial delivery roadmap later features
type: reference
sources: [S003]
updated: 2026-09-15
---

# Initial delivery roadmap later features

This continuation records Features 013 through 020 from the current roadmap ledger. Features 021 through 025 continue on [Initial delivery roadmap recent features](./initial-delivery-roadmap-recent-features.md). (S003)

13. **013 — Source Path Input Normalization (`implemented`):** accepts ordinary project-contained relative source spellings while retaining strict canonical source declarations. (S003)
14. **014 — Relative Destination Paths (`verified`):** extends destination declarations with paths relative to the selected project root while retaining exact declaration text and resolving it separately for operations. (S003)
15. **015 — Simplify Status Output (`verified`):** replaces verbose default human status detail with concise, path-centered conflicts, push, pull, and baseline-attention groups while preserving detailed JSON, `diff`, selection, and exit contracts. Its attributable implementation debrief recorded `PROCEED` with no findings. (S003)
16. **016 — Current-Directory Status Paths (`implemented`):** makes default-human source labels relative to the invocation directory and lets an ordinary displayed label select the same entry in a following `push`. (S003)
17. **017 — Simplify Default Command Output (`implemented`):** makes mapping, status, mutation, and error output concise while preserving JSON and detailed `diff` contracts. (S003)
18. **018 — Source-Authoritative Mapping Addition (`implemented`):** records the destination as the initial comparison reference for an unequal newly added source-defined member, so ordinary synchronization offers a push without payload mutation during `add`. (S003)
19. **019 — Executable Force-Resolution Guidance (`implemented`):** emits force guidance only for exact resolvable entries and directs aggregate conflicts to inspect `grip diff SOURCE`. (S003)
20. **020 — Binary Size Investigation (`abandoned`):** was not pursued because the reported approximately 4.5 MB release artifact already met its less-than-10 MiB stretch target. (S003)
Verified Features 014 and 015 extend the destination-declaration and initial status-presentation contracts. Implemented Features 013 and 016 through 019 extend source input, status-path, concise-output, initial-add, and force-guidance behavior pending debrief; Feature 020 was abandoned without implementation. (S003)

## Related pages

- [Initial delivery roadmap](./initial-delivery-roadmap.md)
- [Initial delivery roadmap recent features](./initial-delivery-roadmap-recent-features.md)
- [Implementation and delivery direction](./implementation-and-delivery-direction.md)
