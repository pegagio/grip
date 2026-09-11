---
title: Configuration and state
type: component
sources: [S001, S015]
updated: 2026-09-11
---

# Configuration and state

Portable mapping declarations live in the Grip project. Mutable current evidence lives in the ignored `.grip/state/` directory. State contains only baselines for active managed membership and is pruned when mappings are removed or ignore policy removes entries. (S001)

The descriptor retains the exact accepted destination declaration, including absolute or lexically non-normalized home-relative spelling. Runtime validation and state binding use the separately resolved endpoint rather than rewriting that declaration. (S001)

Registry publication and State V4 identity carry the original declaration, which lets absolute destinations and non-normalized `~/` spellings survive reload and later operations. See [Destination path forms](./destination-path-forms.md). (S015)

The state is deployment evidence, not an archive. It contains no retained payload history and must not revive a baseline when a mapping or ignored entry is later reintroduced. (S001)
