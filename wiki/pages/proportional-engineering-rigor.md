---
title: Proportional engineering rigor
type: decision
sources: [S002]
updated: 2026-09-03
---

# Proportional engineering rigor

Grip uses the simplest design that protects a local user's managed files and accepted state. Complexity must address demonstrated local failure modes rather than hypothetical distributed-system conditions outside the product boundary. (S002)

Long-lived filesystem locks, background watchers, persistent inode identity, kernel integration, distributed coordination, and snapshot-isolation machinery are excluded unless a feature specification shows why inspection, revalidation, atomic publication, and recovery cannot satisfy its requirement. A proposal for such machinery must name the concrete failure, simpler alternatives, and operational cost. (S002)

Local concurrency is handled by narrow evidence capture, pre-action revalidation, safe publication, and explicit drift errors. Grip may use a short-lived per-user lock to protect its own registry or state publication when concurrent Grip processes could corrupt that data, but it cannot depend on inode identity remaining stable or prevent unrelated applications from editing user files. (S002)

Performance work follows the same proportional rule: common read-only and planning workflows remain responsive, redundant filesystem work is avoided, and measurement of representative workloads must justify caches, parallelism, or additional indexing. (S002)

## Related pages

- [Safety and recovery model](./safety-and-recovery-model.md)
- [Implementation and delivery direction](./implementation-and-delivery-direction.md)
