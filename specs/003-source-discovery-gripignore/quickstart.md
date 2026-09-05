# Quickstart: Validate Source Discovery and Gripignore

This guide defines end-to-end validation for Feature 003. Every fixture uses one disposable root and the command remains read-only.

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

The test suite must use isolated Grip homes and payload roots. It must not inspect or mutate the operator's real Grip home or files.

## Create an isolated mapping

```bash
grip_discovery_root="$(mktemp -d)"
mkdir -p "$grip_discovery_root/grip-home" "$grip_discovery_root/source/editor/themes/empty" "$grip_discovery_root/source/editor/cache" "$grip_discovery_root/destination/editor/local"
printf '%s\n' 'visible' > "$grip_discovery_root/source/editor/config.txt"
printf '%s\n' 'ignored' > "$grip_discovery_root/source/editor/cache/session.dat"
printf '%s\n' 'dark' > "$grip_discovery_root/source/editor/themes/dark.txt"
printf '%s\n' 'local' > "$grip_discovery_root/destination/editor/local/database.dat"
printf '%s\n' 'cache/' > "$grip_discovery_root/source/editor/.gripignore"
printf '%s\n' '*.tmp' '!keep.tmp' > "$grip_discovery_root/source/editor/themes/.gripignore"
```

Create the registry through the accepted mapping command:

```bash
printf '%s\n' 'schema_version = 1' 'mappings = []' > "$grip_discovery_root/grip-home/config.toml"
GRIP_HOME="$grip_discovery_root/grip-home" mise exec -- cargo run -- mapping add tree "$grip_discovery_root/source/editor" "$grip_discovery_root/destination/editor"
```

Inspect the printed disposable root before continuing:

```bash
printf '%s\n' "$grip_discovery_root"
```

## Inspect membership

Run both human and machine-readable inspection:

```bash
GRIP_HOME="$grip_discovery_root/grip-home" mise exec -- cargo run -- mapping inspect "$grip_discovery_root/source/editor"
GRIP_HOME="$grip_discovery_root/grip-home" mise exec -- cargo run -- --output json mapping inspect "$grip_discovery_root/source/editor"
```

Expected outcome: `config.txt`, `themes`, `themes/dark.txt`, and `themes/empty` are eligible; `cache` is one ignored exclusion root; both `.gripignore` files are policy-only and absent from payload records; `local` and its descendants are destination-only; output order is deterministic; `blocking_count` is zero.

## Prove ignore authority

Add authority-poison files that would hide an eligible sentinel if Grip consulted them:

```bash
printf '%s\n' 'sentinel.txt' > "$grip_discovery_root/source/editor/.gitignore"
printf '%s\n' 'sentinel.txt' > "$grip_discovery_root/source/editor/.ignore"
printf '%s\n' 'visible' > "$grip_discovery_root/source/editor/sentinel.txt"
```

Repeat discovery. `sentinel.txt`, `.gitignore`, `.ignore`, and hidden entries remain eligible unless `.gripignore` itself excludes them. Run the fixed corpus in [the Gripignore contract](contracts/gripignore.md) to verify precedence, escaping, wildcards, negation, BOM, CRLF, and excluded-parent pruning.

## Report unsupported boundaries

Create an isolated symbolic link and hard link:

```bash
ln -s "$grip_discovery_root/source/editor/config.txt" "$grip_discovery_root/source/editor/config-link"
ln "$grip_discovery_root/source/editor/config.txt" "$grip_discovery_root/source/editor/config-hardlink"
```

Repeat JSON discovery. The link is `unsupported_source` with reason `symlink`. Both hard-linked names are unsupported with reason `hard_link`. The command still returns a complete `ok` inventory with a nonzero `blocking_count` and does not open either target as payload.

Exercise platform-available sparse, FIFO, socket, non-UTF-8, and nested-mount fixtures through the integration suite rather than placing privileged nodes on the developer filesystem.

## Validate stale evidence and zero mutation

Run the focused suites:

```bash
mise exec -- cargo test --test gripignore_conformance
mise exec -- cargo test --test discovery_filesystem_integration
mise exec -- cargo test --test discovery_cli_contract
```

The tests must compare before/after snapshots of the Grip home, source, and destination; verify deterministic stale changes through the test-only pass barrier; and prove that success, blockers, malformed policy, unreadable entries, stale evidence, and output failure create no files or metadata changes.

## Measure representative performance

```bash
mise exec -- cargo build --release
mise exec -- cargo test --release --test performance_acceptance -- --ignored --nocapture
```

The Feature 003 harness creates one representative 10,000-entry tree with nested policy, warms the command, measures exactly 100 complete two-pass discoveries, verifies byte-equivalent output on every run, records the environment, and requires at least 95 runs to complete within two seconds.

## Observed validation

Validation was executed on 2026-09-05 using macOS on ARM64 with Rust 1.98.0. The isolated quickstart produced four eligible records, one ignored exclusion root, two destination-only records, and zero blockers. The authority-poison files remained eligible, and the link fixture produced the expected symbolic-link blocker plus two hard-link blockers while returning a complete successful inventory.

The final release performance harness completed 100 deterministic two-pass inspections of the representative 10,000-entry tree with a p95 of 129.403125 ms. The same run measured help at 2.322666 ms, version at 1.903167 ms, validation over 1,000 mappings at 709.340208 ms, and mapping-list output over 1,000 mappings at 729.250042 ms. All applicable thresholds passed.

## Cleanup

After confirming the printed path is the disposable root created above, remove only that exact directory using the environment's normal recoverable cleanup workflow. Never adapt cleanup to a broad path, an empty variable, a real home directory, or the repository root.
