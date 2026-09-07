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

## Push safely to destinations

`push` copies supported source files and directories to their mapped destinations. It mutates by default; use `-n` or `--dry-run` to inspect the complete deterministic plan without acquiring a mutation lock, creating operation evidence, changing payloads, or publishing a baseline.

```text
grip push [-n|--dry-run] [--destination] [--] [PATH]
```

The optional path uses source space unless `--destination` is supplied; that option changes selection only and never reverses the source-to-destination direction. Grip reports every blocker before mutation. An actionful execution acquires the global writer lock, repeats inspection under the lock, and stops at the first failed action.

File writes use non-following directory handles, exclusive sibling staging, content and mode verification, atomic rename, and containing-directory sync. Replacements first preserve the prior destination in a private, operation-local recovery entry. Additions have no backup. Successful execution publishes one accepted-state generation only after all selected actions and a final complete observation verify.

Machine-readable results include the complete selected entries, dependency-ordered actions, blockers, SHA-256 plan identity, opaque operation ID, recovery availability, and independent baseline visibility and durability. A partial failure retains completed effects and recovery evidence, leaves later actions unattempted, and does not publish a baseline. Recovery inspection and rollback remain outside this command.

Removed mappings and newly ignored accepted entries remain visible as pending retirement; acceptance cannot silently discard them. Payload copying, conflict resolution, deletion, and retirement are outside these commands.

## Pull safely to sources

`pull` copies an eligible complete destination file state back to its established managed source. Like `push`, it mutates by default; use `-n` or `--dry-run` for a lock-free, side-effect-free preview.

```text
grip pull [-n|--dry-run] [--destination] [--] [PATH]
```

Selection follows the same contract as inspection and push: source space by default, destination space with `--destination`, and `--` before a dash-prefixed path. The option changes selection only; pull direction always remains destination to source. Human and JSON results describe the same ordered plan, and shared mutation JSON includes `operation: pull` and `direction: pull`.

Only an established accepted entry classified as a destination-only change is actionable. Synchronized entries are no-ops. Conflicts, incomplete inspection, unsafe ancestry, unsupported nodes, stale evidence, and missing source content block mutation. Destination-only content outside accepted managed membership is reported item by item as unmanaged and non-actionable; it is never imported, although an unsafe collision at a managed path remains a blocker.

An actionful pull acquires the same short-lived per-user mutation lock as push, revalidates the complete plan after locking, and revalidates each action immediately before publication. Destination bytes and supported mode are staged beside the source, the prior source is preserved and verified in the operation-local recovery area, and replacement uses the strongest supported atomic rename. Pull never creates a missing source or source parent.

After all actions verify, Grip performs a final complete observation and publishes one accepted generation containing only the verified actioned identities while preserving no-action, out-of-scope, and pending-retirement records. A no-op publishes nothing. On failure, completed replacements remain in place, later actions remain unattempted, recovery evidence is retained, and the prior baseline remains authoritative unless a state-publication result explicitly reports that a new generation became visible.

Pull does not propagate source changes, resolve conflicts, delete or retire entries, import unmanaged destination content, inspect or clean recovery data, repair corrupt state, resume interrupted operations, or roll back automatically.

## Synchronize and resolve conflicts

`sync` plans both eligible directions together and mutates by default. Its optional selector follows the same source-space, `--destination`, and `--` rules as push and pull. Use `-n` or `--dry-run` for a lock-free preview.

```text
grip sync [-n|--dry-run] [--destination] [--] [PATH]
```

Source additions and source-only changes produce push actions; established destination-only changes produce pull actions. Converged identical changes require no payload replacement but are accepted only after the complete selected sync succeeds. A synchronized-only selection is a true no-op and creates no operation record or baseline generation. Any conflict, deletion, retirement, unsupported node, unsafe path, or incomplete observation blocks the entire selected plan before mutation.

Resolve one exact divergent entry by naming its source path and choosing the complete winning side explicitly:

```text
grip resolve [-n|--dry-run] (--source|--destination) [--] PATH
```

`PATH` is always interpreted in source-path space; `--destination` chooses the destination state as winner and does not change path interpretation. Resolution never infers a winner, accepts a subtree, combines attributes, or changes adjacent entries. It freshly verifies the conflict, preserves the complete losing state in operation-local recovery, applies the winner through the corresponding push or pull pipeline, verifies equality, and publishes one accepted generation. Both preview aliases produce the equivalent plan without changing payloads or Grip-owned state.

Sync and resolution reuse the global mutation lock, descriptor-safe staging, verified recovery, stop-after-first-failure behavior, and versioned result envelope. Machine output identifies `operation: sync|resolve`; every payload action carries `direction: push|pull`; resolution also carries `winner: source|destination`. Partial results retain completed effects and recovery evidence while leaving the prior baseline authoritative.

## Registry publication safety

`<GRIP_HOME>/config.toml` is the accepted v1 registry. It must be a current-user-owned, non-symlink regular file with group and other write bits unset; add and remove also require owner write permission. Deterministic rewrites preserve its exact permission mode, though presentation-only TOML whitespace, comments, and ordering may be normalized.

Writers coordinate through the stable owner-only `<GRIP_HOME>/.registry.lock`. Before replacement, Grip verifies that the accepted bytes and mapping paths have not changed, then retains the exact prior registry under `<GRIP_HOME>/state/recovery/registry/sha256-<digest>/config.toml`. Recovery generations are immutable and content-addressed. Grip stages and verifies the complete candidate beside `config.toml`, atomically renames it, and syncs the containing directory. Lock, staging, and recovery artifacts never contain accepted synchronization state, and no mapping command mutates `<GRIP_HOME>/state/state.json`.

Grip is offline and does not delete files, invoke Git, elevate privileges, or contact remote services. Synchronization remains limited to explicit mapping ownership and the supported push, pull, sync, and exact conflict-resolution contracts.

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
| 13 | `state_contention` | Another Grip writer holds the global mutation lock |
| 20 | `operational_failure` | I/O, output, or another runtime failure |

Registry-publication contention is reported as an operational failure. `push`, `pull`, `sync`, `resolve`, and explicit `baseline accept` can publish synchronization state; status, check, diff, and every dry-run mutation are read-only.
