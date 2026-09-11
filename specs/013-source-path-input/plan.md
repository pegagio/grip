# Implementation Plan: Source Path Input Normalization

**Branch**: `013-source-path-input` | **Date**: 2026-09-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from [spec.md](spec.md)

## Summary

Accept conventional relative source arguments such as `./app/` and normalize them to the existing portable project-relative declaration before endpoint lookup, mapping identity, or publication. Keep descriptor decoding strict so persisted declarations remain canonical. Route every source-space CLI input through the same command-input parser; destination behavior remains unchanged.

## Technical Context

Grip is a local Rust CLI. The change extends the existing path-policy boundary and command callers; it adds no dependency, storage format, service, or background process.

- **Language/Version**: Rust edition 2024; minimum Rust 1.98
- **Primary Dependencies**: Existing `clap`, `serde`, `toml`, `thiserror`, and standard filesystem APIs; no additions
- **Storage**: Version-controlled `.grip/config.toml` declarations and ignored `.grip/state/` operational evidence
- **Testing**: Rust unit and integration tests using `tempfile`; full validation via `mise run validate`
- **Target Platform**: Local macOS and Unix filesystems visible to the invoking user
- **Project Type**: Single Rust CLI application
- **Performance Goals**: Constant-space lexical normalization per submitted source argument; no new traversal, hashing, caching, indexing, or coordination
- **Constraints**: Persist only normalized source declarations; preserve strict descriptor decoding; reject results outside the project or within `.grip`; retain existing non-following endpoint validation and destination behavior
- **Scale/Scope**: One source input value shared by `add`, `list`, `remove`, and source-space `status`, `diff`, `push`, `pull`, and `sync` selectors; no new syntax, migration, or compatibility alias

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **I. Proportional Rigor**: Add one lexical command-input normalization path beside the existing strict declaration parser; do not introduce a subsystem or dependency.
- **II. Explicit Ownership and Least Surprise**: Normalize only a submitted source argument, then retain existing project containment, `.grip` exclusion, endpoint, ownership, and topology checks.
- **III. Validate, Revalidate, and Verify**: Normalization occurs before existing validation. It does not canonicalize the filesystem or follow symlinks, so current inspection and revalidation remain authoritative.
- **IV. Bounded Concurrency**: Reuse existing narrow registry and state publication coordination; no lock, watcher, or contention behavior changes.
- **V. Fast, Observable, and Testable**: Use deterministic parser and isolated CLI tests for accepted input, retained declarations, selectors, and rejected boundaries. No state is created for rejected input.
- **Merge-bounded persistence**: Keep the specification, plan, tasks, implementation, user documentation, and validation consistent before implementation resumes or the feature is merged.

No constitutional exception or complexity justification is required.

## Project Structure

### Documentation (this feature)

```text
specs/013-source-path-input/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── source-paths.md
└── tasks.md                 # Created later by speckit-tasks
```

### Source Code (repository root)

```text
src/
├── path_policy.rs        # Strict declaration parsing and CLI source-input normalization
├── mapping.rs            # Portable mapping construction from CLI input or stored declarations
├── lib.rs                # add and source-space selector command boundaries
└── registry/
    └── mod.rs            # Strict descriptor validation through deserialization

tests/
├── portable_mapping_model.rs
├── portable_mapping_integration.rs
├── project_reserved_metadata_integration.rs
└── mapping_cli_contract.rs

README.md
docs/
└── product-definition.md
```

**Structure Decision**: Keep the existing single Rust CLI layout. A strict portable-declaration parser remains the descriptor boundary, while a separate command-input parser normalizes user spelling before invoking the existing runtime path and mapping flows.

## Complexity Tracking

No constitutional violations require complexity tracking.
