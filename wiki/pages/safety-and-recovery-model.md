---
title: Safety and recovery model
type: decision
sources: [S001, S002, S004, S005, S007, S008, S009, S010, S011, S013]
updated: 2026-09-09
---

# Safety and recovery model

Grip treats safety as a transaction-like boundary: validate mappings and paths, inspect the full requested scope, classify every entry, construct a deterministic plan, block on conflicts or unsafe conditions, revalidate before execution, stage replacements, verify results, and only then publish the new baseline. (S001)

Ordinary action commands mutate by default, while `-n` and `--dry-run` render the plan without changing payloads, registry data, baselines, or backups. Deletions require additional explicit authorization beyond selecting a mutating command. (S001)

Before replacing or deleting an entry, Grip retains the prior state in an operation recovery namespace under its private state root. Partial failures must be reported precisely and must not publish a baseline that falsely describes incomplete work as accepted. Grip does not automatically elevate privileges or perform Git operations. (S001)

Planning and mutation must guard against path traversal, symlink substitution, overlapping ownership, self-recursive mappings, concurrent state changes, and filesystem evidence changing between inspection and execution. Permission or metadata transitions that cannot be performed are reported rather than silently weakened. (S001)

The constitution makes this safety model binding: every mutation derives from a complete validated inspection, revalidates evidence that could make an action unsafe, stages and atomically publishes replacements where supported, preserves recovery material, verifies results, and publishes a baseline only for a successfully accepted operation. Grip does not promise perfect filesystem snapshot isolation; it detects stale evidence and stops safely. (S002)

Concurrent external edits are handled through evidence capture, pre-action revalidation, safe publication, and explicit drift errors rather than attempts to lock whole payload trees. A short-lived per-user lock may protect Grip-owned registry or state publication, but it must remain bounded and provide actionable contention and stale-lock behavior. (S002)

Registry replacement and payload mutation apply this same boundary with owner-only bounded locks, lock-held plan equality, same-directory staging, private recovery, checkpoints, final verification, and one accepted-state publication. Directional deletion remains separately authorized, and recovery restore and cleanup remain explicit, compatibility-gated operations. (S005) (S008) (S009) (S010) (S011)

Current writers lazily create owner-only state and coordinate through a bounded project-local mutation lock. Read-only commands and dry runs create no state or write coordination, and unrelated projects do not contend through a global authority. (S004)

## Project and rebinding authority

Feature 010 adds project selection to the safety boundary: root, `.grip`, descriptor, ignore file, invoking-user home, descriptor bytes, and resolved topology are retained and revalidated without rediscovery before mutation. Unsafe substitution stops the operation. (S013)

State and recovery authority is portable. A copied State V4 is not accepted merely by prefix substitution; Grip must match every portable identity to the current descriptor, reobserve required accepted entries and recovery targets, and reject missing, ambiguous, stale, incomplete, unsafe, or contradictory evidence. Recovery inventory and restore derive live targets from portable identity plus endpoint role, preventing cross-project restoration through stale absolute paths. (S001) (S013)

## Related pages

- [Grip product model](./grip-product-model.md)
- [Baseline classification and status](./baseline-classification-and-status.md)
- [Mapping registry publication](./mapping-registry-publication.md)
- [Proportional engineering rigor](./proportional-engineering-rigor.md)
