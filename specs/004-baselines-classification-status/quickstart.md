# Quickstart: Validate Baselines, Classification, and Status

This guide validates Feature 004 end to end in one disposable root. It proves read-only classification, explicit state-only baseline acceptance, three-way differences, selection, pending retirement, error exits, and representative performance.

## Prerequisites

- macOS or a supported Unix-like system
- `mise` trusted for this checkout
- The pinned Rust toolchain installed through `mise install`

## Build and static validation

From the repository root:

```bash
mise trust
mise install
mise exec -- cargo fmt --check
mise exec -- cargo clippy --all-targets --all-features -- -D warnings
mise exec -- cargo test
mise exec -- cargo build --release
```

All filesystem tests must use isolated Grip homes and payload roots. They must never inspect or mutate the operator's real Grip home or files.

## Create an isolated equivalent mapping

```bash
grip_baseline_root="$(mktemp -d)"
mkdir -p "$grip_baseline_root/grip-home" "$grip_baseline_root/source/editor/themes" "$grip_baseline_root/destination/editor/themes"
printf '%s\n' 'schema_version = 1' 'mappings = []' > "$grip_baseline_root/grip-home/config.toml"
printf '%s\n' 'dark' > "$grip_baseline_root/source/editor/themes/current.txt"
cp "$grip_baseline_root/source/editor/themes/current.txt" "$grip_baseline_root/destination/editor/themes/current.txt"
chmod 0644 "$grip_baseline_root/source/editor/themes/current.txt" "$grip_baseline_root/destination/editor/themes/current.txt"
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- mapping add tree "$grip_baseline_root/source/editor" "$grip_baseline_root/destination/editor"
printf '%s\n' "$grip_baseline_root"
```

Inspect the printed path and confirm it is the disposable root before continuing.

## Observe an unaccepted initial match

```bash
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- status
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- --output json diff
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- check
printf 'check exit=%s\n' "$?"
```

Expected outcome: status and diff exit `0`; the file is `initial_match`; all three diff comparison fields are present, with baseline comparisons unavailable and source-to-destination empty. Check exits `1` with `attention_required` because equivalent evidence has not been accepted.

Before baseline acceptance, capture source and destination bytes and metadata with the test support snapshot utility or an equivalent fixture assertion. Do not use repository or real home paths as fixtures.

## Accept and reaccept the baseline

```bash
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- baseline accept
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- status
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- check
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- --output json baseline accept
```

Expected outcome: the first acceptance publishes State Envelope V2 generation 0 without changing payloads. Status reports `synchronized`; check exits `0`. The second acceptance reports `already_current`, `published: false`, and generation 0; no new recovery generation or staging file appears.

Decode `state/state.json` through the storage contract tests rather than editing it. Confirm it contains ordered fingerprints and no file content or mtime.

## Exercise three-way classification

Create source-only content drift:

```bash
printf '%s\n' 'source edit' > "$grip_baseline_root/source/editor/themes/current.txt"
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- --output json diff
```

Expected outcome: `source_only_change`; source-to-baseline and source-to-destination contain `content`, destination-to-baseline is empty, and no payload or baseline changes.

Restore and accept equivalent content, then create destination-only permission drift:

```bash
cp "$grip_baseline_root/source/editor/themes/current.txt" "$grip_baseline_root/destination/editor/themes/current.txt"
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- baseline accept
chmod 0600 "$grip_baseline_root/destination/editor/themes/current.txt"
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- diff
```

Expected outcome: `destination_only_change` with `permission_mode` in destination-to-baseline and source-to-destination.

Create divergent content:

```bash
printf '%s\n' 'source conflict' > "$grip_baseline_root/source/editor/themes/current.txt"
printf '%s\n' 'destination conflict' > "$grip_baseline_root/destination/editor/themes/current.txt"
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- check
printf 'check exit=%s\n' "$?"
```

Expected outcome: a complete `divergent_conflict`, blocking and attention counts of one, and check exit `1`; this feature does not resolve or copy either side.

Exercise deletion and timestamp-only behavior through `classification_filesystem_integration` so fixtures can restore exact baselines and assert that mtime alone never changes equality.

## Validate source and destination selection

```bash
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- status -- "$grip_baseline_root/source/editor/themes"
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- status --destination -- "$grip_baseline_root/destination/editor/themes"
```

