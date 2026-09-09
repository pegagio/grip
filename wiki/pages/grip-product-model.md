---
title: Grip product model
type: concept
sources: [S001, S013]
updated: 2026-09-09
---

# Grip product model

Grip is a local, per-user command-line utility for selectively synchronizing files and directory trees through explicit mappings between filesystem locations. It is stateful, bidirectional, and designed as an overlay that manages selected paths without owning the surrounding trees. (S001)

The source and destination labels establish mapping identity, source-side discovery, ignore policy, and the meaning of directional commands. They do not create a permanently authoritative editing side after an entry becomes managed; an unambiguous change may flow in either direction. (S001)

Grip uses an accepted baseline to distinguish one-sided changes from divergent edits. It does not choose winners solely from current timestamps, and its initial scope excludes remote synchronization, background watching, multi-user coordination, privileged services, automatic conflict merging, and broad ownership of destination-only content. (S001)

The name reflects the ownership boundary: Grip has an explicit grip on selected paths and nothing surrounding them. The product should not be defined primarily as “stateful rsync,” because that framing suggests stateless mirroring and broader tree ownership than Grip intends. (S001)

## Project-scoped product boundary

A **Grip project** is an initialized source directory. Its portable mapping intent lives in `.grip/config.toml`, while mutable baselines, locks, operations, staging, and recovery live only beneath ignored `.grip/state/`. The descriptor and `.grip/.gitignore` may be committed without committing local operational evidence. (S001) (S013)

Project selection is explicit through `--project PATH` or implicit through a bounded upward walk. Grip has no global mapping installation, generated project identity, daemon, or persistent project index. A copied project resolves the same declarations against its own root and invoking-user home. (S001) (S013)

## Related pages

- [Mappings and managed membership](./mappings-and-managed-membership.md)
- [Synchronization and conflicts](./synchronization-and-conflicts.md)
- [Safety and recovery model](./safety-and-recovery-model.md)
- [Command-line and path selection](./command-line-and-path-selection.md)
- [Configuration and state](./configuration-and-state.md)
- [Filesystem support boundaries](./filesystem-support-boundaries.md)
- [Implementation and delivery direction](./implementation-and-delivery-direction.md)
