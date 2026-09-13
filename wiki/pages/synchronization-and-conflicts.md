---
title: Synchronization and conflicts
type: component
sources: [S001, S004]
updated: 2026-09-13
---

# Synchronization and conflicts

`grip push` applies only unambiguous source-to-destination changes. `grip pull` applies only unambiguous destination-to-source changes. `grip sync` combines safe changes in both directions and blocks the selected scope on any conflict. `-n` previews each operation without mutation. (S001)

`push -f PATH` makes the source authoritative for exactly one managed entry. `pull -f PATH` makes the destination authoritative for exactly one managed entry. Force may propagate intentional absence. It is rejected for broad, ambiguous, mapping-wide, or tree-wide selection and is unavailable to `sync`. (S001)

Default human conflict guidance presents force commands only when the displayed selector resolves to that one exact managed entry. An aggregate tree conflict instead directs the operator to `grip diff SOURCE` to identify an exact entry before choosing a winner. (S004)
