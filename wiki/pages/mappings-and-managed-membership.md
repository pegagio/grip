---
title: Mappings and managed membership
type: concept
sources: [S001]
updated: 2026-09-03
---

# Mappings and managed membership

A mapping declares a source-to-destination relationship. A file mapping pairs two exact paths, while a tree mapping pairs directory roots and maps source-relative paths to the same destination-relative paths. (S001)

The canonical source path is the user-facing identity of a mapping; Grip does not assign a separate mapping ID. Stored state must still retain and validate the mapping kind, source, and destination, and mappings that overlap or could own the same entry are rejected before discovery or mutation. (S001)

For tree mappings, the dynamically discovered, non-ignored source namespace determines membership. New source entries can be proposed for management, while destination-only entries remain unmanaged unless their relative paths previously entered the managed namespace. Tracking establishes ownership but must not imply an unreviewed bulk mutation, and untracking removes ownership without silently deleting either copy. (S001)

Source-side `.gripignore` files may appear at the mapping root or in nested directories and follow Gitignore-style precedence and negation rules. Destination-side ignore files and ordinary `.gitignore` files do not control membership, and `.gripignore` files are policy rather than synchronized payload by default. A newly ignored managed entry remains pending explicit retirement rather than being silently deleted or untracked. (S001)

## Related pages

- [Grip product model](./grip-product-model.md)
