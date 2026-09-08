---
title: Baseline classification and status
type: component
sources: [S007, S012]
updated: 2026-09-08
---

# Baseline classification and status

Feature 004 separates fresh observation, pure three-way classification, presentation, and explicit baseline publication. `status`, `check`, and `diff` reuse the same deterministic typed classification records; only `baseline accept` may publish machine-owned state, and none of these commands mutates source or destination payloads. (S007)

Each managed identity compares current source evidence, current destination evidence, and optional accepted baseline evidence. Without a baseline, the model distinguishes source addition, initial match, initial collision, and destination-only unmanaged evidence. With a baseline, it distinguishes synchronized, source-only and destination-only changes, converged edits, divergent conflicts, directional deletions, delete/change conflicts, converged deletion, newly ignored pending retirement, and untracked pending retirement. (S007)

Feature 004's first equality contract covered node kind, regular-file content, and permission mode. Feature 009 expands accepted State V3 to the complete file and directory metadata contract, makes modification time equality-defining, and adds explicit absent, unavailable, unsupported, unreadable, unauthorized, and observed evidence states. Field differences remain individually visible while conflict selection treats the entry as one value. (S007) (S012)

Legacy State V2 authority is never silently upgraded. Equal complete current copies are `metadata_migration_ready` and require explicit baseline acceptance; differing copies are `metadata_migration_conflict` and require an explicit whole-entry source or destination winner. (S012)

An optional selector resolves one exact mapping, managed entry, or component-boundary subtree in source space by default or destination space with `--destination`. An omitted selector includes current mappings and retained baseline-only identities, allowing removed mappings and newly ignored accepted entries to remain visible pending explicit retirement. (S007)

Baseline acceptance is an all-or-nothing state transaction. Every selected pair must have complete equivalent supported evidence; scoped acceptance preserves out-of-scope records, and an already-current request publishes no generation. Before replacement, Grip revalidates accepted state, registry, membership and policy, filesystem observations, and semantic classification records; drift leaves the prior baseline authoritative and returns a stable structured reason. (S007)

State Envelope V2 stores ordered baseline records and an integrity digest without retaining payload contents. Publication uses bounded registry-then-state coordination, attempt-owned descriptor verification, identity checks before atomic rename, exact prior-state recovery, and fail-closed handling for unsafe state directories, stale evidence, contention, and generation exhaustion. (S007)

The completed test matrix covers every classification, deterministic repetition, selectors, content and permission changes, unsupported nodes, corruption, substitution races, failed publication, output parity, and non-mutation. Its recorded macOS aarch64 release run classified 10,000 accepted entries with a 100-run p95 of 1.346 seconds, below the two-second criterion without a persistent cache or background service. (S007)

## Related pages

- [Synchronization and conflicts](./synchronization-and-conflicts.md)
- [Configuration and state](./configuration-and-state.md)
- [Command-line and path selection](./command-line-and-path-selection.md)
- [Safety and recovery model](./safety-and-recovery-model.md)
- [Metadata and filesystem contract](./metadata-and-filesystem-contract.md)
