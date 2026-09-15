# Implementation Plan: Contained-Source Tree Mappings

**Branch**: `specs/025-contained-source-tree-mappings` | **Date**: 2026-09-15 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/025-contained-source-tree-mappings/spec.md`

## Summary

Permit exactly one currently rejected root shape: a tree source that is a strict descendant of its destination. Safety moves from an unconditional root-level rejection to a two-stage model. Root validation admits only that narrow shape, then source policy and retained accepted state define a deterministic managed-identity set whose members are checked against the containment-relative path. Destination inspection is refactored from recursive enumeration to exact, no-follow probing of only those managed identities and their required ancestors. Existing baseline, classification, rebinding, drift, locking, force, deletion, and publication behavior remains authoritative.

## Technical Context

**Language/Version**: Rust 1.98, edition 2024

**Primary Dependencies**: Existing `clap`, `home`, `ignore`, `libc`, `rustix`, `serde`, `serde_json`, `sha2`, `thiserror`, and `toml` dependencies; no new dependency is planned

**Storage**: Portable project descriptor V2 and machine-owned accepted state V4 on the local filesystem; both schemas remain unchanged because containment and member-topology evidence is derived at runtime

**Testing**: Rust unit and integration tests via `cargo test`; formatting and linting via `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`; release performance acceptance via the existing ignored harness

**Target Platform**: Existing macOS and Unix local-filesystem contract, with the representative performance gate measured on supported macOS ARM64 release builds

**Project Type**: Single Rust command-line application and library

**Performance Goals**: With at least 100 managed members and 10,000 unrelated destination entries, at least 95 of 100 warm read-only status runs complete within one second; unrelated entries produce no records and require no reads

**Constraints**: Never follow links; never enumerate arbitrary never-managed destination content; preserve accepted source-deletion evidence and ignored retirement; block unsafe members before descriptor publication or payload mutation; retain complete-registry overlap validation and exact-entry force authority; add no cache, watcher, daemon, persistent index, broad lock, or parallel traversal

**Scale/Scope**: One local user's mappings, current source membership, and retained accepted identities; focused changes across mapping validation, discovery, observation, rebinding, classification/result presentation, mutation revalidation, deletion revalidation, documentation, and isolated filesystem tests

## Constitution Check

*GATE: Passed before Phase 0 research and passed again after Phase 1 design.*

| Principle or gate | Result | Design evidence |
|---|---|---|
| I. Proportional Rigor for a Local Tool | PASS | Reuses accepted state as the retained identity index and existing descriptor-relative filesystem primitives. No new persistence, watcher, cache, framework, broad lock, or concurrency mechanism is introduced. |
| II. Explicit Ownership and Least Surprise | PASS | Destination scope is the union of current non-ignored source members and retained accepted identities. Never-managed destination content is neither inventoried nor mutated, and unsafe members fail closed. |
| III. Validate, Revalidate, and Verify | PASS | Addition validates before descriptor or baseline publication. Mutations preserve plan construction, under-lock reinspection, per-action evidence checks, verification, and baseline publication only after success. |
| IV. Bounded Concurrency | PASS | Exact target and ancestor evidence replaces destination-tree snapshots. Existing two-pass discovery and short-lived Grip-owned locks are retained, while selected mappings are revalidated before their first payload action. |
| V. Fast, Observable, and Testable | PASS | Work is proportional to managed identity count rather than destination-tree size. Typed deterministic blockers, isolated filesystem tests, regression suites, and a 100-managed/10,000-unrelated release benchmark provide evidence. |
| Product boundaries and engineering constraints | PASS | The design remains a local, unprivileged CLI and preserves the portable-intent/machine-state split, allowlisted payload types, CLI/domain separation, and no-follow behavior. |
| Merge-bounded persistence | PASS | Feature 025 flows forward from Features 002 and 003 without rewriting merged history. Its spec, plan, design artifacts, future tasks, and implementation remain one mutable pre-merge change set. |

No constitutional exception or complexity waiver is required.

## Project Structure

### Documentation (this feature)

```text
specs/025-contained-source-tree-mappings/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── contained-source-mappings.md
├── checklists/
│   └── requirements.md
└── tasks.md                         # Created later by $speckit-tasks
```

### Source Code (repository root)

```text
src/
├── baseline.rs                      # Candidate-add acceptance and publication gate
├── classification/                  # Managed-record classification and blockers
├── delete/                          # Delete planning and pre-action revalidation
├── discovery/
│   ├── filesystem.rs                # Descriptor-relative exact probes and evidence
│   ├── ignore_policy.rs             # Source membership and retained-ignore coverage
│   ├── mod.rs                       # Source walk, managed identity set, destination probes
│   └── model.rs                     # Discovery records and typed topology detail
├── mapping.rs                       # Root relation and member relation helpers
├── mutation/                        # Push, pull, sync, force, and revalidation
├── observation/
│   ├── fingerprint.rs               # Reusable relative no-follow inspection
│   ├── mod.rs                       # Accepted-state join and selected-scope observation
│   └── model.rs                     # Observation evidence
├── registry/                        # Complete-registry validation
├── state/
│   ├── mod.rs                       # State V4 and binding digest, unchanged schema
│   └── rebinding.rs                 # Retained-member topology assessment
├── lib.rs                           # Add orchestration and command-domain entry points
└── result.rs                        # Deterministic human and JSON diagnostics

