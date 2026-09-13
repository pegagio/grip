# Implementation Plan: Simplify Default Command Output

**Branch**: `017-simplify-command-output` | **Date**: 2026-09-12 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/017-simplify-command-output/spec.md`

## Summary

Replace verbose default human output for mapping and mutation commands with concise, path-and-direction-oriented forms approved in `updated-output.md`. Reorder concise status sections and replace generic force-resolvable-conflict explanations with concrete source-winning and destination-winning commands. Render no-action, blocked, and failed public mutations as concise, actionable terminal results rather than raw execution evidence. Normalize the first line of parser and domain errors to `Error:`. Preserve all domain outcomes, JSON details, `diff`, help, verbose diagnostics, selector semantics, and filesystem behavior.

## Technical Context

**Language/Version**: Rust 1.98, edition 2024

**Primary Dependencies**: Clap 4.6 for argument parsing and help; Serde and serde_json for the stable structured result envelope

**Storage**: Existing project descriptor and local state files; this feature adds no stored data or schema changes

**Testing**: `cargo test` integration and contract tests using isolated temporary roots; `mise run validate` for format, lint, test, and release-build validation

**Target Platform**: macOS and Unix filesystems supported by the existing Grip contract

**Project Type**: Local command-line application

**Performance Goals**: Human output remains proportional to the selected result rows and adds no filesystem inspection, hashing, serialization, or state I/O

**Constraints**: Exact approved default-human headings, indentation, source/destination ordering, and arrows; JSON values and exit behavior remain byte-for-byte contract-compatible where existing tests assert them; `diff` remains unchanged

**Scale/Scope**: Output-only changes in existing command-result rendering, parser-error presentation, command-output contract tests, and user-facing command documentation

## Constitution Check

The pre-design gate passes.

| Principle | Plan response | Gate |
| --- | --- | --- |
| I. Proportional Rigor | Reuse typed command outcomes and renderers; add no service, cache, watcher, lock, or storage. | Pass |
| II. Explicit Ownership and Least Surprise | Change presentation only; mapping selection, dry-run behavior, ownership, and mutation authority are unchanged. | Pass |
| III. Validate, Revalidate, and Verify | Preserve all planning, revalidation, publication, baseline, and failure behavior. Successful human output omits internal evidence only after the existing operation has completed. | Pass |
| IV. Bounded Concurrency | Do not alter locks, contention handling, or concurrent-drift behavior. | Pass |
| V. Fast, Observable, and Testable | Keep JSON and verbose diagnostics intact; add exact human-output tests with isolated temporary roots and run the existing full validation gate. | Pass |
| Merge-Bounded Persistence | Keep the specification, plan, later tasks, implementation, tests, and documentation synchronized; this feature explicitly amends Feature 015 presentation behavior without rewriting its completed artifacts. | Pass |

The post-design recheck also passes: the design adds only renderer projections and test coverage, with no constitutional exception or complexity tracking required.

## Project Structure

The existing single Rust command-line project is retained.

```text
src/
├── lib.rs                 # Process entrypoint and parser-error presentation
├── result.rs              # Typed outcome construction and human/JSON rendering
└── cli.rs                 # Existing command definitions; no syntax change expected

tests/
├── mapping_cli_contract.rs        # Add, list, and remove human-output contracts
├── classification_cli_contract.rs # Status ordering and conflict rendering contracts
├── push_cli_contract.rs           # Push preview, apply, forced, and no-op contracts
├── pull_cli_contract.rs           # Pull preview, apply, blocked, and no-op contracts
└── sync_cli_contract.rs           # Mixed-direction synchronization contracts

README.md
docs/product-definition.md

specs/017-simplify-command-output/
├── plan.md
├── research.md
├── data-model.md
├── contracts/default-human-output.md
├── quickstart.md
├── command-output-audit.md
└── updated-output.md
```

**Structure Decision**: Keep all rendering in the existing outcome/presentation boundary. `src/result.rs` projects already available typed details into the changed human transcript, while `src/lib.rs` handles the parser errors that bypass normal outcome rendering. No command, domain, mutation, state, or persistence module needs a new responsibility.

## Design

1. Keep `CommandOutcome` details and `ResultEnvelopeV1` unchanged. Add or adapt human-only render helpers that derive concise mapping rows and mutation rows from the existing details rather than modifying the structured payload.
2. Route terminal push, pull, sync, and directional-resolution outcomes through one human mutation presenter. Derive each successful action row’s arrow from its individual action direction, map `resolve` to its public push or pull direction, render `Nothing to …` for no-action plans, render resolvable conflict rows and choices, direct technical blocks to `grip status`, and direct execution failures to status-before-retry. Never project planner reason codes, winner fields, milestones, recovery state, baseline authority, or operation records in those human paths.
3. Render mapping results from their declared source and destination only. Use the approved heading and mapping-kind behavior for add, all-mapping list, selected list, and removal without changing mapping JSON.
4. Change default status grouping order to push, pull, conflicts, then needs-baseline. For initial collisions and ordinary divergent conflicts, replace the generic explanation with concrete force commands derived from valid source- and destination-space selectors. Preserve Feature 015’s visibility for technical compatibility and unsupported-state blockers.
5. Normalize the first line of both Clap parser errors and `GripError`-derived human errors to `Error:` while preserving the message body, usage, and exit code. Keep JSON error envelopes unchanged.
6. Attach a human-only source-display projection to all blocked mutation plans, including `sync` and forced directional resolution, so source-winning guidance uses a valid CWD-relative push selector; render the destination endpoint with `--destination` for destination-winning guidance. Keep all values in JSON and diagnostics.
7. Update exact-output tests and user documentation. Preserve dedicated regression tests for unchanged `diff`, JSON, selector, safety behavior, exit categories, and mutation effects.

## Test Strategy

- Add exact default-human transcripts for add, all-list, selected-list, and remove, while retaining existing JSON assertions for declared and resolved mapping data.
- Add exact mutation transcripts for push and pull previews/applies, a forced source-winning push, and a mixed-direction sync preview/apply. Assert absence of internal action names, recovery state, verification fields, generations, and operation IDs.
- Add status tests that prove the new section order, concrete guidance for initial collisions and ordinary divergent conflicts, retained technical blockers, and unchanged CWD-relative source paths from Feature 016.
- Add blocked push and pull tests proving concrete force guidance uses selectors valid from the invocation directory, human output omits baseline authority, and categories, JSON, and safety details remain unchanged.
- Add terminal mutation tests for no-action, force-resolvable blocked, technical blocked, and partial-failure paths; assert no raw planner reason, winner, milestone, recovery, baseline, or operation-record evidence reaches default human output while JSON remains unchanged.
- Add parser and domain error tests for the `Error:` prefix, preserving usage and exit categories.
- Retain or add regression assertions that `diff` output and JSON result envelopes are unchanged, then run `cargo test --all --no-fail-fast` and `mise run validate`.

## Complexity Tracking

No constitutional violations or additional complexity are introduced.
