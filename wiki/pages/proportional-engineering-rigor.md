---
title: Proportional engineering rigor
type: decision
sources: [S002, S005, S006]
updated: 2026-09-05
---

# Proportional engineering rigor

Grip uses the simplest design that protects a local user's managed files and accepted state. Complexity must address demonstrated local failure modes rather than hypothetical distributed-system conditions outside the product boundary. (S002)

Long-lived filesystem locks, background watchers, persistent inode identity, kernel integration, distributed coordination, and snapshot-isolation machinery are excluded unless a feature specification shows why inspection, revalidation, atomic publication, and recovery cannot satisfy its requirement. A proposal for such machinery must name the concrete failure, simpler alternatives, and operational cost. (S002)

Local concurrency is handled by narrow evidence capture, pre-action revalidation, safe publication, and explicit drift errors. Grip may use a short-lived per-user lock to protect its own registry or state publication when concurrent Grip processes could corrupt that data, but it cannot depend on inode identity remaining stable or prevent unrelated applications from editing user files. (S002)

Performance work follows the same proportional rule: common read-only and planning workflows remain responsive, redundant filesystem work is avoided, and measurement of representative workloads must justify caches, parallelism, or additional indexing. (S002)

Feature 002 validates up to 1,000 mappings with an auditable all-pairs ownership check instead of adding a trie, persistent index, graph framework, cache, or async runtime. Canonical accepted-registry validation is performed once per load, and a 100-run release harness verified canonical 1,000-mapping list results while keeping the p95 workflow below the feature's one-second threshold. (S005)

Feature 003 retained sequential source and destination traversal after a release harness measured a 10,000-entry inspection at 129.403125 ms p95 across 100 runs, below its two-second threshold. The evidence did not justify caches, parallel traversal, persistent indexes, or background state. (S006)

## Related pages

- [Safety and recovery model](./safety-and-recovery-model.md)
- [Implementation and delivery direction](./implementation-and-delivery-direction.md)
