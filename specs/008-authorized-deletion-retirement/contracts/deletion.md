# Domain Contract: Authorized Directional Deletion

## Eligibility

| Authority | Required classification | Required remaining peer | Effect |
|---|---|---|---|
| `source` | `source_side_deletion` | Destination equals accepted baseline | Preserve and remove destination |
| `destination` | `destination_side_deletion` | Source equals accepted baseline | Preserve and remove source |

Every selected record must have complete safe evidence. Delete/change conflicts, mismatched authority, initial or unmanaged absence, unsupported nodes, unsafe paths, missing/corrupt authority, and incomplete observation block the complete plan.

## Directory ownership boundary

A directory deletion plan includes every accepted eligible managed descendant and orders children before parents. It blocks before mutation if the target directory contains any descendant that is unmanaged, differently classified, unsupported, mounted, unsafe, or incompletely observed. Immediately before removing a directory, Grip revalidates the planned child set and proves the directory empty through a no-follow descriptor. Recursive removal is forbidden.

## Execution

1. Complete registry validation, selection, observation, classification, descendant inventory, and deterministic planning.
2. Return preview or blocked result without coordination or state changes when applicable.
3. Acquire the mutation lock, reload all evidence, rebuild the same plan, and require exact plan identity.
4. Initialize one operation record before the first recovery or removal side effect.
5. For each child-first action, revalidate authority absence, target evidence, ancestry, identity, and directory child set.
6. Preserve and verify the target under operation-bound recovery.
7. Remove the descriptor-bound file or verified-empty directory, synchronize the parent, and verify absence.
8. After all actions succeed, perform final complete observation and publish one State V2 generation without the selected baseline records.
9. Checkpoint actual visibility, verification, durability, accepted-state authority, and result delivery.

The first action failure stops later actions. Completed removals and recovery remain; automatic rollback does not occur and no baseline retirement is published after partial payload deletion.

## Invariants

- Only `delete` executes directional deletion.
- Authority flags identify accepted absence, not path space or target side.
- Every removed existing entry has verified recovery before removal.
- Unmanaged destination content is never removed.
- Plan and execution use canonical mapping-role paths without following symbolic links or crossing mount boundaries.
- State visibility and durability are reported independently after publication failures.
