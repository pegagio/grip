# Quickstart: Validate the Feature 001 Foundation

This guide defines the end-to-end checks that demonstrate the planned feature. Commands become runnable after implementation; all filesystem effects remain under an isolated temporary root.

## Prerequisites

- macOS or a supported Unix-like system with required advisory locking and atomic same-directory rename
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

These checks must not access a real Grip home. The test suite creates isolated roots and invokes the compiled binary directly.

## Validate a minimal isolated home

Create a disposable test root in your shell and keep both `HOME` and `GRIP_HOME` inside it:

```bash
grip_test_root="$(mktemp -d)"
mkdir -p "$grip_test_root/home" "$grip_test_root/grip-home"
printf '%s\n' 'schema_version = 1' 'mappings = []' > "$grip_test_root/grip-home/config.toml"
HOME="$grip_test_root/home" GRIP_HOME="$grip_test_root/grip-home" mise exec -- cargo run -- validate
```

Expected outcome: exit `0`, the exact alternate root is reported, the registry is valid, state is uninitialized, and no `state/` directory is created.

## Validate machine-readable output

```bash
HOME="$grip_test_root/home" GRIP_HOME="$grip_test_root/grip-home" mise exec -- cargo run -- --output json validate
HOME="$grip_test_root/home" GRIP_HOME="$grip_test_root/grip-home" mise exec -- cargo run -- --output json version
```

Each invocation must emit one JSON object on stdout with the five fields in [the CLI contract](contracts/cli.md), emit no diagnostics by default, and exit `0`.

## Validate path-policy failure

```bash
HOME="$grip_test_root/home" GRIP_HOME="relative/path" mise exec -- cargo run -- --output json validate
```

Expected outcome: exit `10`, symbolic code `invalid_configuration`, no fallback to the default home, and no filesystem creation.

Run the same command with an absolute nonexistent `GRIP_HOME`, then with an existing root whose `config.toml` is absent. Both cases must return exit `10`, report `invalid_configuration`, and create nothing.

## Validate schema failures

Change `schema_version` to `2` and rerun validation. Expected outcome: exit `11` and `unsupported_schema`. Restore version `1`, add an unknown field, and rerun. Expected outcome: exit `10` and `invalid_configuration`.

State examples and integrity rules are defined in [the storage contract](contracts/storage.md). Automated integration tests must cover absent, valid, unsupported, malformed, unknown-field, and digest-mismatch state without writing outside the disposable root.

## Validate publication safety

Run the targeted integration tests after implementation:

```bash
mise exec -- cargo test --test state_integration
```

The suite must prove old-or-new atomic visibility, verified retention of the prior generation before replacement, recovery-copy failure blocking, byte-identical retry reuse, conflicting recovery rejection, interruption before rename, one-writer lock contention, crash-release behavior, final-component symlink and wrong-node rejection, `0700`/`0600` creation, and injected write/sync/verify/rename failures. Lock contention is an internal publication result in Feature 001; public exit `13` remains reserved. The exact publication sequence is defined in [the storage contract](contracts/storage.md).

## Measure acceptance performance

Build once in release mode, then run the ignored workstation harness:

```bash
mise exec -- cargo build --release
mise exec -- cargo test --release --test performance_acceptance -- --ignored --nocapture
```

The harness records OS, filesystem, hardware, compiler, profile, and load assumptions; performs an untimed warm-up; measures exactly 100 subprocess invocations per scenario; and verifies the 95th ordered result against the thresholds in [the specification](spec.md). Shared CI timing is informative, not the workstation acceptance authority.

## Clean up

After inspecting the disposable root, remove only the exact path printed by your shell:

```bash
printf '%s\n' "$grip_test_root"
rm -rf -- "$grip_test_root"
```

Do not adapt this cleanup command to a broad path, an unresolved variable, or a real home directory.

## Verified implementation results

Verified on 2026-09-03 using macOS on ARM64 and Rust 1.98.0. Formatting, Clippy with warnings denied, the complete test suite, and the release build passed. The isolated quickstart produced valid human and JSON success results, returned exit `10` for relative and nonexistent alternate homes, and did not create `state/` during validation.

The release acceptance harness measured 100 warm runs per scenario on this workstation: help p95 `2.99 ms`, version p95 `1.64 ms`, and minimal-home validation p95 `1.68 ms`. All results were below their specified thresholds. These workstation measurements do not extend the durability guarantees to unsupported or user-space filesystems.
