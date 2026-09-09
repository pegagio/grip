---
title: Mappings and managed membership
type: concept
sources: [S001, S004, S005, S006, S011, S013]
updated: 2026-09-09
---

# Mappings and managed membership

A mapping declares a source-to-destination relationship. A file mapping pairs two exact paths, while a tree mapping pairs directory roots and maps source-relative paths to the same destination-relative paths. (S001)

The canonical source path is the user-facing identity of a mapping; Grip does not assign a separate mapping ID. Stored state must still retain and validate the mapping kind, source, and destination, and mappings that overlap or could own the same entry are rejected before discovery or mutation. (S001)

The complete accepted registry is validated as one graph. Equal or nested namespaces, same-mapping overlap, and cross-mapping overlap all block publication. Adding and removing mappings changes registry intent only; it does not touch payloads or synchronization state. (S005)

For tree mappings, the dynamically discovered, non-ignored source namespace determines membership. New source entries can be proposed for management, while destination-only entries remain unmanaged unless their relative paths previously entered the managed namespace. Tracking establishes ownership but must not imply an unreviewed bulk mutation, and untracking removes ownership without silently deleting either copy. (S001)

Source-side `.gripignore` files may appear at the mapping root or in nested directories and follow Gitignore-style precedence and negation rules. Destination-side ignore files and ordinary `.gitignore` files do not control membership, and `.gripignore` files are policy rather than synchronized payload by default. A newly ignored managed entry remains pending explicit retirement rather than being silently deleted or untracked. (S001)

The implemented inspection derives tree membership fresh without creating a manifest or baseline. It includes ordinary files and directories, reports an ignored directory once without traversing descendants, and exposes eligible, ignored, destination-only, unsupported-source, and unsafe-destination-collision categories. File mappings contribute only their exact source and do not enumerate parent directories. (S004)

Accepted entries that later become newly ignored or lose their mapping remain visible as pending-retirement evidence. Scoped baseline acceptance preserves every accepted record outside the selected source or destination subtree and cannot silently retire prior membership. (S004)

Retirement is a state-only transition selected by an exact path or explicit `--all`; it never changes source or destination payloads. Newly ignored, untracked, and converged-deletion records are eligible, active records are rejected, and differing surviving copies require `--force` after their differences are reported. (S011)

## Portable mapping identity

The current mapping identity is `(kind, normalized project-relative source, normalized home-relative destination)`, with entry-relative bytes appended for managed-entry identity. Absolute resolved endpoints exist only in the selected runtime context and are not durable mapping authority. Declared and resolved values are rendered separately. (S013)

Exact `.` is valid only for a tree source and makes the project root the source tree. `.grip/` is pruned structurally before ignore evaluation, so configuration, state, operations, and recovery can never become managed payload. (S013)

> ⚠ conflict: S001, S004, and S005 describe canonical absolute source identity; S013 replaces that current identity with the portable tuple while preserving source-defined membership semantics.

## Related pages

- [Grip product model](./grip-product-model.md)
- [Mapping registry publication](./mapping-registry-publication.md)
