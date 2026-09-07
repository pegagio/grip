---
title: Deletion, retirement, and recovery
type: component
sources: [S011]
updated: 2026-09-07
---

# Deletion, retirement, and recovery

Feature 008 keeps destructive intent outside ordinary synchronization. `grip delete (--source|--destination) [-n|--dry-run] [--] PATH` names the side whose absence is authoritative; a changed remaining peer creates a delete/change blocker, and push, pull, and sync never infer permission to delete. (S011)

Deletion planning covers the complete selected scope and uses canonical child-before-parent ordering. Execution rebuilds the plan under the mutation lock, revalidates every action, preserves and verifies the remaining peer in private recovery, removes it descriptor-relatively, verifies both sides are absent, and retires accepted evidence only after the complete selected operation succeeds. (S011)

Directory removal requires a fresh verified-empty enumeration. Unmanaged descendants, differently classified children, unsupported nodes, mount boundaries, unsafe ancestry, symlink substitution, and concurrent children block the complete deletion before mutation. (S011)

`grip retire [-n|--dry-run] [--destination] [--force] (--all|[--] PATH)` is a state-only transition for newly ignored, untracked, or converged-deletion records. It changes no payloads, requires an exact path or explicit `--all`, rejects active records, and requires `--force` after reporting differing surviving copies. (S011)

`grip recovery list` and `show RECOVERY_REF` expose provenance, integrity, availability, byte count, and restoration eligibility without printing recovered payload content. `restore [-n|--dry-run] RECOVERY_REF` accepts only exact verified evidence whose bound target and remaining authority pass complete compatibility checks; a successful restore retains the source recovery entry and does not accept a new synchronization baseline. (S011)

`grip recovery remove [-n|--dry-run] --confirm RECOVERY_REF...` removes only exact confirmed recoverable bytes. Cleanup preserves immutable provenance and publishes a cleaned tombstone; partial failure distinguishes removed, failed, and unattempted entries without changing accepted registry or baseline authority. (S011)

## Related pages

- [Command-line and path selection](./command-line-and-path-selection.md)
- [Configuration and state](./configuration-and-state.md)
- [Filesystem support boundaries](./filesystem-support-boundaries.md)
- [Safety and recovery model](./safety-and-recovery-model.md)
- [Synchronization and conflicts](./synchronization-and-conflicts.md)
