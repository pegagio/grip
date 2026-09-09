# Implementation Plan: Project-Scoped Initialization and Portable Mappings

**Branch**: `specs/010-project-scoped-initialization` | **Date**: 2026-09-09 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/010-project-scoped-initialization/spec.md`

## Summary

Replace Grip's global registry with initialized Grip projects. `grip init [PATH]` creates a portable `.grip/config.toml` and an exact `.grip/.gitignore`; every project-dependent command resolves one project explicitly or by exhaustive ancestor discovery, interprets sources relative to that project and destinations relative to the invoking user's home, and confines mutable evidence to the ignored `.grip/state/` tree.

The implementation introduces one resolved `ProjectContext`, separates portable declarations and durable identities from absolute runtime endpoints, and carries that context through registry, state, operation, recovery, synchronization, and result delivery. Because existing registry, state, operation, and recovery formats encode the superseded global absolute-path model, Feature 010 performs strict schema cutovers with no compatibility reads, migration, fallback, or dual-mode behavior.

## Technical Context

**Language/Version**: Rust 1.98.0, edition 2024

**Primary Dependencies**: Existing `clap`, `home`, `ignore`, `libc`, `rustix`, `serde`, `serde_json`, `sha2`, `thiserror`, and `toml`; no new dependency is planned

**Storage**: Portable TOML descriptor and exact Git exclusion policy under `<project>/.grip/`; integrity-protected JSON baselines, locks, operation records, recovery evidence, and staging under `<project>/.grip/state/`

**Testing**: `cargo test`, project-oriented temporary fixtures, CLI contract tests, filesystem safety and recovery integration tests, repository-wide legacy-interface scans, and the existing ignored release performance harness through `mise run validate`

**Target Platform**: Current macOS on APFS, preserving Feature 009's qualified filesystem contract

**Project Type**: Local per-user Rust CLI

**Performance Goals**: At least 95 of 100 implicit discoveries from a directory 100 levels below a project root complete within one second; preserve the existing 10,000-entry status and dry-run targets after fixture conversion

**Constraints**: Offline and unprivileged; no daemon, persistent project index, background discovery, generated project ID, global registry, `GRIP_HOME` command scope, compatibility reader, implicit Git operation, symlink escape, partial state rebind, or mutation before complete revalidation

**Scale/Scope**: One selected project per invocation; complete migration of all project-dependent commands and the shared fixture layer; source trees up to the existing representative 10,000-entry workload and ancestor discovery depth of at least 100

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **I. Proportional Rigor for a Local Tool — PASS**: The design remains a single local CLI and reuses the existing inspect-plan-revalidate-apply-publish architecture. It adds no daemon, service, persistent index, watcher, distributed identity, or new dependency.
- **II. Explicit Ownership and Least Surprise — PASS**: One resolved project is the authority boundary. Portable declarations have distinct source and destination roots, `.grip/` is structurally excluded from payload ownership, explicit invalid selection never falls back, and the legacy global registry is ignored.
- **III. Validate, Revalidate, and Recover — PASS**: Initialization publishes complete metadata atomically. Project identity, descriptor bytes, home binding, mapping resolution, accepted state, operation records, and recovery targets are revalidated before mutation. Copied state remains untrusted until complete in-memory rebinding succeeds.
- **IV. Bounded Concurrency — PASS**: A project-local mutation lock serializes writers within one project, narrower registry and state publication locks preserve existing ordering, and unrelated projects never contend through a user-global lock.
- **V. Fast, Observable, and Testable — PASS**: Discovery is a bounded ancestor walk without persistent indexing. One context is retained through dispatch and result finalization. Human and JSON results share the selected project identity, and all acceptance tests use isolated project and home roots.
- **Product boundaries — PASS**: Grip remains local, per-user, offline, allowlist-based, and macOS/APFS-scoped. CLI parsing and presentation remain separate from project resolution and filesystem behavior.
- **Merge-bounded persistence — PASS**: Features 001–009 remain immutable historical evidence. Feature 010 carries the replacement forward, and current product documentation and wiki claims are updated through current artifacts rather than by rewriting historical feature directories.

No constitutional exception is requested.

## Project Structure

### Documentation (this feature)

```text
specs/010-project-scoped-initialization/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── cli.md
│   ├── project-resolution.md
│   ├── storage.md
│   └── state-rebinding.md
├── checklists/
│   └── requirements.md
├── roadmap-reviews/
│   └── brief-20260909T121333Z.md
└── tasks.md                         # Created later by speckit-tasks
```

### Source Code (repository root)

```text
src/
├── project/
│   ├── mod.rs                       # ProjectContext, selection, and revalidation
│   └── init.rs                      # Atomic, non-repairing initialization
├── cli.rs                           # init and global --project parsing
├── lib.rs                           # One-time resolution, dispatch, and result finalization
├── home.rs                          # Narrow invoking-user home resolution only
├── mapping.rs                       # PortableMapping and ResolvedMapping domains
├── path_policy.rs                   # Project-relative and home-relative grammars
├── registry/
│   ├── mod.rs                       # Portable descriptor schema V2
│   └── publication.rs               # Project-local loading and atomic publication
├── observation/model.rs             # Portable durable entry identities
├── discovery/                       # Project-scoped inspection and structural .grip exclusion
├── state/                           # State V4, project-local publication, and locks
├── operation/                       # Portable Operation Record V2
├── recovery/                        # Portable inventory, restore, and cleanup
├── mutation/                        # Portable plans, execution, and recovery
├── push/ pull/ sync/ delete/ retire/ # ProjectContext propagation
├── error.rs                         # Stable project and rebind reasons
└── result.rs                        # Shared project identity in human and JSON output

