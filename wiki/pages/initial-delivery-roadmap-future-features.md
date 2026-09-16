---
title: Initial delivery roadmap future features
type: reference
sources: [S003]
updated: 2026-09-16
---

# Initial delivery roadmap future features

This continuation records planned Features 027 and 028 from the roadmap ledger. (S003)

27. **027 — Aggregate Forced Push (`planned`):** permits `grip push --force` without a selector to make the complete source state authoritative for all managed entries in the selected project. A supplied selector remains under the existing exact-entry force contract. The feature retains dry-run parity, current-evidence revalidation, no-follow safety, and precise reporting; it does not authorize aggregate pull, implicit force, or bypasses for unsupported nodes, symlink ancestors, ownership, topology, or project validation. (S003)
28. **028 — Configurable External Diff Program (`planned`):** makes `grip diff` invoke a configurable external comparison program, defaulting to the `diff` executable. It requires direct executable-and-tokenized-argument invocation rather than shell evaluation, with defined process handling and diagnostics; it does not change classification, selection, JSON output, mutation, ownership, or baseline semantics. (S003)

The roadmap explicitly defers the aggregate forced-push baseline-publication model and the external diff configuration contract to their owning specifications. (S003)

## Related pages

- [Initial delivery roadmap recent features](./initial-delivery-roadmap-recent-features.md)
- [Roadmap open questions recent](./roadmap-open-questions-recent.md)
- [Command-line and path selection](./command-line-and-path-selection.md)
