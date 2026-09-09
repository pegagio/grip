# Grip Product Definition

Grip is the project title, with `grip` as the prospective command name, for a local, per-user command-line utility that selectively synchronizes files and directory trees through explicit mappings between filesystem locations.

Grip is stateful and bidirectional. It remembers the last accepted state of managed entries so it can identify which side changed, copy an unambiguous change in either direction, and stop when both sides changed incompatibly. A destination tree may contain substantially more content than its paired source tree; Grip leaves anything outside the managed source namespace alone.

The implementation is expected to be a new Rust project. This document captures the product definition and working design decisions, not a final command, storage, or compatibility contract.

## Table of Contents

- [Product Summary](#product-summary)
- [Goals and Non-Goals](#goals-and-non-goals)
- [Core Concepts](#core-concepts)
- [Mapping Model](#mapping-model)
- [Tree Discovery and Gripignore](#tree-discovery-and-gripignore)
- [Synchronization Model](#synchronization-model)
- [Content and Metadata](#content-and-metadata)
- [Safety Model](#safety-model)
- [Command-Line Experience](#command-line-experience)
- [Configuration and State](#configuration-and-state)
- [Implementation Direction](#implementation-direction)
- [Incremental Delivery](#incremental-delivery)
- [Potential Future Expansions](#potential-future-expansions)
- [Open Questions](#open-questions)
- [Current Product Decisions](#current-product-decisions)

## Product Summary

Grip is a stateful, selective, bidirectional overlay synchronizer.

Its short product statement is:

> Grip selectively tracks files across paired filesystem locations and safely synchronizes changes to mapped entries in either direction without owning the surrounding trees.

The important terms in that statement are:

- **Stateful**: Grip retains a baseline from the last accepted synchronization instead of choosing a winner from timestamps at each invocation.
- **Selective**: Grip manages explicit file mappings or the dynamically discovered, non-ignored contents of explicit tree mappings.
- **Bidirectional**: Either the source or destination may be edited. A one-sided change can flow to the other side.
- **Overlay**: A tree mapping owns only the namespace discovered from its source. Unrelated destination content remains unmanaged.
- **Local**: The first version operates between paths visible to one local user account. It is not a daemon, privileged system service, or remote replication system.

Each Grip project has a source root: the directory initialized with `grip init`. Mapping sources are portable paths relative to that root, while destinations are portable `~`-relative paths resolved against the invoking user's canonical home. The terms `source` and `destination` identify how a mapping is established and how new tree members enter management; after that, established entries may synchronize in either direction.

`rsync` is useful inspiration and conversational shorthand, but Grip should not be positioned primarily as “stateful rsync.” That phrase creates expectations of stateless mirroring, broad ownership of directory trees, and a direction chosen independently on every invocation. Grip's distinct value is selective membership plus baseline-informed bidirectional synchronization.

The name Grip reinforces the ownership boundary: the tool has an explicit grip on selected paths and nothing surrounding them.

## Goals and Non-Goals

The initial product should solve a small but complete local synchronization problem while preserving room for later expansion.

### Goals

Grip should:

- Create an individual file mapping by pairing one source path with one destination path.
- Create a tree mapping by pairing a source directory with a destination directory root.
- Discover new, non-ignored source-tree entries without requiring users to update a manifest for every file.
- Allow root and nested `.gripignore` files using Gitignore semantics.
- Leave destination-only files and directories unmanaged unless they correspond to previously managed entries.
- Detect content and supported metadata changes on either side.
- Safely propagate unambiguous one-sided changes in either direction.
- Detect divergent two-sided changes and require the user to choose which complete side wins.
- Provide `-n` and `--dry-run` options that preview action commands without mutation.
- Require additional explicit authorization for deletions.
- Preserve replaced or deleted entries for recovery.
- Keep portable configuration in one initialized project and mutable synchronization state in that project's ignored `.grip/state/` directory.
- Provide precise output suitable for both humans and automation.

### Non-Goals for the Initial Release

The first release should not attempt to provide:

- Synchronization across machines or over a network.
- A background daemon, filesystem watcher, or automatic continuous synchronization.
- A privileged machine-wide service.
- Multiple-user coordination.
- Automatic conflict resolution or combining changes from both sides.
- Filesystem-level snapshots or a complete backup product.
- Version control integration or automatic Git operations.
- Universal cross-platform metadata fidelity.
- Automatic ownership of destination-only content.

Remote synchronization may be considered later, but the initial architecture should not pay the complexity cost of host identities, transport security, remote locking, network interruption recovery, or cross-platform attribute translation.

## Core Concepts

Grip separates user intent from observed synchronization history.

### Mapping

A mapping declares a relationship between a source and a destination. A file mapping pairs two files. A tree mapping pairs two directory roots and dynamically discovers its managed namespace from the source.

### Managed Entry

A managed entry is a file, directory, or other explicitly supported filesystem node covered by a mapping. For a tree mapping, its identity is the path relative to the paired roots.

### Source Namespace

The source establishes membership for a tree mapping. A new non-ignored source entry is eligible to become managed. A new destination-only entry is not automatically imported because the destination is allowed to contain unrelated content.

This source-defined membership does not make synchronization permanently one-directional. Once an entry has entered the managed namespace and has an accepted baseline, later destination edits can synchronize back to the source.

### Baseline

The baseline records fingerprints of the last successfully accepted content and the accepted supported metadata for every managed entry; it does not retain exact copies of file contents. Comparing the source, destination, and baseline allows Grip to distinguish one-sided changes from conflicts.

### Plan

A plan is the complete, deterministic set of proposed actions produced from one validated inspection. Dry runs render a plan without mutation. Mutating action commands revalidate relevant filesystem evidence before executing the plan.

### Registry and State

The project descriptor expresses durable user intent: portable mappings, kinds, and options. Machine-owned state contains changing operational evidence: discovered members, baselines, fingerprints, pending retirement information, operation records, and recovery references. The descriptor may be committed with the project; `.grip/state/` is local and ignored.

Users should not need to maintain the derived member inventory by hand.

## Mapping Model

Grip supports two foundational mapping kinds.

### File Mapping

A file mapping pairs one exact source path with one exact destination path.

```toml
schema_version = 2

[[mappings]]
kind = "file"
source = "git/.gitconfig"
destination = "~/.gitconfig"
```

The source resolves below the selected project root and the destination resolves below the invoking user's home. The semantic relationship is one source, one destination, and one managed entry.

### Tree Mapping

A tree mapping pairs a source directory with a destination directory. Relative paths below the source map to the same relative paths below the destination.

```toml
[[mappings]]
kind = "tree"
source = "editor"
destination = "~/.config/editor"
```

For example:

```text
<project>/editor/config.yaml
    <-> ~/.config/editor/config.yaml

<project>/editor/themes/dark.json
    <-> ~/.config/editor/themes/dark.json
```

The destination may also contain entries such as caches, application databases, local workspaces, or unrelated configuration. Those entries are ignored unless their relative paths previously became managed through the source.

### Mapping Ownership

Grip should reject mappings that make ownership ambiguous. Two mappings must not own the same source path, the same destination path, or overlapping tree regions that could resolve to the same entry. Ownership validation should occur before discovery or mutation so adding a new source file cannot make an unchanged registry ambiguous.

The normalized project-relative source path identifies a mapping. Grip does not add a separate user-assigned mapping or project ID because the ownership model already permits only one mapping for a source path and project identity comes from explicit selection or upward discovery. Durable identity stores the portable mapping tuple and entry-relative bytes; resolved absolute endpoints are runtime evidence only.

### Tracking and Untracking

Tracking creates a mapping; it should not imply an unreviewed bulk mutation. For a tree, initial tracking discovers the candidate source namespace and presents the proposed destination additions and any collisions.

Untracking removes Grip's ownership without silently deleting either copy. If a tree entry was previously managed and later becomes ignored, Grip should report it as requiring an explicit untrack decision rather than silently retiring or deleting it.

## Tree Discovery and Gripignore

Tree mappings dynamically discover their source namespace on every inspection. A newly added source entry is identified as a proposed addition even if the user did not update Grip's registry.

### Ignore Files

Grip supports `.gripignore` files at the mapping root and in nested source directories. Their syntax and behavior should reuse Gitignore semantics rather than define a superficially similar custom language.

For example:

```text
GripSource/editor/
├── .gripignore
├── config.yaml
├── cache/
└── themes/
    ├── .gripignore
    ├── dark.json
    └── experimental.json
```

The root `.gripignore` might contain:

```gitignore
cache/
*.log
```

The nested `themes/.gripignore` might contain:

```gitignore
experimental.json
```

In this example, `config.yaml` and `themes/dark.json` are eligible for management. The cache, log files, and experimental theme are excluded.

### Ignore Semantics

Grip should preserve these Gitignore behaviors:

- A `.gripignore` applies to its containing directory and descendants.
- Patterns are interpreted relative to the directory containing the ignore file unless Gitignore syntax gives them another scope.
- Applicable files are evaluated from the mapping root toward the entry.
- Later matching rules take precedence over earlier rules.
- Negated patterns can re-include entries, subject to Gitignore's excluded-parent traversal rules.
- An ignored directory is not traversed unless the rules make traversal necessary for a possible re-inclusion.

The implementation should be validated against a conformance suite covering Gitignore edge cases instead of relying on a loose claim of compatibility.

### Ignore Authority

Only source-side `.gripignore` files define Grip membership. Destination-side ignore files have no effect, and actual `.gitignore` files are not read by default because version-control exclusions and synchronization intent are different concerns.

`.gripignore` files are Grip policy rather than managed payload and are not synchronized by default.

Adding an ignore rule must not silently delete or untrack an entry that has already been synchronized. Grip should retain enough history to report the transition and require an explicit retirement decision.

## Synchronization Model

Grip uses three-way comparison: current source state, current destination state, and the last accepted baseline.

### Entry Classification

The initial classification model should cover at least these cases:

| Source | Destination | Baseline | Classification | Default disposition |
| --- | --- | --- | --- | --- |
| New eligible entry | Absent | Absent | Source addition | Propose source to destination |
| New eligible entry | Equivalent new entry | Absent | Initial match | Propose accepting a baseline without copying |
| New eligible entry | Different new entry | Absent | Initial collision | Require an explicit direction or manual reconciliation |
| Absent | New entry | Absent | Destination-only unmanaged | Ignore and optionally report |
| Equal to baseline | Equal to baseline | Present | Synchronized | No action |
| Changed | Equal to baseline | Present | Source-only change | Propose source to destination |
| Equal to baseline | Changed | Present | Destination-only change | Propose destination to source |
| Changed identically | Changed identically | Present | Converged two-sided change | Accept or refresh baseline |
| Changed differently | Changed differently | Present | Conflict | Require source-wins or destination-wins resolution |
| Deleted | Equal to baseline | Present | Source-side deletion | Require explicit directional deletion |
| Equal to baseline | Deleted | Present | Destination-side deletion | Require explicit directional deletion |
| Deleted | Changed | Present | Delete/change conflict | Block mutation |
| Changed | Deleted | Present | Change/delete conflict | Block mutation |
| Deleted | Deleted | Present | Converged deletion | Retire baseline explicitly or as part of an authorized operation |
| Newly ignored | Present or absent | Present | Managed entry pending retirement | Require explicit untrack decision |

The exact labels may evolve, but the distinctions are part of the product contract.

### Direction

The working direction vocabulary is:

- **Push**: propagate eligible source-side changes to destinations.
- **Pull**: propagate eligible destination-side changes to sources.
- **Sync**: propagate all unambiguous one-sided changes in either direction.

`source` and `destination` remain meaningful for discovery, new-member admission, ignore policy, and directional commands. They do not establish a permanently authoritative content side for existing managed entries.

### Conflicts

Any divergent change on both sides since the baseline is a conflict. Grip should report every conflict found during a complete preflight and block the entire mutating action before the first mutation.

Under the plan of record, the user resolves a conflict by explicitly choosing the complete source state or complete destination state as the winner. The planned conflict workflow does not combine fields or content from the two sides. If content changes on one side while permissions or another supported attribute change on the other, the entry still conflicts and the user still chooses one complete side.

The provisional command form is explicit about that choice:

```text
grip resolve /Users/pegagio/GripSource/editor/config.yaml --source
grip resolve /Users/pegagio/GripSource/editor/config.yaml --destination
```

Before replacing the losing side, Grip preserves that side in the operation's recovery namespace. It then copies and verifies the winning state and publishes the result as the new baseline. Grip must require a fresh inspection if either side changes after the conflict was reported and before the resolution is applied.

If both sides independently reach the same complete supported state, Grip recognizes convergence rather than reporting a conflict.

### Deletions

Deletion is not an ordinary change. A missing previously managed entry must be distinguished from an entry that was never managed.

Grip should report deletions during ordinary inspection and synchronization but require a separate explicit authorization before executing them. A user can combine that authorization with `--dry-run` to preview the deletion plan. Direction matters: accepting a source-side deletion removes or retires the destination copy, while accepting a destination-side deletion removes or retires the source copy.

Grip should never infer deletion from a newly added ignore rule.

## Content and Metadata

Grip synchronizes more than byte content. Supported filesystem metadata participates in change identification, equality, conflict detection, copying, and verification.

### Intended Metadata Scope

The intended Unix and macOS model should investigate support for:

- Filesystem node type.
- File content.
- Permission mode.
- User and group ownership.
- Modification time.
- Extended attributes.
- Access control lists.
- BSD file flags on platforms that expose them.
- Symbolic-link identity without following arbitrary link targets.
- Directory metadata, including metadata for empty directories when directories are supported as managed entries.

Resource forks and macOS-specific metadata may be represented through extended attributes, but the implementation must verify actual filesystem behavior rather than assume complete coverage.

### Preservation Contract

The product should not make an unqualified promise to preserve all attributes. A per-user process cannot always assign arbitrary owners or groups, clear privileged flags, reproduce protected extended attributes, or translate metadata between filesystems with different capabilities.

A defensible contract is:

> Grip synchronizes file content and supported filesystem metadata, preserving ownership and attributes when the operating system and target filesystem permit. Unsupported or unauthorized metadata transitions are reported and never silently discarded.

If Grip cannot reproduce or verify a requested attribute, it must not call the entry synchronized. Depending on the operation phase and risk, the condition should either block the plan or fail the operation while preserving recovery evidence.

### Ownership

Preserving ownership is straightforward when both entries are owned by the invoking user and no ownership transition is required. Assigning another user or an unauthorized group may require privileges that a per-user Grip process deliberately does not have.

Grip should report the required and actual ownership precisely. It should not escalate privileges automatically, invoke `sudo`, or silently substitute the current user's identity.

### Supported Node Boundary

Grip supports ordinary regular files and ordinary directories. Symbolic links remain separately governed because they may have an explicit value-preserving representation, but Grip never follows them implicitly.

Grip does not support:

- Regular files with multiple hard links.
- Sparse files.
- Unix-domain sockets.
- FIFOs or named pipes.
- Character devices.
- Block devices.
- BSD whiteouts.
- Unknown or platform-specific special node types.
- Mounted filesystems encountered beneath a tree mapping root.

Hard-linked and sparse files require explicit detection even though the operating system classifies them as regular files. Grip rejects a regular file whose link count indicates multiple hard links because copying or replacing one path would not preserve inode topology. Copy-on-write filesystem clones are not hard links; Grip may synchronize their logical file state but does not promise to preserve shared physical storage.

Grip rejects a sparse file when the supported platform and filesystem report holes because an ordinary copy could silently expand its physical allocation. Sparse-file support would require hole-aware discovery, copying, target-capability validation, and verification that are outside the product contract.

Grip identifies sockets, FIFOs, devices, whiteouts, and unknown nodes through non-following metadata inspection. It never opens, reads, hashes, copies, backs up, recreates, or follows those nodes. This avoids blocking on a FIFO, interacting with a socket, or causing side effects through a device node.

Tree discovery stays on the filesystem containing the mapping's source root. A nested mount boundary is unsupported and blocks traversal; a user can create a separate explicit mapping whose source root is on that mounted filesystem.

A non-ignored unsupported source entry is reported by exact path and detected type. `status` includes it in the complete classification, `check` returns an attention status, and every mutating command in the selected scope blocks before its first mutation. The user may exclude the entry with `.gripignore`.

A destination-only unsupported node outside the managed namespace remains unmanaged and untouched. An unsupported node occupying a managed destination path, or replacing a previously managed regular file or directory, is an unsafe collision that blocks mutation. Grip uses an allowlist: any filesystem node that is not explicitly supported or separately governed is unsupported.

## Safety Model

Filesystem synchronization is valuable only if the operator can trust its boundaries and recover from mistakes.

### Read-Only Inspection and Dry Runs

Action commands such as `push`, `pull`, and `sync` mutate by default. Their `-n` and `--dry-run` options build and render the same plan without changing payloads, baselines, backups, mappings, or other state. `status`, `check`, and `diff` are intrinsically read-only. Dry runs, read-only commands, and failed operations do not update baselines.

### Complete Preflight

Before mutation, Grip should inspect the complete selected scope and report all known conflicts, unsafe paths, unsupported nodes, ownership overlaps, and metadata operations it cannot perform. A known blocking condition prevents every action in that mutating invocation.

### Revalidation

A mutating action should not blindly execute stale inspection data. Grip should revalidate the mapping, relevant paths, node identities, content and metadata fingerprints, and destination ancestry before replacement. If the evidence changed, Grip stops and asks for a fresh invocation.

### Atomic Publication

Replacement content should be staged on the destination filesystem, fully prepared, and installed with the strongest atomic operation the platform supports. A baseline is published only after every authorized action and verification succeeds.

Atomicity must be described precisely. A multi-entry synchronization cannot generally be globally atomic across arbitrary filesystems, so Grip should not imply that the whole operation rolls back automatically.

### Backups and Partial Failure

Before replacing or deleting an existing managed entry, Grip should preserve the previous entry in a per-operation recovery namespace. Backups are retained until the user explicitly removes them.

If an operation fails after some entries have changed, Grip should stop, preserve completed changes and backups, avoid publishing a new baseline, and report exactly what completed. Automatic rollback is not required initially because rollback itself can fail and obscure evidence.

### Path and Symlink Safety

Grip should canonicalize and validate mapping roots while retaining the lexical relative paths needed to resolve paired entries. It must prevent path traversal, overlapping ownership, and escape through unsafe ancestry.

Tree discovery must not follow symbolic links by default. A symbolic link is its own filesystem node and should either be supported with an explicit link contract or rejected. Grip must never follow an arbitrary destination link and overwrite its target as though the link were the mapped path.

### Concurrency

Grip should serialize mutations to its registry and state. It should detect another active Grip operation and fail safely rather than allow concurrent baseline or backup publication. Read-only inspection may be concurrent only if it can consume a coherent state snapshot.

### No Incidental Git Behavior

Grip may operate on paths inside Git repositories, but it does not stage, commit, reset, clean, or otherwise mutate Git metadata. Version control remains an independent user workflow.

## Command-Line Experience

The command surface begins by selecting one Grip project:

```text
grip init [PATH]
grip [--project PATH] mapping add file SOURCE DESTINATION
grip [--project PATH] mapping add tree SOURCE DESTINATION
grip [--project PATH] mapping list|show|remove|inspect
grip status
grip check
grip diff
grip resolve
grip push
grip pull
grip sync
```

### Working Command Meanings

- **`init [PATH]`**: Atomically initialize `.grip/config.toml` and `.grip/.gitignore` in the current or named directory without creating state or invoking Git.
- **`mapping add`**: Create one portable file or tree mapping after resolving and validating its concrete endpoints.
- **`mapping remove`**: Remove mapping intent without deleting either payload copy.
- **`mapping list|show|inspect`**: Display declared and resolved mappings or inspect current managed membership.
- **`status [PATH]`**: Produce a complete human-readable classification without mutation and return success when inspection itself succeeds. An optional path narrows the classification.
- **`check [PATH]`**: Produce the same or equivalent classification but return a nonzero drift status when attention is required, allowing automation to distinguish drift from inspection failure. An optional path narrows the check.
- **`diff [PATH]`**: Explain content and supported metadata differences without mutation. An optional path narrows the output.
- **`resolve PATH`**: Resolve one exact conflicting entry by explicitly selecting the complete source or destination state as the winner, back up the losing side, make both sides equal, and publish the accepted baseline.
- **`push [PATH]`**: Execute source-to-destination actions in the selected scope, or preview them with `-n` or `--dry-run`.
- **`pull [PATH]`**: Execute destination-to-source actions for previously managed entries in the selected scope, or preview them with `-n` or `--dry-run`.
- **`sync [PATH]`**: Execute all unambiguous one-sided actions in both directions within the selected scope, or preview them with `-n` or `--dry-run`.

The intended command behavior is:

```bash
grip status                 # read-only classification
grip diff                   # read-only details
grip sync --dry-run         # preview proposed synchronization
grip sync                   # perform synchronization
grip push --dry-run         # preview source-to-destination actions
grip push                   # perform source-to-destination actions
```

`status`, `check`, `diff`, `push`, `pull`, and `sync` accept at most one optional path selector in the initial command contract. With no selector, the command covers every mapping. A selector equal to a file mapping source selects that file; a selector equal to a tree mapping source selects the complete tree; and a selector beneath a tree source selects only that managed entry or subtree.

Positional selectors are interpreted in source-path space by default, including for `pull`. The `--destination` option instead interprets the selector in destination-path space and resolves it back to its owning mapping and managed entries. Grip does not silently guess which side a positional path names.

The conventional `--` delimiter terminates option parsing and identifies all remaining arguments as paths. It is useful even with the initial single-selector limit because it handles paths beginning with a hyphen and makes the option/path boundary explicit:

```bash
grip status -- editor/config.yaml
grip status --destination -- ~/.config/editor/config.yaml
grip sync --dry-run -- editor/themes
grip pull --destination -- ~/.config/editor/themes/dark.json
```

Grip resolves source selectors in project-relative source space and destination selectors against the selected destination-home binding without requiring the selected entry to exist. This permits inspection of a managed source that has been deleted. An ignored or unmanaged selector is reported explicitly, and a selector outside every mapping is an error rather than an empty successful result.

A scoped mutating command still validates the complete registry for structural safety, including overlapping mappings. Payload drift and conflicts outside the selected scope do not block the scoped operation. Applicable `.gripignore` rules inherited from ancestors of a selected tree entry remain in force.

Deletions should require an additional explicit option, provisionally `--prune`, or a more precise deletion-specific command. `--prune --dry-run` previews directional deletions without executing them.

Machine-readable output should use stable field names and distinguish synchronized state, drift, conflict, exclusion, unsupported evidence, and operational failure.

## Configuration and State

Grip is project-scoped. `grip init [PATH]` establishes the source root and atomically creates the portable descriptor plus the ignore rule that keeps mutable state out of version control:

```text
<project>/.grip/
├── config.toml
├── .gitignore        # exactly /state/
└── state/            # created lazily by the first actual writer
    ├── state.json
    ├── locks/
    ├── staging/
    ├── operations/
    └── recovery/
```

Descriptor V2 contains normalized project-relative sources and literal `~` or `~/...` destinations. It is stable and portable enough to commit with the surrounding project. State V4, Operation Record V2, recovery metadata, locks, and recovery payloads are owner-only local operational evidence beneath `.grip/state/` and are ignored by the initialized `.grip/.gitignore`.

Project-dependent commands use an explicit `--project PATH` or discover exactly one `.grip` boundary by walking upward from the invocation directory. They fail rather than fall back when no project exists, when nested boundaries make selection ambiguous, or when any selected metadata node is unsafe. There is no global configuration root, global state fallback, environment-selected installation, generated project ID, or persistent project index.

A copied descriptor resolves against the copied project root and current invoking-user home. Copied state retains its bytes but is untrusted until Grip completely rebinds every portable identity and reobserves the required accepted and recovery evidence. Read-only commands report eligibility without rewriting state; a successful state-writing command persists the current binding atomically.

Configuration parsing rejects duplicate or overlapping mappings, unknown fields, unsupported schema versions, path traversal, malformed portable paths, and topology that overlaps `.grip` or another mapping. Read-only and dry-run commands do not create state or acquire writer locks.

## Implementation Direction

Grip should be implemented as a Rust command-line application. Rust is a good fit because Grip performs recursive filesystem traversal, ignore matching, hashing, metadata comparison, staged copying, and safety-sensitive mutation. Its type system and explicit error handling can make state transitions and platform boundaries visible.

Performance is useful for large trees, but correctness and operator trust are the primary reasons for the implementation shape. The design should favor small, composable modules over a framework-heavy architecture.

### Tooling Guidance

This subsection records informed starting guidance, not firm implementation commitments. The choices remain reversible as implementation exposes concrete needs. Introducing or replacing a tool or dependency should be justified by the capability it provides, its maintenance cost, and its effect on Grip's safety and distribution contracts.

#### Rust Toolchain

The starting toolchain should use stable Rust with the Rust 2024 edition. A checked-in [`rust-toolchain.toml`](https://rust-lang.github.io/rustup/overrides.html) should select the development and CI compiler and include `rustfmt` and Clippy. Cargo's separate [`rust-version`](https://doc.rust-lang.org/stable/cargo/reference/rust-version.html) field should state the minimum supported compiler once that policy is chosen. Because Grip is an application, `Cargo.lock` should be committed.

The initial validation surface should remain conventional:

```text
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
cargo build --release
```

[`rust-analyzer`](https://rust-analyzer.github.io/book/) is the preferred editor language server. Additional test runners, coverage tools, fuzzers, profilers, and release automation should be introduced only when the project has a concrete need for them.

#### Command-Line Framework

[`clap`](https://docs.rs/clap/latest/clap/_derive/) with its derive API is the leading candidate. Grip's command set maps naturally to typed subcommand enums and argument structs, while `clap` supplies generated help, typed values, reusable argument groups, validation relationships, and parser-definition checks. The CLI layer should convert parsed arguments into Grip-owned domain commands so framework types and annotations do not spread into synchronization logic.

[`bpaf`](https://docs.rs/bpaf/latest/bpaf/) is the strongest alternative if implementation benefits from a more compositional parser or its completion and documentation generation. [`argh`](https://docs.rs/argh/latest/argh/) is a simpler derive-based alternative. Lower-level parsers such as [`lexopt`](https://docs.rs/lexopt/latest/lexopt/) or `pico-args` would reduce framework weight but make Grip responsible for more help generation, subcommand behavior, validation, and error presentation; they are not the preferred starting point for the planned command surface.

This recommendation does not make `clap` part of the product contract. A small CLI spike should confirm help output, path handling, error behavior, exit-code control, shell completion needs, compile-time cost, and testability before the framework is treated as established.

#### Supporting Libraries

The following library directions are promising but should be adopted incrementally:

- **Tree discovery and ignore matching**: The [`ignore`](https://docs.rs/ignore/latest/ignore/struct.WalkBuilder.html) crate supports recursive traversal, Gitignore-compatible rules, custom ignore filenames, and disabled symlink following. Its defaults also honor `.ignore`, `.gitignore`, Git excludes, global Git ignore rules, and hidden-file filtering. Grip must explicitly disable those behaviors and enable only `.gripignore`, then validate the result with conformance tests.
- **Serialization**: [`Serde`](https://docs.rs/serde/latest/serde/) is the leading type-serialization layer, but it does not decide whether the human-managed registry or machine-owned state uses TOML, YAML, JSON, or another format.
- **Errors**: [`thiserror`](https://docs.rs/thiserror/latest/thiserror/) is a good candidate for typed domain and infrastructure errors. Drift, conflict, invalid configuration, unsafe evidence, and operational failure should remain structured distinctions rather than becoming strings.
- **Diagnostics**: [`tracing`](https://docs.rs/tracing/latest/tracing/) is available if structured diagnostic events prove useful. Human output, machine-readable output, and diagnostic logs must remain separate regardless of the logging implementation.
- **Dependency governance**: [`cargo-deny`](https://embarkstudios.github.io/cargo-deny/checks/index.html) is worth considering early because it can check advisories, licenses, duplicate or banned crates, and dependency sources.

Hashing, filesystem metadata access, temporary-file publication, process locking, CLI integration testing, and release distribution still require focused evaluation. In particular, metadata support should be designed around Grip's explicit macOS and Unix contracts rather than selected by whichever crate appears to offer the broadest abstraction.

### Proposed Components

The initial application can be organized around these responsibilities:

```text
CLI parsing and output
    -> registry loading and validation
    -> source discovery and .gripignore evaluation
    -> filesystem metadata adapter
    -> current snapshot construction
    -> baseline comparison and classification
    -> deterministic plan construction
    -> dry-run rendering
    -> guarded plan execution
    -> backup and baseline publication
```

Platform-specific metadata behavior should sit behind a narrow interface. The first supported contract may target macOS and Unix explicitly rather than invent a broad cross-platform abstraction before the semantics are understood.

Core classification should be modeled with explicit types rather than strings or prompt-like policy. Planning and execution should be separate so read-only behavior can be tested thoroughly before mutations exist.

### Testing Direction

The project should emphasize behavioral and filesystem integration tests covering:

- File and tree mapping boundaries.
- Root and nested `.gripignore` conformance.
- Dynamic source additions and destination-only unmanaged entries.
- Every source/destination/baseline classification.
- Content and metadata-only changes.
- Ownership or attribute transitions that the user cannot perform.
- Overlapping mappings and path traversal attempts.
- Hard-linked files, sparse files, sockets, FIFOs, device nodes, whiteouts, unknown node types, and nested mount boundaries.
- Unsupported source entries, ignored unsupported entries, destination-only unsupported entries, and unsupported collisions at managed destinations.
- Symlinks under their separately governed contract.
- Conflict preflight across multiple otherwise actionable entries.
- Deletion dry runs and explicit authorization.
- Backup creation, partial failure, and baseline non-publication.
- Races between planning, revalidation, and execution.
- Source-side and `--destination` path selection for files, tree roots, nested subtrees, missing managed entries, ignored entries, unmanaged entries, and paths outside every mapping.
- `--` option termination, paths beginning with a hyphen, scoped conflict behavior, and inherited `.gripignore` rules.
- Deterministic human-readable and machine-readable output.

Tests should use temporary roots and must not inspect or mutate the operator's real files or Grip state.

## Incremental Delivery

Grip should grow through small vertical slices that each establish a useful, testable contract.

### Milestone 1: Read-Only Discovery

Establish a Rust CLI and a minimal registry with file and tree mappings. Implement safe path validation, recursive source discovery, nested `.gripignore` behavior, and deterministic listing. Make no payload changes.

### Milestone 2: Snapshot and Status

Capture content and the first supported metadata set, persist a baseline, inspect both sides, and classify synchronized entries, one-sided drift, destination-only unmanaged entries, deletions, and conflicts. Add automation-friendly exit behavior.

### Milestone 3: Safe Push

Implement source-to-destination additions and changes with `-n` and `--dry-run`, revalidation, staging, backups, verification, partial-failure reporting, and baseline publication. The action mutates by default; its dry-run mode renders the plan without mutation. Do not include deletion initially.

### Milestone 4: Reverse Synchronization

Implement destination-to-source changes for established managed entries under the same safety and recovery contract, including mutating and dry-run modes.

### Milestone 5: Bidirectional Sync and Deletion

Plan both directions together, block all conflicts before mutation, and add separately authorized directional deletion and explicit retirement of newly ignored entries.

### Milestone 6: Metadata Expansion

Expand the supported metadata contract deliberately, with platform-specific tests and precise reporting for unsupported or unauthorized transitions.

This order lets the project validate its namespace and state model before taking on destructive behavior. It also provides a useful Rust learning progression without making early compiler lessons coincide with user-data risk.

## Potential Future Expansions

The following ideas are intentionally recorded outside the plan of record. They are not required by the current product definition or assigned to implementation milestones.

### Two-Way Interactive Merge

A future `grip merge` command could help the user reconcile the current source and destination without retaining previous file content. Grip's fingerprints would establish that both sides changed, while a two-way comparison tool could align the current text and let the user construct an accepted result manually.

This would be user-directed reconciliation, not a history-aware three-way merge. Without the previous content, Grip could not reliably infer whether a difference represents an insertion, deletion, or edit relative to the common baseline. The user would remain responsible for the result.

A safe version of this workflow could:

1. Revalidate the current source and destination fingerprints.
2. Materialize a private temporary candidate and open a configured two-way comparison tool.
3. Require the user to accept the candidate explicitly.
4. Confirm that neither original changed during reconciliation.
5. Back up both originals.
6. Publish the accepted candidate to both sides, verify convergence, and record it as the new baseline.

The feature would initially make sense only for supported text files. Binary files, unsupported encodings, metadata conflicts, or an abandoned comparison would continue to require the plan-of-record source-wins or destination-wins resolution.

### Mapping Remap

A future `grip remap` command could preserve mapping and baseline continuity when the user relocates a source or destination path. It would update Grip's registry and state after a filesystem move performed separately by the user; it would not move, copy, or delete filesystem entries itself.

The provisional command shape is:

```bash
grip remap OLD_SOURCE --source NEW_SOURCE --dry-run
grip remap OLD_SOURCE --source NEW_SOURCE

grip remap SOURCE --destination NEW_DESTINATION --dry-run
grip remap SOURCE --destination NEW_DESTINATION
```

A safe remap workflow would lock the registry and state, validate the existing mapping, canonicalize the replacement path, reject ownership overlap, inspect both sides, and preserve the baseline only when current evidence proves continuity. If the mapping contains drift, conflicts, unsupported entries, or content that cannot be reconciled with its existing baseline, Grip would refuse the remap and require separate synchronization or a new `untrack` and `track` lifecycle.

The registry and state transition should publish as one guarded metadata operation after final revalidation. `--dry-run` would report the proposed path and state changes without modifying Grip state. A filesystem-moving `grip mv` command is not part of this expansion because that name would imply responsibility for moving content and introduce a separate partial-failure and recovery contract.

### Multiple Path Selection

A future expansion could allow `status`, `check`, `diff`, `push`, `pull`, and `sync` to accept multiple path selectors in one invocation. The Git-style `--` delimiter would make every remaining argument a path:

```bash
grip status -- SOURCE_PATH_ONE SOURCE_PATH_TWO
grip sync --dry-run -- SOURCE_PATH_ONE SOURCE_PATH_TWO
grip pull --destination -- DESTINATION_PATH_ONE DESTINATION_PATH_TWO
```

All selectors in one invocation would use the same path space: source by default or destination when `--destination` is present. Grip would normalize and validate the complete selector set before mutation, reject the complete invocation if any selector is invalid or ambiguous, remove duplicates, collapse nested selections into their containing scope, and process the resulting union deterministically. Supporting multiple paths would not weaken complete registry validation or allow conflicts within the selected union to be skipped.

## Open Questions

The following decisions are intentionally deferred until the relevant implementation is specified. They should remain explicit rather than being resolved accidentally while coding:

- Which Gitignore specification and edge cases define `.gripignore` compatibility?
- Are `.gripignore` files always implicitly excluded, or can a user explicitly opt into synchronizing one as ordinary content?
- Are empty directories managed in the first release?
- Are directory timestamps and permissions synchronized independently of their children?
- Are symbolic links supported as link objects in the first release or rejected entirely?
- Which metadata fields form the first supported equality contract?
- Are modification times authoritative synchronization data, restored metadata, or only diagnostic evidence?
- How are numeric user and group identities represented, and what happens when names or identifiers cannot be reproduced?
- What command and state transition retire a previously managed entry that becomes ignored?
- How are backups inspected and explicitly removed?
- How does Grip recover when its registry, baseline, or other required state is missing, unreadable, corrupt, or inconsistent, and which recovery actions require the user to select an authoritative side explicitly?
- Which equal, nested, or otherwise recursive source and destination path relationships must mapping validation reject to prevent Grip from discovering or synchronizing its own output?
- Should a multi-entry mutating action continue after an isolated operational failure or stop immediately? The safer initial preference is to stop.
- How should case sensitivity and Unicode normalization differences between source and destination filesystems be detected and reported?
- What stable output and exit-code contract should automation consume?

## Current Product Decisions

The conversation has established the following working decisions:

- Grip is a standalone, general-purpose synchronization utility.
- The project title is `Grip`, with `grip` as the prospective executable name.
- Grip is a local CLI whose configuration and mutable state are scoped to one initialized project.
- `grip init [PATH]` establishes the project source root and creates `.grip/config.toml` plus `.grip/.gitignore`.
- Project commands accept an exact `--project PATH` or discover one project by walking upward; they never fall back to global configuration.
- Descriptor V2 stores portable project-relative sources and `~`-relative destinations and may be committed with the project.
- Mutable State V4, operations, locks, staging, and recovery live beneath ignored `.grip/state/`; no generated project-instance key is required.
- Remote or cross-machine synchronization is outside the initial scope.
- Rust is the preferred implementation language.
- Rust tooling and library selections are advisory and reversible; `clap` is the leading CLI-framework candidate, not a committed product dependency.
- Mappings have no separate user-assigned IDs; the normalized project-relative source path identifies each mapping.
- File mappings pair exact paths.
- Tree mappings pair roots and dynamically discover non-ignored source entries.
- Tree mappings do not require users to enumerate every member.
- New source entries are discovered automatically and proposed for management; discovery alone does not mutate the destination.
- New destination-only entries remain unmanaged.
- Existing managed entries can synchronize in either direction.
- Root and nested `.gripignore` files use Gitignore semantics.
- Actual `.gitignore` files are not synchronization policy by default.
- Ignore policy comes from the source and is not synchronized by default.
- Newly ignored managed entries are not silently deleted or untracked.
- A stored baseline identifies one-sided changes and conflicts.
- Content and supported metadata both participate in change identification.
- Ownership and attributes are preserved when permitted and otherwise reported precisely.
- Grip supports ordinary regular files and directories; hard-linked files, sparse files, sockets, FIFOs, devices, whiteouts, unknown special nodes, and nested mount boundaries are unsupported.
- Non-ignored unsupported source entries and unsupported nodes at managed destinations block mutation, while destination-only unsupported nodes outside the managed namespace remain untouched.
- The plan-of-record conflict workflow treats divergent two-sided changes as a whole-entry conflict and requires an explicit source-wins or destination-wins choice.
- Plan-of-record conflict resolution backs up the losing side and does not combine changes from both sides.
- A two-way, user-directed `grip merge` workflow is recorded only as a potential future expansion and is not part of the plan of record.
- A metadata-only `grip remap` workflow is recorded only as a potential future expansion; it would preserve verified baseline continuity after a user-performed filesystem move without moving content itself.
- `status`, `check`, `diff`, `push`, `pull`, and `sync` accept one optional path selector; selectors name source paths by default, while `--destination` explicitly selects destination-path interpretation.
- `--` terminates option parsing and identifies the remaining positional argument as a path.
- Multiple path selectors with Git-style `--` separation are recorded only as a potential future expansion and are not part of the initial command contract.
- Action commands mutate by default; `-n` and `--dry-run` preview their plans without mutation, while `status`, `check`, and `diff` are always read-only.
- Deletion requires additional explicit authorization even though ordinary action commands mutate by default.
- Conflicts and known unsafe conditions block mutation before the first action.
- Replacements are staged, prior entries are retained for recovery, and baselines publish only after successful verified operations.
- Grip does not automatically elevate privileges or perform Git operations.
