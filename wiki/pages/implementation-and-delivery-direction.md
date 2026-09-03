---
title: Implementation and delivery direction
type: decision
sources: [S001]
updated: 2026-09-03
---

# Implementation and delivery direction

Grip is expected to be a Rust command-line application organized as small, composable responsibilities: CLI and output, registry validation, source discovery and ignore evaluation, filesystem metadata access, snapshot construction, baseline comparison, deterministic planning, dry-run rendering, guarded execution, backup, and baseline publication. Planning and execution remain separate so read-only behavior can be tested before mutation. (S001)

Stable Rust with the 2024 edition is the starting direction. A checked-in toolchain file should align development and CI, and an application lockfile should be committed; exact compiler support and dependency choices remain implementation decisions rather than product commitments. (S001)

`clap` is the leading CLI-framework candidate, while the `ignore`, Serde, `thiserror`, and `tracing` ecosystems are possible supporting directions. These choices are advisory and should be introduced only when their capabilities and maintenance costs are justified; Grip-owned domain commands and structured errors should remain independent of framework details. (S001)

Delivery proceeds through safety-increasing vertical slices: read-only discovery, snapshot and status, guarded push without deletion, reverse synchronization, bidirectional sync with separately authorized deletion, and deliberate metadata expansion. This order validates namespace and state semantics before exposing destructive behavior. (S001)

## Related pages

- [Grip product model](./grip-product-model.md)
