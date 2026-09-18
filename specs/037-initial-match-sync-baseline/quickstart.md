# Quickstart: Initial-Match Baseline Synchronization

Use an isolated temporary Grip project containing a declared tree mapping. Create the same supported regular file below both source and destination trees, but ensure no accepted baseline exists for that child.

1. Run `grip status source-tree/nested/file` and confirm the child appears under `Needs baseline`.
2. Capture endpoint and accepted-state snapshots, then run `grip sync --dry-run source-tree/nested/file`. Confirm the output says it would establish a baseline and every snapshot is unchanged.
3. Run `grip sync source-tree/nested/file`. Confirm the output identifies baseline establishment and lists no payload-copy action.
4. Run `grip status source-tree/nested/file`. Confirm it is current.
5. Repeat with unequal, absent, ignored, destination-only, unsupported, and unsafe cases. Confirm no accepted state is published for each excluded condition.

Run the focused sync tests, then the project validation task before accepting the feature.

## Recorded Outcome

Automated isolated fixtures completed the exact tree-child preview and execution path: preview reported baseline establishment with zero payload actions and no state publication; execution established only the selected child's baseline; and its sibling remained under `Needs baseline`. No-selector, tree-root, and multi-entry scopes remained no-ops. Initial collisions remained blocked, ignored and destination-only entries remained unaccepted, a missing peer remained a directional action rather than acceptance-only work, and existing destination-link safety coverage remained blocking. The focused sync suites and `mise run validate` passed without using real user paths.
