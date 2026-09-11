# Implementation Plan: Simplify Status Output

**Branch**: `015-simplify-status-output` | **Date**: 2026-09-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from [spec.md](spec.md)

## Summary

Replace the default human `grip status` renderer with a concise, deterministic, path-centered presentation: a summary followed only by nonempty `Conflicts`, `Changes to push`, `Changes to pull`, and `Needs baseline` sections. Each row shows the source and destination pair with a status-specific directional symbol. `Needs baseline` covers every nonblocking, non-directional reconciliation state, not only matching payloads. Reuse the existing typed classification result and its safe metadata messages; preserve inspection, classification, JSON serialization, exit behavior, command semantics, and the detailed `diff` presentation. No dependencies, storage, migrations, commands, or background behavior are needed.

## Technical Context

- **Language/Version**: Rust edition 2024; minimum Rust 1.98
- **Primary Dependencies**: Existing `clap`, `serde_json`, and standard-library formatting; no additions
- **Storage**: No storage changes; existing project configuration and ignored state are read unchanged
- **Testing**: Rust unit and integration tests with `tempfile`; full validation via `mise run validate`
- **Target Platform**: Local macOS and Unix filesystems visible to the invoking user
- **Project Type**: Single Rust CLI application
- **Performance Goals**: Preserve one inspection and one classification pass; presentation adds only a linear in-memory projection over status records
- **Constraints**: Human output must be deterministic, path-safe, concise, and status-only; JSON and exit behavior must be byte-for-byte compatible at their existing public boundary; `->`, `<-`, `<->`, and `>-<` are presentation-only and must never imply an unsafe forced action or a conflict winner
- **Scale/Scope**: One default human result path in `src/result.rs`, its focused renderer contracts, and status documentation; `diff`, mutation commands, state, and filesystem behavior remain unchanged

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **I. Proportional Rigor**: Use a presentation-only projection inside the existing result renderer; do not add a formatter framework, additional status mode, cache, index, or persisted view state.
- **II. Explicit Ownership and Least Surprise**: Show only observations already produced for the selected scope. Do not broaden mapping selection, infer ownership, or add automated remediation.
- **III. Validate, Revalidate, and Verify**: Preserve the current observation and classification evidence exactly. The renderer cannot change whether a record is eligible, blocked, or safe to mutate; conflict guidance defaults to read-only inspection.
- **IV. Bounded Concurrency**: Reuse the existing read-only status path. No lock, watcher, contention behavior, or revalidation policy changes are introduced.
- **V. Fast, Observable, and Testable**: Keep JSON diagnostics intact, suppress only no-action information from the default human view, and add deterministic isolated output tests for clean, changed, blocked, metadata, empty, and every nonblocking non-directional classification scope.
- **Merge-bounded persistence**: Keep the specification, plan, tasks, implementation, documentation, and validation synchronized before implementation or merge. This feature supersedes earlier human-status verbosity only; earlier classification and machine-result contracts remain historical and active where not explicitly changed.

No constitutional exception or complexity justification is required.

## Project Structure

Feature artifacts define the presentation contract and validation guide for the existing CLI.

### Documentation

```text
specs/015-simplify-status-output/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── status-output.md
└── tasks.md                 # Created later by speckit-tasks
```

### Source Code

```text
src/
├── cli.rs                    # Existing status command and output selection
├── lib.rs                    # Existing read-only status execution boundary
├── classification/
│   └── model.rs              # Stable records, flags, counts, and ordering
└── result.rs                 # Human and JSON result rendering; feature target

tests/
├── classification_cli_contract.rs
├── metadata_cli_contract.rs
└── product_acceptance.rs

README.md
docs/
└── product-definition.md
```

**Structure Decision**: Keep the existing single Rust CLI layout. `ClassificationResult` remains the source of truth; `result.rs` derives a status-only human projection after typed status data has been serialized for the stable machine-readable boundary.

## Complexity Tracking

No constitutional violations require complexity tracking.
