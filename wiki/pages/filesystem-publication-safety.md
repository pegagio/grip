---
title: Filesystem publication safety
type: reference
sources: [S008, S009]
updated: 2026-09-17
---

# Filesystem publication safety

Feature 005 publishes ordinary files through a no-follow source descriptor and an exclusive verified sibling staging file. Addition uses no-replace rename; replacement first preserves a verified private recovery copy and then atomically renames over the expected destination. The containing directory is synchronized after visibility, and final supported state is reopened and verified so visibility, verification, and durability can be reported independently. (S008)

Missing destination parents are explicit dependency-ordered plan actions derived from endpoint evidence captured during validated registry loading. The pure planner does not inspect the filesystem, and concurrent appearance or ancestry substitution fails instead of being adopted recursively. (S008)

Pull reverses transfer roles without reversing mapping identity. The destination is opened as the transfer origin, but the existing source is the publication target: its full accepted ancestry must remain present and safe, staging occurs beside it, and the prior source is verified in private recovery before atomic replacement. Missing source content or parents block rather than becoming creation actions. (S009)

## Related pages

- [Filesystem support boundaries](./filesystem-support-boundaries.md)
- [Safety model and recovery boundary](./safety-and-recovery-model.md)
