# Implementation Plan: Executable Force-Resolution Guidance

**Branch**: `019-executable-force-guidance` | **Date**: 2026-09-13 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/019-executable-force-guidance/spec.md`

## Summary

Make human conflict guidance executable by deriving eligibility from Grip's existing exact-entry force-resolution rules. Exact-entry conflicts retain their paired `push --force` and `pull --force --destination` instructions. Aggregate conflicts, including an unbaselined tree mapping root, instead show the read-only next step `Run: grip diff SOURCE`. This is a presentation-only correction: force-resolution scope, JSON output, diff behavior, classification, and filesystem mutation behavior remain unchanged.

## Technical Context

**Language/Version**: Rust 1.98, edition 2024

**Primary Dependencies**: clap, serde, serde_json; no new dependencies

**Storage**: Existing on-disk project metadata and accepted baseline state; no schema or migration changes

**Testing**: `cargo test` integration and CLI contract suites; `mise run validate`

**Target Platform**: Local macOS and Unix command-line environments

**Project Type**: Rust CLI

**Performance Goals**: Preserve current status and mutation planning complexity; no additional filesystem scans or serialized output processing

**Constraints**: Human guidance must be copyable and accepted by the existing selector validation; force resolution remains limited to one exact managed entry; JSON output and `grip diff` stay unchanged

**Scale/Scope**: Human rendering and non-serialized guidance derivation for status and blocked `push`, `pull`, and `sync` outcomes

## Constitution Check

| Principle | Plan compliance |
|---|---|
| I. Contract-led behavior | The specification, contract examples, and CLI tests define which instructions may appear for exact and aggregate conflicts. |
| II. Explicit ownership and authority | The established `exact_resolution_selection` policy remains the single authority for whether an action can resolve one entry; rendering does not invent broader force authority. |
| III. Safe, recoverable operations | Aggregate guidance is read-only and force commands are emitted only when existing validation accepts their selector. No mutation path changes. |
| IV. Bounded coordination | This is a local CLI presentation correction with no external coordination, migration, or new service. |
| V. Verifiable implementation | Contract tests exercise exact and aggregate conflict output plus command acceptance; the existing full validation command protects regressions. |

**Gate result**: Pass before research and after design. No constitutional exception or complexity justification is required.

## Project Structure

### Documentation (this feature)

```text
specs/019-executable-force-guidance/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── executable-force-guidance.md
└── tasks.md                     # Created later by speckit-tasks
```

### Source Code (repository root)

```text
src/
├── lib.rs                       # Derives non-serialized action or inspection guidance from existing selection rules
├── result.rs                    # Renders conflict guidance for status and blocked mutations
└── observation/model.rs          # Existing selector-resolution authority; expected to be reused, not broadened

tests/
├── classification_cli_contract.rs # Status guidance contracts
├── push_cli_contract.rs           # Blocked push guidance contracts
├── pull_cli_contract.rs           # Blocked pull guidance contracts
└── sync_cli_contract.rs           # Blocked sync guidance contracts

README.md                        # Concise user-facing explanation if the command guidance contract is documented here
```

**Structure Decision**: Keep classification, selector resolution, and mutation execution in their existing modules. Add only non-serialized human-guidance data at the orchestration/rendering boundary so that status and blocked mutation output share the same eligibility decision without changing persisted or JSON result models.

## Phase 0: Research

Research findings are recorded in [research.md](research.md). The implementation will reuse the exact-entry policy rather than duplicate a weaker classification-only heuristic, preserve the current force surface, and use existing `grip diff SOURCE` as the aggregate inspection step.

## Phase 1: Design

The design artifacts define a non-serialized guidance variant and its transitions in [data-model.md](data-model.md), the visible human-output contract in [contracts/executable-force-guidance.md](contracts/executable-force-guidance.md), and a focused verification sequence in [quickstart.md](quickstart.md).

## Complexity Tracking

No constitution violations or additional complexity require justification.
