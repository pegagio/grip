# Exact Forced Missing-Peer Restoration Contract

## Directional behavior

| Selected command | Selected winner | Peer state | Required outcome |
| --- | --- | --- | --- |
| `grip push --force SOURCE` | Source present | Destination absent | Restore destination from the complete current source state. |
| `grip pull --force --destination DESTINATION` | Destination present | Source absent | Restore source from the complete current destination state. |
| `grip push --force SOURCE` | Source absent | Destination unchanged | Preserve existing behavior: propagate source absence to delete destination. |
| `grip pull --force --destination DESTINATION` | Destination absent | Source unchanged | Preserve existing behavior: propagate destination absence to delete source. |

The present selected winner may equal or differ from accepted state. Direction-incompatible deletion/change conflicts remain blocked.

## Selection contract

The supplied selector must resolve to one exact established managed identity. File mappings and exact managed tree members are eligible. Omitted selectors, mapping roots, subtrees, aggregate tree entries, ambiguous selections, ignored entries, unmanaged paths, and entries with unrelated safety blockers remain rejected before mutation.

## Preview and completion contract

`--dry-run` reports the one selected restoration action without changing endpoints, declarations, state, locks, or operation evidence. Execution revalidates the selected current winner before publication, verifies the restored peer, and publishes fresh accepted evidence only after success.

## Human-output contract

For an exact selectable one-sided absence, user-facing output describes a missing peer rather than claiming that the surviving winner changed. It presents an executable forced direction only when the entry meets exact-selection requirements.
