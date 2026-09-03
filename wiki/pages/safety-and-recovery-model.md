---
title: Safety and recovery model
type: decision
sources: [S001]
updated: 2026-09-03
---

# Safety and recovery model

Grip treats safety as a transaction-like boundary: validate mappings and paths, inspect the full requested scope, classify every entry, construct a deterministic plan, block on conflicts or unsafe conditions, revalidate before execution, stage replacements, verify results, and only then publish the new baseline. (S001)

Ordinary action commands mutate by default, while `-n` and `--dry-run` render the plan without changing payloads, registry data, baselines, or backups. Deletions require additional explicit authorization beyond selecting a mutating command. (S001)

Before replacing or deleting an entry, Grip retains the prior state in an operation recovery namespace under its private state root. Partial failures must be reported precisely and must not publish a baseline that falsely describes incomplete work as accepted. Grip does not automatically elevate privileges or perform Git operations. (S001)

Planning and mutation must guard against path traversal, symlink substitution, overlapping ownership, self-recursive mappings, concurrent state changes, and filesystem evidence changing between inspection and execution. Permission or metadata transitions that cannot be performed are reported rather than silently weakened. (S001)

## Related pages

- [Grip product model](./grip-product-model.md)
