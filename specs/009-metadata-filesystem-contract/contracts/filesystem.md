# Filesystem Contract

This contract defines how Grip inspects, plans, mutates, and verifies managed filesystem entries on current macOS with APFS.

## Descriptor Binding

Traversal and entry access must use non-following descriptor-bound operations. Grip opens relative to validated parent descriptors with no-follow semantics, captures object and ancestry evidence, and re-stats immediately before and after each action. Path strings alone never authorize mutation.

The platform adapter uses descriptor forms of xattr, ownership, mode, timestamp, BSD-flag, ACL, filesystem-stat, and attribute-list operations wherever Darwin exposes them.

## Supported Nodes

Only ordinary regular files and ordinary directories are supported. The following conditions are reported precisely and are blocking when they occur in managed membership:

- Symbolic links, without opening or following their target
- Regular files with link count other than one
- Regular files marked with the authoritative sparse extended flag
- Sockets, FIFOs, character devices, block devices, whiteouts, and unknown node kinds
- Mounted-on directories below a tree mapping root

An unsupported source node excluded before discovery membership is nonblocking. An unsupported destination-only node outside managed ownership remains untouched and nonblocking.

## Endpoint Capabilities

Preflight records capability evidence from each concrete endpoint, including filesystem type, filesystem/device identity, mount flags, case behavior, returned attribute masks, mtime precision, and support for every required metadata inspection, application, and verification operation. Platform name alone is insufficient evidence.

Every blocker identifies endpoint, safe path, field or node property, required value or behavior, evidence state, stable reason, and human explanation. Preflight is read-only and must complete for the whole selected scope before mutation begins.

## Path and Collision Behavior

Managed identity preserves exact filesystem component bytes. Grip never normalizes Unicode, folds case, or renames an entry to manufacture compatibility.

Case behavior comes from APFS volume capability evidence. Unicode-equivalence behavior comes from a comparator qualified against actual APFS lookup fixtures on the recorded OS and APFS build. Distinct selected identities that alias on either endpoint form one collision finding containing every identity and block the mapping or selected scope. Inconclusive comparison capability also blocks.

## Mutation Order

For a created or replaced entry, Grip must:

1. Stage and verify content under restrictive temporary permissions.
2. Apply numeric owner and group.
3. Apply the ordered extended ACL.
4. Apply allowlisted xattrs.
5. Apply the final full permission mode.
6. Apply the final modification time.
7. Apply supported BSD flags.
8. Flush durability and verify the complete state.

Required clearing of supported immutable or append-only flags is an explicit preflighted and revalidated step. Unsupported or protected flags are never cleared opportunistically.

Metadata-only changes are explicit recoverable actions. Child actions complete before directory finalizers; directory finalizers run from deepest to shallowest so descendant mutation cannot invalidate accepted directory metadata.

## Revalidation and Recovery

Immediately before an action, Grip revalidates content fingerprint where applicable, complete metadata, node kind, link count, sparse status, mount and filesystem identity, parent ancestry, target collision assumptions, authorization, capabilities, and accepted-state generation. Drift stops execution under the existing partial-failure contract.

Before replacement, deletion, or in-place metadata change can remove the prior logical state, Grip must preserve and verify sufficient Recovery Metadata V2 evidence. If preservation or binding fails, the original entry remains untouched.

## Verification

Success requires re-reading every supported equality field, checking it against the selected complete winner, confirming action durability, and verifying the complete selected scope. Grip publishes accepted state only after all selected actions and directory finalizers succeed. Unsupported physical representation such as clone or compression identity is never claimed.
