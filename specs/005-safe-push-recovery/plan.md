# Implementation Plan: Safe Push and Recovery

**Branch**: `005-safe-push-recovery` | **Date**: 2026-09-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/005-safe-push-recovery/spec.md`

## Summary

Add Grip's first payload-mutating command by deriving a complete typed push plan from the existing two-pass observation and classification pipeline, rendering the same plan for dry run, and executing only source additions and source-only changes after lock-held revalidation. Publish files through descriptor-relative sibling staging, preserve verified prior destinations under a private per-operation recovery namespace, checkpoint a strict partitioned operation record without repeatedly serializing unchanged plan data, stop on the first execution failure, verify every result, and reuse State Envelope V2 publication only after complete success. Introduce no dependency, cache, background service, automatic rollback, broad filesystem lock, or new accepted-state schema.

## Technical Context

**Language/Version**: Rust 1.98.0, Rust 2024 edition

**Primary Dependencies**: Existing `clap` 4.6, `rustix` 1.1 with `fs` and `process`, `ignore` 0.4.33, Serde/`serde_json`, `sha2` 0.10, `thiserror` 2, and `tempfile` for tests; no new dependency

**Storage**: Preserve accepted State Envelope V2 at `state/state.json`; add a strict partitioned Operation Record V1 under `state/operations/<operation-id>/` with immutable `plan.json`, bounded `operation.json`, per-started-action checkpoints, and private recovery payloads; add stable owner metadata at `.mutation.lock`

**Testing**: Rust unit tests, pure planner matrices, isolated CLI/filesystem/recovery/contention integration tests, strict partitioned-record round trips, bounded-serialization assertions, deterministic typed fault injection, output-writer failure tests, existing regression suites, and the ignored release performance harness

**Target Platform**: macOS and Unix-like systems supported by the existing Rust/rustix build; ordinary regular files and directories only

**Project Type**: Single Rust CLI application with reusable domain and filesystem modules

**Performance Goals**: Equivalent ordered output across 100 unchanged dry runs; at least 95 of 100 warm release-mode plans over 10,000 eligible mixed entries complete within two seconds

**Constraints**: Complete preflight before mutation; dry-run non-mutation; mutation lock acquired only for an actionful unblocked execution; descriptor-relative no-follow access; same-parent staging; verified recovery before replacement; stop after first execution failure; no partial baseline; output failure cannot invalidate an already-published baseline; no pull, sync, deletion, conflict resolution, automatic rollback/recovery, privilege elevation, broad payload lock, snapshot guarantee, cache, index, or parallel execution

**Scale/Scope**: All mappings or one source/destination selector; exactly one complete plan over every selected entry; 18 inherited classifications; source additions and source-only changes are actionable; exactly 10,000 eligible entries in representative performance acceptance

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **I. Proportional rigor**: The design reuses the current crate, observation/classification pipeline, dependencies, publication primitives, and test patterns. One outer advisory writer lock, one strict partitioned operation record, and focused descriptor-relative mutation helpers address demonstrated payload and crash risks without a daemon, watcher, cache, persistent index, parallel runtime, inode identity contract, or snapshot claim.
- **II. Explicit ownership and least surprise**: Current mappings and source-defined membership remain the only ownership authority. Every selected entry has an explicit plan disposition, destination-only content remains untouched, dry run creates no artifacts, unsupported or conflicting evidence blocks before mutation, and deletion is absent.
- **III. Validate, revalidate, and recover**: Push derives a complete deterministic plan, revalidates the complete evidence under writer coordination, stages and verifies payloads, preserves and verifies prior destination state before replacement, checkpoints partial outcomes, verifies final state, and publishes one baseline only after complete success. Automatic rollback is intentionally excluded.
- **IV. Bounded concurrency**: Initial inspection is lock-free. An actionful push acquires one short-lived per-user advisory mutation lock after preflight, then revalidates and holds it through payload execution and baseline publication. All Grip writers participate in the global order `mutation → registry → state`; no payload-tree lock is introduced.
- **V. Fast, observable, and testable**: Typed plans, partitioned operation records, and results separate domain state, human output, JSON, and diagnostics. Immutable plan intent is serialized once, each action checkpoint rewrites only bounded action evidence, and ordinary push does not scan retained operation history. Deterministic fault seams cover every visible side-effect boundary, and the 10,000-entry p95 harness gates optimization.
- **Product boundaries**: The feature remains local, offline, per-user, allowlist-based, and privilege-neutral. Recovery contains only deliberately preserved payloads in Grip-owned private state; no Git or external service participates.
- **Merge-bounded persistence**: Clarifications are reflected in the specification and all Phase 0 decisions are reflected across this plan and Phase 1 artifacts. After task generation, `speckit-analyze` gates implementation; accepted discoveries flow back before merge and `speckit-converge` closes remaining gaps.

Post-design re-check: passed. The data model separates plan evidence, mutable execution state, recovery payloads, and accepted baselines; the CLI, filesystem, and storage contracts preserve every ownership and failure boundary. No exception requires complexity tracking.

## Project Structure

### Documentation (this feature)

```text
specs/005-safe-push-recovery/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── cli.md
│   ├── push.md
│   ├── filesystem.md
│   └── storage.md
└── tasks.md                  # Created later by speckit-tasks
```

### Source Code (repository root)

```text
src/
├── cli.rs                         # Add action-specific PushArgs
├── lib.rs                         # Thin push orchestration and render-failure finalization
├── result.rs                      # Typed push projection and human/JSON rendering
├── error.rs                       # Push blockers/failures and operation-aware contention
├── baseline.rs                    # Build accepted state from verified actioned identities
├── observation/                   # Reuse complete two-pass evidence and selectors
├── classification/                # Reuse pure 18-category classification
├── registry/
│   └── publication.rs             # Participate in outer mutation-lock order
├── state/
│   ├── mod.rs                     # Preserve State V1/V2 unchanged
│   ├── mutation_lock.rs           # Stable owner-aware outer writer lock
│   ├── lock.rs                    # Existing narrow state publication lock
│   └── publication.rs             # Reuse accepted baseline publication
├── operation/
│   ├── mod.rs
│   ├── model.rs                   # Strict partitioned Operation Record V1
│   └── publication.rs             # Private atomic plan, summary, and action checkpoints
└── push/
    ├── mod.rs                     # Public internal push entry points
    ├── model.rs                   # Plans, actions, dispositions, results
    ├── plan.rs                    # Pure planner and dependency ordering
    ├── filesystem.rs              # Descriptor-relative staging/publication
    ├── recovery.rs                # Verified private prior-state copies
    └── execution.rs               # Lock-held checkpointed executor

