# Implementation Plan: Exact Destination Symlink Replacement

**Branch**: `026-exact-destination-symlink-replacement` | **Date**: 2026-09-15 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/026-exact-destination-symlink-replacement/spec.md`

## Summary

Permit one narrowly typed observation outcome: a symbolic link at an exact managed destination leaf. `add` records the mapping and leaves that leaf unresolved without touching its object or target. Only a source-winning, exact-entry forced push may replace the revalidated link object. Files use the existing staged-file atomic rename; directories use a verified private sibling directory and atomic rename, then existing child and metadata actions. No target data is read, no persisted schema changes, and all other link placements remain blocking.

## Technical Context

**Language/Version**: Rust 1.98, edition 2024

**Primary Dependencies**: Existing `clap`, `libc`, `rustix`, `serde`, `serde_json`, `sha2`, `thiserror`, and filesystem modules; no new dependency is planned

**Storage**: Portable project descriptor V2 and machine-owned accepted state V4; the unresolved destination-link evidence is operation-scoped and is not persisted or treated as baseline state

**Testing**: Rust unit and integration tests via `cargo test`; `cargo fmt --check`; `cargo clippy --all-targets --all-features -- -D warnings`; release build; existing ignored release performance harness

**Target Platform**: Existing macOS and Unix local-filesystem contract; representative performance qualification on macOS ARM64 release builds

**Project Type**: Single Rust command-line application and library

**Performance Goals**: At least 95 of 100 warm status and dry-run samples over 100 managed identities plus 10,000 unrelated destination entries complete within one second without reading or emitting unrelated entries

**Constraints**: Preserve `add` payload non-mutation; use no-follow descriptor-relative inspection; never read or modify link targets; require an exact source-space selector and source-winning force; retain whole-mapping safety checks, under-lock rebuild, per-action revalidation, post-action verification, and State V4 publication; introduce no cache, watcher, daemon, index, broad lock, or parallel traversal

**Scale/Scope**: Exact destination leaves of active file and tree mappings whose current source is a supported regular file or directory; no source links, destination-link ancestors, destination-winning replacement, ordinary-operation bypass, aggregate force, or broad destination traversal

## Constitution Check

*GATE: Passed before Phase 0 research and passed again after Phase 1 design.*

| Principle or gate | Result | Design evidence |
|---|---|---|
| I. Proportional Rigor for a Local Tool | PASS | Extends existing exact probes, staged publication, and state publication. The ephemeral leaf evidence avoids a schema, cache, recovery mechanism, or new subsystem. |
| II. Explicit Ownership and Least Surprise | PASS | Only an exact managed destination leaf may enter the unresolved state. `add` changes no payload; the only mutation requires explicit source-winning force for one selected entry. |
| III. Validate, Revalidate, and Verify | PASS | The plan captures no-follow leaf identity, rebuilds under the existing mutation lock, revalidates it immediately before rename, verifies the replacement, and publishes accepted state only after success. |
| IV. Bounded Concurrency | PASS | Existing short-lived Grip-owned lock, plan comparison, and action revalidation remain in use. Link-object evidence replaces no target observation or broad locking. |
| V. Fast, Observable, and Testable | PASS | Exact managed probes and cached safe ancestor descriptors remain bounded by managed identities. Typed diagnostics, isolated temporary-root tests, drift injection, and the existing performance gate provide evidence. |
| Product boundaries and engineering constraints | PASS | The feature remains a local unprivileged CLI, preserves the allowlisted payload boundary, keeps CLI/result presentation separate from domain and filesystem behavior, and does not create history or recovery state. |
| Merge-bounded persistence | PASS | Feature 026 flows forward from Features 009, 011, 019, 023, and 025 without changing their merged artifacts. Its spec, plan, design artifacts, tasks, and implementation remain one mutable change set. |

No constitutional exception or complexity waiver is required.

## Project Structure

### Documentation (this feature)

```text
specs/026-exact-destination-symlink-replacement/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── destination-link-replacement.md
├── roadmap-reviews/
│   └── brief-20260916T000642Z.md
├── checklists/
│   └── requirements.md
└── tasks.md                         # Created later by $speckit-tasks
```

### Source Code (repository root)

```text
src/
├── baseline.rs                      # Add and actioned-entry accepted-state candidates
├── classification/                  # Unresolved destination-link classification and output record
├── discovery/
│   └── filesystem.rs                # No-follow child metadata and link-object identity
├── observation/
│   ├── fingerprint.rs               # Exact probe outcome including a final destination link
│   ├── mod.rs                       # Runtime unresolved-link observation and bounded tree probes
│   └── model.rs                     # Ephemeral link evidence
├── mutation/
│   ├── execution.rs                 # Under-lock revalidation and replacement execution
│   ├── filesystem.rs                # Staged file and directory publication through an opened parent
│   ├── model.rs                     # Planned link-replacement action evidence
│   └── plan.rs                      # Exact source-winner resolution disposition and action choice
├── path_policy.rs                   # Add-time final-leaf link admission without following ancestors
├── lib.rs                           # Add orchestration, exact-force selection, and guidance routing
└── result.rs                        # Deterministic human and JSON unresolved-link guidance

