---
title: Filesystem support boundaries
type: reference
sources: [S001, S004, S005, S006, S008, S009, S012]
updated: 2026-09-13
---

# Filesystem support boundaries

Grip’s initial ordinary payload contract covers regular files and directories. Hard-linked files, sparse files, sockets, FIFOs, device nodes, whiteouts, unknown special nodes, and nested mount boundaries are unsupported until their semantics are deliberately specified and tested. (S001)

Feature 002 rejects final symbolic-link endpoints. It may resolve a safe intermediate directory symlink while canonicalizing the longest existing prefix. Submitted-path and resolved-anchor evidence makes retargeting before publication stale rather than a silent ownership change. (S005)

Mapping sources must exist as the declared regular file or directory kind. A destination may be absent when its nearest existing canonical ancestor is a safe directory; any existing destination must have the corresponding kind. Inputs must be absolute UTF-8 paths without parent traversal, and accepted registry paths must already equal their resolved canonical identities. (S005)

The current portable interface replaces those historical absolute declarations: sources are normalized relative to the selected project, while destinations may be absolute, home-relative, or project-relative and resolve through non-following validated ancestry within their permitted roots. The README records the same selected-project interpretation for relative destination declarations. (S001) (S004)

A non-ignored unsupported source entry or an unsupported node colliding with a managed destination blocks mutation. An unsupported destination-only entry outside the managed namespace remains untouched because Grip does not own it. Ignored unsupported source entries may be skipped when ignore evaluation excludes them before management. (S001)

Feature 003 inspection reports symbolic links, hard links, sparse files, special nodes, non-UTF-8 source names, and nested mount boundaries without following or opening them as payload. Unsupported source nodes and unsafe paired-destination collisions are blocking findings, while unsupported destination-only nodes remain nonblocking and unmanaged. Non-UTF-8 path identity is preserved through escaped display plus `raw_hex` in machine output. (S004)

The implementation uses descriptor-relative, no-follow inspection for traversed entries and performs separate source and destination passes. Policy files are accepted only as regular, single-link, non-sparse UTF-8 files; unreadable or unsupported `.gripignore` files fail inspection instead of weakening policy silently. (S006)

Feature 005 publishes ordinary files through a no-follow source descriptor and an exclusive verified sibling staging file. Addition uses no-replace rename; replacement first preserves a verified private recovery copy and then atomically renames over the expected destination. The containing directory is synchronized after visibility, and final supported state is reopened and verified so visibility, verification, and durability can be reported independently. (S008)

Missing destination parents are explicit dependency-ordered plan actions derived from endpoint evidence captured during validated registry loading. The pure planner does not inspect the filesystem, and concurrent appearance or ancestry substitution fails instead of being adopted recursively. (S008)

Pull reverses transfer roles without reversing mapping identity. The destination is opened as the transfer origin, but the existing source is the publication target: its full accepted ancestry must remain present and safe, staging occurs beside it, and the prior source is verified in private recovery before atomic replacement. Missing source content or parents block rather than becoming creation actions. (S009)

Feature 009 finalizes this boundary for current macOS with APFS. Regular files and directories compare complete supported metadata; modification time independently defines equality. Grip preserves exact path bytes, qualifies case and Unicode behavior per endpoint, blocks alias collisions, and treats unavailable, unauthorized, or lossy evidence as a precise blocker rather than coercing it. Other platforms and filesystems remain outside initial acceptance. (S012)

## Related pages

- [Grip product model](./grip-product-model.md)
- [Deletion, retirement, and recovery](./deletion-retirement-and-recovery.md)
- [Metadata and filesystem contract](./metadata-and-filesystem-contract.md)
