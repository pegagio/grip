# Implementation Plan: Configurable External Diff Program

**Branch**: `028-configurable-external-diff` | **Date**: 2026-09-16 | **Spec**: [spec.md](./spec.md)

## Summary

Make a human-readable, exactly selected `grip diff` hand its already-resolved source and destination endpoints to one directly invoked external comparison program. The effective program is chosen by `GRIP_EXTERNAL_DIFF`, then merged selected-project and machine-wide named-tool settings, then `diff`. The implementation preserves the existing inspection pipeline, no-follow endpoint checks, read-only JSON output, mapping ownership, and baseline behavior; it neither starts a shell nor creates comparison copies.

## Technical Context

**Language/Version**: Rust 1.98, edition 2024

**Primary Dependencies**: Existing `clap`, `serde`, `serde_json`, `toml`, `thiserror`, `rustix`, and `home`

**Storage**: Existing selected-project `.grip/config.toml` Descriptor V2 plus optional read-only machine-wide `~/.grip/config.toml`; no state or baseline format change

**Testing**: `cargo test`, focused isolated CLI integration tests using `tempfile`, then `mise run validate`

**Target Platform**: macOS and Unix local filesystems

**Project Type**: Single Rust CLI application

**Performance Goals**: Retain one existing inspection of the selected scope and launch at most one child process for an eligible exact human selection

**Constraints**: Direct `Command` invocation only; literal arguments; no shell evaluation, temporary material, network access, tool installation, mapping mutation, or JSON-contract change

**Scale/Scope**: One selected project, one exact file or directory comparison, two direct endpoints, and at most two configuration layers

## Constitution Check

| Principle | Plan response | Status |
|---|---|---|
| I. Proportional Rigor | Reuse the local inspection, configuration, and process primitives. Add no daemon, watcher, cache, service, or broad coordination. | PASS |
| II. Ownership and Least Surprise | Launch only after one existing managed comparison resolves exactly and remains safe. The handoff is read-only and never expands managed ownership. | PASS |
| III. Validate, Revalidate, and Verify | Retain current selection and inspection checks, then perform direct endpoint eligibility checks immediately before spawn. No payload, registry, or baseline publication occurs. | PASS |
| IV. Bounded Concurrency | Add no locks or filesystem-wide coordination. A changed/unsafe endpoint prevents launch through existing evidence and direct no-follow checks. | PASS |
| V. Fast, Observable, and Testable | Keep JSON and unselected inspection paths process-free; produce deterministic Grip-owned completion diagnostics and cover argv, precedence, non-launch, and exit cases with isolated fixtures. | PASS |
| Merge-Bounded Persistence | Keep this plan, its design artifacts, future tasks, implementation, and validation results consistent; run analyze after tasking and converge after implementation. | PASS |

No constitutional exception is needed.

## Project Structure

### Documentation

```text
specs/028-configurable-external-diff/
├── spec.md
├── checklists/
├── roadmap-reviews/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── external-diff.md
└── tasks.md                 # Created later by speckit-tasks
```

### Source Code

```text
src/
├── cli.rs                   # Existing diff selector parsing
├── lib.rs                   # Inspection routing, prepared handoff, child lifecycle
├── home.rs                  # Validated invoking-home boundary
├── project/
│   ├── mod.rs               # Selected-project descriptor validation
│   └── init.rs              # Initial Descriptor V2 serialization
├── registry/
│   ├── mod.rs               # Descriptor V2 and deferred diff-profile representation
│   └── publication.rs       # Safe no-follow descriptor loading and descriptor-preserving updates
├── path_policy.rs           # Direct endpoint safety/revalidation
└── result.rs                # Existing inspection rendering and comparison completion rendering

tests/
├── support/project.rs       # Isolated HOME/project and executable fixtures
├── diff_cli_contract.rs     # New external-diff precedence, argv, exit, and non-launch coverage
├── classification_cli_contract.rs
├── registry_integration.rs
└── project_metadata_security_integration.rs

README.md
docs/product-definition.md
```

