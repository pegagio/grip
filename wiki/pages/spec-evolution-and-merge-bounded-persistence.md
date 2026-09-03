---
title: Spec evolution and merge-bounded persistence
type: decision
sources: [S002]
updated: 2026-09-03
---

# Spec evolution and merge-bounded persistence

Grip uses the Merge-Bounded Flow-Back Spec Persistence Model. Before merge, a feature's `spec.md`, `plan.md`, `tasks.md`, and implementation form one mutable, reviewable change set. Accepted behavioral, technical, or work discoveries must flow back to the corresponding artifacts so lower-level work does not silently contradict higher-level intent. (S002)

Flow-back cannot introduce material scope without review. Independently valuable behavior, substantial expansion, or work requiring separate acceptance becomes a separate feature. After tasking or consequential reconciliation, `/speckit.analyze` gates implementation or resumption; after implementation, `/speckit.converge` runs until known gaps are reconciled or explicitly removed from scope. (S002)

Acceptance into the designated integration branch freezes the merged feature directory as a semantically immutable historical record. Later behavioral changes flow forward through a new feature directory that references material amendments, replacements, or dependencies; only meaning-preserving editorial corrections may alter merged history. (S002)

The constitution takes precedence over conflicting project artifacts. Amendments require explicit user approval, a Sync Impact Report, and a semantic version increment, while feature specifications and plans identify applicable principles and justify any narrow exception before implementation or merge. (S002)

## Related pages

- [Implementation and delivery direction](./implementation-and-delivery-direction.md)
