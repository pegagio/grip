# Grip Product Definition

Grip is a local, per-user deployment and selective synchronization tool. It maps files and directory trees between a project and a destination without taking ownership of either surrounding filesystem.

## Table of Contents

- [Product model](#product-model)
- [Command-line interface](#command-line-interface)
- [Mappings and membership](#mappings-and-membership)
- [Inspection and synchronization](#inspection-and-synchronization)
- [State and safety](#state-and-safety)
- [Non-goals](#non-goals)

## Product model

Grip is stateful: it records an accepted baseline for each currently managed entry. It uses the source, destination, and that baseline to distinguish unambiguous one-sided change from conflict. It is selective: only declared file mappings and non-ignored members of declared tree mappings are managed. It is bidirectional: either endpoint may be authoritative when the operation makes that direction explicit.

A source is a path relative to the project root. The command line accepts ordinary relative spellings such as `./app/` and normalizes them before storing the portable declaration. A destination can be an absolute path, `~`, any path beginning `~/`, or a non-empty path relative to the selected project root. Grip retains the exact accepted destination spelling in the descriptor, including lexical dot components or repeated separators, then resolves it for operational validation without following symbolic links. `~/` forms are interpreted from the invoking user's home and may resolve outside it; project-relative forms are interpreted from the selected project root even when the command runs from a descendant directory and may explicitly use `..` to address an adjacent location; absolute forms trade descriptor portability for an explicit target. Empty, other-user-home, and expansion-like destination forms are rejected. A file mapping manages one file pair. A tree mapping manages source-side, non-ignored descendants at matching relative destination paths; unrelated destination content remains unmanaged.

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

`grip add` declares a mapping without copying or creating either endpoint. At least one endpoint must already exist. For example, `grip add README.md ~/Working/grip-dst/README.md` and `grip add README.md /absolute/grip-dst/README.md` are both valid. Grip infers file or tree kind from the existing endpoints and rejects incompatible kinds or two missing endpoints. If both endpoints are equivalent, Grip establishes the existing baseline. If they differ, Grip records the destination's supported state as the initial comparison reference, so the source appears as an ordinary pending push without changing either payload. Source-only members retain their existing addition behavior, and destination-only tree content remains unmanaged.

`grip add`, unselected `grip list`, selected `grip list SOURCE`, and `grip remove` show concise declared mapping rows. Add and unselected list rows omit a kind prefix; selected-list and removal rows retain it. Default human output never repeats resolved endpoints, while JSON retains them. `grip remove` removes the declaration and all associated baseline evidence, without changing either endpoint. Re-adding the same mapping is a new declaration; it never revives old baseline evidence.

Tree membership is defined by source-side discovery and `.gripignore`. Ignored entries are outside Grip's active membership. A direct ignore-file edit is therefore a policy change: it prunes relevant state and a later unignore is treated as new membership rather than a restored history.

A tree source may be strictly beneath its own resolved destination root, enabling a repository `home/` tree to map to `~/`. This is a narrow exception: equal roots, destination-beneath-source roots, file containment, and cross-mapping ownership conflicts remain invalid. Destination inspection is limited to exact current or retained managed identities, so unrelated destination siblings are not enumerated, classified, baselined, or used as drift evidence.

For an admitted contained-source mapping, Grip compares every active managed relative path with the destination-relative source location using component boundaries. Equal, ancestor, and descendant relations block as `recursive_member_topology`; disjoint paths remain safe. The diagnostic identifies the relation and resolved pair, and an intentionally unmanaged unsafe subtree must be excluded through ordinary `.gripignore` policy. Force and narrow selectors do not override the mapping-wide topology blocker.

## Inspection and synchronization

`grip status` reports the current classification without changing state. Its default human output is a concise summary followed only by nonempty `Changes to push`, `Changes to pull`, `Conflicts`, and `Needs baseline` sections in that order. Rows use `SOURCE -> DESTINATION`, `SOURCE <- DESTINATION`, `SOURCE <-> DESTINATION`, and `SOURCE >-< DESTINATION` respectively; `>-<` indicates non-directional baseline or reconciliation work, not a copy direction. An initial collision, ordinary divergent conflict, or one-sided absence follows its `<->` row with commands to keep the source (`grip push --force SOURCE`) or destination (`grip pull --force --destination DESTINATION`); technical blockers retain their safety-significant detail. In default human status, the source side is relative to the process current working directory while the destination side retains its endpoint representation. An ordinary displayed source path can be supplied to `grip push` from that same directory; parent components are allowed when the source is elsewhere in the selected project. Git-style quoting makes unusual source names unambiguous for humans but is not a shell-token guarantee. `grip diff` remains the detailed diagnostic view, while `-o json` retains structured evidence. `status -e` returns a nonzero attention exit status when an entry needs action. `-d` interprets a path selector in destination space; without it, selectors are source-space paths.

Ordinary synchronization is conservative:

- `push` applies unambiguous source-to-destination changes.
- `pull` applies unambiguous destination-to-source changes.
- `sync` combines only unambiguous changes in both directions and blocks the selected scope if a conflict remains.

All three accept `-n` to render the complete plan without mutation. Ordinary operations block divergent entries, unbaselined collisions, and one-sided absence.

Successful action-bearing output is concise: previews begin `Would push N file(s):`, `Would pull N file(s):`, or `Would synchronize N file(s):`; completed commands begin `Pushed N file(s):`, `Pulled N file(s):`, or `Synchronized N file(s):`. Each following row states only its actual source, destination, and arrow. This applies to forced directional resolutions as well. A no-action mutation says `Nothing to push.`, `Nothing to pull.`, or `Nothing to synchronize.` A baseline-only mutation says it would establish or established a baseline for the accepted files.

When an otherwise blocked `push`, `pull`, or `sync` contains only initial collisions or ordinary divergent conflicts, Grip follows each displayed path pair with the same two concrete force choices. A technical or mixed block instead tells the operator to run `grip status` for current detailed safety information. Default human mutation output omits raw planner identifiers, winner fields, action milestones, recovery state, baseline authority, and operation-record evidence; the category, exit behavior, and JSON result remain unchanged. If execution fails after it starts, Grip preserves the completed-action count and directs the operator to `grip status` before retrying.

Source selectors for mapping commands, `status`, `diff`, `pull`, and `sync` retain their project-relative interpretation. A relative source selector for `push`, including dry-run and forced forms, is resolved from the process current working directory and must remain inside the selected project after traversal and symlink checks. Absolute source selectors remain invalid. Destination selectors retain destination-space interpretation.

`push -f PATH` and `pull -f PATH` deliberately select a winner for exactly one managed entry. `push -f` makes the source authoritative; `pull -f` makes the destination authoritative. A present selected winner restores a missing peer; for an existing simple one-sided deletion, a selected absent winner propagates intentional absence by removing the unchanged losing endpoint. Force is never mapping-wide, tree-wide, ambiguous, or available through `sync`.

## State and safety

Grip stores portable mapping declarations in the project and mutable current evidence in `.grip/state/`. State contains active-membership baselines only; it is pruned when a mapping is removed or an entry leaves membership. For an unequal add, a private mapping-scoped publication fence is created before the descriptor and State V4 candidate are published and is cleared only after both verify. Grip uses temporary staging, lock-held revalidation, post-publication verification, and atomic baseline publication to make mutations safe.

Grip creates no retained payload copies, recovery archives, restore commands, or history. Git remains the authority for repository history and recovery. Grip's role is deployment and synchronization, not backup or version control.

## Non-goals

Grip does not provide remote synchronization, a background daemon, automatic conflict merging, a filesystem snapshot service, Git history management, or ownership of destination-only content. It also does not automatically choose a winner for a conflict or absence: the operator selects a direction through `push -f` or `pull -f`.
