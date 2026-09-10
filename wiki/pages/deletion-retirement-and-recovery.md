---
title: Absence, removal, and recovery boundaries
type: component
sources: [S001]
updated: 2026-09-10
---

# Absence, removal, and recovery boundaries

Grip has no public `delete`, `retire`, `resolve`, or recovery command. `grip remove SOURCE` removes only the mapping declaration and baseline evidence; it does not delete either endpoint. (S001)

One-sided absence blocks ordinary synchronization. An operator may intentionally propagate absence only through an exact-one-entry `push -f` or `pull -f`, which chooses source or destination as the winner. (S001)

Git, not Grip, owns repository history and recovery. (S001)
