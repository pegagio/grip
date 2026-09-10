# Grip Product Definition

Grip is a local, per-user deployment and selective synchronization tool. It maps files and directory trees between a project and a destination home without taking ownership of either surrounding filesystem.

## Table of Contents

- [Product model](#product-model)
- [Command-line interface](#command-line-interface)
- [Mappings and membership](#mappings-and-membership)
- [Inspection and synchronization](#inspection-and-synchronization)
- [State and safety](#state-and-safety)
- [Non-goals](#non-goals)

## Product model

Grip is stateful: it records an accepted baseline for each currently managed entry. It uses the source, destination, and that baseline to distinguish unambiguous one-sided change from conflict. It is selective: only declared file mappings and non-ignored members of declared tree mappings are managed. It is bidirectional: either endpoint may be authoritative when the operation makes that direction explicit.

A source is project-relative. A destination is relative to the invoking user's home. A file mapping manages one file pair. A tree mapping manages source-side, non-ignored descendants at matching relative destination paths; unrelated destination content remains unmanaged.

## Command-line interface

The public command hierarchy is deliberately flat and Git-inspired:

```text
grip [-p|--project PATH] [-o|--output json] [-v...]

grip init [PATH]
grip version
grip add <SOURCE> <DESTINATION>
grip list [SOURCE]
grip remove <SOURCE>
grip status [-e|--exit-code] [-d|--destination] [PATH]
grip diff [-d|--destination] [PATH]
grip push [-n|--dry-run] [-f|--force] [-d|--destination] [PATH]
grip pull [-n|--dry-run] [-f|--force] [-d|--destination] [PATH]
grip sync [-n|--dry-run] [-d|--destination] [PATH]
```

Human-readable output is the default. `-o json` provides structured output and leaves room for future formats. `-p` is invalid with `init` and `version`; those commands do not operate on an existing project.

There are no public command families or aliases for `mapping`, `validate`, `check`, `fsck`, `show`, `resolve`, `delete`, `retire`, `accept`, or recovery operations. `.gripignore` is edited directly, just as `.gitignore` is.

## Mappings and membership

`grip add` declares a mapping without copying or creating either endpoint. At least one endpoint must already exist. Grip infers file or tree kind from the existing endpoints and rejects incompatible kinds or two missing endpoints. If both endpoints are equivalent, Grip establishes a baseline; otherwise the declaration begins unbaselined and requires an appropriate synchronization decision.

`grip list` displays all mappings or one mapping selected by source path. `grip remove` removes the declaration and all associated baseline evidence, without changing either endpoint. Re-adding the same mapping is a new declaration; it never revives old baseline evidence.

Tree membership is defined by source-side discovery and `.gripignore`. Ignored entries are outside Grip's active membership. A direct ignore-file edit is therefore a policy change: it prunes relevant state and a later unignore is treated as new membership rather than a restored history.

## Inspection and synchronization

`grip status` reports the current classification without changing state. `status -e` returns a nonzero attention exit status when an entry needs action. `grip diff` gives a non-mutating view of observed differences. `-d` interprets a path selector in destination space; without it, selectors are source-space paths.

Ordinary synchronization is conservative:

- `push` applies unambiguous source-to-destination changes.
- `pull` applies unambiguous destination-to-source changes.
- `sync` combines only unambiguous changes in both directions and blocks the selected scope if a conflict remains.

All three accept `-n` to render the complete plan without mutation. Ordinary operations block divergent entries, unbaselined collisions, and one-sided absence.

`push -f PATH` and `pull -f PATH` deliberately select a winner for exactly one managed entry. `push -f` makes the source authoritative; `pull -f` makes the destination authoritative. They can propagate intentional absence, including removal of the losing endpoint. Force is never mapping-wide, tree-wide, ambiguous, or available through `sync`.

## State and safety

Grip stores portable mapping declarations in the project and mutable current evidence in `.grip/state/`. State contains active-membership baselines only; it is pruned when a mapping is removed or an entry leaves membership. Grip uses temporary staging, lock-held revalidation, post-publication verification, and atomic baseline publication to make mutations safe.

Grip creates no retained payload copies, recovery archives, restore commands, or history. Git remains the authority for repository history and recovery. Grip's role is deployment and synchronization, not backup or version control.

## Non-goals

Grip does not provide remote synchronization, a background daemon, automatic conflict merging, a filesystem snapshot service, Git history management, or ownership of destination-only content. It also does not automatically choose a winner for a conflict or absence: the operator selects a direction through `push -f` or `pull -f`.
