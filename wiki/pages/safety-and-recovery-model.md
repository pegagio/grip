---
title: Safety and recovery model
type: decision
sources: [S001, S002, S004, S005, S007, S008, S009]
updated: 2026-09-07
---

# Safety and recovery model

Grip treats safety as a transaction-like boundary: validate mappings and paths, inspect the full requested scope, classify every entry, construct a deterministic plan, block on conflicts or unsafe conditions, revalidate before execution, stage replacements, verify results, and only then publish the new baseline. (S001)

Ordinary action commands mutate by default, while `-n` and `--dry-run` render the plan without changing payloads, registry data, baselines, or backups. Deletions require additional explicit authorization beyond selecting a mutating command. (S001)

Before replacing or deleting an entry, Grip retains the prior state in an operation recovery namespace under its private state root. Partial failures must be reported precisely and must not publish a baseline that falsely describes incomplete work as accepted. Grip does not automatically elevate privileges or perform Git operations. (S001)

Planning and mutation must guard against path traversal, symlink substitution, overlapping ownership, self-recursive mappings, concurrent state changes, and filesystem evidence changing between inspection and execution. Permission or metadata transitions that cannot be performed are reported rather than silently weakened. (S001)

The constitution makes this safety model binding: every mutation derives from a complete validated inspection, revalidates evidence that could make an action unsafe, stages and atomically publishes replacements where supported, preserves recovery material, verifies results, and publishes a baseline only for a successfully accepted operation. Grip does not promise perfect filesystem snapshot isolation; it detects stale evidence and stops safely. (S002)

Concurrent external edits are handled through evidence capture, pre-action revalidation, safe publication, and explicit drift errors rather than attempts to lock whole payload trees. A short-lived per-user lock may protect Grip-owned registry or state publication, but it must remain bounded and provide actionable contention and stale-lock behavior. (S002)

Feature 002 applies this boundary to registry replacement. A writer locks a stable owner-only file, rereads the accepted bytes, revalidates the exact submitted-path evidence and complete candidate, publishes and verifies a content-addressed recovery copy of the prior document, and verifies a same-directory staged candidate before rename. Failure before rename leaves the prior registry authoritative; a directory-sync failure after rename reports that visibility changed but durability was not confirmed. (S005)

Feature 005 applies the full boundary to source-to-destination payload mutation. An actionful push acquires the per-user mutation lock only after complete preflight, rebuilds and compares the plan under that lock, initializes a durable operation record immediately before mutation, and checkpoints revalidation, recovery, staging, publication, verification, and terminal baseline evidence. It stops on the first failure without rollback and leaves every later action unattempted. (S008)

Replacement actions preserve and verify the prior destination as a private operation-local payload before publishing the staged source. A partial failure retains visible changes and recovery evidence but leaves the prior baseline authoritative; a successful run publishes exactly one accepted generation after every action and a final complete observation verify. Result-delivery failure cannot revoke an already published baseline. (S004) (S008)

Feature 006 applies the same boundary with destination as transfer origin and source as transfer target. It stages destination bytes beside the source, preserves and verifies the prior source, never creates a missing source or source parent, and publishes one accepted generation only after every replacement and a final complete observation verify. A failed pull keeps completed effects and recovery evidence while leaving later actions unattempted and the prior baseline authoritative unless state publication explicitly became visible. (S004) (S009)

## Related pages

- [Grip product model](./grip-product-model.md)
- [Baseline classification and status](./baseline-classification-and-status.md)
- [Mapping registry publication](./mapping-registry-publication.md)
- [Proportional engineering rigor](./proportional-engineering-rigor.md)
