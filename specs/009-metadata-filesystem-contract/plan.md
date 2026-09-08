# Implementation Plan: Metadata and Filesystem Contract Completion

**Branch**: `009-metadata-filesystem-contract` | **Date**: 2026-09-07 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/009-metadata-filesystem-contract/spec.md`

## Summary

Complete Grip's initial equality and mutation contract for regular files and directories on current macOS with APFS. The implementation extends observation, accepted state, three-way classification, planning, execution, recovery, verification, and results with numeric ownership, nanosecond modification time, an exact extended-attribute policy, ordered macOS ACLs, and an allowlist of ordinary user BSD flags. It also adds descriptor-bound capability checks for unsupported nodes, sparse files, hard links, mount boundaries, and APFS name collisions.

The design keeps existing CLI workflows and the sequential inspect-plan-revalidate-apply-publish architecture. A small `metadata` domain module owns platform-neutral evidence, comparison, and transition types; a narrow macOS adapter performs non-following descriptor operations and APFS capability discovery. Accepted state moves to State Envelope V3, with explicit read-only migration classifications for older baselines. Recovery metadata moves to V2 so every destructive transition remains reversible.

## Technical Context

**Language/Version**: Rust 1.98.0, edition 2024

**Primary Dependencies**: Existing `clap`, `home`, `ignore`, `rustix`, `serde`, `serde_json`, `sha2`, `thiserror`, and `toml`; approved direct `libc 0.2` declaration only for missing Darwin ACL, BSD-flag, and volume-capability bindings

**Storage**: Integrity-protected JSON under exact absolute `GRIP_HOME`, including State Envelope V3, Recovery Metadata V2, and existing Operation Record V1 partitions with additive metadata details

**Testing**: `cargo test`, isolated APFS filesystem integration tests, CLI contract fixtures, schema migration fixtures, and the existing ignored 10,000-entry performance acceptance harness

**Target Platform**: Current macOS on APFS; both default case-insensitive APFS and case-sensitive APFS are qualification targets. Other Unix platforms and filesystems are explicitly deferred.

**Project Type**: Local per-user Rust CLI

**Performance Goals**: At least 95 of 100 `status` and dry-run invocations over the documented 10,000-entry tree finish within two seconds on the recorded qualification workstation. Synchronization is measured and published without inventing an unsupported threshold.

**Constraints**: No daemon, privilege elevation, broad filesystem locks, persistent inode identity, watchers, cross-operation caches, parallel traversal, persistent indexes, implicit link following, partial metadata fidelity, or destination-only ownership expansion. Complete preflight is read-only, all mutations are revalidated, recovery precedes destructive publication, and accepted state publishes only after full verification.

**Scale/Scope**: One local user's mappings and trees up to the representative 10,000-entry workload; ordinary files and directories only; seven equality dimensions in addition to node kind and content; complete integration across existing status, check, diff, baseline, push, pull, sync, resolve, recovery, operation-record, and result paths

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **I. Proportional Rigor for a Local Tool — PASS**: The design extends the existing sequential architecture and introduces no daemon, watcher, cross-operation cache, index, broad lock, snapshot machinery, or general portability layer. The macOS adapter is limited to concrete platform gaps. A measured, operation-local endpoint-profile reuse map is bounded to one immutable mapping snapshot and is discarded after inspection.
- **II. Explicit Ownership and Least Surprise — PASS**: Only selected managed entries can mutate. Unknown metadata, unsupported nodes, ambiguous names, unauthorized transitions, and unavailable capabilities block before mutation. Excluded provenance attributes are reported without becoming managed.
- **III. Validate, Revalidate, and Recover — PASS**: Capability preflight is read-only and complete. Descriptor-bound evidence is revalidated per action. Recovery Metadata V2 preserves complete prior logical state, final verification covers every supported dimension, and accepted-state publication remains last.
- **IV. Bounded Concurrency — PASS**: The plan relies on narrow descriptor binding, identity rechecks, and the existing Grip-owned state lock. It adds no tree lock or claim that inode identity survives replacement.
- **V. Fast, Observable, and Testable — PASS**: Metadata reads are collected once per observation and reused through planning. Human and JSON results share typed records. Isolated APFS tests cover every promised field, blocker, recovery path, and schema migration. Optimization remains measurement-gated.
- **Product boundaries — PASS**: The design is local, per-user, unprivileged, allowlist-based, macOS/APFS-scoped, and keeps CLI presentation separate from filesystem behavior.
- **Merge-bounded persistence — PASS**: Feature 009 artifacts remain one mutable pre-merge change set. `speckit.analyze` is required after task generation and before implementation; `speckit.converge` remains required after implementation.

No constitutional exception is requested.

## Project Structure

### Documentation (this feature)

```text
specs/009-metadata-filesystem-contract/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── cli.md
│   ├── filesystem.md
│   ├── metadata.md
│   └── storage.md
├── checklists/
│   └── requirements.md
├── roadmap-reviews/
│   └── brief-20260907T214346Z.md
└── tasks.md                 # Created later by speckit-tasks
```

### Source Code (repository root)

```text
src/
├── metadata/
│   ├── mod.rs               # Platform-neutral evidence and transition service
│   ├── model.rs             # Metadata, capability, collision, and finding types
│   └── macos.rs             # Narrow descriptor-bound Darwin/APFS adapter
├── observation/
│   ├── fingerprint.rs       # Complete-state fingerprint construction
│   └── model.rs             # SupportedState V3 and explicit evidence states
├── discovery/
│   ├── filesystem.rs        # Non-following nodes, mounts, hard links, sparse state
│   └── model.rs
├── classification/
│   ├── mod.rs               # V2 migration and complete-state three-way rules
│   └── model.rs             # Metadata dimensions and compatibility findings
├── mutation/
│   ├── filesystem.rs        # Metadata staging, application, and verification
│   ├── model.rs             # Metadata and directory-finalization actions
│   ├── plan.rs              # Dependency ordering and complete preflight
│   ├── execution.rs         # Side-effect-aware action execution
│   └── recovery.rs          # Full-state preservation integration
├── recovery/
│   ├── model.rs             # Recovery Metadata V2
│   └── restore.rs           # Complete metadata restoration and verification
├── state/
│   ├── mod.rs               # State Envelope V2 decode and V3 publication
│   └── publication.rs
├── operation/
│   └── model.rs             # Additive typed metadata details in V1 records
├── baseline.rs              # Explicit legacy-baseline acceptance rules
├── result.rs                # Shared human/JSON compatibility reporting
└── cli.rs                   # Existing command surfaces only

