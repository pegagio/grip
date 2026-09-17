# Implementation Plan: Destination Adoption

**Branch**: `036-destination-adoption` | **Date**: 2026-09-17 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/036-destination-adoption/spec.md`

## Summary

Add an explicit, exact-path adoption mode to `grip pull`: `-a` / `--adopt DESTINATION`. The mode resolves one existing regular destination file under one existing tree mapping, verifies that the paired source path is absent and safely admissible, then copies complete supported state from destination to source and publishes accepted evidence. It deliberately bypasses ambient destination traversal: only the requested file and any necessary source ancestor directory chain participate. `--force` changes only ignore-policy eligibility for this mode; it is not conflict resolution.

## Technical Context

**Language/Version**: Rust, edition 2024 (repository toolchain)

**Primary Dependencies**: Clap CLI parsing, Serde result serialization, tempfile-based filesystem integration tests

**Storage**: Project `.grip/` accepted-state records and per-user Grip registry/state

**Testing**: `cargo test`; focused CLI contract and isolated filesystem integration tests

**Target Platform**: macOS and Unix local filesystems

**Project Type**: Stateful local CLI application

**Performance Goals**: One exact destination inspection plus a bounded ancestor-chain inspection; no destination-tree traversal or new persistent index

**Constraints**: No-follow validation; deterministic plan, pre-action revalidation, staged atomic publication, full managed-state verification; dry run does not mutate; `.gripignore` remains source-defined policy

**Scale/Scope**: One regular file per invocation under an already-declared tree mapping; required ancestor directories only; no directory or bulk adoption

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

| Principle | Plan response | Status |
| --- | --- | --- |
| I. Proportional Rigor | Reuse existing pull planning, mutation, verification, and publication paths; add a narrow explicit adoption inspection rather than a destination index, watcher, or broad traversal. | Pass |
| II. Explicit Ownership and Least Surprise | Require one exact destination selector under a declared tree mapping. Admit only the selected file and required ancestor chain; ordinary discovery remains source-defined. | Pass |
| III. Validate, Revalidate, and Verify | Plan an explicit adoption operation with complete no-follow evidence, before-action revalidation, staged source publication, post-copy verification, then accepted-state publication. | Pass |
| IV. Bounded Concurrency | Revalidate the exact source, destination, mapping/registry, ignore policy, and accepted state; do not lock either tree. | Pass |
| V. Fast, Observable, and Testable | Avoid traversal, preserve deterministic human/JSON output, and cover success, ignore override, failure, dry run, stale evidence, and no-sibling admission in isolated roots. | Pass |
| Merge-Bounded Flow-Back | The source-ancestor clarification requires baseline entries for newly created ancestor directories; the specification was corrected before task generation. | Pass |

No constitutional exception or complexity justification is required.

## Project Structure

### Documentation (this feature)

```text
specs/036-destination-adoption/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── pull-adopt.md
└── tasks.md                 # Created later by speckit-tasks
```

### Source Code (repository root)

```text
src/
├── cli.rs                    # Pull option parsing and CLI contract
├── lib.rs                    # Pull/adopt command orchestration
├── observation/
│   ├── mod.rs                # Normal and exact adoption inspection
│   └── model.rs              # Mapping, identity, selection, and evidence types
├── discovery/
│   └── ignore_policy.rs      # Source-side .gripignore evaluation
├── mutation/
│   ├── model.rs              # Adoption operation, plan, actions, and outcomes
│   ├── plan.rs               # Deterministic adoption plan construction
│   ├── execution.rs          # Revalidation, staging, apply, and verification
│   └── filesystem.rs         # No-follow filesystem evidence and publication
├── baseline.rs               # Accepted-state candidate construction
└── result.rs                 # Human and JSON result rendering

tests/
├── pull_cli_contract.rs      # Parsing, selectors, and rendered outcomes
├── pull_filesystem_integration.rs
├── contained_source_tree_integration.rs
├── gripignore_conformance.rs
└── support/                  # Isolated roots and invocation helpers

README.md
docs/product-definition.md
```

**Structure Decision**: Keep the existing Rust CLI structure. The feature adds an explicit adoption branch next to normal pull orchestration and narrow helpers in existing observation, planning, execution, baseline, and result modules; it introduces no service, framework, or persistent index.

## Delivery Sequence

1. Add the CLI option contract and route `pull --adopt` before normal and forced pull execution. Reject `--source` with adoption and require a path.
2. Resolve exactly one eligible tree-owned destination file without using normal source discovery. Evaluate the effective project-root `.gripignore` policy for the paired source-relative path.
3. Build an adoption-specific deterministic plan for the selected file and only missing source ancestors, preserving existing metadata compatibility, topology, and no-follow checks.
4. Reuse staged source publication and post-action verification, then publish accepted baselines only for the verified adoption set. For a forced ignored entry, compute and validate the precise policy-file placement and complete exemption rule set before rendering the advisory warning.
5. Add isolated integration/contract coverage, then update operator documentation and run the focused suite followed by the full test suite.
