---
title: Spec evolution and merge-bounded persistence
type: decision
sources: [S002, S005, S006, S008, S009]
updated: 2026-09-07
---

# Spec evolution and merge-bounded persistence

Grip uses the Merge-Bounded Flow-Back Spec Persistence Model. Before merge, a feature's `spec.md`, `plan.md`, `tasks.md`, and implementation form one mutable, reviewable change set. Accepted behavioral, technical, or work discoveries must flow back to the corresponding artifacts so lower-level work does not silently contradict higher-level intent. (S002)

Flow-back cannot introduce material scope without review. Independently valuable behavior, substantial expansion, or work requiring separate acceptance becomes a separate feature. After tasking or consequential reconciliation, `/speckit.analyze` gates implementation or resumption; after implementation, `/speckit.converge` runs until known gaps are reconciled or explicitly removed from scope. (S002)

Acceptance into the designated integration branch freezes the merged feature directory as a semantically immutable historical record. Later behavioral changes flow forward through a new feature directory that references material amendments, replacements, or dependencies; only meaning-preserving editorial corrections may alter merged history. (S002)

The constitution takes precedence over conflicting project artifacts. Amendments require explicit user approval, a Sync Impact Report, and a semantic version increment, while feature specifications and plans identify applicable principles and justify any narrow exception before implementation or merge. (S002)

Feature 002 demonstrates the pre-merge feedback loop: successive convergence passes appended tasks for descriptor identity, error precedence, absent selectors, canonical stored paths, performance, recovery staging, unsupported registry nodes, bounded path inspection, and submitted-path revalidation. Those tasks were implemented and checked before a complete-delta roadmap debrief returned `PROCEED` with no findings and recommended verification. (S005)

Feature 003 likewise kept specification, plan, tasks, contracts, implementation, tests, and performance evidence in one reviewable pre-merge boundary. Its complete-delta debrief returned `PROCEED` with no findings, after which the roadmap transitioned the feature from `in-progress` to `verified`. (S006)

Feature 005 required a convergence phase after its first implementation pass. The appended work removed planner filesystem reads, made operation and recovery access descriptor-relative, preserved phase-specific side-effect truth, routed post-journal failures through terminal evidence, completed result parity, and expanded adversarial tests before the complete-delta debrief returned `PROCEED` with no findings. (S008)

Feature 006 likewise flowed implementation discoveries back before merge: direction-neutral failure and lifecycle evidence, source-side stale-evidence checks, operation-record retry behavior, renderer parity, and representative performance results were reconciled across specification artifacts, code, tests, and documentation. Its complete-delta debrief returned `PROCEED` with no findings and recommended roadmap verification. (S009)

## Related pages

- [Implementation and delivery direction](./implementation-and-delivery-direction.md)
