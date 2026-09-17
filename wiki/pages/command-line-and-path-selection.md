---
title: Command-line and path selection
type: component
sources: [S003, S004, S014, S015, S016, S017, S028]
updated: 2026-09-17
---

# Command-line and path selection

Grip uses a flat, Git-inspired command surface: (S004)

```text
grip [-p|--project PATH] [-o|--output json] [-v...]
grip init [PATH] | version | add SOURCE DESTINATION | list [SOURCE] | remove SOURCE
grip status [-e] [-d] [PATH] | diff [-d] [PATH]
grip push [-n] [-f] [-d] [PATH] | pull [-n] [-f] [-d] [PATH] | sync [-n] [-d] [PATH]
```

Human output is the default; `-o json` is the supported machine format. `-p` is invalid with `init` and `version`. A path selector is source-space by default and destination-space with `-d`. No nested `mapping` commands or aliases such as `check`, `validate`, `show`, or `resolve` are public. (S004)

`add` accepts ordinary relative source spellings such as `./app/`, normalizes them to a project-relative declaration, and retains that normalized source. It accepts an absolute, `~`-relative, or project-relative destination declaration and retains its exact submitted spelling. (S004) (S016) (S017)

Source-space selectors apply the same source normalization before lookup. Destination-aware operations resolve relative destination declarations from the selected project root, never the current directory, while preserving the declaration separately. (S016) (S017)

`pull PATH` uses destination space by default; `-s` and `--source` opt into source-space selection. `status`, `diff`, `push`, `pull`, and `sync` ignore modification times by default, while `-m` and `--use-modification-time` enable comparison and transfer for that one operation. `pull -a` and `pull --adopt` are the explicit destination-only tree-file enrollment form and cannot be combined with `--source`. (S004)

Default human `status` renders an actionable source path relative to the invocation directory. An ordinary displayed source path can be passed unchanged to `push` from that same directory; other source-space commands retain project-relative selector interpretation. [Mapping registry publication](./mapping-registry-publication.md) records the corresponding persisted-declaration boundary. (S004) (S016)

The destination declaration decision preserves lexical spelling while keeping resolved endpoint safety checks and selector behavior unchanged. [Destination path forms](./destination-path-forms.md) records the scope and persistence boundary. (S015) (S017)

The hierarchy is an intentional pre-release replacement, so superseded commands are rejected rather than accepted as compatibility aliases. [Git-inspired command hierarchy](./git-inspired-command-hierarchy.md) records the durable decision and its force and baseline boundaries. (S014)

No-selector `grip push --force` is an explicit aggregate source-winning operation across all managed project entries, while a supplied selector retains the exact-entry force contract. `grip push --force --dry-run` previews that aggregate without payload or accepted-state mutation. Verified Feature 028 makes an exact human-readable `grip diff` invoke a configured external program while preserving unselected and JSON inspection. [Initial delivery roadmap future features](./initial-delivery-roadmap-future-features.md) records its bounded scope. (S003) (S004)

[External diff program](./external-diff-program.md) records the configuration, direct invocation, safety, and exit boundary for an exact selected diff. (S028)
