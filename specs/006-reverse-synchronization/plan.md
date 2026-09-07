# Implementation Plan: Reverse Synchronization

**Branch**: `006-reverse-synchronization` | **Date**: 2026-09-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/006-reverse-synchronization/spec.md`

**Note**: This plan ends after Phase 1 design; `tasks.md` is generated separately by `$speckit-tasks`.

## Summary

Add `grip pull` by extending the verified Feature 005 mutation pipeline with an explicit direction rather than duplicating its safety logic. Extract direction-neutral plan, action, filesystem, recovery, execution, failure, operation-record, and result projections into a focused `mutation` module; retain thin `push` and `pull` command adapters and compatibility re-exports where existing internal tests use push paths. Pull makes only `destination_only_change` for established accepted identities actionable, treats every destination-only unmanaged item as a reported non-action, requires the source and its ancestry to remain present and safe, stages destination bytes beside the source, preserves the prior source, verifies publication, and publishes State V2 only after complete success.

## Technical Context

**Language/Version**: Rust 1.98.0, Rust 2024 edition

**Primary Dependencies**: Existing `clap` 4.6, `rustix` 1.1 with `fs` and `process`, `ignore` 0.4.33, Serde/`serde_json`, `sha2` 0.10, `thiserror` 2, and `tempfile` for tests; no new dependency

**Storage**: Preserve accepted State Envelope V2 and Partitioned Operation Record V1; extend operation identity from `push` to `push | pull`, reuse the existing action checkpoints and recovery layout, and bind direction through the immutable plan and operation summary without adding a second history tree

**Testing**: Rust unit tests, direction-table planner tests, shared mutation regression tests, isolated pull CLI/filesystem/recovery/contention/failure integration tests, strict operation-record round trips for both directions, deterministic typed fault injection, output-writer failure tests, all existing push regressions, and the ignored release performance harness

**Target Platform**: macOS and Unix-like systems supported by the existing Rust/rustix build; ordinary regular files and directories only

**Project Type**: Single Rust CLI application with reusable domain and descriptor-oriented filesystem modules

**Performance Goals**: Equivalent ordered output across 100 unchanged dry runs; at least 95 of 100 warm release-mode pull plans over 10,000 eligible managed entries complete within two seconds

**Constraints**: Complete preflight before mutation; dry-run non-mutation; only accepted `destination_only_change` identities are actionable; unmanaged destination items remain reported non-actions; missing sources and parent ancestry block rather than create; mutation lock acquired only for an actionful unblocked execution; no-follow descriptor access; same-parent staging at the source; verified source recovery before replacement; stop after first failure; no partial baseline; shared versioned result details with `direction: pull`; no deletion, import, bidirectional execution, conflict resolution, automatic rollback, broad lock, cache, index, parallel execution, or new dependency

**Scale/Scope**: All mappings or one source/destination selector; one complete plan over every selected managed or observed unmanaged entry; 18 inherited classifications; only destination-only changes to established managed entries are actionable; exactly 10,000 eligible managed entries in representative performance acceptance

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **I. Proportional rigor**: The design generalizes the proven push pipeline once and retains command-specific adapters. It adds no dependency, service, cache, persistent index, parallel runtime, broad lock, inode identity promise, or snapshot claim. Direction-neutral types remove duplicated safety behavior that would otherwise diverge across pull and later sync.
- **II. Explicit ownership and least surprise**: Pull actions require accepted managed identity and `destination_only_change`. Destination-only unmanaged items remain visible non-actions, missing sources are deletion classifications rather than creation opportunities, and unsupported or conflicting evidence blocks before mutation.
- **III. Validate, revalidate, and recover**: Pull uses complete planning, lock-held plan equivalence, per-action evidence revalidation, source-side sibling staging, verified preservation of the prior source, post-publication verification, and one accepted baseline publication after complete success. Partial outcomes retain the prior baseline and are never rolled back automatically.
- **IV. Bounded concurrency**: Read-only preflight stays lock-free. Actionful execution reuses the short-lived per-user mutation lock and global `mutation → registry → state` order. Descriptor-relative revalidation detects external edits without locking mapped trees.
- **V. Fast, observable, and testable**: One shared typed result schema carries `direction`; human output, JSON, and diagnostics remain separate. Direction-table planner tests, shared push regressions, pull-specific integration tests, typed fault seams, and the 10,000-entry harness cover behavior without duplicating traversal or serialization.
- **Product boundaries**: The feature stays local, offline, per-user, allowlist-based, and privilege-neutral. It neither imports unmanaged destination content nor adds deletion, Git behavior, remote access, or automatic recovery.
- **Merge-bounded persistence**: Clarified behavior is present in the specification and all Phase 0 choices are represented in this plan and its Phase 1 artifacts. After task generation, `speckit-analyze` gates implementation; accepted discoveries flow back before merge, and `speckit-converge` closes remaining gaps.

Post-design re-check: passed. The design keeps mapping roles (`source_path`, `destination_path`) distinct from transfer roles (`origin`, `target`), makes direction explicit in plans and results, and preserves the verified state/recovery boundaries. No exception requires complexity tracking.

## Project Structure

### Documentation (this feature)

```text
specs/006-reverse-synchronization/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── cli.md
│   ├── pull.md
│   ├── filesystem.md
│   └── storage.md
└── tasks.md                  # Created later by speckit-tasks
```

### Source Code (repository root)

```text
src/
├── cli.rs                         # Add PullArgs and pull command routing
├── lib.rs                         # Thin push/pull orchestration
├── result.rs                      # Shared versioned mutation projections and rendering
├── error.rs                       # Direction-neutral mutation failure evidence
├── baseline.rs                    # Reuse accepted-state construction for actioned identities
├── observation/                   # Reuse selectors and complete two-pass evidence
├── classification/                # Reuse the 18-category three-way model
├── operation/
│   ├── model.rs                   # Admit push and pull operation identity
│   └── publication.rs             # Initialize either direction from shared plans
├── mutation/
│   ├── mod.rs                     # Direction and shared fault boundaries
│   ├── model.rs                   # Shared plan, action, disposition, result types
│   ├── plan.rs                    # Direction-table planning and canonical order
│   ├── filesystem.rs              # Origin-to-target staging, publication, verification
│   ├── recovery.rs                # Verified preservation of the replaced target
│   └── execution.rs               # Lock-held direction-neutral executor
├── push/
│   └── mod.rs                     # Thin push adapter and compatibility re-exports
├── pull/
│   └── mod.rs                     # Thin pull adapter
├── registry/publication.rs        # Preserve registry evidence and lock order
└── state/                          # Preserve State V2 and mutation coordination

tests/
├── push_*.rs                       # Existing regressions preserved through shared core
├── pull_cli_contract.rs
├── pull_planning.rs
├── pull_filesystem_integration.rs
├── pull_failure_integration.rs
├── pull_recovery_integration.rs
├── pull_contention_integration.rs
├── operation_record_integration.rs # Both direction identities and strict decoding
├── performance_acceptance.rs       # Add 10,000-entry pull measurements
└── support/mod.rs                  # Direction-neutral fixtures and fault helpers
```

**Structure Decision**: Preserve the single-crate architecture. Extract the already-coupled push model/executor/filesystem/recovery code into one internal `mutation` module because pull needs the same invariants with origin and target reversed. Keep `push` and `pull` as thin directional policies so command meaning remains explicit, preserve mapping-role field names in public records, and avoid a premature generalized sync scheduler or plugin abstraction.

## Complexity Tracking

No constitutional violations require justification.
