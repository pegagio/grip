# Quickstart: Validate Bidirectional Synchronization and Conflict Resolution

This guide defines disposable end-to-end evidence for Feature 007. Detailed behavior is in the [CLI](contracts/cli.md), [sync](contracts/sync.md), [resolution](contracts/resolution.md), [filesystem](contracts/filesystem.md), and [storage](contracts/storage.md) contracts.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Build and regression baseline](#build-and-regression-baseline)
- [Preview and execute mixed sync](#preview-and-execute-mixed-sync)
- [Verify complete conflict blocking](#verify-complete-conflict-blocking)
- [Resolve both winner directions](#resolve-both-winner-directions)
- [Verify converged acceptance](#verify-converged-acceptance)
- [Verify failures and recovery](#verify-failures-and-recovery)
- [Run complete gates](#run-complete-gates)

## Prerequisites

From the repository root:

```bash
mise trust
mise install
```

All manual scenarios and tests must use disposable `HOME`, `GRIP_HOME`, source, and destination roots. Never use real managed files.

## Build and regression baseline

```bash
mise run build
mise run test
```

Existing push, pull, classification, baseline, operation-record, state, mapping, and discovery suites must remain green.

## Preview and execute mixed sync

Create isolated accepted entries containing one source-only change, one destination-only change, one synchronized entry, and one unmanaged destination-only sibling.

```bash
grip --output json sync --dry-run
grip --output json sync
grip --output json status
```

Expected evidence:

- Preview reports one push and one pull action in canonical Entry Identity order and changes no payload or Grip state.
- The unmanaged destination sibling is a reported non-action and is never imported.
- Execute preserves each replaced target, verifies both transfers, and publishes exactly one accepted generation.
- Status then reports every managed pair synchronized.

## Verify complete conflict blocking

Add divergent source/destination changes to one accepted entry while leaving eligible one-sided changes elsewhere in the selected scope.

```bash
grip --output json sync --dry-run
grip --output json sync
```

Expected evidence:

- Both results include the complete blocker set and the otherwise eligible directional actions.
- Execute starts zero actions, acquires no mutation side effect, creates no operation/recovery record, and publishes no baseline.

## Resolve both winner directions

For one exact divergent entry identified by its source path, exercise both decisions from equivalent fixtures:

```bash
grip --output json resolve --dry-run --source -- /tmp/grip-source/config
grip --output json resolve --source -- /tmp/grip-source/config
grip --output json resolve -n --destination -- /tmp/grip-source/config
grip --output json resolve --destination -- /tmp/grip-source/config
```

Expected evidence:

- Each preview matches its corresponding execution plan over unchanged evidence and mutates nothing.
- Source winner produces one push action and preserves the destination; destination winner produces one pull action and preserves the source.
- Both successful executions verify complete supported equality and publish one baseline.
- Destination paths passed as `PATH`, missing/conflicting winner flags, and multi-entry selectors are rejected.

## Verify converged acceptance

Change both sides of an accepted entry to the same complete supported state while leaving another entry synchronized.

```bash
grip --output json sync --dry-run
grip --output json sync
grip --output json status
```

Expected evidence:

- Preview identifies the converged entry as accept-only with no payload action.
- Execute creates no payload recovery or staging entry, but coordinates and publishes one accepted generation for the converged state.
- A subsequent synchronized-only sync is a no-op and creates no operation record or generation.

## Verify failures and recovery

Use typed test-only fault injection at mixed-direction recovery, staging, publication, verification, final observation, baseline publication, and result delivery boundaries.

Expected evidence:

- The first failure stops later actions independent of direction.
- Completed effects remain, losing-target recovery remains verified and operation-local, and the prior baseline stays authoritative unless a new generation is explicitly reported visible.
- Converged entries are not accepted after any payload failure.
- A later invocation performs fresh inspection and never resumes a prior nonterminal record.
- Result-delivery failure after successful acceptance returns exit `20` without revoking payload or baseline authority.

Focused suites should include:

```bash
cargo test --test sync_cli_contract
cargo test --test sync_planning
cargo test --test sync_filesystem_integration
cargo test --test sync_failure_integration
cargo test --test sync_recovery_integration
cargo test --test sync_contention_integration
cargo test --test resolve_cli_contract
cargo test --test resolve_planning
cargo test --test resolve_filesystem_integration
cargo test --test operation_record_integration
```

## Run complete gates

```bash
mise run validate
mise run performance
```

`validate` must pass formatting, Clippy with warnings denied, all default tests, and the release build. The ignored performance harness must document OS, architecture, Rust version, profile, fixture mix, and 100-run p95 results. Sync preview and pure planning over 10,000 mixed classifications must complete within two seconds in at least 95 of 100 warm runs without a new cache, background service, broad payload lock, or unmeasured parallelism.

Feature 007 acceptance evidence recorded on 2026-09-07:

- **Environment**: macOS on ARM64, Rust 1.98.0, release profile, 100 warm measured runs after one unmeasured warm-up.
- **Fixture**: 10,000 accepted file entries containing synchronized, source-only, destination-only, converged, and divergent-conflict classifications; the resulting sync plan contained actions in both directions, accept-only entries, and blockers.
- **Sync preview p95**: 1.146991333 seconds for `grip --output=json sync --dry-run`.
- **Pure sync planning p95**: 0.942500750 seconds for repeated inspect, classify, and deterministic plan construction.
- **Result**: Both Feature 007 measurements passed the two-second SC-009 threshold. The complete `mise run performance` harness passed in 739.56 seconds.

Before implementation, generate tasks with `$speckit-tasks` and run `$speckit-analyze` as required by the constitution.