tests/
├── support/
│   └── mod.rs                              # macOS/APFS fixtures and metadata helpers
├── metadata_model.rs                       # Canonical representation unit tests
├── metadata_filesystem_integration.rs      # File/directory metadata round trips
├── metadata_capability_integration.rs      # Permissions and endpoint evidence
├── metadata_migration_integration.rs       # State V2 to V3 workflows
├── metadata_recovery_integration.rs        # Recovery Metadata V2 round trips
├── filesystem_boundary_integration.rs      # Nodes, hard links, sparse files, mounts
├── apfs_name_compatibility_integration.rs  # Case and Unicode collision fixtures
├── metadata_cli_contract.rs                # Paired human/JSON contract tests
├── product_acceptance.rs                   # Feature 001-009 classification and mutation matrix
└── performance_acceptance.rs               # Extended 10,000-entry measurements
```

**Structure Decision**: Preserve the single-crate architecture. Add one cohesive `metadata` module because the same contract is consumed by observation, classification, mutation, recovery, and results. Keep APFS and Darwin specifics behind `metadata::macos`; do not distribute raw FFI or platform constants across domain modules. Extend existing feature modules rather than introducing a repository layer, service process, or cross-platform abstraction.

## Implementation Strategy

### 1. Establish the complete evidence model and strict storage transition

Introduce a typed evidence state that distinguishes `observed`, `absent`, `unavailable`, `unsupported`, `unreadable`, and `unauthorized`. Replace the current file-only `SupportedState` assumptions with a V3 complete entry state containing node kind, optional content, and independent metadata. Extend changed dimensions for mode, owner, group, mtime, each allowlisted xattr, ACL, and BSD flags.

Publish State Envelope V3 rather than adding optional metadata fields to V2. Strictly decode existing State V2 as legacy-incomplete evidence. Read-only commands never rewrite it: equal current copies become `metadata_migration_ready`, differing copies become `metadata_migration_conflict`, and only explicit `baseline accept` or whole-entry `resolve --source|--destination` publishes V3.

### 2. Add descriptor-bound macOS/APFS observation

Collect node identity, device boundary, link count, sparse indicator, mode, numeric ownership, mtime, xattrs, ACL, BSD flags, and endpoint capabilities from non-following descriptors. Preserve semantic ACL entry sequence exactly while canonicalizing only the unordered permission and inheritance-flag sets within each entry. Preserve raw path component bytes. Query filesystem type and case behavior from the concrete volume; qualify Unicode-equivalence behavior against current APFS fixtures rather than claiming a universal filename comparator.

Reject unsupported nodes before payload access. Ordinary files with link count other than one, authoritative sparse indicators, or nested mount boundaries become structured blockers. Unknown xattrs block mutation; explicitly excluded xattrs remain visible diagnostics.

### 3. Extend classification, planning, and public results

Compare complete states without field merging. Report every differing dimension while retaining entry-level conflict and winner selection. Add typed endpoint, path, field, required value, observed capability, and reason to compatibility findings. Preserve current command names and result categories; extend the Result Envelope V1 details additively so paired human and JSON renderers consume the same typed records.

Plan full replacements for content or node changes and metadata transitions for metadata-only changes. Add a directory-finalization action ordered after every descendant, with deeper directories finalized before parents. Finish complete read-only preflight for the selected scope before the first mutation.

### 4. Apply, recover, and verify complete state

For created or replaced entries, stage content under restrictive mode, then apply owner/group, ordered ACL, allowlisted xattrs, final mode, mtime, and supported BSD flags. Apply immutable and append flags last. For existing immutable or append-only targets, represent the required clear-and-restore transition explicitly in preflight and revalidation.

Recovery Metadata V2 records the exact prior logical metadata and verification evidence. Recovery payload objects may retain full xattr bytes on their private filesystem objects while their manifest stores deterministic length and digest references; private recovery permissions remain a storage-security property separate from the recorded original mode. Metadata-only directory changes receive recovery evidence before in-place mutation. Re-read and compare the complete supported state after every action, confirm durability, and publish the accepted State V3 generation only after the entire selected operation succeeds.

### 5. Qualify the macOS/APFS contract

Run the full matrix on the exact recorded macOS, Darwin, APFS bundle, filesystem type, mount flags, and capability masks. Cover file and directory values, absent/present transitions, same- and cross-volume APFS mappings, case-sensitive and case-insensitive volumes, canonical Unicode equivalents, migration paths, authorization blockers, unknown/excluded attributes, unsupported nodes, drift, recovery, partial failure, and paired output contracts.

Extend the existing ignored performance test to execute 100 status and 100 dry-run samples against the documented 10,000-entry tree and report p50, p95, maximum, host, and build data. Measure representative synchronization separately. Introduce optimization only after a concrete measurement misses the two-second target and record the choice below.

### Measured performance correction

The first release-mode exploratory sample missed the two-second target for `status` at 2.833 seconds, push dry-run at 2.159 seconds, pull dry-run at 2.661 seconds, and sync dry-run at 2.754 seconds. Inspection was rediscovering identical endpoint capability profiles for every entry and rereading xattrs and BSD flags while constructing both metadata state and diagnostics.

The smallest correction reuses each endpoint capability profile only within one `observation::inspect` call, keyed by the immutable mapping snapshot, consolidates xattr and BSD-flag reads into one complete metadata observation per entry, and projects the legacy compatibility fingerprint from that complete observation instead of opening and hashing the payload a second time. The map is proportional to mappings rather than entries, never survives the command, never crosses an accepted-state boundary, and does not replace per-action revalidation. Result construction also emits shared endpoint profiles once per mapping, emits each action's complete before/after state through its metadata evidence rather than duplicating the same state in legacy source/destination fields, and omits only synchronized no-action entry repetitions while retaining the exact selected and no-action counts. Unmanaged and other diagnostic no-action entries remain explicit.

Parallel traversal was rejected because it would complicate deterministic ordering and resource bounds. A persistent index or cross-operation cache was rejected because it would require invalidation and stronger filesystem identity guarantees. Repeating all capability system calls per entry was simpler but had already missed the acceptance threshold. The selected operation-local reuse adds small mapping-count memory and local lookup cost while avoiding those correctness and maintenance risks.

## Dependency Approval Gate

The existing `rustix 1.1` dependency covers xattrs, ownership, modes, timestamps, and filesystem stats. Darwin ACL APIs, `fchflags`, and volume capability queries are not all exposed by the current dependency surface. The approved implementation is a tiny `cfg(target_os = "macos")` FFI adapter using a direct `libc 0.2` declaration rather than shelling out, maintaining handwritten system declarations, or adding a broad filesystem crate. The dependency choice was explicitly approved on 2026-09-07.

## Post-Design Constitution Re-check

Phase 1 adds only durable schemas, explicit contracts, and tests required by the specification. The State V3 and Recovery Metadata V2 transitions avoid silent reinterpretation; the adapter stays local and unprivileged; complete preflight, revalidation, recovery, verification, and ownership boundaries remain intact. No new background process, broad lock, cross-operation cache, persistent index, cross-platform promise, or unrelated feature entered the design. The operation-local endpoint-profile reuse described above followed a measured threshold miss and remains bounded by the same snapshot and revalidation rules. All gates remain passed.

## Complexity Tracking

No constitutional violations require justification.
