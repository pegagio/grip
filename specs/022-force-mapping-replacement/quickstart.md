# Quickstart: Validate Force Mapping Replacement

Use the project’s isolated filesystem tests to validate this metadata-only feature. Do not run examples against real managed files.

## Prerequisites

- Run commands from the repository root.
- Ensure the configured Rust toolchain is available with `mise install` if needed.
- Review the [CLI contract](contracts/force-add-replacement.md) and [data model](data-model.md) before interpreting a failure.

## Focused Validation

Run the feature’s primary integration suite while developing:

```bash
cargo test --test mapping_cli_contract
cargo test --test mapping_topology_integration
cargo test --test state_integration
```

The focused fixtures must prove all of the following:

- A distinct source can replace exactly one equal-destination file mapping with `grip add --force`, while both endpoint payloads remain byte-for-byte unchanged.
- Human output identifies the old and new mapping, and JSON exposes both `mapping` and `replaced_mapping`.
- Ordinary add remains rejected while an active owner exists.
- Removing the exact owner permits a different source to be added normally; removing an unrelated source does not.
- Force preserves source-overlap, tree, nested, multiple-match, endpoint-validation, and topology rejection behavior.
- An injected publication or drift failure completes or restores the whole descriptor/state pair, removes its fence only after verification, and never leaves stale displaced evidence.

## Full Validation

Run the complete project validation before review:

```bash
mise run validate
```

This checks formatting, linting, all default tests, and the release build. The performance acceptance task is intentionally separate and is not required for this metadata behavior:

```bash
mise run performance
```

## Manual Contract Smoke Test

In a disposable temporary project with temporary source and destination files, create an initial file mapping and replace it with a different source pointed at the same destination:

```bash
grip add target/debug/grip ~/.local/bin/grip
grip add --force target/release/grip ~/.local/bin/grip
grip list
grip status
```

Expected outcomes are defined in the [CLI contract](contracts/force-add-replacement.md): only the release mapping remains, `status` reflects the new mapping’s normal initial comparison state, and neither endpoint file was changed by either add command. Repeat after `grip remove target/release/grip` using an ordinary add with another source; it must not require `--force` when no other active owner remains.
