---
title: Destination path forms
type: decision
sources: [S015]
updated: 2026-09-11
---

# Destination path forms

Grip accepts destination declarations that are absolute paths, exactly `~`, or begin with `~/`. Sources remain normalized paths relative to the selected project root; relative destination forms are rejected rather than being interpreted from the current directory or project. (S015)

Grip stores each accepted destination spelling unchanged, including `.` and `..` components or repeated separators in a `~/` form. It expands home-relative declarations and lexically normalizes a separate operational path without filesystem canonicalization, so existing endpoint, ownership, topology, and revalidation checks still apply to the resolved endpoint. (S015)

Absolute declarations intentionally trade cross-machine portability for an explicit target. Equivalent resolved destinations still conflict even when their stored spellings differ, and declaration-aware registry publication and State V4 identity preserve the original declaration instead of reconstructing it from a runtime path. (S015)

## Related pages

- [Command-line and path selection](./command-line-and-path-selection.md)
- [Mappings and managed membership](./mappings-and-managed-membership.md)
- [Configuration and state](./configuration-and-state.md)
