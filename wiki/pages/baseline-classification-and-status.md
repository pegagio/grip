---
title: Baselines, classification, and status
type: component
sources: [S001, S004, S018]
updated: 2026-09-11
---

# Baselines, classification, and status

A baseline is current accepted evidence for an actively managed entry, not stored file content or recovery history. `grip status` classifies source, destination, and baseline without mutation. `status -e` returns attention when work is needed. `grip diff` is likewise read-only. (S001)

Default human status output is path-centered: it summarizes the selected scope and shows only nonempty `Conflicts`, `Changes to push`, `Changes to pull`, and `Needs baseline` groups. The arrows respectively express a conflict, source-to-destination work, destination-to-source work, and non-directional reconciliation attention; detailed comparison evidence is available through `grip diff` or JSON output. (S004)

The `>-<` baseline signal does not select a payload-copy direction. It distinguishes baseline or reconciliation work from safe push and pull directions while leaving `status -e` as the attention exit-status interface. (S001)

Feature 015 fixes the human-status group order as conflicts, push, pull, then baseline attention; empty groups and individually current entries are omitted. Safety blockers remain visible in plain language, whereas informational compatibility findings remain in detailed machine output but are quiet in the default view. (S018)

Unambiguous one-sided changes can synchronize normally. Divergence, initial collision, unbaselined differences, and one-sided absence block ordinary operations. Removing a mapping or ignoring an entry prunes its baseline; later reintroduction is new membership. (S001)
