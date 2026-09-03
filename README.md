# Grip

Grip is an offline, per-user command-line tool for safely managing file mappings. Feature 001 establishes its CLI, configuration, and machine-state foundation; it does not yet create mappings, copy payloads, synchronize files, delete files, use Git, or contact remote services.

## Install and build

Install the pinned Rust toolchain and build the executable:

```bash
mise trust
mise install
mise exec -- cargo build --release
```

Common project workflows are available as mise tasks:

```bash
mise run build
mise run clean
mise run test
mise run validate
mise run performance
```

`clean` removes Cargo build artifacts when a clean rebuild is needed. `validate` runs the formatting check, Clippy with warnings denied, all default tests, and the release build. `performance` builds in release mode before running the ignored 100-run acceptance harness.

## Configure

Grip uses `~/.grip` by default. Set `GRIP_HOME` to a non-empty absolute path to select one exact alternate root; Grip does not fall back to the default or to machine-wide configuration.

Create `config.toml` in the selected root with the minimal v1 registry:

```toml
schema_version = 1
mappings = []
```

Grip-owned state is optional. `validate` treats an absent `state/state.json` as uninitialized and never creates it.

## Commands and output

Use `grip version` to report the application version and `grip validate` to validate the selected home, registry, and optional state. Both support `--output human` (the default) or `--output json`. Repeat `-v` or `--verbose` to emit redacted diagnostics on stderr. Conventional `--help` and `--version` displays do not access a Grip home.

```bash
GRIP_HOME=/Users/pegagio/example-grip-home grip validate
grip --output json version
```

## Exit codes

| Exit | Symbol | Meaning |
|---:|---|---|
| 0 | `ok` | Success |
| 2 | `invalid_usage` | Invalid command or arguments |
| 10 | `invalid_configuration` | Invalid home policy or registry |
| 11 | `unsupported_schema` | Unsupported registry or state schema |
| 12 | `corrupt_state` | Malformed or inconsistent machine state |
| 20 | `operational_failure` | I/O, output, or another runtime failure |

State-publication contention is internal in Feature 001; no public command publishes state yet.
