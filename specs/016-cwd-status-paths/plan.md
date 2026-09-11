# Implementation Plan: Current-Directory Status Paths

**Branch**: `016-cwd-status-paths` | **Date**: 2026-09-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from [spec.md](spec.md)

## Summary

Render default human `grip status` source paths relative to the invocation directory, using Git-style C quoting only when a human display needs escaping. Preserve destination display and JSON record fields. Let `grip push` resolve a relative source selector from the same invocation directory, retain the existing rejection of absolute source selectors, reject a selector that escapes the selected project before unmanaged-path inspection, and then reuse the existing selection and mutation pipeline.

## Technical Context

**Language/Version**: Rust edition 2024; minimum Rust 1.98

**Primary Dependencies**: Standard library and existing `clap`/`serde_json`; no additions

**Storage**: No storage or schema changes; mapping declarations and State V4 remain unchanged

**Testing**: Rust unit and integration tests with `tempfile`; full validation through `mise run validate`

**Target Platform**: Local macOS and Unix filesystems visible to the invoking user

**Project Type**: Single Rust CLI application

**Performance Goals**: One constant-size invocation-directory capture and linear display projection; no additional recursive tree traversal, hashing, caching, or serialization pass. Existing candidate paths may undergo the bounded canonical containment check required for safe selection.

**Constraints**: Default human status only; JSON must remain byte-for-byte compatible at its existing public boundary; only relative source selectors for `push` change interpretation; absolute source selectors and all non-push selector commands retain their existing behavior; a selector must be contained by the selected project before unmanaged-path inspection

**Scale/Scope**: One source-selector resolver, the human status renderer, focused integration contracts, and documentation; no command, mapping, state, or mutation-pipeline expansion

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **I. Proportional Rigor**: Add a small lexical relative-path and display projection at the existing CLI/presentation boundary. Do not add a path library, shell integration, cache, or persisted path state.
- **II. Explicit Ownership and Least Surprise**: Resolve a copied push selector from the invocation directory, validate project containment before unmanaged-path inspection, and retain existing managed-entry selection. Do not broaden ownership or destination selection.
- **III. Validate, Revalidate, and Verify**: Reuse the existing inspection, revalidation, planning, and mutation pipeline after source-selector resolution. The display change must not influence classification or action eligibility.
- **IV. Bounded Concurrency**: Capture the invocation directory once per process command. No lock, watcher, or revalidation policy changes are required.
- **V. Fast, Observable, and Testable**: Preserve structured records and add isolated nested-directory, sibling-directory, explicit-project, escaping, and project-escape contracts.
- **Merge-bounded persistence**: This new feature follows verified Feature 015 without altering its frozen artifacts. Specification, plan, tasks, implementation, documentation, and validation will remain one mutable review boundary.

No constitutional exception or complexity justification is required.

## Project Structure

Feature artifacts define the status-display and selector contract for the existing CLI.

### Documentation

```text
specs/016-cwd-status-paths/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── status-paths.md
└── tasks.md                 # Created later by speckit-tasks
```

### Source Code

```text
src/
├── lib.rs                 # Invocation directory and push selector resolution
├── path_policy.rs         # Contained CWD-relative source selector and display helpers
└── result.rs              # Default human status source-path projection

tests/
├── classification_cli_contract.rs  # Human status and JSON/exit contracts
└── support/project.rs               # Commands invoked from controlled directories

README.md
docs/
└── product-definition.md
```

**Structure Decision**: Keep the existing single Rust CLI. The classification result remains the JSON and domain source of truth; a presentation-only status projection receives the captured invocation directory. `push` alone gets a new CWD-relative source-selector resolver that validates the selected-project boundary before the existing selection code runs.

## Complexity Tracking

No constitutional violations require complexity tracking.
