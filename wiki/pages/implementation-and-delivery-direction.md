---
title: Implementation and delivery direction
type: decision
sources: [S001, S002, S003, S004, S006]
updated: 2026-09-05
---

# Implementation and delivery direction

Grip is expected to be a Rust command-line application organized as small, composable responsibilities: CLI and output, registry validation, source discovery and ignore evaluation, filesystem metadata access, snapshot construction, baseline comparison, deterministic planning, dry-run rendering, guarded execution, backup, and baseline publication. Planning and execution remain separate so read-only behavior can be tested before mutation. (S001)

Stable Rust with the 2024 edition is the starting direction. A checked-in toolchain file should align development and CI, and an application lockfile should be committed; exact compiler support and dependency choices remain implementation decisions rather than product commitments. (S001)

Feature 001 realizes that direction with a pinned Rust toolchain installed through mise and exposes named mise tasks for development builds, cleanup, tests, the full validation suite, and the release performance harness. (S004)

Feature 002 extends the same single application with separate mapping-domain, path-policy, registry-publication, CLI, and result responsibilities. It records intent only, validates the complete ownership graph, revalidates accepted bytes and path evidence under a short-lived registry lock, retains exact prior-registry recovery, and publishes deterministic complete candidates without touching payloads or synchronization state. (S004)

`clap` is the leading CLI-framework candidate, while the `ignore`, Serde, `thiserror`, and `tracing` ecosystems are possible supporting directions. These choices are advisory and should be introduced only when their capabilities and maintenance costs are justified; Grip-owned domain commands and structured errors should remain independent of framework details. (S001)

Delivery proceeds through safety-increasing vertical slices: read-only discovery, snapshot and status, guarded push without deletion, reverse synchronization, bidirectional sync with separately authorized deletion, and deliberate metadata expansion. This order validates namespace and state semantics before exposing destructive behavior. (S001)

Implementation choices must use the simplest mechanism that protects managed files and accepted state. New frameworks, services, caches, concurrency mechanisms, or persistent indexes require a concrete capability need and an explanation of their maintenance and correctness costs; reversible choices belong in feature plans rather than durable product governance. (S002)

Common inspection and planning workflows must remain responsive on representative local trees. Performance changes are driven by measurement, and behavior that can change payloads, registry data, or baselines requires automated coverage in isolated temporary roots rather than the developer's real files or Grip home. (S002)

The durable roadmap refines the product's six delivery milestones into nine planned specifications. It separates CLI and state foundations, mapping ownership, and discovery before baseline classification, then introduces push, pull, bidirectional conflict handling, authorized deletion, and final metadata and filesystem completion in dependency order. (S003)

Features 001 through 003 are verified. Feature 003 adds deterministic read-only source discovery, nested source-side `.gripignore` policy, destination-only reporting, and unsupported-node boundaries without baseline or payload mutation, making Feature 004 dependency-eligible. (S003)

Feature 003 implements that slice with Grip-owned discovery models, `ignore` for matching only, and `rustix` descriptor-relative filesystem access. Two sequential traversal passes keep source policy and destination-only classification explicit; no cache, persistent index, watcher, lock, or parallel traversal was introduced. (S006)

## Related pages

- [Grip product model](./grip-product-model.md)
- [Proportional engineering rigor](./proportional-engineering-rigor.md)
- [Spec evolution and merge-bounded persistence](./spec-evolution-and-merge-bounded-persistence.md)
- [Initial delivery roadmap](./initial-delivery-roadmap.md)
- [Local development workflows](./local-development-workflows.md)