**Structure Decision**: Extend the existing Rust CLI, descriptor, safe-file, and inspection layers. A typed comparison handoff separates process execution from the existing `CommandOutcome` result category model, so child exit statuses do not distort Grip error categories or JSON output.

## Phase 0: Research Decisions

The resolved decisions and rejected alternatives are in [research.md](./research.md). In particular, the existing Descriptor V2 is strict and mapping mutations reconstruct it, so the implementation must preserve optional diff-profile data rather than treating this as a separate untracked side file or silently dropping settings.

## Phase 1: Design and Implementation Approach

1. Extend the accepted Descriptor V2 document surface with optional `[diff]` and `[difftool.<name>]` data while retaining `schema_version = 2` and strict mapping validation. Keep profile values as an accepted, deferred configuration fragment so malformed diff settings are diagnosed only by an eligible human external-diff request, not by JSON inspection or mapping commands. Preserve profile data semantically whenever mapping add/remove rewrites the descriptor.
2. Add a read-only machine-wide profile loader for `~/.grip/config.toml`. An absent file is an empty layer; a present file and containing directory must satisfy current-user ownership, non-symlink, safe-mode, and no-follow opening requirements. It is a tool-preference document, never a mapping registry or state store, and Grip never creates or changes it.
3. Define typed partial configuration layers and merge them field-by-field: machine-wide first, selected-project second. The selected project may select a globally defined tool; project `program` and `args` replace matching global fields, with `args` replacing as a whole. Resolve and validate the selected tool only after the layers merge. A non-empty `GRIP_EXTERNAL_DIFF` bypasses both layers and contributes no configured arguments.
4. Retain `execute_inspection` as the source of truth for selector interpretation, registry/state loading, classification, and read-only revalidation. Only a human `diff` with a supplied selector may produce a prepared handoff. JSON output and unselected human inspection render the existing result and never load a global profile or spawn a child.
5. Derive direct source/destination paths from the exact resolved selection, supporting a selected file, tree member, or mapping-root directory. Immediately before spawn, require both endpoints to be present supported files or directories and pass existing no-follow/unsafe checks; never infer tree-root eligibility only from descendant classification records.
6. Refactor the process entrypoint to distinguish an ordinary rendered command outcome from a prepared external handoff. Render the human detailed inspection, invoke `std::process::Command` with the executable, literal arguments, source path, and destination path as separate arguments, and inherit normal terminal I/O. Report the Grip-owned completion deterministically after `wait`.
7. Return a normal child exit code unchanged. On Unix signal termination, identify the signal and return `128 + signal`; on preflight/configuration/launch failure, retain a Grip-owned diagnostic and its existing result-category exit behavior. Do not route a started child result through the static `ResultCategory` exit mapping.
8. Add focused integration coverage and update the README and product definition with the two configuration locations, TOML shape, precedence, wrapper guidance for environment arguments, and non-launch boundaries.

The data model and command/configuration contract are in [data-model.md](./data-model.md) and [contracts/external-diff.md](./contracts/external-diff.md).

## Validation Strategy

- Descriptor and loader tests for legacy mapping-only V2 documents, optional profile round trips, strict unknown-key rejection, deferred profile diagnostics, no-follow safe-mode handling, and no loss of profile settings during mapping updates.
- Isolated child-process tests for global/project inheritance, project selection and field replacement, environment override, literal space/shell-like arguments, source and destination selectors, direct file and directory endpoints, normal exits, and signals.
- Regression tests proving no child process for JSON, no selector, missing/unsupported endpoints, and unsafe symbolic-link conditions; retain existing classification output and state snapshots.
- Run focused diff and classification tests, then `cargo test` and `mise run validate` after implementation. This planning change itself is checked with `git diff --check` and placeholder scans.

## Post-Design Constitution Check

All principles remain PASS. The design adds one bounded local child process only after an exact safe read-only selection, has no mutation or new coordination, preserves machine-readable behavior, and keeps configuration handling isolated from mapping ownership and accepted state.

## Complexity Tracking

No constitutional violations or complexity exceptions are required.
