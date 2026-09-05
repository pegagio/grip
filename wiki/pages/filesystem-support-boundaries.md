---
title: Filesystem support boundaries
type: reference
sources: [S001, S004, S005, S006]
updated: 2026-09-05
---

# Filesystem support boundaries

Grip’s initial ordinary payload contract covers regular files and directories. Hard-linked files, sparse files, sockets, FIFOs, device nodes, whiteouts, unknown special nodes, and nested mount boundaries are unsupported until their semantics are deliberately specified and tested. (S001)

For Feature 002 mapping identities, a final symbolic-link endpoint is rejected. A safe intermediate symlink to a directory may be resolved while canonicalizing the longest existing prefix, including for an absent destination or absent `show`/`remove` selector. The submitted path and resolved anchor evidence are retained so retargeting that intermediate alias before publication is detected as stale evidence rather than silently changing ownership. (S005)

Mapping sources must exist as the declared regular file or directory kind. A destination may be absent when its nearest existing canonical ancestor is a safe directory; any existing destination must have the corresponding kind. Inputs must be absolute UTF-8 paths without parent traversal, and accepted registry paths must already equal their resolved canonical identities. (S005)

A non-ignored unsupported source entry or an unsupported node colliding with a managed destination blocks mutation. An unsupported destination-only entry outside the managed namespace remains untouched because Grip does not own it. Ignored unsupported source entries may be skipped when ignore evaluation excludes them before management. (S001)

Feature 003 inspection reports symbolic links, hard links, sparse files, special nodes, non-UTF-8 source names, and nested mount boundaries without following or opening them as payload. Unsupported source nodes and unsafe paired-destination collisions are blocking findings, while unsupported destination-only nodes remain nonblocking and unmanaged. Non-UTF-8 path identity is preserved through escaped display plus `raw_hex` in machine output. (S004)

The implementation uses descriptor-relative, no-follow inspection for traversed entries and performs separate source and destination passes. Policy files are accepted only as regular, single-link, non-sparse UTF-8 files; unreadable or unsupported `.gripignore` files fail inspection instead of weakening policy silently. (S006)

Content and the supported metadata set both participate in equality and conflict detection. The exact first-release metadata fields remain open, but the product intends to preserve attributes when permitted and report unsupported or unauthorized transitions precisely rather than silently degrading them. (S001)

The initial platform contract may target macOS and Unix explicitly. Cross-platform metadata fidelity, case sensitivity, Unicode normalization, user and group identity reproduction, and symlink behavior require explicit contracts rather than assumptions inherited from a broad filesystem abstraction. (S001)

## Related pages

- [Grip product model](./grip-product-model.md)