tests/
├── cli_contract.rs                # Existing grammar regressions
├── baseline_integration.rs        # Shared writer coordination regressions
├── state_integration.rs           # State V1/V2 preservation
├── push_cli_contract.rs
├── push_planning.rs
├── push_filesystem_integration.rs
├── push_failure_integration.rs
├── push_recovery_integration.rs
├── push_contention_integration.rs
├── performance_acceptance.rs
└── support/
    └── mod.rs
```

**Structure Decision**: Preserve the single-crate architecture and keep CLI routing thin. Reuse `observation`, `classification`, and State V2 publication as authoritative inputs and outputs. Isolate pure push planning from side-effecting execution; isolate operation history from accepted state; and centralize descriptor-relative payload and recovery operations behind narrow modules rather than adding general filesystem abstractions.

## Complexity Tracking

No constitutional violations require justification.

## Implementation traceability audit

The completed implementation audit found no requirement correction necessary. The implementation and acceptance evidence cover every requirement group without introducing a cache, service, dependency, deletion behavior, reverse propagation, or automatic rollback.

| Requirements | Primary implementation evidence | Primary verification evidence |
|---|---|---|
| FR-001–FR-009 | `src/cli.rs`, `src/push/model.rs`, `src/push/plan.rs`, `src/lib.rs` | `tests/push_cli_contract.rs`, `tests/push_planning.rs` |
| FR-010–FR-020 | `src/push/execution.rs`, `src/push/filesystem.rs`, `src/push/recovery.rs` | `tests/push_filesystem_integration.rs`, `tests/push_failure_integration.rs`, `tests/push_recovery_integration.rs` |
| FR-021–FR-025 | `src/baseline.rs`, `src/state/mutation_lock.rs`, `src/state/publication.rs` | `tests/baseline_integration.rs`, `tests/push_contention_integration.rs`, `tests/state_integration.rs` |
| FR-026–FR-032 | `src/operation/model.rs`, `src/operation/publication.rs`, `src/result.rs` | `tests/push_cli_contract.rs`, `tests/push_recovery_integration.rs`, `tests/performance_acceptance.rs` |
| SC-001–SC-007, SC-009 | Typed push, operation, recovery, baseline, and result paths above | Focused push suites plus existing discovery, classification, registry, state, and baseline regressions |
| SC-008 | Pure deterministic planning and lock-free dry run | Measured 10,000-entry release harness; dry-run p95 948.758083 ms and execute-mode planning p95 833.376917 ms |
