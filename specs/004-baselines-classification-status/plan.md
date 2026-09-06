# Implementation Plan: Baselines, Classification, and Status

**Branch**: `004-baselines-classification-status` | **Date**: 2026-09-05 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/004-baselines-classification-status/spec.md`

## Summary

Extend Grip's stable two-pass discovery into a fingerprinted observation pipeline, join current source and destination evidence with versioned accepted baselines, and classify the complete union through a pure exhaustive domain model. Add read-only `status`, `check`, and `diff` commands plus state-only `baseline accept`; persist accepted evidence in a strict State Envelope V2 through bounded registry-then-state coordination, hardened recovery and atomic publication, while continuing to read State V1 as an empty-baseline predecessor. Reuse the existing SHA-256, rustix, Serde, CLI, and testing stack; add no dependency, cache, index, background work, or payload mutation.

## Technical Context

**Language/Version**: Rust 1.98.0, Rust 2024 edition

**Primary Dependencies**: Existing `clap` 4.6, `rustix` 1.1 with `fs` and `process`, `ignore` 0.4.33, Serde/`serde_json`, `sha2` 0.10, `thiserror` 2, and `tempfile` for tests; no new dependency

**Storage**: Read strict State Envelope V1 as an empty-baseline predecessor; publish strict, integrity-checked State Envelope V2 at `state/state.json`, with immutable prior generations under `state/recovery/generation-<N>/state.json`

**Testing**: Rust unit tests, exhaustive table-driven classifier tests, isolated filesystem and CLI integration tests, deterministic race/publication fault seams, existing state/registry compatibility tests, and the ignored release performance harness

**Target Platform**: macOS and Unix-like systems supported by the existing Rust/rustix build; permission mode follows the Unix `0o7777` contract

**Project Type**: Single Rust CLI application and reusable library modules

**Performance Goals**: Equivalent ordered output across 100 unchanged runs; at least 95 of 100 warm release-mode full status runs over 10,000 equivalent paired entries and accepted baselines complete within two seconds

**Constraints**: `status`, `check`, and `diff` are lock-free and non-mutating; baseline acceptance changes Grip-owned state only; SHA-256 content is streamed from descriptor-bound ordinary files; mtime is diagnostic only; unsupported nodes are never opened; no payload mutation, automatic recovery, multiple selectors, cache, persistent index, parallel traversal, background service, broad tree lock, or snapshot-isolation claim

**Scale/Scope**: Every current accepted mapping plus retained baseline-only identities, or one source/destination selector covering one mapping, entry, or subtree; complete union of active discovery and persisted baseline identities; 18 stable classification categories; one accepted-state generation per semantic update

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **I. Proportional rigor**: The design reuses the current crate and dependencies, adds pure classification plus focused fingerprint/state responsibilities, and introduces no daemon, watcher, cache, persistent index, parallel runtime, payload lock, inode identity, or snapshot claim. Descriptor-bound hashing and expected-snapshot publication address demonstrated local races.
- **II. Explicit ownership and least surprise**: Current mappings and source policy define active ownership. Destination-only entries stay unmanaged, unsupported nodes are never opened, removed-mapping baselines remain pending retirement, and the feature never copies, deletes, retires, resolves, or otherwise mutates payloads.
- **III. Validate, revalidate, and recover**: Baseline acceptance derives one deterministic complete candidate, locks only Grip-owned registry/state publication, revalidates state, mapping, membership, and fingerprints, preserves the prior state generation, verifies the staged candidate, and publishes only after all selected evidence remains acceptable. Read-only commands publish nothing.
- **IV. Bounded concurrency**: Inspection compares fresh complete passes without locks. State mutation uses a single documented lock order—registry, then state—and releases both after expected-snapshot revalidation and atomic state publication. External payload changes become stale-evidence errors.
- **V. Fast, observable, and testable**: Typed domain results drive deterministic human and JSON forms while diagnostics stay on stderr. Exhaustive pure tests and isolated filesystem tests cover every classification and failure boundary; the 10,000-entry release measurement gates any later optimization.
- **Product boundaries**: The feature remains local, offline, per-user, allowlist-based, and privilege-neutral. State V2 contains fingerprints and supported metadata, never payload contents, and the CLI/domain/rendering separation remains explicit.
- **Merge-bounded persistence**: Clarified behavior is reflected in the spec and Phase 0 decisions are reflected in this plan and its Phase 1 artifacts. After tasks are generated, `speckit-analyze` gates implementation; accepted implementation discoveries must flow back before merge and `speckit-converge` must close remaining gaps.

Post-design re-check: passed. The data model makes equality, identity, ordering, schema evolution, and publication transitions explicit; the three CLI contracts preserve established result and filesystem boundaries. No exception requires complexity tracking.

## Project Structure

### Documentation (this feature)

```text
specs/004-baselines-classification-status/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── cli.md
│   ├── classification.md
│   └── storage.md
└── tasks.md                  # Created later by speckit-tasks
```

### Source Code (repository root)

```text
src/
├── cli.rs                    # Add status/check/diff/baseline grammar and shared selector
├── error.rs                  # Activate attention and state-contention categories
├── lib.rs                    # Thin command routing
├── result.rs                 # Typed classification/baseline projections and human rendering
├── baseline.rs               # Candidate construction, locked revalidation, and acceptance result
├── path_policy.rs            # Separate durable mapping validity from current presence
├── observation/
│   ├── mod.rs                # Stable two-pass observation orchestration and selection
│   ├── model.rs              # Evidence-bearing normalized source/destination snapshot
│   └── fingerprint.rs        # Descriptor-bound streamed SHA-256 and supported metadata
├── classification/
│   ├── mod.rs                # Pure exhaustive three-way classifier
│   └── model.rs              # Categories, comparisons, counts, directions, result DTOs
├── discovery/
│   ├── mod.rs                # Preserve mapping-inspect projection over shared observation
│   ├── model.rs              # Reuse SafePath, node kinds, raw ordering, and evidence
│   ├── filesystem.rs         # Extend safe descriptors for ordinary-file hashing
│   └── ignore_policy.rs      # Reuse source-side membership policy
├── registry/
│   └── publication.rs        # Expose durable load/lock/revalidation seams
└── state/
    ├── mod.rs                # Version-neutral state plus strict V1/V2 codecs
    ├── lock.rs               # Reuse bounded advisory state lock
    └── publication.rs        # Expected-snapshot, recovery, staging, and atomic publish

tests/
├── classification_matrix.rs
├── classification_filesystem_integration.rs
├── classification_cli_contract.rs
├── baseline_integration.rs
├── state_integration.rs
├── performance_acceptance.rs
└── support/
    └── mod.rs
```

**Structure Decision**: Preserve the single-crate architecture. Extract a shared internal observation layer because Feature 003's rendered inventory intentionally discards evidence needed for classification; keep mapping inspection as a stable projection. Keep filesystem reads out of the pure classifier and state wire concerns out of CLI routing. Harden the existing state publisher rather than create a second persistence mechanism.
