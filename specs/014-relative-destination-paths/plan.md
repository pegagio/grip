# Implementation Plan: Relative Destination Paths

**Branch**: `014-relative-destination-paths` | **Date**: 2026-09-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from [spec.md](spec.md)

## Summary

Accept a non-empty relative destination declaration, retain its exact UTF-8 spelling in the project descriptor, and resolve it from the selected Grip project root only when an operation needs a filesystem location. Preserve absolute, `~`, and `~/` behavior. Reuse the existing declaration, resolved-mapping, ownership, state, and CLI boundaries; no new commands, dependencies, state format, migration, or background process are needed.

## Technical Context

Grip is a local Rust CLI. This feature broadens one existing portable declaration parser and threads the selected project root through its operational resolver.

- **Language/Version**: Rust edition 2024; minimum Rust 1.98
- **Primary Dependencies**: Existing `clap`, `serde`, `toml`, `thiserror`, and standard filesystem APIs; no additions
- **Storage**: Version-controlled `.grip/config.toml` mapping declarations and ignored `.grip/state/` operational evidence
- **Testing**: Rust unit and integration tests using `tempfile`; full validation via `mise run validate`
- **Target Platform**: Local macOS and Unix filesystems visible to the invoking user
- **Project Type**: Single Rust CLI application
- **Performance Goals**: Constant-space lexical normalization per resolved destination; no added traversal, hashing, caching, indexing, or coordination
- **Constraints**: Persist exact relative spelling; resolve relative declarations from the selected project root, never CWD; do not filesystem-canonicalize or follow symlinks; retain endpoint, ownership, topology, drift, and publication checks
- **Scale/Scope**: One destination declaration type across mapping creation, descriptor reload, destination-space selectors, state binding/rebinding, remove publication, documentation, and isolated tests

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **I. Proportional Rigor**: Extend the existing `DestinationPath` parser and resolver rather than introduce a new path subsystem, option, storage format, or migration.
- **II. Explicit Ownership and Least Surprise**: Resolve the declaration to an operational absolute endpoint before existing ownership and topology validation. Raw spelling never bypasses existing safety checks.
- **III. Validate, Revalidate, and Verify**: Keep lexical normalization separate from filesystem inspection and revalidation. It must not canonicalize paths or follow symbolic links.
- **IV. Bounded Concurrency**: Reuse existing descriptor and state publication coordination; no new locks, watchers, or contention behavior.
- **V. Fast, Observable, and Testable**: Add deterministic parser, descriptor, selector, copy-portability, topology, and removal-regression tests using isolated temporary roots. Keep declared and resolved values distinct where output exposes both.
- **Merge-bounded persistence**: Keep the specification, plan, tasks, code, user documentation, and validation consistent before implementation or merge.

No constitutional exception or complexity justification is required.

## Project Structure

Feature artifacts record the design and validation contract for the existing single CLI project.

### Documentation

```text
specs/014-relative-destination-paths/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── destination-paths.md
└── tasks.md                 # Created later by speckit-tasks
```

### Source Code

The implementation stays within the existing Rust CLI layout.

```text
src/
├── path_policy.rs           # Destination declaration parsing and resolution
├── mapping.rs               # Portable declaration and resolved-mapping boundary
├── lib.rs                   # Add and destination-space selector boundaries
├── registry/
│   ├── mod.rs               # Descriptor loading and resolved ownership validation
│   └── publication.rs       # Declaration-preserving descriptor publication
└── state/
    ├── mod.rs               # State binding and runtime resolution
    └── rebinding.rs         # Rebinding from portable declarations

tests/
├── portable_mapping_model.rs
├── portable_mapping_integration.rs
├── mapping_cli_contract.rs
├── mapping_topology_integration.rs
├── registry_integration.rs
├── project_clone_portability_integration.rs
└── project_state_rebinding_integration.rs

README.md
docs/
└── product-definition.md
```

**Structure Decision**: Keep the existing single Rust CLI layout. The general destination declaration type remains distinct from strict source declarations, and receives both project root and home only when deriving an operational path.

## Complexity Tracking

No constitutional violations require complexity tracking.
