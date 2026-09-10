---
title: Configuration and state
type: component
sources: [S001]
updated: 2026-09-10
---

# Configuration and state

Portable mapping declarations live in the Grip project. Mutable current evidence lives in the ignored `.grip/state/` directory. State contains only baselines for active managed membership and is pruned when mappings are removed or ignore policy removes entries. (S001)

The state is deployment evidence, not an archive. It contains no retained payload history and must not revive a baseline when a mapping or ignored entry is later reintroduced. (S001)
