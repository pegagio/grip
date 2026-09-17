---
title: Filesystem support boundaries
type: reference
sources: [S001, S004, S005, S006, S012, S026]
updated: 2026-09-17
---

# Filesystem support boundaries

Grip’s initial ordinary payload contract covers regular files and directories. Hard-linked files, sparse files, sockets, FIFOs, device nodes, whiteouts, unknown special nodes, and nested mount boundaries are unsupported until their semantics are deliberately specified and tested. (S001)

Feature 002 rejects final symbolic-link endpoints. It may resolve a safe intermediate directory symlink while canonicalizing the longest existing prefix. Submitted-path and resolved-anchor evidence makes retargeting before publication stale rather than a silent ownership change. (S005)

Mapping sources must exist as the declared regular file or directory kind. A destination may be absent when its nearest existing canonical ancestor is a safe directory; any existing destination must have the corresponding kind. Inputs must be absolute UTF-8 paths without parent traversal, and accepted registry paths must already equal their resolved canonical identities. (S005)

The current portable interface replaces those historical absolute declarations: sources are normalized relative to the selected project, while destinations may be absolute, home-relative, or project-relative and resolve through non-following validated ancestry within their permitted roots. The README records the same selected-project interpretation for relative destination declarations. (S001) (S004)

A non-ignored unsupported source entry or an unsupported node colliding with a managed destination blocks mutation. An unsupported destination-only entry outside the managed namespace remains untouched because Grip does not own it. Ignored unsupported source entries may be skipped when ignore evaluation excludes them before management. (S001)

Feature 003 inspection reports symbolic links, hard links, sparse files, special nodes, non-UTF-8 source names, and nested mount boundaries without following or opening them as payload. Unsupported source nodes and unsafe paired-destination collisions are blocking findings, while unsupported destination-only nodes remain nonblocking and unmanaged. Non-UTF-8 path identity is preserved through escaped display plus `raw_hex` in machine output. (S004)

The implementation uses descriptor-relative, no-follow inspection for traversed entries and performs separate source and destination passes. Policy files are accepted only as regular, single-link, non-sparse UTF-8 files; unreadable or unsupported `.gripignore` files fail inspection instead of weakening policy silently. (S006)

Feature 009 finalizes this boundary for current macOS with APFS. Regular files and directories compare complete supported metadata; modification time is unmanaged by default and can be included only through the operation-local `-m` option. Grip preserves exact path bytes, qualifies case and Unicode behavior per endpoint, blocks alias collisions, and treats unavailable, unauthorized, or lossy evidence as a precise blocker rather than coercing it. Other platforms and filesystems remain outside initial acceptance. (S001) (S012)

Feature 026 adds one runtime-only exception: an exact managed destination leaf that is a symbolic link can be observed without target access and remain an unresolved obstacle. Only an exact source-winning forced push can atomically replace the revalidated link object with staged supported state; source links and required destination-link ancestors remain blocking. (S026)

## Related pages

- [Filesystem publication safety](./filesystem-publication-safety.md)
- [Grip product model](./grip-product-model.md)
- [Deletion, retirement, and recovery](./deletion-retirement-and-recovery.md)
- [Metadata and filesystem contract](./metadata-and-filesystem-contract.md)
- [Exact destination symlink replacement](./exact-destination-symlink-replacement.md)
