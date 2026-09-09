---
title: Implementation and delivery direction
type: decision
sources: [S001, S002, S003, S004, S006, S008, S009, S013]
updated: 2026-09-09
---

# Implementation and delivery direction

Grip is expected to be a Rust command-line application organized as small, composable responsibilities: CLI and output, registry validation, source discovery and ignore evaluation, filesystem metadata access, snapshot construction, baseline comparison, deterministic planning, dry-run rendering, guarded execution, backup, and baseline publication. Planning and execution remain separate so read-only behavior can be tested before mutation. (S001)

Stable Rust with the 2024 edition is the starting direction. A checked-in toolchain file should align development and CI, and an application lockfile should be committed; exact compiler support and dependency choices remain implementation decisions rather than product commitments. (S001)

Delivery proceeds through safety-increasing vertical slices: read-only discovery, snapshot and status, guarded push without deletion, reverse synchronization, bidirectional sync with separately authorized deletion, and deliberate metadata expansion. This order validates namespace and state semantics before exposing destructive behavior. (S001)

Implementation choices must use the simplest mechanism that protects managed files and accepted state. New frameworks, services, caches, concurrency mechanisms, or persistent indexes require a concrete capability need and an explanation of their maintenance and correctness costs; reversible choices belong in feature plans rather than durable product governance. (S002)

Common inspection and planning workflows must remain responsive on representative local trees. Performance changes are driven by measurement, and behavior that can change payloads, descriptor data, or baselines requires automated coverage in isolated temporary projects rather than the developer's real files. (S002)

Features 001 through 009 established the single-crate CLI, registry, discovery, state, mutation, deletion, recovery, and metadata boundaries. Shared direction-neutral safety machinery keeps push, pull, sync, and resolution behavior consistent without duplicating locking, revalidation, recovery, operation-record, or result semantics. (S003) (S004) (S006) (S008) (S009)

Feature 010 corrects the product boundary by introducing one selected `ProjectContext`, portable declarations and durable identities, and project-local mutable state. Existing discovery, classification, planning, mutation, deletion, retirement, recovery, metadata, and result behavior remains in the single crate and receives the selected context rather than consulting a global installation. (S013)

The cutover deliberately adds no dependency, daemon, watcher, cache, persistent index, generated project ID, compatibility reader, or automatic Git behavior. The required verification surface includes deep upward discovery, clone isolation and rebinding, current-interface scanning, the complete prior regression matrix, and a 100-sample release performance harness. (S013)

## Related pages

- [Grip product model](./grip-product-model.md)
- [Proportional engineering rigor](./proportional-engineering-rigor.md)
- [Spec evolution and merge-bounded persistence](./spec-evolution-and-merge-bounded-persistence.md)
- [Initial delivery roadmap](./initial-delivery-roadmap.md)
- [Local development workflows](./local-development-workflows.md)
