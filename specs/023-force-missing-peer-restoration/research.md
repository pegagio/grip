# Research: Forced Missing-Peer Restoration

## Decision: Treat a present winner with a missing peer as a forced restoration action

The exact forced-resolution path will use the existing directional mutation publication path when the selected winner exists and its peer is absent. A source winner restores the destination; a destination winner restores the source.

**Rationale**: The selected winner's complete current state is already observed, revalidated, staged, verified, and eligible for accepted-state publication by the existing mutation model. The reported defect is that these classifications are blocked before reaching that model.

**Alternatives considered**:

- Create a new restore operation or public command: rejected because force already expresses the operator's exact winner choice.
- Route restoration through the deletion planner: rejected because it models an absent winner removing the remaining peer, not a present winner recreating one.
- Make ordinary push and pull accept one-sided absence: rejected because ordinary operations must continue to block absence until an operator explicitly chooses a winner.

## Decision: Preserve absent-winner deletion as a distinct force path

The existing source-absent push and destination-absent pull cases remain deletion operations. Present-winner restoration adds only the opposite direction-compatible one-sided-absence classifications to exact conflict-resolution planning, including cases where the present winner changed after acceptance.

**Rationale**: This retains explicit deletion authority and allows an operator's force selection to use the winner's complete current state rather than treating changed-winner absence as an unsolvable conflict.

**Alternatives considered**:

- Collapse deletion and restoration into one generalized plan: rejected because their target-state verification and accepted-baseline outcomes differ, increasing risk without user value.

## Decision: Reuse exact-entry selection and action-level revalidation unchanged

No tree root, subtree, or aggregate selector becomes force-resolvable. The repaired cases require an established exact identity and use the existing mutation lock, complete inspection, action preflight, post-publication verification, and baseline refresh.

**Rationale**: The regression occurs after exact selection succeeds. Selection and safety boundaries are already correct and must not be broadened to fix it.

**Alternatives considered**:

- Permit mapping-wide force to simplify restoration: rejected because it could overwrite or delete unrelated managed members.
- Add new persisted restoration evidence: rejected because the existing plan and accepted baseline evidence are sufficient.

## Decision: Restore missing files through add-without-replace publication

An exact present-winner file restoration uses the existing add-file action rather than a replacement action. This preserves the existing no-replace filesystem publication check if an external process recreates the missing peer after planning.

**Rationale**: A missing peer must not be silently overwritten if it reappears between inspection and publication.
