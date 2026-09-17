---
title: Synchronization and conflicts
type: component
sources: [S001, S004, S023, S027]
updated: 2026-09-17
---

# Synchronization and conflicts

`grip push` applies only unambiguous source-to-destination changes. `grip pull` applies only unambiguous destination-to-source changes. `grip sync` combines safe changes in both directions and blocks the selected scope on any conflict. `-n` previews each operation without mutation. (S001)

`push -f PATH` makes the source authoritative for exactly one managed entry. `pull -f PATH` makes the destination authoritative for exactly one managed entry. Force may propagate intentional absence. It is rejected for broad, ambiguous, mapping-wide, or tree-wide selection and is unavailable to `sync`. (S001)

Default human conflict guidance presents force commands only when the displayed selector resolves to that one exact managed entry. An aggregate tree conflict instead directs the operator to `grip diff SOURCE` to identify an exact entry before choosing a winner. (S004)

One-sided absence is an ordinary synchronization blocker; human status presents source- and destination-winning force commands for an exact selectable entry. (S001)

For an exact one-sided absence, a present selected winner restores only its missing peer; a selected absent winner retains the established deletion behavior only when the present peer remains unchanged. Deletion/change conflicts remain blocked. (S004) (S023)

An exact forced restoration accepts a current winner that differs from its prior accepted state, applies the same identity during dry-run and execution, and refreshes accepted evidence only after verification. (S023)

No-selector `grip push --force` deliberately selects the complete source-winning state for all managed entries in one project. It performs full preflight validation, keeps destination leaf links and symlink ancestry blocking, revalidates each action, and publishes accepted evidence only after each entry verifies. A later failure stops the aggregate, records completed, failed, and unattempted entries distinctly, and retains only earlier verified publications. (S027)
