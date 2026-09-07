# Implementation Plan: Authorized Deletion and Retirement

**Branch**: `008-authorized-deletion-retirement` | **Date**: 2026-09-07 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/008-authorized-deletion-retirement/spec.md`

**Note**: This template is filled in by the `$speckit-plan` command; its definition describes the execution workflow.

## Summary

Add separately authorized child-first directional deletion, state-only retirement, and a unified recovery inspection/restore/cleanup interface over Grip's three existing recovery stores. Preserve the verified classification, path, mutation-lock, operation-record, registry-publication, State V2, Result V1, and descriptor-safe filesystem boundaries. Add command-owned planners/executors plus immutable recovery manifests and cleanup tombstones; retain historical evidence byte-for-byte and fail closed when legacy provenance cannot support restoration.

## Technical Context

**Language/Version**: Rust 1.98.0, Rust 2024 edition

**Primary Dependencies**: Existing `clap` 4.6, `rustix` 1.1 with `fs` and `process`, `ignore` 0.4.33, Serde/`serde_json`, `sha2` 0.10, `thiserror` 2, and `tempfile` for tests; no new dependency

**Storage**: Preserve Registry V1, State Envelope V2, Partitioned Operation Record V1, existing Recovery Metadata V1, and all historical recovery bytes. Add immutable integrity-protected Recovery Manifest V1 and Cleanup Tombstone V1 sidecars; no database, central recovery index, or eager migration

**Testing**: Rust unit tests; exhaustive deletion/retirement policy tables; strict reference/manifest/tombstone tests; isolated CLI, filesystem, recovery, contention, stale-evidence, interrupted-operation, restore-fault, partial-failure, authority-restore, cleanup, and output-delivery integration tests; all existing regressions; ignored release performance harness

**Target Platform**: macOS and Unix-like systems supported by the existing Rust/rustix build; ordinary regular files and directories under the current supported-state contract

**Project Type**: Single Rust CLI application with reusable domain and descriptor-oriented filesystem modules

**Performance Goals**: Equivalent ordered output across 100 unchanged previews; at least 95 of 100 warm release-mode deletion previews over 10,000 managed entries and recovery inventories over 10,000 retained entries complete within two seconds

**Constraints**: Explicit delete authority; path or `--all` retirement selection; force limited to differing pending-retirement evidence; complete preflight; child-first non-recursive removal; unmanaged descendants block; per-action revalidation; verified pre-deletion recovery; no partial baseline retirement; exact bound-target restore; legacy provenance fails closed; immutable cleanup tombstones; no automatic cleanup/rollback/reconstruction, arbitrary restore path, payload-tree lock, cache, index, background service, new dependency, or Feature 009 metadata expansion

**Scale/Scope**: One exact or subtree deletion selector; one retirement path or explicit all scope; one exact restore reference; one or more exact cleanup references; existing mapping set; representative 10,000-entry deletion and recovery inventories

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **I. Proportional rigor**: The design adds focused command modules and two small integrity-protected sidecars while reusing existing observation, lock, operation, publication, and result primitives. It introduces no service, cache, central index, database, dependency, parallel executor, broad lock, inode promise, archive format, or snapshot claim.
- **II. Explicit ownership and least surprise**: Delete requires an explicit authoritative absent side and acts only on accepted entries. Any unmanaged directory descendant blocks. Retirement requires a path or `--all`; force authorizes only explicit comparison-history discard. Restore targets only a manifest-bound original location, and cleanup accepts exact private recovery identities.
- **III. Validate, revalidate, and recover**: Every mutation derives a complete deterministic plan, rebuilds it under coordination, checkpoints before side effects, revalidates action evidence, preserves deleted or displaced payloads, verifies outcomes, and publishes accepted state only after complete success. Partial effects and actual authority remain explicit.
- **IV. Bounded concurrency**: Read-only inventory and previews remain lock-free. Actionful workflows reuse `.mutation.lock` and the existing `mutation -> registry -> state` order. Files and directories are accessed descriptor-relatively; external edits are detected rather than prevented by payload-tree locks.
- **V. Fast, observable, and testable**: Typed results preserve human/JSON/diagnostic separation. Deterministic fault hooks and isolated temporary roots cover deletion, retirement, recovery, cleanup, corruption, concurrency, and authority publication. Representative measurement precedes any recovery index or cache.
- **Product boundaries**: The feature remains local, offline, per-user, privilege-neutral, allowlist-based, and limited to current regular-file/directory supported state. It does not add snapshots, automatic rollback, destination-only ownership, Git behavior, remote access, or universal metadata fidelity.
- **Merge-bounded persistence**: All clarified behavior is present in the specification and Phase 0 decisions are reflected across the plan and Phase 1 artifacts. After task generation, `speckit-analyze` gates implementation; accepted discoveries must flow back through this artifact set before merge.

Post-design re-check: passed. Command-owned workflow types prevent recovery and state-only behavior from weakening transfer invariants. Manifest and tombstone sidecars add only the provenance required for safe restore and truthful cleanup; historical artifacts remain immutable. No constitutional exception requires complexity tracking.

## Project Structure

### Documentation (this feature)

```text
specs/008-authorized-deletion-retirement/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── cli.md
│   ├── deletion.md
│   ├── retirement.md
│   ├── recovery.md
│   ├── storage.md
│   └── filesystem.md
└── tasks.md                  # Created later by speckit-tasks
```

### Source Code (repository root)

```text
src/
├── cli.rs                         # Add delete, retire, and recovery grammars
├── lib.rs                         # Thin routing and orchestration
├── result.rs                      # Typed Result V1 projections and rendering
├── error.rs                       # Stable blockers and operational failures
├── classification/               # Reuse 18 categories and expose survivor differences
├── observation/                  # Inspect retained ignored/untracked identities safely
├── delete/
│   ├── mod.rs
│   ├── model.rs                   # Delete authority, plan, action, evidence, result
│   ├── plan.rs                    # Eligibility, subtree blockers, child-first ordering
│   └── execution.rs               # Recovery, descriptor removal, final retirement
├── retire/
│   ├── mod.rs
│   ├── model.rs                   # Retirement request, disposition, plan, result
│   ├── plan.rs                    # Eligibility and force-required policy
│   └── execution.rs               # State-only accepted-record publication
├── recovery/
│   ├── mod.rs
│   ├── model.rs                   # References, manifests, tombstones, public entries
│   ├── inventory.rs               # Read-only adapters over three existing stores
│   ├── restore.rs                 # Payload, registry, and state restore plans/execution
│   └── cleanup.rs                 # Exact-reference byte removal and tombstones
├── mutation/
│   ├── filesystem.rs              # Add descriptor removal and restore primitives
│   └── recovery.rs                # Write manifests for new recovery; preserve V1 reads
├── operation/
│   ├── model.rs                   # Admit and validate four new operation plan kinds
│   └── publication.rs             # Generic typed plan initialization/checkpointing
├── registry/publication.rs        # Exact verified registry restore primitive
└── state/publication.rs           # Baseline retirement and exact state restore primitive

tests/
├── delete_cli_contract.rs
├── delete_planning.rs
├── delete_filesystem_integration.rs
├── delete_recovery_integration.rs
├── delete_failure_integration.rs
├── delete_contention_integration.rs
├── retire_cli_contract.rs
├── retire_integration.rs
├── recovery_cli_contract.rs
├── recovery_storage_integration.rs
├── recovery_filesystem_integration.rs
├── recovery_inventory_integration.rs
├── recovery_restore_integration.rs
├── recovery_authority_restore_integration.rs
├── recovery_cleanup_integration.rs
├── operation_record_integration.rs # Extend operation/plan compatibility coverage
├── performance_acceptance.rs       # Add deletion and recovery workloads
└── support/mod.rs                  # Add retained identity, directory, recovery, fault fixtures
```

**Structure Decision**: Preserve the single-crate architecture. Keep deletion and retirement separate because deletion mutates payload child-first while retirement changes accepted state only. Give public recovery one cohesive module with inventory, restore, and cleanup submodules because all three share typed references and manifest validation. Extend shared filesystem, recovery, operation, registry, and state modules only for invariants already owned there; do not force non-transfer plans into `MutationPlan`.

## Complexity Tracking

No constitutional violations require justification.
