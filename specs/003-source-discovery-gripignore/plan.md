# Implementation Plan: Source Discovery and Gripignore

**Branch**: `003-source-discovery-gripignore` | **Date**: 2026-09-04 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/003-source-discovery-gripignore/spec.md`

## Summary

Add a read-only `grip mapping inspect [SOURCE]` workflow that validates the complete registry, derives file and tree membership dynamically, applies only source-side root and nested `.gripignore` policy, inventories destination-only entries, and reports unsupported boundaries without following or opening them. A small discovery domain coordinates a descriptor-relative Unix filesystem adapter built on the existing `rustix` dependency and a new `ignore` dependency used only for Gitignore-compatible pattern evaluation. Two complete sequential passes compare ephemeral evidence before returning one deterministic inventory; the feature publishes no state and introduces no locks, caches, background work, or payload mutation.

## Technical Context

**Language/Version**: Rust 1.98.0, Rust 2024 edition

**Primary Dependencies**: Existing `clap` 4.6, `rustix` 1.1 with `fs`, Serde/`serde_json`, `thiserror`, and `tempfile` for tests; add `ignore` 0.4.33 for `GitignoreBuilder` matching only

**Storage**: Read existing `config.toml`; no discovery inventory, baseline, lock, staging file, or other state is persisted

**Testing**: Rust unit tests, isolated `cargo test` filesystem and CLI integration tests, deterministic fault/barrier tests, and the existing ignored release performance harness

**Target Platform**: macOS and Unix-like systems supported by the existing Rust/rustix build; platform-only node types use conditional classification tests

**Project Type**: Single Rust CLI application and reusable library modules

**Performance Goals**: Equivalent ordered output across 100 unchanged runs; at least 95 of 100 warm release-mode discoveries of a representative 10,000-entry tree with nested policy complete within two seconds

**Constraints**: Read-only and offline; no following symbolic links; no payload reads or hashes; only `.gripignore` controls membership; destination-only paths remain unmanaged; no broad locks, persistent inode identity, cache, index, background traversal, baseline comparison, or mutation

**Scale/Scope**: All accepted mappings or one mapping selected by canonical source identity; ordinary files and directories including empty directories; five inventory categories; root and nested policy; one sequential two-pass traversal per request

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **I. Proportional rigor**: The design adds one focused matcher dependency to avoid maintaining a partial Gitignore implementation. Traversal stays sequential and local, reuses `rustix`, and adds no daemon, watcher, cache, index, parallel runtime, persistent identity, or snapshot-isolation claim.
- **II. Explicit ownership and least surprise**: Source membership comes only from accepted mappings and source-side policy. Destination-only entries remain unmanaged, `.gripignore` remains policy-only, and unsupported nodes are reported without being followed or coerced.
- **III. Validate, revalidate, and recover**: This feature is read-only and therefore creates no recovery data. It validates the complete registry, captures narrow ephemeral evidence, performs a second complete pass, and fails rather than returning a knowingly inconsistent inventory.
- **IV. Bounded concurrency**: No filesystem or Grip-owned lock is acquired. Directory-handle-relative inspection narrows pathname races, and a pass mismatch becomes explicit `stale_discovery_evidence` rather than a false completeness claim.
- **V. Fast, observable, and testable**: Results are deterministic and separate human, JSON, and diagnostic channels. Real isolated filesystem tests cover every supported and unsupported boundary; the release harness measures the required 10,000-entry workload before any optimization is considered.
- **Product boundaries**: The workflow remains a local per-user CLI. It does not contact remote services, elevate privileges, invoke Git, publish state, compare content/metadata equality, or mutate source or destination paths.
- **Merge-bounded persistence**: Phase 0 decisions are reflected in the plan and Phase 1 contracts. Tasking must run `speckit-analyze` before implementation; implementation discoveries must flow back before merge and `speckit-converge` must close post-implementation gaps.

Post-design re-check: passed. The data model is non-persistent, the CLI contract adds one read-only subcommand, and the filesystem and policy contracts preserve every pre-research gate. No exception requires Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/003-source-discovery-gripignore/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── cli.md
│   ├── discovery.md
│   └── gripignore.md
└── tasks.md                  # Created later by speckit-tasks
```

### Source Code (repository root)

```text
src/
├── cli.rs                    # Add mapping inspect grammar
├── error.rs                  # Stable discovery failure reasons
├── lib.rs                    # Route the command and expose discovery
├── mapping.rs                # Reuse mapping identity and ordering
├── path_policy.rs            # Reuse endpoint/selector policy concepts
├── result.rs                 # Render deterministic inventory results
├── discovery/
│   ├── mod.rs                # Orchestration, two-pass comparison, public result
│   ├── filesystem.rs         # Descriptor-relative Unix inspection/evidence
│   ├── ignore_policy.rs      # Nested .gripignore matcher stack
│   └── model.rs              # Records, safe paths, reasons, and ordering
└── registry/
    └── publication.rs        # Expose read-only snapshot revalidation

tests/
├── discovery_cli_contract.rs
├── discovery_filesystem_integration.rs
├── gripignore_conformance.rs
├── performance_acceptance.rs
└── support/
    └── mod.rs
```

**Structure Decision**: Preserve the existing single-crate CLI. Add one cohesive `discovery` module because traversal, ignore policy, and result modeling are independently testable responsibilities reused by Feature 004; keep entrypoint, registry, and rendering changes narrow.
