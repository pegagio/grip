# Implementation Plan: Destination Path Forms

**Branch**: `012-destination-path-forms` | **Date**: 2026-09-10 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from [spec.md](spec.md)

## Summary

Broaden destination declarations from normalized home-relative paths to three explicit forms: absolute, `~`, and any `~/`-prefixed spelling. Preserve the accepted destination text in project mapping intent, derive a separately normalized operational path only while validating or using a mapping, and retain project-relative source behavior. Update the product documentation and all destination-aware command paths so absolute and lexically non-normalized home-relative mappings remain usable after `add`.

## Technical Context

Grip is a local Rust CLI. The plan extends the existing mapping domain and project descriptor rather than adding a service, dependency, or storage backend.

- **Language/Version**: Rust edition 2024; minimum Rust 1.98
- **Primary Dependencies**: `clap`, `serde`, `toml`, `thiserror`, and standard filesystem APIs; no additions
- **Storage**: Version-controlled `.grip/config.toml` mapping declarations and ignored `.grip/state/` operational evidence
- **Testing**: Rust unit and integration tests using `tempfile`; full validation via `mise run validate`
- **Target Platform**: Local macOS and Unix filesystems visible to the invoking user
- **Project Type**: Single Rust CLI application
- **Performance Goals**: No additional traversal, hashing, caching, indexing, or long-lived coordination for this path-form change
- **Constraints**: Keep the submitted destination spelling byte-for-byte in mapping intent; derive a lexical operational path without following symlinks; retain existing endpoint, ownership, topology, drift, and publication checks
- **Scale/Scope**: One mapping declaration type and every existing destination-aware command path; no remote paths, source-form changes, migration command, or CLI aliases

## Constitution Check

The design passes the applicable constitutional gates before research and again after Phase 1.

- **I. Proportional Rigor**: Reuse the existing local path-policy and registry boundaries; introduce one destination declaration value type, not a new subsystem.
- **II. Explicit Ownership and Least Surprise**: Resolve destination declarations before existing ownership and topology validation; raw lexical spelling never bypasses those checks.
- **III. Validate, Revalidate, and Verify**: Preserve existing endpoint inspection, stale-evidence, and publication flows. Lexical normalization is only an input-to-inspection step and must not canonicalize through symlinks.
- **IV. Bounded Concurrency**: Reuse existing narrow registry/state publication coordination; no new locks or watchers.
- **V. Fast, Observable, and Testable**: Add isolated temporary-root coverage for accepted, rejected, persisted, reloaded, and duplicate-resolved path forms. Keep diagnostics distinct from machine output.
- **Merge-bounded persistence**: Update the specification, plan, tasks, implementation, user docs, and validation together before merge.

No constitutional exception or complexity justification is required.

## Project Structure

Feature artifacts record the design and validation contract for the existing single CLI project.

```text
specs/012-destination-path-forms/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── destination-paths.md
└── tasks.md                 # Created later by speckit-tasks

src/
├── path_policy.rs           # Declaration parsing and operational-path inspection
├── mapping.rs               # Portable and resolved mapping domain values
├── lib.rs                   # add command and destination-space selection boundary
├── registry/
│   ├── mod.rs               # Descriptor loading and runtime resolution
│   └── publication.rs       # Declaration-aware descriptor publication
└── state/mod.rs             # Portable identity and state binding preservation

tests/
├── portable_mapping_model.rs
├── portable_mapping_integration.rs
├── mapping_cli_contract.rs
├── registry_integration.rs
└── support/project.rs

docs/
└── product-definition.md
```

**Structure Decision**: Keep the single Rust CLI layout. The feature changes the existing destination declaration boundary and propagates that value through mapping, registry publication, state identity, selector resolution, documentation, and tests.

## Complexity Tracking

No constitutional violations require complexity tracking.
