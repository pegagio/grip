---
title: Destination path forms
type: decision
sources: [S015, S017]
updated: 2026-09-11
---

# Destination path forms

Grip accepts destination declarations that are absolute paths, exactly `~`, begin with `~/`, or are relative paths. A relative destination is resolved from the selected project root, never the current directory. Sources remain normalized paths relative to that same selected project root. (S017)

Grip stores each accepted destination spelling unchanged, including `.` and `..` components or repeated separators in home-relative and project-relative forms. It expands home-relative declarations or resolves project-relative declarations, then lexically normalizes a separate operational path without filesystem canonicalization, so existing endpoint, ownership, topology, and revalidation checks still apply to the resolved endpoint. (S015) (S017)

Absolute declarations intentionally trade cross-machine portability for an explicit target. Equivalent resolved destinations still conflict even when their stored spellings differ, and declaration-aware registry publication and State V4 identity preserve the original declaration instead of reconstructing it from a runtime path. (S015) (S017)

## Related pages

- [Command-line and path selection](./command-line-and-path-selection.md)
- [Mappings and managed membership](./mappings-and-managed-membership.md)
- [Configuration and state](./configuration-and-state.md)
