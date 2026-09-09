# Grip

Grip is a local, project-scoped synchronizer for explicit file and tree mappings. A Grip project keeps portable mapping intent in version-controlled metadata while keeping mutable baselines, operation records, locks, staging files, and recovery bytes local to that project.

## Initialize a project

Initialize the current directory or an explicit directory:

```sh
grip init
grip init /path/to/dotfiles
```

Initialization atomically creates:

```text
<project>/.grip/
├── config.toml
└── .gitignore
```

The descriptor starts as strict Descriptor V2 and `.grip/.gitignore` contains exactly `/state/`. Grip never runs Git itself; committing `.grip/config.toml` and `.grip/.gitignore` remains the user's choice.

Project-dependent commands accept `--project PATH` or discover a project by walking upward from the current directory. An explicit path must name the exact project root. Discovery fails when it finds no project, more than one project boundary, or unsafe or invalid project metadata. `grip version`, `--version`, and `--help` remain project-independent.

## Configure portable mappings

Mapping sources are relative to the project root. Destinations use literal home-relative syntax (`~` or `~/...`) and resolve against the invoking user's canonical home:

```sh
grip mapping add file shell/gitconfig "~/.gitconfig"
grip mapping add tree editor "~/.config/editor"
grip mapping list
grip mapping show shell/gitconfig
grip mapping remove shell/gitconfig
```

Grip stores only the portable declarations. Human and JSON results show declared values separately from safely rendered resolved endpoints. Absolute sources, absolute destinations, traversal, environment expansion, other-user home syntax, symbolic-link endpoints, unsafe ancestry, and overlapping ownership are rejected.

For a tree mapping, Grip discovers ordinary non-ignored source entries on every inspection. Source-side `.gripignore` files use Gitignore-compatible rules. The project metadata directory `.grip` is structurally reserved and is never mapping payload, even if ignore rules attempt to re-include it.

## Inspect and synchronize

```sh
grip mapping inspect [PATH]
grip status [PATH]
grip check [PATH]
grip diff [PATH]
grip baseline accept [PATH]
grip push [-n|--dry-run] [PATH]
grip pull [-n|--dry-run] [PATH]
grip sync [-n|--dry-run] [PATH]
grip resolve [-n|--dry-run] (--source|--destination) PATH
```

Selectors use project-relative source space by default. `--destination` selects in destination space where supported; `--` terminates option parsing for dash-prefixed paths.

Grip compares the current source, current destination, and last accepted baseline. It propagates unambiguous one-sided changes, reports converged or synchronized entries as no-ops, and blocks divergent conflicts until `resolve` names the complete winning side. Destination-only content outside source-defined managed membership remains unmanaged.

Read-only commands and dry runs do not create `.grip/state`, acquire writer locks, publish operation evidence, or change payloads. An actual writer lazily creates owner-only project state, takes a bounded project-local mutation lock, repeats inspection and revalidation, stages and verifies each replacement, and publishes State V4 only after final verification.

## Delete, retire, and recover

Deletion and retirement are explicit:

```sh
grip delete --source [--dry-run] PATH
grip delete --destination [--dry-run] PATH
grip retire [--force] [--dry-run] PATH
grip recovery list
grip recovery show REFERENCE
grip recovery restore REFERENCE
grip recovery remove REFERENCE
```

Before replacing or deleting existing payload, Grip preserves verified recovery bytes under the operation that produced them. Operation Record V2, recovery manifests, and payload metadata use portable managed identities and project-relative private references. Recovery inventory and restore derive live targets from the currently selected project and endpoint role; copied project state cannot direct a restore back into the original project.

## Project metadata and local state

The complete layout is:

```text
<project>/.grip/
├── config.toml              # portable Descriptor V2; suitable for version control
├── .gitignore               # exactly /state/
└── state/                   # owner-only, local, ignored, and created lazily
    ├── state.json           # integrity-protected State V4
    ├── locks/
    ├── staging/
    ├── operations/          # portable Operation Record V2 and action evidence
    └── recovery/            # descriptor and accepted-state recovery generations
```

State V4 stores portable entry identities plus a local binding to the resolved project root, destination home, descriptor digest, and resolved topology. Copying a project retains its state as untrusted evidence. Read-only commands may report `rebind_eligible` without writing; mutation is blocked on missing, ambiguous, stale, incomplete, unsafe, or contradictory evidence. A successful state-writing command records the new binding atomically.

There is no global Grip registry, global mutable state root, generated project identifier, compatibility reader for older schemas, or environment-variable override for command scope.

## Output and exit codes

All commands support `--output human|json`; repeat `-v` for redacted diagnostics on stderr. JSON results use stable categories and safe path values, including `raw_hex` when necessary to preserve exact non-UTF-8 identity.

| Exit | Symbol | Meaning |
| ---: | --- | --- |
| 0 | `ok` | Successful command |
| 1 | `attention_required` | Inspection succeeded and found entries requiring attention |
| 2 | `invalid_usage` | Invalid arguments |
| 10 | `invalid_configuration` | Project, descriptor, mapping, or binding is invalid |
| 11 | `unsupported_schema` | Unsupported current metadata schema |
| 12 | `corrupt_state` | Malformed, unsafe, or inconsistent state |
| 13 | `state_contention` | Another writer owns a required project-local lock |
| 20 | `operational_failure` | I/O, result delivery, or another runtime failure |

## Build and validate

```sh
mise trust
mise install
mise run build
mise run test
mise run validate
mise run performance
```

`validate` checks formatting, runs Clippy with warnings denied, runs the default test suite, and builds the release binary. The ignored performance and platform-specific suites are explicit release gates.
