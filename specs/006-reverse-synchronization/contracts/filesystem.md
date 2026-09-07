# Filesystem Contract: Pull Publication and Recovery

This contract reverses transfer roles while preserving the Feature 005 descriptor, staging, recovery, publication, verification, and durability boundary.

## Descriptor boundary

- Open mapping roots and descendant directories with close-on-exec and no-follow behavior.
- Retain mapping-role source and destination identity while deriving destination as origin and source as target.
- Resolve descendant components from validated directory descriptors and inspect final nodes without following links.
- Require the established source target and all accepted source ancestry to remain present, supported, and unchanged.
- Revalidate device, inode, kind, link count, sparse evidence, supported mode, length, content fingerprint, and relevant ancestry immediately before side effects.
- Never reopen a display string to make a mutation decision.

## Staging

For each pull replacement:

1. Open the destination origin through its validated descriptor chain and require its complete supported state to equal `expected_destination`.
2. Create one exclusive attempt-owned hidden sibling in the source target's validated parent with mode `0600`.
3. Stream destination bytes into staging with a bounded buffer while calculating SHA-256 and length.
4. Revalidate destination descriptor evidence before and after streaming.
5. Apply the planned supported destination permission mode through the staging descriptor.
6. Sync, rewind, reread, and verify staging bytes, length, mode, and descriptor/path binding.

Attempt-owned cleanup removes only a staging pathname still bound to the opened staged file. A substituted or unknown path is preserved for explicit inspection and reported as unsafe.

## Source recovery

Before source replacement:

1. Reopen the current source without following links and require equality with `expected_source`.
2. Copy it into the action's exclusive private operation recovery directory.
3. Sync and publish the recovery payload without replacement.
4. Reopen and verify recovery against the prior source supported state.
5. Publish recovery metadata bound to operation ID, action index, and Entry Identity.
6. Checkpoint `recovery: preserved` before source publication.
7. Revalidate source and destination again immediately before replacement.

## Replacement publication

Rename verified sibling staging over the source target, sync the source parent, reopen the source without following links, and require equality with the planned destination supported state. Visibility, durability, and verification are reported independently. No automatic rollback is attempted.

## Source ancestry

Pull creates no source node and no parent directory. If the source or any required ancestry component is absent, replaced, a symbolic link, the wrong kind, unsupported, inaccessible, or outside accepted mapping ancestry, the action does not begin. That evidence remains a deletion, conflict, stale, or unsafe condition under the inherited classification contract.

## Unsupported conditions

Pull never opens or mutates symbolic links, hard-linked files, sparse files, sockets, FIFOs, devices, whiteouts, unknown special nodes, nested mounts, wrong-kind collisions, or entries whose required supported mode cannot be applied and verified.

## Test-only fault points

The shared typed fault seam covers lock acquisition, action revalidation, source recovery, staging, payload publication, source-parent synchronization, source verification, final observation, state publication, terminal summary, and result delivery. It is unavailable through production arguments or environment variables.
