# Domain Contract: Whole-Entry Conflict Resolution

## Eligibility

A request is eligible only when complete registry validation succeeds, the source-space path resolves to one exact established Entry Identity, current source and destination observations are complete and supported, accepted baseline evidence exists, and classification is `divergent_conflict`.

| Winner | Action direction | Preserved loser | Final state |
|---|---|---|---|
| `source` | `push` | destination | Both sides equal complete source state |
| `destination` | `pull` | source | Both sides equal complete destination state |

The winning state includes all currently supported content and metadata. Attributes are never combined across sides.

## Preview and execution

- Preview produces the same one-action semantic plan as execute over unchanged evidence and mutates nothing.
- Execution acquires mutation coordination, rebuilds the exact conflict plus winner plan, and rejects changed evidence before record creation or payload mutation.
- The losing side is preserved and verified before staging is published.
- Publication and final verification reuse the transfer direction's established filesystem contract.
- One baseline generation is published only after both sides equal the selected complete winner.

## Rejections

Resolution rejects stale evidence, missing baseline, initial collision, synchronized or converged state, one-sided change, deletion, retirement, unsupported or unsafe evidence, a non-exact selector, a destination-space path, or ambiguous/missing winner flags. It never broadens to adjacent entries or remembers an earlier decision.
