---
title: Synchronization and conflicts
type: component
sources: [S001]
updated: 2026-09-10
---

# Synchronization and conflicts

`grip push` applies only unambiguous source-to-destination changes. `grip pull` applies only unambiguous destination-to-source changes. `grip sync` combines safe changes in both directions and blocks the selected scope on any conflict. `-n` previews each operation without mutation. (S001)

`push -f PATH` makes the source authoritative for exactly one managed entry. `pull -f PATH` makes the destination authoritative for exactly one managed entry. Force may propagate intentional absence. It is rejected for broad, ambiguous, mapping-wide, or tree-wide selection and is unavailable to `sync`. (S001)
