# Filesystem Contract: Mixed Direction and Resolution

Feature 007 adds no new filesystem primitive. Every action derives internal origin and target from its direction while retaining stable mapping-role paths.

## Per-action boundary

1. Revalidate registry, baseline, identity, both observed sides, and target ancestry.
2. Open the origin without following symbolic links.
3. Create an exclusive sibling staging entry beside the target.
4. Copy complete supported state, synchronize it, and verify it.
5. Preserve and verify the existing target in operation-local recovery when replacement is required.
6. Revalidate origin and target immediately before publication.
7. Publish with the strongest supported atomic target operation, synchronize the parent, and report visibility separately from durability.
8. Verify the final target against the expected winner/origin state.

Push additions retain existing safe destination-parent actions. Pull and resolution never synthesize missing source parents. Directory metadata transitions remain blocked under the current supported boundary where the verified directional pipeline blocks them.

## Mixed-plan guarantees

- Actions on different filesystems stage beside their own targets.
- Canonical action ordering does not imply multi-file or cross-filesystem atomicity.
- The first failure stops all later actions regardless of direction.
- Completed effects and recovery remain visible evidence; automatic rollback is excluded.
- External changes are handled by revalidation, not payload-tree locks.
