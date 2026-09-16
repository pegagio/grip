---
title: Exact destination symlink replacement
type: component
sources: [S026]
updated: 2026-09-15
---

# Exact destination symlink replacement

An exact managed destination leaf that is a symbolic link is an unresolved obstacle, not supported payload or accepted baseline evidence. `grip add` can record an otherwise valid file or source-defined tree mapping without following, replacing, or modifying that link or its target. (S026)

Status and dry-run output identify the managed member and exact destination path. Ordinary push, pull, sync, deletion paths, destination-winning force, aggregate selection, source links, and destination-link ancestors remain blocked. (S026)

Only `grip push --force SOURCE` for one exact supported source entry can replace the revalidated leaf link object. File replacement uses staged source state; directory replacement publishes an empty private sibling directory before applying descendants and metadata. The former target is never read or changed, and accepted state is published only after verification. (S026)

## Related pages

- [Mappings and managed membership](./mappings-and-managed-membership.md)
- [Force mapping replacement](./force-mapping-replacement.md)
- [Filesystem support boundaries](./filesystem-support-boundaries.md)
- [Baselines, classification, and status](./baseline-classification-and-status.md)
