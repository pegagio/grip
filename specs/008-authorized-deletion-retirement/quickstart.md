# Quickstart: Validate Authorized Deletion and Retirement

This guide defines disposable end-to-end evidence for Feature 008. Detailed behavior is in the [CLI](contracts/cli.md), [deletion](contracts/deletion.md), [retirement](contracts/retirement.md), [recovery](contracts/recovery.md), [storage](contracts/storage.md), and [filesystem](contracts/filesystem.md) contracts.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Build and regression baseline](#build-and-regression-baseline)
- [Preview and execute directional deletion](#preview-and-execute-directional-deletion)
- [Protect unmanaged directory descendants](#protect-unmanaged-directory-descendants)
- [Retire accepted evidence](#retire-accepted-evidence)
- [Inspect and restore recovery](#inspect-and-restore-recovery)
- [Clean recovery bytes](#clean-recovery-bytes)
- [Verify failures and authority](#verify-failures-and-authority)
- [Run complete gates](#run-complete-gates)

## Prerequisites

From the repository root:

```bash
mise trust
mise install
```

All scenarios and automated tests must use disposable `HOME`, `GRIP_HOME`, source, and destination roots. Never use real managed files. The examples below use placeholder paths beneath `/tmp`; the test harness creates and removes its own isolated roots.

## Build and regression baseline

```bash
mise run build
mise run test
```

Existing mapping, discovery, classification, baseline, push, pull, sync, resolution, recovery, operation-record, state, and registry tests must remain green.

## Preview and execute directional deletion

Create one accepted fixture in which the source is absent and the unchanged destination remains. From equivalent snapshots, run:

```bash
grip --output json delete --dry-run --source -- /tmp/grip-source/config
grip --output json delete --source -- /tmp/grip-source/config
grip --output json status
```

Expected evidence:

- Preview and execution expose the same plan identity and destination removal action; preview changes no payload or Grip state.
- Execution preserves verified destination recovery before descriptor-safe removal.
- Final observation proves both sides absent, then exactly the selected baseline record is retired.
- Status no longer reports the retired accepted identity; unrelated baselines and payloads are unchanged.

Repeat from a symmetric fixture with destination absent and `--destination`; the source is the only removal target.

## Protect unmanaged directory descendants

Create an accepted managed directory deletion whose remaining directory contains both eligible managed children and one unmanaged child. Run:

```bash
grip --output json delete --dry-run --source -- /tmp/grip-source/tree
grip --output json delete --source -- /tmp/grip-source/tree
```

Expected evidence:

- Both commands identify the unmanaged descendant and block the complete plan.
- No mutation lock side effect, operation record, recovery entry, payload removal, or baseline publication occurs.
- After independently relocating the unmanaged child, the rebuilt plan orders files and child directories before parents and removes a directory only after fresh empty enumeration.

## Retire accepted evidence

Prepare newly ignored, removed-mapping, converged-deletion, and active records. Exercise exact and explicit-all selection:

```bash
grip --output json retire --dry-run -- /tmp/grip-source/tree/old.conf
grip --output json retire -- /tmp/grip-source/tree/old.conf
grip --output json retire --dry-run --all
```

Expected evidence:

- Bare `grip retire` is rejected; a path or `--all` is mandatory.
- Eligible selected records are removed from one newly published State V2 generation without payload mutation.
- Active, unsafe, unsupported, and incomplete entries block regardless of force.
- A pending-retirement entry with differing surviving copies reports `force_required`; the same reviewed request with `--force` may discard its selected comparison history while leaving both copies unchanged.

## Inspect and restore recovery

Produce payload, registry, and accepted-state recovery through isolated prior operations, then run:

```bash
grip --output json recovery list
grip --output json recovery show payload:OPERATION:0
grip --output json recovery restore --dry-run -- payload:OPERATION:0
grip --output json recovery restore -- payload:OPERATION:0
```

Use actual opaque references returned by list; do not construct private paths.

Expected evidence:

- List and show report typed references, provenance, integrity, availability, byte count, bound target, and eligibility without printing file content.
- Restore accepts an absent target or an occupied target exactly equal to recorded post-action evidence; an occupied eligible target is preserved first.
- A successful payload restore reproduces exact supported state, retains the original recovery entry, and does not accept a baseline automatically.
- A legacy entry without sufficient immutable provenance remains inspectable but blocks with `legacy_provenance_incomplete`.

For missing/corrupt registry and accepted-state fixtures, verify an exact recovered document can restore only when its manifest, bytes, schema, remaining authority, ownership, paths, and live payload compatibility all validate. A different valid current authority must block.

## Clean recovery bytes

From a fixture with at least two available recovery entries, run:

```bash
grip --output json recovery remove --dry-run --confirm REF_A REF_B
grip --output json recovery remove --confirm REF_A REF_B
grip --output json recovery show REF_A
```

Expected evidence:

- Preview reports the same ordered byte-removal plan and changes nothing.
- Execute requires exact unique references and confirmation.
- Each success removes only recoverable bytes, verifies absence, and publishes an immutable cleanup tombstone while retaining manifest and operation provenance.
- Show reports `cleaned`; it never treats missing bytes without a valid tombstone as completed cleanup.
- An injected first failure reports completed, failed, and unattempted entries without claiming multi-reference atomicity.

## Verify failures and authority

Use typed test-only faults at recovery preservation, unlink, directory sync, absence verification, final observation, State V2 publication, registry/state restore publication, cleanup unlink, tombstone publication, and result delivery.

Expected evidence:

- Lock-held evidence drift fails before operation initialization or mutation.
- The first side-effect failure stops later actions and preserves every completed effect and recovery entry.
- Partial deletion never retires the baseline.
- Registry/state restore reports actual visibility and durability after publication faults.
- Cleanup interrupted after byte removal but before tombstone publication reports `cleanup_incomplete`; a fresh confirmed invocation may finish the tombstone after revalidation.
- Result-delivery failure never revokes already visible payload or authority state.

Focused suites should include:

```bash
cargo test --test delete_cli_contract
cargo test --test delete_planning
cargo test --test delete_filesystem_integration
cargo test --test delete_recovery_integration
cargo test --test delete_failure_integration
cargo test --test delete_contention_integration
cargo test --test retire_cli_contract
cargo test --test retire_integration
cargo test --test recovery_cli_contract
cargo test --test recovery_inventory_integration
cargo test --test recovery_restore_integration
cargo test --test recovery_authority_restore_integration
cargo test --test recovery_cleanup_integration
cargo test --test recovery_storage_integration
cargo test --test recovery_filesystem_integration
cargo test --test operation_record_integration
```

## Run complete gates

```bash
mise run validate
mise run performance
```

`validate` must pass formatting, Clippy with warnings denied, all default tests, and the release build. The ignored performance harness must document OS, architecture, Rust version, profile, fixture construction, and 100-run distributions. At least 95 of 100 warm deletion previews over 10,000 managed entries and recovery inventories over 10,000 retained entries must complete within two seconds without a new cache, persistent index, background service, broad payload lock, or unmeasured parallelism.

Representative-workstation qualification on macOS/AArch64 with Rust 1.98 and the release profile completed 100 warm runs on 2026-09-07. The observed p95 was 1.055 seconds for deletion preview over 10,000 managed entries and 1.412 seconds for integrity-verified recovery inventory over 10,000 retained operation entries; both satisfied the two-second acceptance threshold. The complete performance suite passed in 924.79 seconds.

Before implementation, generate tasks with `$speckit-tasks` and run `$speckit-analyze` as required by the constitution.
