---
title: Command-line and path selection
type: component
sources: [S004, S014]
updated: 2026-09-10
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

The hierarchy is an intentional pre-release replacement, so superseded commands are rejected rather than accepted as compatibility aliases. [Git-inspired command hierarchy](./git-inspired-command-hierarchy.md) records the durable decision and its force and baseline boundaries. (S014)
