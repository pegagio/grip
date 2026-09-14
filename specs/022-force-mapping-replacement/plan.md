# Implementation Plan: Force Mapping Replacement

**Branch**: `022-force-mapping-replacement` | **Date**: 2026-09-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/022-force-mapping-replacement/spec.md`

## Summary

Add `grip add --force SOURCE DESTINATION` as a deliberately narrow mapping replacement. The command will remove only one distinct active file mapping with the same canonical destination, construct and completely validate the full replacement registry, and retain every other ownership rejection. It will publish the candidate descriptor and State V4 evidence as one recoverable fenced transition, without changing either endpoint payload. The same transition discipline will make successful exact removal leave no stale declaration or accepted baseline that prevents an ordinary later add.

## Technical Context

**Language/Version**: Rust 1.98, edition 2024

**Primary Dependencies**: clap derive for CLI parsing; serde/serde_json/toml for persisted and machine-readable data; sha2 for publication evidence; thiserror for errors

**Storage**: Project descriptor at `.grip/config.toml`; Grip-owned State V4 and bounded publication fence beneath the project state directory

**Testing**: `cargo test`, focused integration suites under `tests/`, `mise run validate`

**Target Platform**: Local macOS and Unix CLI environments

**Project Type**: Stateful local CLI application

**Performance Goals**: Preserve the existing one-pass add-time inspection model; do not add caches, background work, broad locks, or extra endpoint traversal beyond the candidate validation and existing revalidation required for a mutation

**Constraints**: Force is an explicit exact file-mapping replacement only; all ownership/topology validation remains authoritative; add never mutates endpoint payloads; descriptor and State V4 transitions must be recoverable and externally revalidated

**Scale/Scope**: One CLI option, one descriptor/state transition path, deterministic human/JSON outcomes, and isolated filesystem regression coverage; no daemon, service, history, or recovery UI

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

| Principle | Plan response | Status |
|---|---|---|
| I. Proportional Rigor for a Local Tool | Reuses the existing short-lived mutation lock and publication fence; no new cache, service, broad lock, or persistence system. | PASS |
| II. Explicit Ownership and Least Surprise | `--force` selects exactly one equal-destination file declaration and rebuilds the full validated registry; it does not copy payloads or relax other topology rules. | PASS |
| III. Validate, Revalidate, and Verify | Candidate descriptor and State V4 are built before publication, endpoint/project evidence is revalidated, and a persisted fence permits completion or restoration after an interrupted transition. | PASS |
| IV. Bounded Concurrency | Coordination remains limited to Grip-owned metadata, with existing durable evidence checks immediately before publication. | PASS |
| V. Fast, Observable, and Testable | Uses the existing scoped observation path and adds deterministic human/JSON outcomes plus isolated filesystem tests for success, rejection, drift, and non-mutation. | PASS |
| Product boundaries | Does not introduce a daemon, elevated privileges, remote transport, automatic merge, or user-facing history/recovery interface. | PASS |

**Post-design result**: No constitutional exception or complexity justification is required.

## Project Structure

### Documentation (this feature)

```text
specs/022-force-mapping-replacement/
├── plan.md              # This file ($speckit-plan command output)
├── research.md          # Phase 0 output ($speckit-plan command)
├── data-model.md        # Phase 1 output ($speckit-plan command)
├── quickstart.md        # Phase 1 output ($speckit-plan command)
├── contracts/           # Phase 1 output ($speckit-plan command)
└── tasks.md             # Phase 2 output ($speckit-tasks command - NOT created by $speckit-plan)
```

### Source Code (repository root)
```text
src/
├── cli.rs                         # Add option parsing
├── lib.rs                         # Add/remove transitions and fence recovery
├── mapping.rs                     # Complete ownership validation
├── registry/
│   ├── mod.rs                     # Descriptor construction and resolution
│   └── publication.rs             # Validated descriptor publication
├── result.rs                      # Human and JSON mapping results
└── state/
    ├── add_fence.rs               # Generalized descriptor/state transition fence
    ├── mod.rs                     # State V4 descriptor binding
    ├── mutation_lock.rs           # Short-lived metadata coordination
    └── publication.rs             # State candidate publication

tests/
├── mapping_cli_contract.rs        # CLI, output, state, drift, and payload fixtures
├── mapping_topology_integration.rs # Ownership and topology rejection matrix
├── mapping_registry_integration.rs # Registry ownership coverage
└── state_integration.rs           # State V4 binding and publication coverage
```

**Structure Decision**: Keep the established single Rust CLI layout. The feature changes the existing add/remove, state-publication, and result seams rather than introducing a new service or abstraction layer.

## Complexity Tracking

No constitutional violations require tracking.
