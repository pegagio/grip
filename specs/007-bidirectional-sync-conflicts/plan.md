# Implementation Plan: Bidirectional Synchronization and Conflict Resolution

**Branch**: `007-bidirectional-sync-conflicts` | **Date**: 2026-09-07 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/007-bidirectional-sync-conflicts/spec.md`

## Summary

Add `sync` as one complete deterministic plan containing both push and pull actions, and add exact-entry `resolve` with explicit source/destination winner flags and preview support. Extend the verified direction-neutral mutation core with an operation discriminator, per-action transfer direction, operation-specific planning policies, converged-entry baseline acceptance, and backward-compatible operation-record validation. Reuse the existing descriptor-safe staging, recovery, execution, coordination, result, and State V2 publication boundaries.

## Technical Context

**Language/Version**: Rust 1.98.0, Rust 2024 edition

**Primary Dependencies**: Existing `clap` 4.6, `rustix` 1.1 with `fs` and `process`, `ignore` 0.4.33, Serde/`serde_json`, `sha2` 0.10, `thiserror` 2, and `tempfile` for tests; no new dependency

**Storage**: Preserve State Envelope V2, Recovery Metadata V1, and Partitioned Operation Record V1. Extend the closed operation vocabulary to `sync` and `resolve`, retain old push/pull record validity, and add per-action direction plus optional resolution winner inside new immutable plans without a parallel history tree

**Testing**: Rust unit tests; exhaustive operation/classification policy tables; plan identity and legacy-record tests; isolated sync and resolution CLI, filesystem, recovery, contention, stale-evidence, partial-failure, and output-delivery integration tests; all push/pull regressions; ignored release performance harness

**Target Platform**: macOS and Unix-like systems supported by the existing Rust/rustix build; ordinary regular files and directories under the current supported-state contract

**Project Type**: Single Rust CLI application with reusable domain and descriptor-oriented filesystem modules

**Performance Goals**: Equivalent ordered output across 100 unchanged previews; at least 95 of 100 warm release-mode sync plans over 10,000 selected entries complete within two seconds

**Constraints**: Complete selected-scope preflight; conflict blocks every sync action; source-space-only resolution identity; exactly one explicit winner; preview non-mutation; per-action revalidation; verified losing-side recovery; stop after first execution failure; no partial baseline; accept converged evidence only after complete success; no deletion, retirement, initial-collision adoption, merge, import, rollback, broad payload lock, cache, index, parallel execution, schema fork, or new dependency

**Scale/Scope**: All mappings or one sync selector; exactly one established conflicting identity for resolution; 18 inherited classifications; mixed push/pull actions plus converged no-replacement entries; representative 10,000-entry plan

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **I. Proportional rigor**: The design adds two thin command adapters and extends the existing mutation policy/model. It adds no dependency, service, cache, persistent index, parallel executor, broad lock, inode promise, or snapshot claim.
- **II. Explicit ownership and least surprise**: Sync acts only on selected mapped entries with established classification policy. Unmanaged destination content remains non-actionable. Resolution requires one exact source-space identity and one explicit winner. Deletion and retirement remain blocked.
- **III. Validate, revalidate, and recover**: Both commands derive complete plans, require lock-held plan equivalence, revalidate each action, preserve the losing target, verify publication, and publish one baseline only after complete success. An operation record is initialized before the first payload mutation or, for converged-only sync, before accepted-state mutation. Preview, blocked, and synchronized-only semantic no-op plans mutate nothing.
- **IV. Bounded concurrency**: Read-only planning stays lock-free. Actionful commands reuse the per-user mutation lock and `mutation -> registry -> state` order. External edits are detected through descriptor-oriented revalidation, not payload-tree locks.
- **V. Fast, observable, and testable**: Operation and per-action direction are structured fields. Human, JSON, and diagnostic channels remain separate. Exhaustive policy tests, isolated filesystem tests, injected failures, legacy-record checks, and representative measurements cover the new behavior.
- **Product boundaries**: The feature remains local, offline, per-user, privilege-neutral, and allowlist-based. It adds no automatic merging, deletion, remote access, Git behavior, or unsupported node coercion.
- **Merge-bounded persistence**: The clarified CLI decisions appear in the spec and every Phase 0 decision is reflected across this plan and Phase 1 artifacts. After task generation, `speckit-analyze` gates implementation and accepted discoveries flow back before merge.

Post-design re-check: passed. Separating invoked operation from per-action transfer direction preserves mapping vocabulary and old directional behavior while enabling mixed plans. The storage extension remains backward compatible and no constitutional exception requires complexity tracking.

## Project Structure

### Documentation (this feature)

```text
specs/007-bidirectional-sync-conflicts/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── cli.md
│   ├── sync.md
│   ├── resolution.md
│   ├── filesystem.md
│   └── storage.md
└── tasks.md                  # Created later by speckit-tasks
```

### Source Code (repository root)

```text
src/
├── cli.rs                         # Add SyncArgs, ResolveArgs, WinnerArg, routing
├── lib.rs                         # Thin sync/resolve orchestration
├── result.rs                      # Operation-aware plan/applied/failure projection
├── error.rs                       # Operation-aware stale and mutation failures
├── baseline.rs                    # Accept actioned plus converged selected identities
├── observation/                   # Reuse selectors; exact source-space resolution
├── classification/                # Reuse the 18-category three-way model
├── operation/
│   ├── model.rs                   # Admit sync/resolve and validate old/new plans
│   └── publication.rs             # Initialize records by invoked operation
├── mutation/
│   ├── model.rs                   # Operation, direction, winner, plan/action/result fields
│   ├── plan.rs                    # Push/pull/sync/resolve policy builders
│   ├── execution.rs               # Operation-aware rebuild and per-action execution
│   ├── filesystem.rs              # Reuse origin-to-target descriptor-safe transfer
│   └── recovery.rs                # Reuse verified target preservation
├── push/mod.rs                    # Existing thin adapter
├── pull/mod.rs                    # Existing thin adapter
├── sync/mod.rs                    # Thin bidirectional adapter
├── resolve/mod.rs                 # Thin exact-conflict adapter
└── state/                         # Preserve State V2 and mutation coordination

tests/
├── push_*.rs                       # Existing directional regressions
├── pull_*.rs                       # Existing directional regressions
├── sync_cli_contract.rs
├── sync_planning.rs
├── sync_filesystem_integration.rs
├── sync_failure_integration.rs
├── sync_recovery_integration.rs
├── sync_contention_integration.rs
├── resolve_cli_contract.rs
├── resolve_planning.rs
├── resolve_filesystem_integration.rs
├── operation_record_integration.rs
├── performance_acceptance.rs
└── support/mod.rs                  # Mixed-direction/conflict fixtures and fault helpers
```

**Structure Decision**: Preserve the single-crate architecture and the existing internal `mutation` module. Add one invoked-operation enum above the two-value transfer-direction enum, record direction on every payload action, and use focused sync/resolve planning policies. Keep the executor, filesystem, recovery, and operation publication shared because their invariants already vary by target direction rather than command name.

## Complexity Tracking

No constitutional violations require justification.
