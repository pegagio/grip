---
title: External diff program
type: component
sources: [S028]
updated: 2026-09-16
---

# External diff program

For an exact human-readable `grip diff` selection, Grip renders its normal detailed inspection and then hands the two validated endpoints to one external comparison program. JSON output and an unselected human diff remain inspection-only and never launch a child process. (S028)

The selected executable is resolved in order: a non-empty `GRIP_EXTERNAL_DIFF`, a named tool assembled from machine-wide `~/.grip/config.toml` and selected-project `.grip/config.toml` fields, then `diff`. A project replaces matching global fields; an explicit project `args` array replaces the whole global array. The environment override is executable-only and receives no configured arguments. (S028)

Named tools use a `[diff] tool` selection with a `[difftool.<name>]` `program` and ordered literal `args`. Grip invokes the program directly, passing each configured token and both endpoints as separate arguments. It does not evaluate shell syntax, expand values, create temporary comparison copies, or change mapping, baseline, or JSON contracts. (S028)

Grip revalidates both endpoints immediately before launch. Missing, unsafe, unsupported, or otherwise ineligible endpoints prevent a launch and retain Grip-owned diagnostics. A normal child exit code is returned unchanged; a signal termination is reported with `128 + signal`. (S028)

## Related pages

- [Configuration and state](./configuration-and-state.md)
- [Command-line and path selection](./command-line-and-path-selection.md)
