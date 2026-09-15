# Implementation Plan: Forced Missing-Peer Restoration

**Branch**: `023-force-missing-peer-restoration` | **Date**: 2026-09-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/023-force-missing-peer-restoration/spec.md`

**Note**: This template is filled in by the `$speckit-plan` command; its definition describes the execution workflow.

## Summary

Repair exact forced synchronization when the selected winner exists and its managed peer is missing. Extend the existing exact conflict-resolution path to plan a verified add from the present source to a missing destination or from the present destination to a missing source. Preserve the separate deletion path for an explicitly selected absent winner, exact-entry selection, and all existing mutation revalidation and baseline publication boundaries.

## Technical Context

**Language/Version**: Rust 1.85+, 2024 edition

**Primary Dependencies**: Existing Clap, Serde, SHA-256, Rustix, and Tempfile dependencies only

**Storage**: Existing project-local Descriptor V2 and State V4 accepted baseline evidence; no schema change

**Testing**: `cargo test`, focused CLI, planning, and filesystem integration tests, then `mise run validate`

**Target Platform**: macOS and supported Unix-like filesystems

**Project Type**: Single Rust CLI application

**Performance Goals**: No additional tree traversal, hashing, storage, or background work beyond the existing exact-entry force operation

**Constraints**: Exact selected entry only; offline; no new recovery/history, locks, cache, index, or persistent state; preserve source/destination ownership and revalidation

**Scale/Scope**: Four direction-compatible present-winner, missing-peer classifications and their dry-run, file/tree-entry, baseline, and human-output regressions

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **I. Proportional rigor**: Reuse the existing exact force planner and publication pipeline. No new service, storage, framework, cache, or concurrency mechanism is required.
- **II. Explicit ownership and least surprise**: Restore only the one established identity selected by existing force validation. Keep all broad, ambiguous, unmanaged, unsupported, and ignored selections rejected.
- **III. Validate, revalidate, and verify**: Keep complete discovery, exact selection, pre-action revalidation, staged publication, post-action verification, and fresh baseline publication. Preserve absent-winner deletion as a separate explicit path.
- **IV. Bounded concurrency**: Reuse the existing writer lock and action-level evidence checks; do not expand locking scope.
- **V. Fast, observable, and testable**: Add isolated regression coverage for both directions, dry runs, unaffected siblings, absent-winner deletion, and rejection boundaries. Preserve human, JSON, and diagnostics separation.
- **Product boundaries**: No automatic conflict winner, aggregate force, mapping mutation, history, recovery, or remote behavior.

Post-design re-check: passed. The design corrects the existing exact directional contract using the established mutation model and has no constitutional exception.

## Project Structure

### Documentation (this feature)

```text
specs/023-force-missing-peer-restoration/
├── plan.md
├── research.md
├── data-model.md
├── contracts/force-restoration.md
├── quickstart.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── lib.rs                         # Exact force dispatch and absent-winner deletion split
├── mutation/plan.rs               # Exact present-winner restoration disposition and actions
└── mutation/execution.rs          # Resolution revalidation for present-winner restoration

tests/
├── push_filesystem_integration.rs # Present source restores missing destination
├── pull_filesystem_integration.rs # Present destination restores missing source
├── push_planning.rs               # Exact and broad force planning boundaries
├── pull_planning.rs               # Exact and broad force planning boundaries
└── classification_cli_contract.rs # Accurate executable human guidance
```

**Structure Decision**: Keep the existing single-crate layering. Force dispatch remains the only orchestration point that distinguishes absent-winner deletion from present-winner restoration; the mutation planner and executor remain the reusable verified-copy path.
