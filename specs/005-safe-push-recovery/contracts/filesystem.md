# Filesystem Contract: Push Publication and Recovery

This contract applies to ordinary regular files and directories on the supported macOS and Unix boundary. It does not promise filesystem snapshot isolation or metadata outside Feature 004's supported equality state.

## Descriptor boundary

- Open every existing root and directory with directory, close-on-exec, and no-follow semantics.
- Resolve descendants from already validated directory descriptors using raw component bytes.
- Inspect final components without following symbolic links.
- Open regular source and destination files from their validated parent descriptors.
- Never reopen a composed display path to make a mutation decision.
- Treat changed device, inode, kind, link count, sparse evidence, supported mode, length, or content fingerprint as stale or unsupported according to the established observation contract.

The implementation extends the current `discovery::filesystem::Directory` and `observation::fingerprint` patterns behind a focused mutation interface. Planner types never receive unrestricted descriptors.

## Directory creation

For one planned parent or managed-directory action:

1. Revalidate the opened parent and require the child name to be absent through no-follow inspection.
2. Create exactly one child directory with requested mode `0700`.
3. Reopen the child as a no-follow directory and verify its kind and identity.
4. Sync the parent directory.
5. Mark the action visible, verified, and durable.

A concurrent child appearance is stale evidence even when the node is a directory. Recursive directory creation is forbidden. Directory mode parity with the source remains Feature 009 work.

## File staging

For each managed file action:

1. Create an exclusive attempt-owned hidden sibling in the final destination parent with no-follow and close-on-exec flags and initial mode `0600`.
2. Open the source through its validated descriptor chain.
3. Stream source bytes to staging with a bounded buffer while calculating SHA-256 and length.
4. Revalidate source descriptor metadata before and after streaming.
5. Apply the planned supported permission mode through the staging descriptor.
6. Sync, rewind, reread, and verify the staging bytes, length, mode, and descriptor/path identity against the planned source Supported State.
7. Checkpoint staging as verified before publication.

If staging fails, remove only an attempt-owned pathname whose current device and inode still identify the opened staging file. Preserve substituted or unknown nodes for explicit inspection and report the unsafe condition.

## Addition publication

Immediately before publication, revalidate the source, destination ancestry, and expected destination absence. Publish with same-directory no-replace rename. If a destination appeared, stop as stale evidence without overwriting it.

After rename, sync the destination parent, reopen the destination no-follow, fingerprint it, and require equality with the planned source Supported State. Record visibility, verification, and durability separately.

## Replacement recovery

Before replacement publication:

1. Reopen the current destination no-follow and require equality with the plan's expected prior state.
2. Stream it into an exclusive private recovery staging file under the operation's numeric action directory.
3. Sync, atomically publish without replacement, and sync the recovery directory.
4. Reopen and verify the recovery payload against the prior Supported State.
5. Checkpoint the operation record with the relative recovery reference and `preserved` state.
6. Revalidate the current destination again immediately before replacement rename.

Recovery payloads use private mode `0600`; the original supported mode lives in typed recovery metadata. A non-identical existing recovery target is a fail-closed collision.

## Replacement publication

After verified recovery and final revalidation, rename the verified sibling staging file over the destination. The rename provides atomic visibility on the supported filesystem but is not compare-and-swap. Sync the parent, reopen the destination no-follow, and verify its complete Supported State.

Grip reports these cases distinctly:

| Rename | Parent sync | Final verification | Report |
|---|---|---|---|
| not attempted or failed | not attempted | not attempted | not visible; prior destination expected |
| succeeded | failed | unknown or verified | visible; durability unconfirmed |
| succeeded | succeeded | failed | visible and durable; verification failed |
| succeeded | succeeded | passed | visible, durable, and verified |

The executor never attempts automatic rollback.

## Parent ancestry constraints

- Synthetic parent actions may create only components between an already validated existing destination ancestor and an accepted mapped destination.
- Every component is a literal raw-byte child name; parent traversal and symbolic-link substitution are rejected.
- A synthetic parent records every managed identity that authorized it.
- Destination-only siblings and descendants outside source-defined membership remain untouched.

## Durability boundary

File synchronization plus containing-directory synchronization is Grip's supported OS-level durability confirmation. It does not claim physical-media cache flush guarantees beyond the host operating system contract.

## Unsupported conditions

Grip never opens or mutates symbolic links, hard-linked files, sparse files, sockets, FIFOs, devices, whiteouts, unknown special nodes, nested mounts, wrong-kind collisions, or filesystem entries whose required supported mode cannot be applied and verified.

## Test-only fault points

The implementation exposes typed, hidden test hooks for:

- after initial preflight and after mutation-lock acquisition;
- before action revalidation;
- before and after parent creation;
- source change during streaming;
- staging create, write, mode, sync, corruption, and path substitution;
- recovery create, write, publish, sync, corruption, and verification;
- destination appearance, disappearance, or change before publication;
- before and after payload rename;
- parent-directory sync failure;
- final destination verification failure;
- operation-record checkpoint failure;
- accepted-state publication faults; and
- final result-writer failure.

Fault injection is unavailable through production CLI options or environment variables.
