---
title: Baselines, classification, and status
type: component
sources: [S001, S004, S018, S019, S020, S026]
updated: 2026-09-17
---

# Baselines, classification, and status

A baseline is current accepted evidence for an actively managed entry, not stored file content or recovery history. `grip status` classifies source, destination, and baseline without mutation. `status -e` returns attention when work is needed. `grip diff` is likewise read-only. (S001)

Default human status output is path-centered: it summarizes the selected scope and shows only nonempty `Conflicts`, `Changes to push`, `Changes to pull`, and `Needs baseline` groups. The arrows respectively express a conflict, source-to-destination work, destination-to-source work, and non-directional reconciliation attention; detailed comparison evidence is available through `grip diff` or JSON output. (S004)

The `>-<` baseline signal does not select a payload-copy direction. It distinguishes baseline or reconciliation work from safe push and pull directions while leaving `status -e` as the attention exit-status interface. (S001)

Feature 015 introduced concise path-centered status groups. Feature 017 defines their current order as current entries, changes to push, changes to pull, conflicts, then needs baseline; empty groups are omitted. Default human mutation results likewise use concise action, no-action, blocked, and failure projections while JSON and `grip diff` retain detailed evidence. (S018) (S019) (S001)

For a newly added unequal source-defined member, Grip records the destination as the initial comparison reference without changing either payload. The existing classifier then presents the source as an ordinary pending push; later destination drift remains a conflict. [Mapping addition and initial baselines](./mapping-addition-and-initial-baselines.md) records the publication boundary. (S020)

Unambiguous one-sided changes can synchronize normally. Divergence, initial collision, unbaselined differences, and one-sided absence block ordinary operations. Removing a mapping or ignoring an entry prunes its baseline; later reintroduction is new membership. (S001)

An exact managed destination symlink is classified as `unresolved_destination_link`, not as supported payload or accepted baseline evidence. Status and blocked dry runs identify the managed source and link path; only an executable exact source-winning force command is presented. (S026)

## Related pages

- [Exact destination symlink replacement](./exact-destination-symlink-replacement.md)
