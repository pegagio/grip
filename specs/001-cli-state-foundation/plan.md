# Implementation Plan: CLI, Configuration, and State Foundation

**Git Branch**: `develop` | **Feature Identifier**: `001-cli-state-foundation` | **Date**: 2026-09-03 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/001-cli-state-foundation/spec.md`

## Summary

Build a single-crate Rust CLI that resolves the exact per-user Grip home, validates a strict TOML registry and integrity-protected JSON state, and renders deterministic human or JSON results without touching mapped payloads. Keep `main` and CLI parsing thin; model home resolution, document validation, error classification, state publication, and rendering as independent library responsibilities. Before replacing accepted state, retain and verify it by generation in a recovery namespace, then publish the replacement through a same-directory staged file under a short-lived OS advisory lock. Prove all filesystem behavior in isolated temporary roots.

## Technical Context

**Language/Version**: Rust 1.98.0, Rust 2024 edition, pinned in `mise.toml`; `rust-version = "1.98"`

**Primary Dependencies**: `clap` 4.6, `serde` 1, `toml` 1.1, `serde_json` 1, `thiserror` 2, `home` 0.5, `rustix` 1.1, and `sha2` 0.10; `tempfile` 3 for tests only

**Storage**: User-authored `config.toml`; machine-owned `state/state.json`, `state/state.lock`, `state/recovery/generation-<N>/state.json`, and same-directory staging files under the selected Grip home

**Testing**: Built-in Rust unit and integration tests, black-box subprocess tests through `CARGO_BIN_EXE_grip`, isolated `tempfile` roots, targeted fault injection, and a non-default release-mode performance acceptance harness

**Target Platform**: Local macOS and Unix-like systems on filesystems that support required advisory locking, file synchronization, and atomic same-directory rename

**Project Type**: Single command-line application crate with a reusable library boundary

**Performance Goals**: At least 95 of 100 warm release-mode runs complete within 100 ms for help/version and 250 ms for minimal-home validation on a documented representative workstation

**Constraints**: Offline, per-user, no mapped-payload mutation, no privilege elevation, exact `GRIP_HOME`, strict schemas, deterministic result categories, owner-only Grip-owned state and recovery, retained prior accepted state, atomic visibility for state publication, and no promise of universal sudden-power-loss durability

**Scale/Scope**: One executable, two application commands, one empty v1 registry schema, one minimal v1 state schema, six public result categories, one reserved internal state-contention category, and no mapping or synchronization behavior

## Constitution Check

The pre-research and post-design gates both pass with no exception:

- **I. Proportional Rigor**: A single crate and eight narrowly justified runtime dependencies are sufficient. No daemon, watcher, cache, persistent index, async runtime, broad lock, snapshot mechanism, or database is introduced.
- **II. Explicit Ownership**: The feature reads or publishes only documents beneath the selected Grip home. It never discovers, opens, or mutates mapped payloads and never invokes Git or privilege elevation.
- **III. Validate, Revalidate, and Recover**: Readers reject malformed or incompatible documents. Writers revalidate Grip-owned paths while holding the publication lock, retain and verify the prior accepted state before replacement, stage and verify the complete replacement, and make only the final rename authoritative.
- **IV. Bounded Concurrency**: One stable, nonblocking advisory lock protects only the state-publication critical section. Lock-file existence is not ownership, and kernel release replaces unsafe age- or PID-based stale-lock deletion.
- **V. Fast, Observable, and Testable**: Domain results, human/JSON rendering, and optional diagnostics are separate. All filesystem tests use isolated roots, and the explicit performance harness measures the user-visible thresholds without adding a benchmark framework.
- **Product boundaries**: The design remains a local per-user CLI, targets macOS/Unix semantics explicitly, and introduces no remote, multi-user, privileged, or payload behavior.
- **Merge-bounded persistence**: Research-driven clarification of help/version and `validate` behavior has been flowed back into `spec.md`; design artifacts remain one mutable feature change set.

## Project Structure

### Documentation (this feature)

```text
specs/001-cli-state-foundation/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── cli.md
│   └── storage.md
└── tasks.md
```

### Source Code (repository root)

```text
Cargo.toml
Cargo.lock
rust-toolchain.toml
mise.toml
src/
├── main.rs
├── lib.rs
├── cli.rs
├── error.rs
├── home.rs
├── registry.rs
├── result.rs
└── state/
    ├── mod.rs
    ├── lock.rs
    └── publication.rs
tests/
├── support/
│   └── mod.rs
├── cli_contract.rs
├── home_integration.rs
├── registry_integration.rs
├── state_integration.rs
└── performance_acceptance.rs
```

**Structure Decision**: Use one application crate because the feature has one executable and one cohesive domain. `src/main.rs` only connects process I/O and exit status; `src/lib.rs` exposes Grip-owned behavior for direct tests. Modules follow actual responsibilities rather than speculative architectural layers. Unit tests live beside modules; cross-boundary and filesystem behavior lives under `tests/`.

## Complexity Tracking

No constitutional violations require justification.
