# Quickstart: Validate Mapping Registry and Ownership

This guide defines the end-to-end checks that demonstrate Feature 002 after implementation. All created paths remain beneath one disposable root.

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

## Create an isolated registry

```bash
grip_plan_root="$(mktemp -d)"
mkdir -p "$grip_plan_root/grip-home" "$grip_plan_root/source/editor" "$grip_plan_root/destination"
touch "$grip_plan_root/source/gitconfig"
printf '%s\n' 'schema_version = 1' 'mappings = []' > "$grip_plan_root/grip-home/config.toml"
```

Inspect the printed value before using it in later commands:

```bash
printf '%s\n' "$grip_plan_root"
```

## Add and inspect mappings

```bash
GRIP_HOME="$grip_plan_root/grip-home" mise exec -- cargo run -- mapping add file "$grip_plan_root/source/gitconfig" "$grip_plan_root/destination/gitconfig"
GRIP_HOME="$grip_plan_root/grip-home" mise exec -- cargo run -- mapping add tree "$grip_plan_root/source/editor" "$grip_plan_root/destination/editor"
GRIP_HOME="$grip_plan_root/grip-home" mise exec -- cargo run -- mapping list
GRIP_HOME="$grip_plan_root/grip-home" mise exec -- cargo run -- --output json mapping show "$grip_plan_root/source/gitconfig"
```

Expected outcome: both additions succeed without creating either destination, list output is canonical-source ordered, show returns one complete file mapping, synchronization state remains absent, and verified prior-registry generations exist beneath `state/recovery/registry/`. The accepted TOML and recovery shapes are defined in [the storage contract](contracts/storage.md).

## Reject ambiguous ownership

```bash
mkdir -p "$grip_plan_root/source/editor/themes"
GRIP_HOME="$grip_plan_root/grip-home" mise exec -- cargo run -- --output json mapping add tree "$grip_plan_root/source/editor/themes" "$grip_plan_root/destination/themes"
```

Expected outcome: exit `10`, `invalid_configuration`, and a stable source-overlap reason. `config.toml` remains unchanged. Continue with equal endpoints and a cross-mapping recursive relationship from [the topology contract](contracts/topology.md); each must be rejected before publication.

## Remove intent without removing data

```bash
GRIP_HOME="$grip_plan_root/grip-home" mise exec -- cargo run -- mapping remove "$grip_plan_root/source/gitconfig"
GRIP_HOME="$grip_plan_root/grip-home" mise exec -- cargo run -- --output json mapping list
```

Expected outcome: the file mapping is absent, the tree mapping remains, the source file remains unchanged, and no destination or synchronization state was created.

## Validate publication safety

Run the targeted integration tests:

```bash
mise exec -- cargo test --test mapping_registry_integration
mise exec -- cargo test --test mapping_topology_integration
mise exec -- cargo test --test mapping_cli_contract
```

The suites must cover complete-registry rejection, canonical identity, absent destinations, endpoint symlinks, intermediate ancestry, non-UTF-8 rejection, stale accepted bytes, stale path evidence, lock contention, prior-registry recovery creation/reuse/collision/failure, interruption before rename, deterministic serialization, exact safe-mode preservation, and zero unintended mutation.

## Measure representative performance

```bash
mise exec -- cargo build --release
mise exec -- cargo test --release --test performance_acceptance -- --ignored --nocapture
```

The Feature 002 harness creates 1,000 disjoint mappings under its isolated root, performs one warm-up, measures exactly 100 list/validation runs, records the environment, and requires at least 95 runs to complete within one second.

## Cleanup

After confirming that the printed path is the disposable root created above, remove only that exact directory using the environment's normal recoverable cleanup workflow. Never adapt cleanup to a broad path, an empty variable, a real home directory, or the repository root.

## Validation results

Validated on 2026-09-03 on macOS ARM64 with Rust 1.98.0. The isolated workflow added file and tree mappings, returned two canonical mappings in source order, showed one exact mapping, rejected a nested source with exit 10 while preserving the accepted registry bytes, and removed only the selected intent. The source remained present, both absent destinations remained absent, `state/state.json` remained absent, the accepted `0640` registry mode was preserved, and three exact prior-registry recovery generations were present after the successful updates.

`mise run validate` passed formatting, Clippy with warnings denied, all default tests, and the release build. The final ignored release acceptance harness passed 100 measured runs over 1,000 mappings with byte-identical stdout and stderr on every list run and p95 results of 2.67 ms for help, 1.68 ms for version, 713.69 ms for complete-registry validate, and 746.61 ms for deterministic mapping list.
