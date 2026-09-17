# Implementation Plan: Operational Output and State Rebinding

**Branch**: `029-operational-output-and-state` | **Date**: 2026-09-16 | **Spec**: [spec.md](./spec.md)

## Summary

Refine human operational output around selected external diffs and pushes, while making State V4 rebinding depend on mapping-relevant identity rather than unrelated descriptor settings. Existing JSON, mapping, process-safety, and mutation-safety boundaries remain unchanged.

## Technical Context

**Language/Version**: Rust 1.98, edition 2024

**Primary Dependencies**: Existing `clap`, `serde_json`, `libc`, and standard process and filesystem APIs

**Storage**: Existing Descriptor V2 and State V4 binding; no schema change

**Testing**: Focused Rust CLI integration tests with isolated temporary roots, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `git diff --check`

**Target Platform**: macOS and Unix local filesystems

**Project Type**: Single Rust CLI application

**Performance Goals**: Reuse captured inspection and action evidence; no additional traversal, external process, cache, or lock

**Constraints**: Preserve JSON output, direct external-tool invocation, no-follow safety, mapping ownership, accepted-state publication safety, and aggregate-force authority

**Scale/Scope**: One selected external comparison, one human mutation transcript, and one selected project's existing binding

## Constitution Check

| Principle | Plan response | Status |
|---|---|---|
| I. Proportional Rigor | Adjust existing renderers and binding comparison only; add no service, cache, watcher, or new state format. | PASS |
| II. Explicit Ownership and Least Surprise | Continue to count and report only managed payload work; destination-only content remains unmanaged. | PASS |
| III. Validate, Revalidate, and Verify | Rebinding remains strict for mapping and location changes; output changes do not bypass mutation verification. | PASS |
| IV. Bounded Concurrency | Add no lock or filesystem coordination; reuse existing inspection and publication boundaries. | PASS |
| V. Fast, Observable, and Testable | Separate program output from diagnostics, retain JSON, and cover all changed behavior with isolated fixtures. | PASS |
| Merge-Bounded Persistence | Record implemented changes in new forward-flowing artifacts, analyze them, and converge against current code. | PASS |

No constitutional exception is needed.

## Project Structure

```text
specs/029-operational-output-and-state/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── human-output.md
├── checklists/
│   └── requirements.md
└── tasks.md

src/
├── lib.rs
├── result.rs
└── state/rebinding.rs

tests/
├── diff_cli_contract.rs
├── push_cli_contract.rs
└── project_state_rebinding_integration.rs
```

**Structure Decision**: `lib.rs` selects output ownership for an external handoff, `result.rs` formats human explanation from existing structured evidence, and `state/rebinding.rs` compares existing location and resolved-mapping fields.

## Design

1. An external handoff bypasses normal human result rendering. A verbose request renders a diagnostic-only explanation to standard error before direct process execution.
2. Verbose differences are grouped by managed property, showing source, baseline, and destination values; unmanaged compatibility noise is omitted.
3. Human synchronization counts use file-level payload actions. Aggregate force blocks use existing blocker reason and path evidence.
4. A changed raw descriptor is rebind-eligible when project root, destination home, and resolved mapping digest match; mapping- or location-affecting changes retain full inspection.
5. Documentation and isolated regression tests record the behavior.

## Validation Strategy

- Run selected external-diff tests for stdout/stderr ownership and verbose property explanations.
- Run push tests for payload-file counts and aggregate blocker rendering.
- Run rebinding integration tests for a changed source plus a valid diff profile, retaining mapping/destination drift coverage.
- Run formatting, Clippy, and diff checks.

## Post-Design Constitution Check

All principles remain PASS. The design changes presentation and refines existing binding authorization without weakening mapping, filesystem, or mutation safety.

## Complexity Tracking

No constitutional violations or complexity exceptions are required.