tests/
├── support/mod.rs                   # Isolated project and home fixture builder
├── init_cli_contract.rs
├── project_selection_integration.rs
├── portable_mapping_integration.rs
├── project_state_rebinding_integration.rs
├── project_isolation_integration.rs
├── project_metadata_security_integration.rs
├── cli_contract.rs
├── product_acceptance.rs
├── performance_acceptance.rs
└── existing integration suites      # Converted to project-scoped fixtures
```

**Structure Decision**: Preserve the single-crate architecture. Add a cohesive `project` module because selection and revalidation are now the authority boundary shared by every command. Keep `home.rs` only for trusted destination-home resolution. Extend the existing registry, state, operation, recovery, discovery, and mutation modules rather than adding a repository layer or compatibility subsystem.

## Implementation Strategy

### 1. Establish the project authority boundary

Add `grip init [PATH]` and global `--project PATH`. Central dispatch resolves a `ProjectContext` exactly once for every project-dependent command and carries it through result-delivery finalization. `init` resolves only its target; application `version` and clap-provided help/version return without project discovery. Explicit selection requires the exact root and never falls back. Implicit discovery inspects the complete ancestor chain, rejects invalid candidate metadata, and succeeds only with exactly one valid project.

Initialization stages the complete `.grip` directory inside the target, writes and syncs Descriptor V2 plus exact `/state/\n`, atomically renames the staged directory without replacement, and syncs the project root. It creates no state, Git metadata, mapping, or payload. Existing exact metadata is a no-op; partial, unsafe, unsupported, or non-equivalent metadata is never repaired or overwritten.

### 2. Separate portable intent from runtime endpoints

Replace the current overloaded absolute `Mapping` representation with `PortableMapping` and `ResolvedMapping`. Descriptor V2 stores only normalized project-relative sources and literal `~` or `~/...` destinations. Runtime resolution binds those declarations to the canonical selected project and invoking-user home and retains revalidatable ancestry evidence. Mapping ownership is checked after resolution, while durable identity uses the portable tuple and entry-relative bytes.

Source selectors use project-relative grammar and destination-space selectors use home-relative grammar. A tree mapping may declare `.` as its source, but `.grip/` is pruned structurally before ignore-policy evaluation and can never become managed payload. Absolute paths, parent traversal, dot or repeated components other than the exact tree-root `.`, environment expressions, other-user tilde notation, and symlink escapes are rejected.

### 3. Cut over every durable state and recovery identity

Publish State Envelope V4 with portable entry identities and machine-local binding evidence for the canonical project root, canonical home, accepted descriptor digest, and resolved mapping-set digest. Introduce new operation, mutation-recovery, recovery-manifest, and recovery-metadata schema versions wherever the prior format embeds an absolute `MappingSnapshot`, bound target, or restoration authority. Reject all superseded schemas; do not translate global state or infer a new identity by replacing path prefixes.

On a binding mismatch, retain the bytes but treat them as untrusted. Re-resolve all portable identities, match them to the current descriptor or explicit pending-retirement evidence, reobserve every accepted entry and recovery target required by the command, and revalidate project, home, descriptor, and topology evidence. Read-only and dry-run commands report the rebind result without writing. A successful state-writing operation records the current binding in its final atomic state publication; any incomplete, stale, ambiguous, or contradictory evidence blocks acceptance and mutation.

### 4. Relocate all mutable coordination beneath ignored state

Derive baseline, operation, recovery, staging, and lock paths exclusively from `<project>/.grip/state/`. The first stateful writer creates strict owner-only state directories and files lazily. One project-local mutation lock serializes config, state, recovery, and payload writers; narrower registry and state locks preserve deterministic publication order. Read-only and dry-run commands neither create state nor acquire write locks.

Registry recovery remains available for Grip-authored config changes, but recovery bindings identify the selected project's descriptor and portable mappings rather than an absolute global target. Unrelated projects use disjoint lock and recovery namespaces.

### 5. Preserve behavior and replace current interfaces holistically

Propagate `ProjectContext` through mapping, inspection, baseline, push, pull, sync, resolve, delete, retire, recovery, and validation paths. Add the selected project to shared human and Result Envelope V1 details after resolution without changing the envelope's top-level schema. Discovery failures report safe candidate evidence but do not claim a selected project.

Replace global workflow text in `README.md`, `docs/product-definition.md`, current wiki claims, runtime code, and active tests. Preserve Features 001–009 unchanged as historical evidence. Convert the shared fixture layer first, then run every existing synchronization and recovery suite with distinct temporary project and home roots plus a populated legacy `GRIP_HOME` trap.

### 6. Qualify safety, isolation, and performance

Test atomic initialization, exact reinitialization, concurrent creation, unsafe metadata, Git-clone permissions, nested projects, exhaustive discovery, descriptor substitution, path grammar, structural `.grip` exclusion, clone isolation, same-project contention, cross-project independence, copied-state rebinding, recovery authority, dry-run purity, and paired output. Extend the ignored release harness with 100-level discovery and 100 samples, recording p50, p95, maximum, revision, and host evidence. Add no cache or index unless measurement shows a concrete miss.

## Post-Design Constitution Re-check

Phase 1 resolves every technical unknown without adding a new dependency or compatibility layer. Project discovery remains bounded; state and recovery authority are portable but revalidated against concrete endpoints; all mutable artifacts share one ignored project-local boundary; initialization and publication remain atomic and non-following; and read-only behavior remains side-effect free. The broad fixture migration is required to prove the complete product cutover, while historical feature artifacts remain untouched. All gates remain passed.

## Complexity Tracking

No constitutional violations require justification.
