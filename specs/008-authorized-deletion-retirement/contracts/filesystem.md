# Filesystem Contract: Removal and Restoration

## Descriptor-safe removal

1. Validate and open every ancestor without following symbolic links.
2. Revalidate the target's node kind, supported state, identity evidence, and authoritative peer absence.
3. Preserve and verify the target in private operation recovery.
4. For a file, issue descriptor-relative unlink against the validated parent/name pair.
5. For a directory, verify all planned managed children absent, enumerate the directory again, require it empty, and issue descriptor-relative directory removal.
6. Synchronize the containing directory and report visibility separately from durability.
7. Reinspect through the validated parent and require target absence.

Recursive removal, symlink following, mounted-subtree crossing, and removal of unmanaged descendants are forbidden.

## Payload restoration

1. Open and verify recovery bytes within their private recovery directory without following links.
2. Validate the manifest's bound mapping-role target and current ancestry.
3. Require current target absence or exact equality to recorded post-action evidence.
4. If occupied and eligible, preserve the displaced state under the restore operation.
5. Stage recovered content beside the target, apply the recorded supported mode, synchronize and verify staging, then publish with the strongest supported atomic operation.
6. Synchronize the target parent and verify exact restored supported state.

Directory recovery is composed from individual directory and file entries; no archive extraction is introduced. Restoration never follows links, crosses mounts, or opens unsupported nodes as payload.

## Drift and failure

Every action revalidates evidence immediately before its first side effect. Changed ancestry, target, peer absence, directory children, recovery bytes, authority document, or accepted membership stops the action. Completed effects and newly created recovery remain; later actions are unattempted. No workflow claims multi-entry or cross-filesystem atomicity.