Both selectors resolve to the same component-boundary subtree and return equivalent records. Add a fixture whose selected path begins with `-` and prove `--` preserves it as a path. More than one selector must exit `2`.

## Validate scoped acceptance preservation

Add a second equivalent file, then accept only it:

```bash
printf '%s\n' 'second' > "$grip_baseline_root/source/editor/second.txt"
cp "$grip_baseline_root/source/editor/second.txt" "$grip_baseline_root/destination/editor/second.txt"
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- baseline accept -- "$grip_baseline_root/source/editor/second.txt"
```

Expected outcome: only the selected record changes. Existing current, drifted, and pending-retirement baseline records remain byte-semantically unchanged in the new full state generation. The integration test must decode both generations and compare every out-of-scope record.

## Validate untracked pending retirement

After restoring both sides to a baseline-acceptable state, accept and remove the mapping:

```bash
cp "$grip_baseline_root/source/editor/themes/current.txt" "$grip_baseline_root/destination/editor/themes/current.txt"
chmod 0644 "$grip_baseline_root/source/editor/themes/current.txt" "$grip_baseline_root/destination/editor/themes/current.txt"
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- baseline accept
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- mapping remove "$grip_baseline_root/source/editor"
GRIP_HOME="$grip_baseline_root/grip-home" mise exec -- cargo run -- status
```

Expected outcome: mapping removal leaves accepted state unchanged. Status reports retained records as `untracked_pending_retirement` without reopening former payload paths. Check reports attention. Feature 004 performs no retirement or deletion.

## Run focused suites

```bash
mise exec -- cargo test --test classification_matrix
mise exec -- cargo test --test classification_filesystem_integration
mise exec -- cargo test --test classification_cli_contract
mise exec -- cargo test --test baseline_integration
mise exec -- cargo test --test state_integration
```

The suites must cover all classifications, all three comparison fields, V1/V2/absent state, whole-request rejection, no-op behavior, scoped preservation, lock contention, publication visibility, unsupported nodes, stale file reads, two-pass drift, and read-only before/after snapshots.

## Measure representative performance

```bash
mise exec -- cargo build --release
mise exec -- cargo test --release --test performance_acceptance -- --ignored --nocapture
```

The Feature 004 harness creates 10,000 equivalent paired files with accepted baselines, warms JSON status, measures exactly 100 complete runs, verifies byte-equivalent stdout and stderr, records environment details, and requires at least 95 runs within two seconds. Sequential behavior remains the default unless this evidence justifies optimization.

## Cleanup

After confirming the printed path is the disposable root created above, remove only that exact directory using the environment's normal recoverable cleanup workflow. Never adapt cleanup to a broad path, an empty variable, a real home directory, or the repository root.

## Recorded validation

Feature 004 was validated on 2026-09-05 on macOS/aarch64 with Rust 1.98. The isolated integration suites exercised the initial-match, acceptance/reacceptance, source-only, destination-only permission, divergent-conflict, deletion, timestamp-only, source/destination selector, scoped-preservation, pending-retirement, error-exit, publication-fault, and non-mutation scenarios above. All default tests passed, including 100 byte-equivalent unchanged-evidence runs for each of status, check, and diff.

`mise run validate` passed formatting, Clippy with warnings denied, all default tests, and the release build. Mise emitted sandbox-only cache-write warnings outside the checkout; they did not affect validation.

The ignored release harness passed 100 warm runs with this environment and p95 measurements: help 1.715ms, version 1.741ms, validate 711.726ms, 1,000-mapping list 713.312ms, 10,000-entry discovery 195.440ms, and accepted paired 10,000-entry JSON status 1.952s. Output and diagnostics were byte-equivalent across the measured status runs, and no cache, persistent index, parallel traversal, or background service was introduced.

After convergence hardening, the exact-current implementation was revalidated on 2026-09-06 with the same macOS/aarch64 Rust 1.98 release harness. The p95 measurements were: help 2.489ms, version 2.308ms, validate 693.606ms, 1,000-mapping list 719.578ms, 10,000-entry discovery 213.455ms, and accepted paired 10,000-entry JSON status 1.346s. All 100 measured status runs remained byte-equivalent and the two-second acceptance threshold passed.
