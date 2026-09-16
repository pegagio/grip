# Research: Exact Destination Symlink Replacement

## Decision 1: Observe the final destination link object without following it

The destination exact-probe outcome gains a narrow `destination_leaf_link` form containing the managed identity, safe opened ancestors, and no-follow metadata identity for the final link object. It does not read link text, open the referent, or include target-derived state. A source link, ancestor link, wrong-kind leaf, special node, mount boundary, or inaccessible path remains the existing blocking outcome.

This reuses the descriptor-relative child metadata and probe boundary introduced by Features 009 and 025. The parent descriptor proves safe ancestry, while final child metadata distinguishes the object without target access.

Rejected alternatives:

- Reclassifying all unsupported destination nodes as replaceable would weaken the allowlisted payload boundary.
- Calling `readlink` or canonicalizing the final link would access target information outside the feature's authority.
- Treating the link as absence would lose the evidence needed to prevent a stale or substituted replacement.

## Decision 2: Keep unresolved link evidence runtime-only

`DestinationLeafLinkEvidence` is attached to current observation and the planned replacement action. It contains no-follow final-object identity sufficient to detect replacement or retargeting of the directory entry, plus the existing required-ancestor evidence. Project descriptor V2 and State V4 remain unchanged. The admitted member receives no accepted baseline until successful forced replacement and verification.

This keeps portable user intent separate from local transient filesystem evidence and avoids treating an unsupported node as accepted payload.

Rejected alternatives:

- Persisting link object identity would become stale across normal filesystem replacement and add a machine-local migration obligation.
- Persisting target text would violate the no-follow boundary and provide no required synchronization value.

## Decision 3: Require a supported source for add-time link admission

When the destination endpoint itself is a link, `add` derives the mapping kind from an existing supported source file or directory. A destination-only link remains rejected because the existing CLI has no declaration for its intended file-versus-tree kind. For a tree mapping, only current non-ignored source identities and retained accepted identities are probed; a final leaf link is admitted while a root or intermediate link blocks.

This preserves the current interface and avoids inferring ownership or payload type from a link target.

Rejected alternatives:

- Inferring type from the target would follow the link and make admission depend on target content.
- Adding a type flag or destination-only link mode exceeds Feature 026's scope.

## Decision 4: Add a dedicated unresolved-link classification

An exact managed destination link is classified as an unresolved destination-link obstacle. It is blocking for `add` baseline acceptance, ordinary push, pull, sync, delete, and destination-winning force. Its diagnostic identifies the managed source and exact destination link path. It offers `grip push --force SOURCE` only after the standard source display maps back to one exact active identity.

The exact-source forced path may select an active identity without a baseline only after reinspection establishes that the selector resolves to exactly one unresolved destination-link record. All mapping, subtree, aggregate, destination-space, and ambiguous selections remain rejected.

Rejected alternatives:

- Reusing the generic unsafe collision classification would make every unsupported leaf eligible for force.
- Showing a pull command or a broad force command would advertise authority the executor cannot safely honor.

## Decision 5: Use atomic sibling rename for files and directories

For a regular file, retain the existing no-follow source staging and verified sibling rename. For a directory, create and verify an empty private sibling directory through the opened safe parent, then atomically rename that sibling over the revalidated link object. Existing planned descendants populate the new directory, and the established directory-metadata finalizer runs last.

Atomic rename replaces the link entry itself without resolving its target. The design never performs an unlink-then-create sequence, so a concurrent object cannot silently occupy the destination between two operations.

Rejected alternatives:

- Unlinking the link and then creating a directory leaves a non-atomic gap and violates the replacement safety boundary.
- Copying a directory tree into an arbitrary temporary location would duplicate the existing action ordering and broaden the failure surface.
- Supporting nonempty target-directory replacement is irrelevant because the destination is a link object, not its target directory.

## Decision 6: Revalidate the exact link object under the mutation lock

The planner carries exact leaf evidence into the action. Under the existing mutation lock, execution rebuilds the plan, verifies registry and State V4 snapshots, reinspects the selected mapping, and compares final link-object identity and safe ancestry before staging and immediately before the rename. A changed, absent, supported, ancestor-unsafe, or otherwise different object is stale evidence and stops before replacement.

Link-target changes are intentionally not observed or compared: they cannot authorize a different destination object because Grip never follows the link.

Rejected alternatives:

- Path-string-only checks cannot distinguish a link replacement at the same spelling.
- Revalidating only after rename would allow a substituted link object to be removed.

## Decision 7: Preserve contained-source scope and performance boundaries

Feature 025's managed identity set remains authoritative. Destination probing visits only those identities and required ancestors. Final-leaf link handling does not enumerate siblings, read a link target, or retain an index. The existing release benchmark is extended with relevant link fixtures while retaining 100 managed identities, 10,000 unrelated entries, 100 warm samples, and p95 at or below one second.

Rejected alternatives:

- Scanning a destination tree for links would make unrelated home content a discovery, performance, and drift input.
- Adding a cache or index before a measured shortfall conflicts with the proportional-rigor constraint.
