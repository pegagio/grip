---
title: Git-inspired command hierarchy
type: decision
sources: [S014]
updated: 2026-09-10
---

# Git-inspired command hierarchy

Grip intentionally uses a flat, Git-inspired public interface: `init`, `version`, `add`, `list`, `remove`, `status`, `diff`, `push`, `pull`, and `sync`. The hierarchy describes the operator's deployment workflow rather than internal concepts such as baselines, retirement, or recovery. (S014)

`add` records mapping intent without copying payloads and can infer a file or tree mapping when at least one endpoint exists. `remove` deletes only the declaration and its baseline evidence. `.gripignore` remains a directly edited policy file, so Grip supplies no per-entry tracking or ignore command. (S014)

Normal synchronization does not choose a winner for divergent, one-sided-absent, or unbaselined-different entries. An exact-one-entry `push -f` chooses the source state and `pull -f` chooses the destination state; either complete winner may be absent. Git or another operator-selected system owns history and recovery, leaving Grip with internal synchronization evidence rather than a recovery interface. (S014)

## Related pages

- [Command-line and path selection](./command-line-and-path-selection.md)
- [Mappings and managed membership](./mappings-and-managed-membership.md)
- [Synchronization and conflicts](./synchronization-and-conflicts.md)
- [Safety model and recovery boundary](./safety-and-recovery-model.md)
- [Implementation and delivery direction](./implementation-and-delivery-direction.md)
