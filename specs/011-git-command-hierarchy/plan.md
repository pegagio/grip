# Implementation Plan: Git-Inspired Command Hierarchy

**Branch**: `011-git-command-hierarchy` | **Date**: 2026-09-09 | **Spec**: [spec.md](spec.md)

## Summary

Replace Grip’s nested, overlapping CLI with the approved flat Git-inspired surface: `init`, `version`, `add`, `list`, `remove`, `status`, `diff`, `push`, `pull`, and `sync`. The implementation will remove all superseded public commands and aliases, use directional force to make an explicitly selected endpoint authoritative, and make internal baseline data follow current mapping membership rather than recovery history.

The main design change is a shared membership-reconciliation transition. It prunes baseline records when a mapping is removed or an entry becomes ignored, so re-adding or unignoring an entry starts without stale conflict evidence. Publication still uses safe, short-lived staging and revalidation, but it retains neither payload recovery copies nor a public recovery interface.

## Technical Context

**Language/Version**: Rust 2024, MSRV 1.98.

**Primary Dependencies**: `clap` 4.6 with derive support, `serde`/`serde_json`, `toml`, and `sha2`.

**Storage**: Project-local `.grip/config.toml` mapping registry, `.gripignore`, and private state/baseline data under `.grip`; temporary publication artifacts only while an operation is in progress.

**Testing**: Rust unit and integration tests with `cargo test`; repository gate `mise run validate` runs formatting, Clippy with warnings denied, tests, and release build. `mise run performance` remains the explicit ignored release-performance gate.

**Target Platform**: Local Unix/macOS command-line tool.

**Project Type**: Single Rust CLI crate.

**Performance Goals**: Preserve the current bounded local-operation responsiveness and release-performance acceptance coverage; no new performance threshold is introduced by this interface redesign.

**Constraints**: No network service, daemon, public recovery/history interface, legacy command aliases, or retained payload backups. Force acts only on exactly one resolved managed entry and may propagate absence.

**Scale/Scope**: Full public CLI replacement, associated registry/state lifecycle refactor, CLI contracts, tests, README/product documentation, and current wiki content. Historical feature artifacts remain historical records.

## Constitution Check

The plan passes all constitutional gates before research and after the Phase 1 design.

| Principle | Plan response |
|---|---|
| I. Proportional rigor | Reuse the existing local publication and lock model; introduce no watcher, daemon, index, or service. |
| II. Ownership and scope | Every mutating directional command resolves to managed mappings, and `--force` requires exactly one entry. |
| III. Validate, revalidate, and verify | Plan before mutation, revalidate immediately before publication, stage payloads only transiently, and verify the winner afterward. |
| IV. Bounded concurrency | Keep registry/state and endpoint locks short and scoped to the operation; membership pruning uses the same bounded publication path. |
| V. Testable and observable behavior | Add parser, contract, state-lifecycle, failure-path, and documentation coverage with human and JSON result assertions. |

No exception or complexity justification is required.

## Project Structure

### Documentation

```text
specs/011-git-command-hierarchy/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── contracts/
    └── cli.md
```

### Source and test areas

```text
src/
├── cli.rs                 # Clap grammar and output parsing
├── lib.rs                 # Command dispatch and project selection
├── result.rs              # Human/JSON result and exit behavior
├── mapping.rs             # Registry mutations and endpoint validation
├── state/                 # Baseline binding, validation, and persistence
├── classification/        # Baseline-aware comparison states
├── mutation/              # Shared plans and bounded publication
├── push/
├── pull/
├── sync/
├── delete/                # Internal directional-absence publication only
├── registry/
└── project/

tests/
├── cli_contract.rs
├── mapping_cli_contract.rs
├── classification_matrix.rs
├── push_cli_contract.rs
├── pull_cli_contract.rs
├── sync_cli_contract.rs
├── state_integration.rs
└── project_state_rebinding_integration.rs
```

The implementation remains a single Rust CLI crate. Existing deletion and publication primitives may be retained behind the new force behavior, while the legacy public recovery, retirement, resolve, and command modules are removed or reduced to internal-only utilities as appropriate.

## Implementation Approach

1. Define the new `clap` grammar in `src/cli.rs`: global `-p`/`--project`, `-o`/`--output json`, and repeated verbosity; direct subcommands only; no `human` output value; and no aliases for removed commands. Keep `init [PATH]` and `version` independent of project selection.
2. Refactor `src/lib.rs` dispatch and `src/result.rs` exit handling around the new commands. `status -e` converts ordinary attention into a nonzero exit; malformed state, invalid arguments, and failed operations remain errors regardless of `-e`.
3. Replace mapping-add validation with two-endpoint discovery. Require at least one endpoint, infer one compatible mapping kind from what exists, record a mapping without copying data, and create a baseline only when both endpoints match.
4. Add a single membership-reconciliation path owned by registry/state code. A baseline records the relevant `.gripignore` policy-revision token when it is established. Any later policy revision mismatch makes that baseline unusable before classification, even if a rule was added and then removed between Grip commands. Reconciliation prunes no-longer-membership or policy-stale baselines on `status` and real mutating paths only; `diff` and every dry run apply the same invalidation in memory without changing state.
5. Make `remove` use a coordinated registry-and-state transition: prevalidate the descriptor and state candidates, publish the mapping change and pruned baseline state under the existing bounded locks, and use an ephemeral completion marker only if needed to finish an interrupted publication. The marker is removed after durable completion and contains no payload backup, user-visible history, or restoration capability.
6. Simplify classification and planning. Remove pending-retirement classifications and recovery-oriented actions. Ordinary `push`, `pull`, and `sync` act only on non-absent unambiguous entries; `sync` also rejects differing unbaselined entries. Force permits an exact one-entry directional winner, including absence.
7. Retain safe temporary staging for individual endpoint writes, revalidate before publish, verify the selected winner afterward, and update baselines only after a successful publication or matched registration. Delete recovery storage and public recovery/retire/resolve/delete command surfaces.
8. Replace affected CLI-contract and lifecycle tests, then update current user documentation and wiki pages. Assert that every legacy command and alias is rejected, and that output defaults to human text while `-o json` is stable machine-readable output.

## Complexity Tracking

No constitutional violations require tracking.
