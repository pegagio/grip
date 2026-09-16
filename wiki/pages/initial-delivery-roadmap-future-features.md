---
title: Initial delivery roadmap future features
type: reference
sources: [S003]
updated: 2026-09-16
---

# Initial delivery roadmap future features

This continuation records Feature 027 and planned Feature 028 from the roadmap ledger. (S003)

27. **027 — Aggregate Forced Push (`verified`):** permits `grip push --force` without a selector to make the complete source state authoritative for all managed entries in the selected project. A supplied selector remains under the existing exact-entry force contract. Verified entries publish accepted evidence independently after revalidation and verification; a later failure leaves the aggregate failed and later entries unaccepted. The feature does not authorize aggregate pull, implicit force, or bypasses for unsupported nodes, symlink ancestors, ownership, topology, or project validation. (S003)
28. **028 — Configurable External Diff Program (`planned`):** makes `grip diff` invoke a configurable external comparison program, defaulting to the `diff` executable. It requires direct executable-and-tokenized-argument invocation rather than shell evaluation, with defined process handling and diagnostics; it does not change classification, selection, JSON output, mutation, ownership, or baseline semantics. (S003)

The roadmap resolves aggregate forced-push baseline publication through Feature 027 and defers the external diff configuration contract to Feature 028. (S003)

## Related pages

- [Initial delivery roadmap recent features](./initial-delivery-roadmap-recent-features.md)
- [Roadmap open questions recent](./roadmap-open-questions-recent.md)
- [Command-line and path selection](./command-line-and-path-selection.md)
