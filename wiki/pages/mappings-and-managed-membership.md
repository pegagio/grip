---
title: Mappings and managed membership
type: concept
sources: [S001, S004, S005, S006]
updated: 2026-09-06
---

# Mappings and managed membership

A mapping declares a source-to-destination relationship. A file mapping pairs two exact paths, while a tree mapping pairs directory roots and maps source-relative paths to the same destination-relative paths. (S001)

The canonical source path is the user-facing identity of a mapping; Grip does not assign a separate mapping ID. Stored state must still retain and validate the mapping kind, source, and destination, and mappings that overlap or could own the same entry are rejected before discovery or mutation. (S001)

Feature 002 makes that namespace rule concrete. File mappings own one exact path, tree mappings own a root and every possible descendant, and comparisons are component-aware. The complete accepted registry is validated as one graph: equal or nested source namespaces, equal or nested destination namespaces, same-mapping source/destination overlap, and cross-mapping source/destination overlap in either direction all block publication. This conservative cross-side rule also prevents longer ownership cycles without a separate graph traversal. (S005)

Canonical source identity is the only lookup key for `show` and `remove`; aliases resolving to the same source cannot coexist. Adding and removing mappings changes registry intent only: it does not traverse tree members, create absent destinations, copy or delete payloads, or create synchronization state. (S005)

For tree mappings, the dynamically discovered, non-ignored source namespace determines membership. New source entries can be proposed for management, while destination-only entries remain unmanaged unless their relative paths previously entered the managed namespace. Tracking establishes ownership but must not imply an unreviewed bulk mutation, and untracking removes ownership without silently deleting either copy. (S001)

Source-side `.gripignore` files may appear at the mapping root or in nested directories and follow Gitignore-style precedence and negation rules. Destination-side ignore files and ordinary `.gitignore` files do not control membership, and `.gripignore` files are policy rather than synchronized payload by default. A newly ignored managed entry remains pending explicit retirement rather than being silently deleted or untracked. (S001)

The implemented inspection derives tree membership fresh without creating a manifest or baseline. It includes ordinary files and directories, reports an ignored directory once without traversing descendants, and exposes eligible, ignored, destination-only, unsupported-source, and unsafe-destination-collision categories. File mappings contribute only their exact source and do not enumerate parent directories. (S004)

Feature 003 fixes ignore evaluation to source-relative, hierarchical policy: root and nested `.gripignore` files apply in traversal order, later matches override earlier ones, negation can re-include entries, and neither ordinary Git exclusions nor destination-side policy participates. Ignored directories prune their descendants from both discovery and destination-only reporting. (S006)

Accepted entries that later become newly ignored or lose their mapping remain visible as pending-retirement evidence. Scoped baseline acceptance preserves every accepted record outside the selected source or destination subtree and cannot silently retire prior membership. (S004)

## Related pages

- [Grip product model](./grip-product-model.md)
- [Mapping registry publication](./mapping-registry-publication.md)
