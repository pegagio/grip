# Implementation Plan: Initial-Match Baseline Synchronization

**Branch**: `037-initial-match-sync-baseline` | **Date**: 2026-09-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from [spec.md](spec.md)

## Summary

Permit an exact selected ordinary sync to accept one complete equivalent managed entry that lacks a baseline, including an exact child under a tree mapping. Gate initial-match acceptance on one resolved managed entry so no-selector, mapping-root, and multi-entry subtree sync retain their existing behavior. Reuse the existing acceptance-only plan/execution path and accepted-state publication; no payload action, new command, state schema, or selector behavior is introduced.

## Technical Context

**Language/Version**: Rust edition 2024; MSRV Rust 1.98.

**Primary Dependencies**: Existing `clap`, `serde`, `serde_json`, `sha2`, `rustix`, and standard library; no additions.

**Storage**: Existing project mapping descriptor and machine-owned accepted state generation; no schema changes.

**Testing**: `cargo test`, focused CLI/filesystem integration tests, `cargo fmt --check`, Clippy with warnings denied, and the repository validation task.

**Target Platform**: Existing macOS and Unix filesystem contract.

**Project Type**: Local Rust CLI.

**Performance Goals**: Preserve one selected-scope inspection and existing acceptance-only execution behavior; no additional traversal, caching, or background work.

**Constraints**: Exactly one resolved source-path managed entry; complete equivalent supported evidence only; dry runs remain state- and payload-non-mutating; no public baseline command, broad-tree acceptance, or force expansion.

**Scale/Scope**: One planner classification-to-disposition correction, its contract coverage, and user documentation.

## Constitution Check

*GATE: Passed before Phase 0 research and after Phase 1 design.*

- **I. Proportional rigor**: Pass. The change reuses the existing acceptance-only execution path and adds no abstraction, lock, cache, or service.
- **II. Explicit ownership**: Pass. Selection remains limited to an already active source-defined managed entry; destination-only and ineligible entries remain excluded.
- **III. Validate, revalidate, and verify**: Pass. The existing mutation lock, plan rebuild, final observation, candidate construction, and state publication apply before an accepted baseline is published.
- **IV. Bounded concurrency**: Pass. Existing short-lived Grip-owned mutation and state locks remain unchanged.
- **V. Fast, observable, and testable**: Pass. Add isolated tree-child, preview, safety, and result regression coverage; no extra scans beyond current sync.
- **Merge-bounded persistence**: Pass. The specification, plan, tasks, implementation, tests, documentation, and roadmap status will move together; analysis gates implementation and convergence closes the feature.

## Project Structure

### Documentation (this feature)

```text
specs/037-initial-match-sync-baseline/
├── plan.md              # This file ($speckit-plan command output)
├── research.md          # Phase 0 output ($speckit-plan command)
├── data-model.md        # Phase 1 output ($speckit-plan command)
├── quickstart.md        # Phase 1 output ($speckit-plan command)
├── contracts/           # Phase 1 output ($speckit-plan command)
└── tasks.md             # Phase 2 output ($speckit-tasks command - NOT created by $speckit-plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
src/
├── lib.rs                 # Selected sync orchestration
├── mutation/
│   ├── plan.rs            # Classification-to-disposition policy
│   └── execution.rs       # Existing acceptance-only execution and publication
└── result.rs              # Existing baseline-only result rendering

tests/
├── sync_cli_contract.rs
└── sync_filesystem_integration.rs
```

**Structure Decision**: Retain the single Rust CLI structure. The behavior belongs in the existing sync planning policy; execution, serialization, locking, and result rendering already support acceptance-only plans.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
