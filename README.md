# Grip

Grip is a local, per-user tool for managing explicit file and tree mapping intent. Feature 002 records ownership only: mapping commands do not copy, delete, discover, reconcile, or otherwise modify payload files.

## Mapping lifecycle

Add mappings with canonical source-path identity:

```sh
grip mapping add file /absolute/source/file /absolute/destination/file
grip mapping add tree /absolute/source/tree /absolute/destination/tree
```

The source must exist and match the requested kind. The destination may be absent, but its nearest existing ancestor must be safe. Both arguments must be absolute UTF-8 paths without `..`; symbolic-link endpoints and unsupported node kinds are rejected. Grip canonicalizes accepted paths and rejects duplicate, overlapping, nested, equal, or cross-recursive ownership across the complete registry.

Inspect or remove intent by canonical source identity:

```sh
grip mapping list
grip mapping show /absolute/source
grip mapping remove /absolute/source
```

`mapping list` is ordered by canonical source. `mapping show` and `mapping remove` select exactly one mapping. Removal changes registry intent only and never removes payload data.

All commands support `--output human|json`. JSON mapping values have stable `kind`, `source`, and `destination` fields, while failures include stable `operation` and `reason` details.

## Inspect managed membership

Inspect the current source-defined membership of every mapping or one mapping selected by canonical source identity:

```sh
grip mapping inspect
grip mapping inspect /absolute/source
```

Inspection is read-only. It derives tree membership fresh on every invocation, includes ordinary files and directories (including empty directories), and never creates a manifest, baseline, lock, recovery record, or payload change. File mappings contribute only their exact source and do not enumerate either parent directory.

Source-side `.gripignore` files define policy for their containing directory and descendants using Gitignore-compatible comments, escaping, anchoring, wildcards, directory rules, negation, and last-match precedence. A nested policy can re-include an entry only while its ancestors remain traversable. An ignored directory is reported once as an exclusion root and is not traversed. `.gripignore` itself is always policy rather than payload; `.gitignore`, `.ignore`, global Git settings, destination-side policy files, and hidden-file status have no authority.

The inventory distinguishes eligible entries, ignored source paths, destination-only unmanaged paths, unsupported source entries, and unsafe nodes occupying an eligible paired destination. Symbolic links, hard links, sparse files, special nodes, non-UTF-8 source names, and nested mount boundaries are never followed or opened as payload. Unsupported source entries and paired collisions are blocking findings, but a complete inspection still exits `0` and reports `blocking_count`; failures to produce a complete inventory return a nonzero error.

Human output is deterministic and machine output uses stable categories plus escaped Safe Path values with `raw_hex` when exact non-UTF-8 identity must be retained. Diagnostics remain on stderr when verbosity is enabled.

## Baselines and synchronization status

Grip compares current source and destination metadata with explicitly accepted evidence. It fingerprints ordinary file content with SHA-256 and records the first supported metadata set: node kind, file content and length where applicable, and the full Unix permission mode. Modification time is diagnostic only and does not make content different.

```sh
grip status [PATH]
grip check [PATH]
grip diff [PATH]
grip baseline accept [PATH]
```

Each optional path selects one mapping, entry, or component-boundary subtree in source space. Add `--destination` to interpret it in destination space, and use `--` before a dash-prefixed path. `status` reports the complete deterministic classification; `check` returns exit `1` and `attention_required` when its otherwise successful result needs attention; `diff` reports the available source-to-baseline, destination-to-baseline, and source-to-destination metadata dimensions without printing payload content.

`baseline accept` records evidence only when every selected source and destination entry is complete and equivalent. A scoped acceptance preserves all evidence outside the scope. Repeating an already-current acceptance is an exact no-op. State Envelope V2 generations are integrity checked, published atomically under `<GRIP_HOME>/state/state.json`, and retain exact prior bytes as immutable recovery evidence. Grip continues to read State V1 as an empty-baseline predecessor.

Removed mappings and newly ignored accepted entries remain visible as pending retirement; acceptance cannot silently discard them. Payload copying, conflict resolution, deletion, and retirement are outside these commands.

## Registry publication safety

`<GRIP_HOME>/config.toml` is the accepted v1 registry. It must be a current-user-owned, non-symlink regular file with group and other write bits unset; add and remove also require owner write permission. Deterministic rewrites preserve its exact permission mode, though presentation-only TOML whitespace, comments, and ordering may be normalized.

Writers coordinate through the stable owner-only `<GRIP_HOME>/.registry.lock`. Before replacement, Grip verifies that the accepted bytes and mapping paths have not changed, then retains the exact prior registry under `<GRIP_HOME>/state/recovery/registry/sha256-<digest>/config.toml`. Recovery generations are immutable and content-addressed. Grip stages and verifies the complete candidate beside `config.toml`, atomically renames it, and syncs the containing directory. Lock, staging, and recovery artifacts never contain accepted synchronization state, and no mapping command mutates `<GRIP_HOME>/state/state.json`.

Grip is offline and does not synchronize files, delete files, use Git, or contact remote services. Mapping declarations establish intent for later synchronization features.

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

Use `grip version` to report the application version and `grip validate` to validate the selected home, registry, and optional state. These commands and the mapping lifecycle support `--output human` (the default) or `--output json`. Repeat `-v` or `--verbose` to emit redacted diagnostics on stderr. Conventional `--help` and `--version` displays do not access a Grip home.

```bash
GRIP_HOME=/Users/pegagio/example-grip-home grip validate
grip --output json version
```

## Exit codes

| Exit | Symbol | Meaning |
|---:|---|---|
| 0 | `ok` | Success |
| 1 | `attention_required` | Check completed with entries requiring attention |
| 2 | `invalid_usage` | Invalid command or arguments |
| 10 | `invalid_configuration` | Invalid home policy or registry |
| 11 | `unsupported_schema` | Unsupported registry or state schema |
| 12 | `corrupt_state` | Malformed or inconsistent machine state |
| 13 | `state_contention` | Another baseline publication holds the state lock |
| 20 | `operational_failure` | I/O, output, or another runtime failure |

Registry-publication contention is reported as an operational failure. Only explicit `baseline accept` publishes synchronization state; status, check, and diff are read-only.
