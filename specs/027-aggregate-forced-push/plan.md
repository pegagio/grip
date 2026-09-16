# Implementation Plan: Aggregate Forced Push

**Branch**: `027-aggregate-forced-push` | **Date**: 2026-09-16 | **Spec**: [spec.md](./spec.md)

## Summary

Add the narrowly authorized no-selector form of `grip push --force`. It creates a source-winning aggregate operation for all managed entries in the selected project, fully validates that scope before mutation, verifies and publishes accepted state one completed entry at a time, refreshes retained state after each publication, and stops at the first execution failure without attempting later entries. Existing exact-selector force behavior and all non-push directions remain unchanged.

## Technical Context

**Language/Version**: Rust 1.98, edition 2024
**Primary Dependencies**: `clap`, `serde`, `serde_json`, `thiserror`, `rustix`
**Storage**: Local Grip registry and State V4 accepted-baseline records
**Testing**: `cargo test`, isolated temporary-root integration tests
**Target Platform**: macOS and Unix local filesystems
**Project Type**: Single Rust CLI application
**Performance Goals**: Preserve one complete traversal/inspection per aggregate plan; avoid repeated tree traversal or hashing outside existing revalidation needs
**Constraints**: No daemon, broad locks, new persistence system, safety bypass, implicit deletion, aggregate pull, or aggregate sync
**Scale/Scope**: One selected local project and its managed mappings; deterministic results and machine-readable operation records

## Constitution Check

| Principle | Plan response | Status |
|---|---|---|
| I. Proportional Rigor | Reuse existing inspection, action execution, and State V4 publication; add no coordination or indexing infrastructure. | PASS |
| II. Ownership and Least Surprise | Aggregate selection is explicit only for no-selector source-winning push; complete-state absence is handled only under that explicit authority. | PASS |
| III. Validate, Revalidate, and Verify | Validate the entire scope before mutation, revalidate each action, verify it, then publish only the completed entry's accepted evidence. Stop at first failure. | PASS |
| IV. Bounded Concurrency | Refresh the in-memory state snapshot after each accepted publication; retain narrow existing state coordination and per-action revalidation. | PASS |
| V. Fast, Observable, and Testable | Preserve deterministic output; cover selection, safety, dry run, drift, publication, partial failure, and compatibility with isolated tests. | PASS |
| Merge-Bounded Persistence | Keep specification, plan, later tasks, and implementation consistent; run analysis after tasking and convergence after implementation. | PASS |

The constitution version 3.0.0 explicitly permits the two required exceptions: aggregate managed-scope selection for a feature-authorized forced directional operation and publication for verified entries before a later aggregate failure. No constitutional exception remains.

## Project Structure

### Documentation

```text
specs/027-aggregate-forced-push/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── aggregate-forced-push.md
└── tasks.md                 # Created later by speckit-tasks
```

### Source Code

```text
src/
├── cli.rs                   # Push argument parsing and command presentation
├── lib.rs                   # Direction selection and operation construction
├── mutation/
│   ├── plan.rs              # Aggregate action planning and entry/action association
│   └── execution.rs         # Revalidation, execution, verification, result assembly
├── operation/               # Persisted operation model and result rendering
├── state/
│   └── publication.rs       # State V4 accepted-baseline publication
└── delete/                  # Typed source-absence planning and execution support

tests/
├── push_cli_contract.rs
├── push_planning.rs
├── push_filesystem_integration.rs
├── push_failure_integration.rs
├── operation_record_integration.rs
├── contained_source_tree_integration.rs
└── performance_acceptance.rs
```

**Structure Decision**: Extend the existing Rust CLI and mutation/state layers. The aggregate behavior is a new typed operation policy, not a broader interpretation of ordinary `Resolve`.

## Phase 0: Research Decisions

The decisions and rejected alternatives are recorded in [research.md](./research.md).

## Phase 1: Design and Implementation Approach

1. Add a distinct `AggregateForcePush` operation type and result vocabulary. Route only omitted-selector `push --force` into it; retain the current exact-entry path for supplied selectors.
2. Derive one deterministic complete source-winning entry scope, including divergent and missing-destination states and source absence where the existing complete-state contract supports it. Keep destination-leaf link replacement exact-only and block it in aggregate scope.
3. Perform complete aggregate validation before producing mutable work. Dry run follows the same selection and result path but performs no mutation or state publication.
4. Preserve entry-to-action indexes in the mutation plan. Execute actions in deterministic entry order, revalidate immediately before each action, and verify each destination result.
5. Publish accepted baseline evidence only after every action for one entry verifies. Reload the State V4 snapshot after publication before revalidating later actions; never treat a stale snapshot as the authoritative current baseline.
6. On the first execution, verification, publication, or drift failure, stop before later entries. Return a failed aggregate record with completed publications, the failed entry, and later unattempted identities clearly distinguished.
7. Add contract and integration coverage without weakening existing exact-force, deletion, symlink, or ordinary push behavior.

The data model and external behavior contract are in [data-model.md](./data-model.md) and [contracts/aggregate-forced-push.md](./contracts/aggregate-forced-push.md).

## Validation Strategy

- Unit and planning tests for omitted-selector aggregate selection, deterministic ordering, source-complete classifications, and exact-selector compatibility.
- Filesystem integration tests for divergent, missing-destination, and supported source-absence cases; no-follow safety and preflight blocker non-mutation.
- Dry-run tests that compare the selected scope and prove payload and State V4 non-mutation.
- Failure and drift tests that prove stop-on-first-failure, earlier-entry accepted publication, failed/later unaccepted state, and deterministic operation records.
- Existing performance acceptance coverage to ensure no redundant traversal or hashing is introduced.
- Run the focused test files, `cargo test`, and the repository validation command recorded in [quickstart.md](./quickstart.md).

## Post-Design Constitution Check

All principles remain PASS. The design publishes only verified completed entries, retains a failed aggregate outcome after partial failure, and introduces no expanded ownership, background coordination, or safety bypass.

## Complexity Tracking

No constitutional violations or complexity exceptions are required.
