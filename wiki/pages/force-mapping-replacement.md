---
title: Force mapping replacement
type: component
sources: [S022]
updated: 2026-09-14
---

# Force mapping replacement

`grip add --force SOURCE DESTINATION` has one narrow authority: it may replace exactly one distinct active file mapping whose resolved destination equals the request's resolved destination. The full candidate registry remains subject to existing source-overlap, tree, nested, unsafe, incompatible, and ambiguous ownership validation. (S022)

The transition removes the displaced declaration and its accepted comparison identities, establishes the requested mapping's normal initial comparison evidence, and publishes the descriptor and State V4 candidate together through a verified fence. A retry completes only the recorded candidate or restores the recorded prior pair; it reports a replacement rather than an ordinary add when completion succeeds. (S022)

Neither success nor rejection copies, deletes, or changes either endpoint payload. Exact `grip remove` uses the same descriptor/state retirement boundary, allowing a later ordinary add to reuse the old destination when no active owner remains. (S022)

## Related pages

- [Mapping addition and initial baselines](./mapping-addition-and-initial-baselines.md)
- [Mapping registry publication](./mapping-registry-publication.md)
- [Safety and recovery model](./safety-and-recovery-model.md)
