---
title: Local development workflows
type: howto
sources: [S004, S008, S009, S010, S011]
updated: 2026-09-07
---

# Local development workflows

Install the pinned toolchain with `mise trust` followed by `mise install`. A direct release build is available through `mise exec -- cargo build --release`. (S004)

The preferred project entry points are `mise run build` for a development build, `mise run test` for the default test suite, and `mise run clean` to remove Cargo build artifacts before a clean rebuild. (S004)

Use `mise run validate` for the complete verification sequence: formatting, Clippy with warnings denied, default tests, and a release build. Use `mise run performance` to build in release mode and execute the otherwise ignored 100-run performance acceptance harness. (S004)

Feature 005's representative harness constructs 10,000 eligible mixed entries and compares 100 JSON dry-run plans with 100 internal execute-mode plans stopped before mutation. The accepted run kept both p95 planning distributions below the two-second requirement without adding a cache, persistent index, parallel traversal, service, or new dependency. (S008)

Feature 006 extends that harness with 6,700 synchronized and 3,300 destination-only changed accepted entries. On the documented macOS/aarch64 release run, pull dry-run p95 was 997.840917 ms and execution-planning p95 was 834.299833 ms across 100 warm runs, satisfying the two-second requirement without new caching, indexing, parallelism, or coordination machinery. (S009)

Feature 007's 10,000-entry mixed fixture covers synchronized, source-only, destination-only, converged, and conflicting classifications. Across 100 warm macOS ARM64 release runs, sync preview p95 was 1.146991333 seconds and pure planning p95 was 0.942500750 seconds; both passed the two-second threshold without new caching, services, broad payload locks, or unmeasured parallelism. (S010)

Feature 008 adds documented 10,000-entry deletion-preview and recovery-inventory workloads. Across 100 warm release runs on the representative workstation, deletion preview p95 was 1.054690542 seconds and recovery inventory p95 was 1.412352292 seconds, satisfying the two-second threshold without a persistent cache, background service, broad payload lock, or automatic cleanup policy. (S011)

## Related pages

- [Implementation and delivery direction](./implementation-and-delivery-direction.md)
- [Command-line and path selection](./command-line-and-path-selection.md)
