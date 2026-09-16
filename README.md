# Grip

Grip is a local, project-scoped synchronizer for explicit file and tree mappings. A Grip project keeps portable mapping intent in version-controlled metadata while keeping mutable baselines, operation records, locks, and staging files local to that project. Git remains responsible for project history and recovery.

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

Project-dependent commands accept `-p PATH` or `--project PATH`, or discover a project by walking upward from the current directory. An explicit path must name the exact project root. Discovery fails when it finds no project, more than one project boundary, or unsafe or invalid project metadata. `grip version`, `--version`, and `--help` remain project-independent.

## Configure portable mappings

Mapping sources are paths relative to the project root. Grip accepts ordinary relative spellings such as `./editor/` and normalizes them before storing the declaration. Destinations may be an absolute path, literal `~`, any `~/...` spelling, or a path relative to the selected project root. Grip preserves the exact accepted destination declaration and resolves it only for operations:

```sh
grip add shell/gitconfig "~/.gitconfig"
grip add editor "~/.config/editor"
grip add README.md ~/Working/grip-dst/README.md
grip add README.md /absolute/grip-dst/README.md
grip add ./app/ ../grip-dst/app/
grip list
grip list shell/gitconfig
grip remove shell/gitconfig
```

If exactly one active file mapping already owns a destination, `grip add --force SOURCE DESTINATION` deliberately replaces that mapping without copying either endpoint. It is limited to one equal resolved destination; it does not override source, tree, nested, or other ownership conflicts. The result identifies both declarations:

```text
Mapping replaced:
  old: target/debug/grip -> ~/.local/bin/grip
  new: target/release/grip -> ~/.local/bin/grip
```

After `grip remove` succeeds for the exact source that owns a destination, a normal add may use that destination again without `--force`. Removing a different mapping does not release the active owner.

Grip stores only the declarations. Home-relative and project-relative forms are portable; an absolute destination intentionally binds a declaration to a specific filesystem location. A project-relative destination is resolved from the selected project root, never the command's current directory, and may explicitly use `..` to address an adjacent location. Default human mapping output shows concise declared rows; JSON retains both declared and safely rendered resolved endpoints. Source links, destination-link ancestors, empty destination forms, environment expansion, other-user home syntax, unsafe ancestry, and overlapping ownership are rejected. An exact paired destination leaf link is the narrow exception: `add` records it as unresolved without reading or modifying its target.

```text
Mapped:
 shell/gitconfig -> ~/.gitconfig

2 mapping(s):
 shell/gitconfig -> ~/.gitconfig
 editor -> ~/.config/editor
```

For a tree mapping, Grip discovers ordinary non-ignored source entries on every inspection. Source-side `.gripignore` files use Gitignore-compatible rules. The project metadata directory `.grip` is structurally reserved and is never mapping payload, even if ignore rules attempt to re-include it.

A tree source may be strictly beneath its own destination root, which supports a dotfiles repository layout such as `grip add home/ ~/`. Grip inspects only current non-ignored source members and retained accepted identities at their exact paired destination paths; it does not enumerate unrelated home content. Equal roots, destinations beneath their source, contained file mappings, and ownership overlap between mappings remain invalid.

For a contained-source tree, each managed relative path must be disjoint from the destination-relative path that locates the source tree. If a managed member equals, contains, or falls beneath that path, Grip blocks with `recursive_member_topology` and reports the relation. This blocker cannot be forced or bypassed with an exact selector. If the subtree is intentionally unmanaged, exclude it explicitly in the source tree's `.gripignore`.

## Inspect and synchronize

```sh
grip status [-e|--exit-code] [-d|--destination] [PATH]
grip diff [-d|--destination] [PATH]
grip push [-n|--dry-run] [-f|--force] [-d|--destination] [PATH]
grip pull [-n|--dry-run] [-f|--force] [-d|--destination] [PATH]
grip sync [-n|--dry-run] [-d|--destination] [PATH]
```

Source selectors for mapping commands, `status`, `diff`, `pull`, and `sync` use project-relative source space. A relative source selector for `push` is resolved from the current working directory so an ordinary source path displayed by `grip status` can be used directly from that same directory. Absolute source selectors remain invalid. `--destination` selects in destination space where supported; `--` terminates option parsing for dash-prefixed paths.

Grip compares the current source, current destination, and last accepted baseline. Adding unequal existing endpoints records the destination as that mapping's initial comparison reference without copying either side, so the source is immediately offered as an ordinary push. It propagates unambiguous one-sided changes, reports converged or synchronized entries as no-ops, and blocks divergent conflicts until force chooses a source or destination winner. `grip push --force PATH` and `grip pull --force PATH` remain exact-entry operations; `grip push --force` without `PATH` is the explicit aggregate source-winning operation for all eligible managed entries in the selected project. Destination-only content outside source-defined managed membership remains unmanaged.

Default `grip status` is a concise, path-centered summary. It lists only entries needing attention in this order: `Changes to push`, `Changes to pull`, `Conflicts`, then `Needs baseline`. Rows use `->`, `<-`, `<->`, and `>-<` respectively. An initial collision or ordinary divergent conflict includes the commands for keeping either endpoint only when its displayed selector resolves to one exact managed entry. Aggregate tree conflicts instead say `Run: grip diff SOURCE` so you can inspect exact entries first; technical blockers retain their safety detail. Clean entries appear only in the summary; `grip diff` remains the unchanged detailed diagnostic view, and `-o json` retains structured evidence.