tests/
├── contained_source_tree_integration.rs
├── filesystem_boundary_integration.rs
├── force_resolution_guidance_contract.rs
├── push_filesystem_integration.rs
├── sync_filesystem_integration.rs
└── performance_acceptance.rs

docs/
├── product-definition.md
└── README.md
```

**Structure Decision**: Keep the single-crate architecture and current responsibility boundaries. Observation owns no-follow link evidence; classification and result rendering expose the unresolved condition; the pure mutation planner grants the narrow forced disposition; filesystem primitives publish a replacement through an already validated parent; execution owns lock-held revalidation, verification, and accepted-state publication.

## Phase 0: Research Decisions

Research resolved every planning question; there are no remaining clarification markers. Details and rejected alternatives are in [research.md](research.md).

1. Reuse Feature 025's bounded managed-identity probe and Feature 009's no-follow child metadata so unrelated destination siblings and every link target remain outside evidence.
2. Model an exact destination-leaf link as ephemeral `DestinationLeafLinkEvidence`, not as a supported payload, baseline, descriptor field, or persisted state.
3. Admit destination-link mappings only when an existing supported source supplies the file or tree kind; a destination-only link remains ambiguous and rejected without a new declaration syntax.
4. Permit only `Resolve + Source winner + exact destination leaf link`; keep ordinary operations and every other force shape blocked.
5. Reuse staged-file rename for files. For directories, create and verify an empty private sibling directory and atomically rename it over the checked link before existing descendant actions and directory metadata finalization.
6. Compare the reobserved leaf object identity immediately before rename; target changes are intentionally irrelevant because Grip never follows or reads the target.
7. Preserve Feature 025's hard ancestor-link blocker and complete selected-mapping revalidation; the leaf exception never bypasses topology, ownership, or metadata blockers.
8. Extend existing integration and performance suites instead of adding test infrastructure or optimization machinery.

## Phase 1: Design

The runtime data and state transitions are defined in [data-model.md](data-model.md), externally observable behavior in [contracts/destination-link-replacement.md](contracts/destination-link-replacement.md), and operator validation scenarios in [quickstart.md](quickstart.md).

### Implementation sequence

1. Introduce an ephemeral exact destination-leaf link observation containing only no-follow object identity and safe parent ancestry. Distinguish it from unsupported source links and ancestor links.
2. Adjust add-time endpoint and prospective-tree inspection to admit only that leaf outcome, retain no baseline for it, preserve fencing and all-or-nothing descriptor/state publication, and replace the generic baseline rejection with path-specific diagnostics.
3. Add the explicit unresolved-link classification and result rendering. Status and dry-run show its entry and only a source-winning force command when the displayed source selector resolves to exactly one managed identity.
4. Extend exact-force selection to accept one active, unbaselined source identity only after post-observation confirms exactly one unresolved destination leaf; retain the existing rejection for mapping, subtree, aggregate, and destination-space ambiguity.
5. Add planned file and directory link-replacement actions that carry expected link evidence. Do not relax generic `UnsupportedManaged` or `UnsafeCollision` planning.
6. Rebuild the complete plan under the mutation lock; revalidate registry, State V4, selected source, managed-mapping invariants, safe ancestor chain, and exact leaf object before staging and again immediately before rename.
7. Publish a staged regular file or staged empty directory by atomic sibling rename over the verified link object. Then run the existing child actions, metadata finalization, supported-state verification, operation record, and actioned-entry baseline publication.
8. Update README and product guidance, then add isolated admission, force, no-follow, drift, failure, selector, output, bounded-inspection, and performance tests.

### Validation strategy

- Cover file and tree admission with exact destination links; prove descriptor, state, source, link object, and link target are unchanged by `add`.
- Cover exact forced replacement of regular-file and directory source identities. Assert only the selected link object is replaced, the former target is unchanged, and a baseline generation publishes only after verification.
- Assert status, JSON, and dry-run identify the member and link path; show source-only exact-force guidance; reject pull-side, ordinary, mapping, subtree, aggregate, and multiple-selector attempts.
- Inject link-object substitution, ancestor substitution, source substitution, and post-stage/pre-rename failures. Assert no target following and no false accepted-state publication.
- Retain source-link, ancestor-link, unsupported-node, recursive-member, ownership, and topology regression cases.
- Demonstrate 10,000 unrelated destination entries and links are neither read nor output, then run the existing 100-sample macOS ARM64 release performance gate.
- Run focused integration tests followed by `mise run validate`; run `mise run performance` separately on the supported performance platform.

## Post-Design Constitution Check

The Phase 1 design still passes every gate. The new evidence is ephemeral and no-follow, the file and directory replacement paths use atomic rename through existing descriptor-relative primitives, and the existing lock, revalidation, verification, and publication pipeline prevents false convergence. The plan does not add storage, broad synchronization, target access, or a generalized link-support mode.

## Complexity Tracking

No constitution violations require justification.
