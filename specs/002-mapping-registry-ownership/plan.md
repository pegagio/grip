# Implementation Plan: Mapping Registry and Ownership Validation

**Branch**: `002-mapping-registry-ownership` | **Date**: 2026-09-03 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/002-mapping-registry-ownership/spec.md`

## Summary

Extend the existing single-crate Rust CLI with an intent-only mapping lifecycle for file and tree mappings. Parse the complete v1 TOML registry into versioned wire types, convert mappings into canonical domain values, validate all source and destination namespaces as one ownership graph, retain and verify the prior accepted registry by content digest, and atomically publish deterministic updates under a stable owner-only advisory lock. Keep path resolution, topology validation, persistence, CLI parsing, and presentation independently testable; do not traverse tree members or modify payload or synchronization state.

## Technical Context

**Language/Version**: Rust 1.98.0, Rust 2024 edition, pinned in `mise.toml`; `rust-version = "1.98"`

**Primary Dependencies**: Existing `clap` 4.6, `serde` 1, `toml` 1.1, `serde_json` 1, `thiserror` 2, `home` 0.5, `rustix` 1.1, and `sha2` 0.10; existing `tempfile` 3 for tests only; no new dependency

**Storage**: User-authored `<GRIP_HOME>/config.toml`, deterministically rewritten as a complete v1 registry; stable owner-only `<GRIP_HOME>/.registry.lock`; same-directory `<GRIP_HOME>/.config.tmp-<unique-token>` staging files; immutable `<GRIP_HOME>/state/recovery/registry/sha256-<digest>/config.toml` prior-registry generations

**Testing**: Built-in Rust unit tests, black-box CLI contract tests through `CARGO_BIN_EXE_grip`, isolated `tempfile` roots, publication fault injection, and an ignored release-mode 1,000-mapping performance harness

**Target Platform**: Local macOS and Unix-like systems with non-following metadata inspection, advisory file locking, file synchronization, and atomic same-directory rename

**Project Type**: Single command-line application crate with a reusable library boundary

**Performance Goals**: Complete validation and deterministic listing of 1,000 mappings within one second in at least 95 of 100 warm release-mode runs on a documented representative workstation

**Constraints**: Offline and per-user; canonical source-path identity; complete-registry validation for every mapping operation; no tree-member traversal; no payload, baseline, or synchronization-state mutation; verified prior-registry recovery before replacement; no privilege elevation; safe ancestry handling; deterministic human/JSON results; owner-only lock, staging, and recovery nodes; atomic accepted-registry visibility; no universal sudden-power-loss guarantee

**Scale/Scope**: Two mapping kinds, five lifecycle commands, one v1 registry document, one stable registry lock, immutable content-addressed prior-registry recovery generations, all-pairs topology validation for up to 1,000 mappings, and no discovery or synchronization behavior

## Constitution Check

The pre-research and post-design gates both pass with no exception:

- **I. Proportional Rigor**: The design extends the existing crate and dependencies. A straightforward all-pairs ownership check is easier to audit and is sufficient for 1,000 mappings; no cache, persistent index, graph framework, database, async runtime, daemon, watcher, or broad lock is introduced.
- **II. Explicit Ownership**: Only explicitly submitted file or tree mappings can enter the registry. Complete namespace validation rejects ambiguity before ownership is recorded. Mapping commands never traverse tree members or mutate payloads, synchronization state, backups, baselines, Git state, or privileged locations.
- **III. Validate, Revalidate, and Recover**: Every operation validates the current complete registry and path evidence. Writers hold one registry lock, reread and compare the accepted bytes, revalidate the complete candidate and path evidence, retain and verify the exact prior registry in an immutable content-addressed recovery generation, stage and verify the serialized candidate, and atomically rename it. Failure before rename leaves the prior registry accepted; successful replacement retains recovery evidence without creating payload backups.
- **IV. Bounded Concurrency**: One nonblocking advisory lock protects only registry publication. Its stable inode remains at `.registry.lock`; lock-file existence is not ownership, and no age- or PID-based stale-lock deletion is used.
- **V. Fast, Observable, and Testable**: Domain failures carry stable reason values, rendering remains separate, and every filesystem test uses isolated roots. A representative 1,000-mapping harness measures before any indexing or parallelism is considered.
- **Product boundaries**: The feature remains a local per-user CLI and extends only mapping intent. It does not introduce discovery, ignore evaluation, remote access, multi-user coordination, privilege elevation, copying, deletion, or synchronization.
- **Merge-bounded persistence**: Planning and analysis discoveries about safe intermediate-symlink canonicalization, the stable registry lock, path revalidation, registry permissions, canonical TOML publication, and prior-registry recovery were flowed back into `spec.md` before implementation resumes. Post-implementation convergence remains an explicit task.

## Project Structure

### Documentation (this feature)

```text
specs/002-mapping-registry-ownership/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── cli.md
│   ├── storage.md
│   └── topology.md
├── checklists/
│   └── requirements.md
└── roadmap-reviews/
    └── brief-20260903T215150Z.md
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
├── mapping.rs
├── path_policy.rs
├── registry/
│   ├── mod.rs
│   └── publication.rs
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
├── mapping_cli_contract.rs
├── mapping_registry_integration.rs
├── mapping_topology_integration.rs
├── performance_acceptance.rs
├── registry_integration.rs
└── state_integration.rs
```

**Structure Decision**: Retain one application crate and the Feature 001 entrypoint/result boundaries. Move `src/registry.rs` to `src/registry/mod.rs`, add `registry/publication.rs` for persistence, and keep mapping semantics and path policy in separate modules so registry serialization does not own domain validation. Extend existing integration suites rather than adding a workspace or generic repository abstraction.

## Complexity Tracking

No constitutional violations require justification.
