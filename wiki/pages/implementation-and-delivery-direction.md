---
title: Implementation and delivery direction
type: decision
sources: [S001, S002, S003, S004, S006, S008, S009, S013, S014]
updated: 2026-09-10
---

# Implementation and delivery direction

Grip is a local command-line application organized as small, composable responsibilities: CLI and output, registry validation, source discovery and ignore evaluation, filesystem metadata access, baseline comparison, deterministic planning, dry-run rendering, guarded execution, temporary staging, and baseline publication. Planning and execution remain separate so read-only behavior can be tested before mutation. (S014)

The implementation remains a single Rust command-line crate. Exact compiler support and dependency choices remain implementation decisions rather than product commitments. (S014)

Historical delivery proceeded through safety-increasing vertical slices: read-only discovery, snapshot and status, guarded push, reverse synchronization, bidirectional sync, and metadata expansion. That sequence remains useful provenance, but its public recovery, retirement, deletion, and separate-resolution interfaces are superseded rather than current product behavior. (S003) (S013) (S014)

Implementation choices must use the simplest mechanism that protects managed files and accepted state. New frameworks, services, caches, concurrency mechanisms, or persistent indexes require a concrete capability need and an explanation of their maintenance and correctness costs; reversible choices belong in feature plans rather than durable product governance. (S002)

Common inspection and planning workflows must remain responsive on representative local trees. Performance changes are driven by measurement, and behavior that can change payloads, descriptor data, or baselines requires automated coverage in isolated temporary projects rather than the developer's real files. (S002)

Features 001 through 010 established the single-crate CLI, project scope, registry, discovery, state, mutation, and metadata foundations. Their retained safety mechanics support bounded staging, locking, revalidation, verification, and truthful result reporting; they do not imply that earlier public recovery or resolution commands remain available. (S003) (S004) (S006) (S008) (S009) (S013) (S014)

Feature 010 introduced one selected project context, portable declarations and durable identities, and project-local mutable state. Feature 011 then completed the current interface replacement: the public surface is the flat ten-command hierarchy, internal baseline evidence follows active membership, and Git or another operator-selected system owns history and recovery. (S003) (S013) (S014)

The current product deliberately adds no daemon, watcher, persistent index, compatibility reader, automatic Git behavior, retained payload backup, or user-facing recovery interface. Its verification surface includes current-interface rejection coverage, isolated filesystem and state tests, the repository quality gate, and explicit representative performance qualification. (S014)

The roadmap records Feature 011 as verified after a trustworthy HEAD-to-worktree debrief returned `PROCEED` with no findings, completing the eleven-feature initial delivery sequence. (S003)

Feature 011 records a clean convergence pass, formatting, lint, default-test, and release-build validation, plus representative performance acceptance for the revised command hierarchy. (S014)

## Related pages

- [Grip product model](./grip-product-model.md)
- [Proportional engineering rigor](./proportional-engineering-rigor.md)
- [Spec evolution and merge-bounded persistence](./spec-evolution-and-merge-bounded-persistence.md)
- [Initial delivery roadmap](./initial-delivery-roadmap.md)
- [Local development workflows](./local-development-workflows.md)
- [Git-inspired command hierarchy](./git-inspired-command-hierarchy.md)