tests/
├── contained_source_tree_integration.rs
├── mapping_topology_integration.rs
├── mapping_cli_contract.rs
├── gripignore_conformance.rs
├── project_state_rebinding_integration.rs
├── apfs_name_compatibility_integration.rs
├── performance_acceptance.rs
└── push, pull, sync, delete, force, and publication regression suites

docs/
├── product-definition.md
└── user-facing command and mapping guidance
```

**Structure Decision**: Keep the existing single-crate architecture. Place component-aware topology rules in `mapping`, managed-set construction and exact destination evidence in `discovery`, accepted-state reconciliation in `observation`, and operation-specific stale-plan enforcement in the existing mutation and deletion executors. This preserves current responsibility boundaries and avoids a parallel synchronization path for contained mappings.

## Phase 0: Research Decisions

Research resolved the design questions without remaining clarification markers. The full rationale and rejected alternatives are recorded in [research.md](research.md).

1. Split static root validation from dynamic managed-member validation.
2. Derive a raw-relative-path managed identity set from current source policy plus retained active baselines.
3. Probe only exact managed destinations and required ancestors through no-follow directory descriptors.
4. Preserve ignored accepted identities as retirement evidence without opening ignored subtrees.
5. Derive containment evidence on each machine and keep descriptor V2 and state V4 unchanged.
6. Extend existing deterministic blocker output additively with `recursive_member_topology` and a typed relation.
7. Revalidate the entire selected mapping before its first mutation so an exact selector cannot bypass an unsafe sibling.
8. Extend the existing release performance harness rather than adding an optimization subsystem.

## Phase 1: Design

The domain model is defined in [data-model.md](data-model.md), the externally observable behavior in [contracts/contained-source-mappings.md](contracts/contained-source-mappings.md), and executable operator scenarios in [quickstart.md](quickstart.md).

### Implementation sequence

1. Add component-aware helpers that distinguish allowed contained tree roots from prohibited root shapes and classify a member relative to the containment path.
2. Introduce the ephemeral managed identity set and retained-ignore coverage, threading accepted active identities into discovery without changing persisted schemas.
3. Replace recursive destination walking with canonical-order exact probes that cache safe opened ancestors and return absent, supported leaf, or blocking ancestor/leaf outcomes.
4. Feed the managed set into APFS/name-compatibility assessment, observation, and classification while preserving retained deletions and ignored retirement.
5. Add member-topology blockers to add, status/diff planning, rebinding, mutation, force, and deletion paths; ensure selected-map revalidation occurs before the first payload action and again at existing action boundaries.
6. Update human/JSON diagnostics and user documentation without changing commands, selectors, or envelope versions.
7. Add the contained-source, topology matrix, bounded-discovery, ignore, rebinding, drift, and performance tests; update only assertions superseded by the new destination-scope contract.

### Validation strategy

- Unit-test root and member relation matrices using path components, including prefix-like siblings.
- Exercise `home/` to `~/` in isolated temporary homes across add, status, diff, push, pull, sync, exact selection, force, and delete behavior.
- Demonstrate that 10,000 unrelated destination entries, including unreadable and unsupported subtrees, produce no managed records or stale evidence.
- Preserve exact observation of retained source-absent identities and ignored accepted identities.
- Inject a recursive member or topology drift between planning, locking, and action and assert zero payload mutation and zero baseline publication.
- Run the existing ownership, mapping, ignore, APFS, synchronization, deletion, force, state, filesystem-safety, and publication regression suites.
- Measure 100 warm release-mode status samples and assert the specified p95 threshold.

## Post-Design Constitution Check

The Phase 1 design still passes every pre-research gate. The managed identity set is ephemeral and based on existing authority, exact probing narrows rather than expands filesystem access, mutation safety uses the existing revalidation/publication pipeline, and the test plan measures both correctness and the representative performance claim. No new complexity-tracking entry is needed.

## Complexity Tracking

No constitution violations require justification.