When status runs from a nested directory, its source side is shown relative to that directory. Use the ordinary displayed path with `push` from the same directory:

```text
$ cd app
$ grip status
Changes to push:
  main.py -> ~/workspace/app/main.py

$ grip push main.py
```

Paths elsewhere in the selected project use parent components when needed, such as `../shared/config.yml`. Human status paths use Git-style quoting when a name needs escaping; this is a display convention, not a shell-specific escaping guarantee.

```text
Status: 2 entries checked; 2 current; no action needed.
```

```text
Status: 4 entries checked; 1 current; 1 to push; 1 conflict; 1 needs baseline.

Changes to push:
  app/main.py -> ~/workspace/app/main.py

Conflicts:
  README.md <-> ~/workspace/README.md
    Keep source: grip push --force README.md
    Keep destination: grip pull --force --destination ~/workspace/README.md

Needs baseline:
  CHANGELOG.md >-< ~/workspace/CHANGELOG.md
```

Read-only commands and dry runs do not create `.grip/state`, acquire writer locks, publish operation evidence, or change payloads. `grip push --force --dry-run` previews the complete aggregate scope and its blockers without changing payloads or State V4. An actual writer lazily creates owner-only project state, takes a bounded project-local mutation lock, repeats inspection and revalidation, stages and verifies each replacement, and publishes State V4 only after final verification. Aggregate force publishes accepted evidence after each fully verified entry; a later failure stops the run, retains those earlier publications, and leaves the failed and later entries unaccepted.

`remove` changes only Grip's declaration and baseline; it never changes either endpoint. Normal synchronization blocks one-sided absence and unresolved destination links. An exact source-side `push --force SOURCE` may replace one revalidated destination-link object with a supported file or directory source state; it never follows, reads, or modifies the former target. Pull-side, aggregate, mapping-wide, and ordinary-operation link replacement remain blocked. Use Git for history and recovery.

When `push`, `pull`, or `sync` is blocked by an exact initial collision or ordinary divergent conflict, its output lists the conflicting path and repeats the two force choices beneath it. An aggregate tree conflict instead gives `Run: grip diff SOURCE`; a technical or mixed blocker says `Run: grip status`, where the current detailed safety evidence remains available. The ordinary human result omits planner IDs, winner fields, baseline authority, and operation-record evidence; the blocked result and exit status remain unchanged until you explicitly resolve it.

Action-bearing previews and completions show only the requested direction and affected files. A no-action mutation says `Nothing to push.`, `Nothing to pull.`, or `Nothing to synchronize.` A baseline-only mutation says it would establish or established a baseline for its accepted files. A partial failure keeps its completed-action count and says `Run: grip status before retrying.` The same forms apply to a forced directional resolution; `sync` may contain both arrows under one heading:

```text
Would push 1 file(s):
  app/main.py -> ~/workspace/app/main.py

Pulled 1 file(s):
  docs/config.yml <- ~/workspace/docs/config.yml

Synchronized 2 file(s):
  docs/config.yml <- ~/workspace/docs/config.yml
  app/main.py -> ~/workspace/app/main.py
```

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
```

State V4 stores portable entry identities plus a local binding to the resolved project root, destination home, descriptor digest, and resolved topology. Copying a project retains its state as untrusted evidence. Read-only commands may report `rebind_eligible` without writing; mutation is blocked on missing, ambiguous, stale, incomplete, unsafe, or contradictory evidence. A successful state-writing command records the new binding atomically.

There is no global Grip registry, global mutable state root, generated project identifier, compatibility reader for older schemas, or environment-variable override for command scope.

## Output and exit codes

All commands support `-o json`; human-readable output is the default. Repeat `-v` for redacted diagnostics on stderr. JSON results use stable categories and safe path values, including `raw_hex` when necessary to preserve exact non-UTF-8 identity.

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

`build` produces the debug binary at `target/debug/grip`; run `mise run build --release` to produce the optimized release binary at `target/release/grip`. `validate` checks formatting, runs Clippy with warnings denied, runs the default test suite, and builds the release binary. The ignored performance and platform-specific suites are explicit release gates.

## Prepare a local release

On a clean `master` checkout on macOS Apple Silicon, `mise run release` runs both release gates, packages `grip`, `README.md`, and `LICENSE`, writes an archive and SHA-256 checksum under `dist/`, and creates an annotated local `v<version>` tag for that exact commit. The version comes from the Cargo package metadata. Existing artifact files, checksums, and tags are never overwritten.

Inspect the prepared release before publishing it:

```sh
(cd dist && shasum -a 256 -c grip-v<version>-darwin-arm64.tar.gz.sha256)
tar -tzf dist/grip-v<version>-darwin-arm64.tar.gz
git show v<version>
```

The task does not push anything or create a GitHub Release. After inspection, push deliberately with `git push origin v<version>`, then create the GitHub Release and upload the archive plus checksum. Run `mise run test-release` to execute the isolated release-task contract suite.
