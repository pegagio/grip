---
title: Command-line and path selection
type: component
sources: [S004, S014, S015]
updated: 2026-09-11
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

`add` accepts a normalized project-relative source and a destination expressed as an absolute path, `~`, or any `~/` spelling. The declaration is retained exactly, while destination-aware operations resolve it separately; other relative or expansion-like destination forms are rejected. (S004)

The destination declaration decision preserves lexical spelling while keeping resolved endpoint safety checks and selector behavior unchanged. [Destination path forms](./destination-path-forms.md) records the scope and persistence boundary. (S015)

The hierarchy is an intentional pre-release replacement, so superseded commands are rejected rather than accepted as compatibility aliases. [Git-inspired command hierarchy](./git-inspired-command-hierarchy.md) records the durable decision and its force and baseline boundaries. (S014)
