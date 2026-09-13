# Implementation Plan: Source-Authoritative Mapping Addition

**Branch**: `018-source-authoritative-add` | **Date**: 2026-09-12 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/018-source-authoritative-add/spec.md`

## Summary

Keep `grip add` non-mutating for endpoint payloads while treating the destination’s complete supported state as the initial accepted comparison reference for a newly added unequal source-defined member. The existing classifier will then classify the source as the only changed side, so ordinary `status`, `push`, and `sync` offer a push without classifier or planner changes. Extend add-time baseline construction for file and tree mappings, and coordinate descriptor and required initial-state publication with a short-lived, mapping-scoped fence that is created and verified before publication and cleared only after the matching descriptor and State V4 state are verified.

## Technical Context

**Language/Version**: Rust 1.98, edition 2024

**Primary Dependencies**: `clap`, `serde`, `serde_json`, `sha2`, `rustix`, and the Rust standard library

**Storage**: Version-controlled `.grip/config.toml` descriptor plus owner-only local `.grip/state/state.json` State V4 accepted baselines and bounded, owner-only incomplete-add fence state

**Testing**: `cargo test`; `mise run validate` runs formatting, Clippy with warnings denied, the full suite, and the release build

**Target Platform**: Local macOS and Unix filesystems visible to the invoking user

**Project Type**: Rust command-line application

**Performance Goals**: Add inspects only the newly declared mapping and its admitted source-defined tree members; it introduces no background work or durable index.

**Constraints**: Preserve endpoint payloads during add; preserve portable declaration spelling, destination-only ownership, State V4 binding/integrity checks, bounded locks, and existing human/JSON and `diff` contracts outside the new classification.

**Scale/Scope**: One declared file mapping or one source-defined tree mapping per invocation. Tree work is linear in admitted members and uses the existing ignore-policy/discovery path.

## Constitution Check

| Principle | Plan response | Gate |
|---|---|---|
| I. Proportional Rigor | Reuse the existing baseline map, inspection, and atomic single-file publishers; add only a narrowly scoped add-publication coordinator and a short-lived mapping-scoped fence, rather than a daemon, history, or general index. | Pass |
| II. Explicit Ownership and Least Surprise | Baseline only source-defined, non-ignored active members. Destination-only members remain unmanaged, and `add` changes no endpoint payload. | Pass |
| III. Validate, Revalidate, and Verify | Capture endpoint evidence, revalidate before publication, create and verify the mapping-scoped fence, stage/verify descriptor and state against its digests, then clear the fence only after matching verification. Retry either completes the fenced transition or restores the prior descriptor. | Pass |
| IV. Bounded Concurrency | Retain the existing project mutation lock and establish explicit, bounded fence → registry → state → verification → fence-clear ordering for the compound add path. | Pass |
| V. Fast, Observable, and Testable | Preserve concise command output and machine results; cover files, mixed trees, later drift, complete payload snapshots, JSON compatibility, fence creation/clearing, retry completion/restoration, mapping-scoped blocking, and registry/state fault paths with isolated roots. | Pass |

**Post-design check**: Pass. The design uses existing State V4 persistence and narrow publication primitives. The small compound add coordinator and pre-publication mapping-scoped fence are necessary because a descriptor without required source-authoritative state, or a publication transition whose coherence cannot be verified, would violate the feature’s safety contract. The fence is bounded coordination state rather than a durable recovery index.

## Project Structure

### Documentation (this feature)

```text
specs/018-source-authoritative-add/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── add-initial-authority.md
└── tasks.md              # Created by speckit-tasks
```

### Source Code

```text
src/
├── lib.rs                    # CLI command orchestration, including add
├── baseline.rs               # Accepted-baseline candidate construction
├── classification/mod.rs     # Existing three-way classification
├── observation/              # Mapping and tree inspection
├── registry/publication.rs   # Descriptor staging and publication
└── state/
    ├── mod.rs                # State V4 identities and bindings
    ├── publication.rs        # State staging and publication
    └── mutation_lock.rs      # Bounded project mutation coordination

tests/
├── mapping_cli_contract.rs
├── classification_cli_contract.rs
├── classification_matrix.rs
├── state_integration.rs
├── mapping_topology_integration.rs
└── support/
```

**Structure Decision**: Extend the existing add, baseline, registry, and state modules. Keep the public CLI surface and the generic classifier/planner unchanged; use focused integration and contract tests in the established test layout.

## Complexity Tracking

No constitutional violation or additional complexity justification is required.
