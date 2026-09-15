# Research: Contained-Source Tree Mappings

## Decision 1: Separate root topology from member topology

Root validation will continue to reject equal endpoints, a destination beneath its source, every contained file mapping, and all existing cross-mapping ownership conflicts. It will exempt exactly a tree source that is a strict descendant of its destination. A reusable runtime helper will derive the non-empty containment-relative path `P` from resolved endpoints and another helper will compare a managed member's relative path `R` to `P` by path components.

This split is necessary because registry validation knows resolved roots but does not yet know source membership or ignore policy. The static check can establish that the mapping shape is eligible, while the later source-policy stage decides whether the shape is safe for the actual members.

Rejected alternatives:

- A `--force` or generic recursive-topology bypass would weaken file mappings, reverse containment, and cross-mapping ownership guarantees.
- A string-prefix comparison would confuse component siblings such as `dotfiles` and `dotfiles-old` and would not preserve raw filesystem identity.
- Persisting `P` would place machine-specific resolved paths in portable intent and create stale migration state after a move or home change.

## Decision 2: Use an ephemeral managed identity set

Each active tree mapping will build a deterministic `ManagedIdentitySet`, keyed by raw relative path bytes, from the union of current non-ignored source members and retained accepted identities. Entries carry current source kind when present, accepted kind when retained, and active or ignored membership evidence. This set drives member-topology checks, name-compatibility analysis, exact destination probes, and record production.

The tree root is a mapping anchor, not a payload member, so the empty relative path never enters the set. This preserves the established source-defined tree model and prevents the destination anchor itself from becoming implicitly owned.

For a contained mapping, `R` is unsafe when it equals `P`, is an ancestor of `P`, or is a descendant of `P`. A disjoint `R` is safe. Current members excluded by `.gripignore` do not acquire ownership. Retained identities remain in scope until normal accepted-state retirement removes them.

This model reuses state V4 baselines as the durable retained-identity index and adds no second index or schema. It also ensures that source deletion remains observable after destination-tree enumeration is removed.

Rejected alternatives:

- Inspecting only current source members would lose deletion and conflict classifications for retained source-absent identities.
- Persisting a destination index would duplicate accepted state and introduce invalidation and recovery obligations.
- Validating only the exact selected entry would let an exact selector bypass an unsafe sibling in the same mapping.

## Decision 3: Preserve ignored accepted identities explicitly

Source walking will receive a retained-identity trie or equivalent prefix index. When source ignore policy prunes a directory, discovery will mark exact retained identities at or beneath that ignored prefix as ignored without opening or fabricating descendants. Observation will preserve their existing retirement semantics rather than reinterpret them as ordinary source deletions.

This is needed because a pruned walk cannot otherwise distinguish an ignored accepted path from an absent source path. The prefix structure is ephemeral and scoped to accepted identities, so it does not turn ignored content into an inventory.

Rejected alternatives:

- Descending into ignored directories would violate source policy and could expose unsupported or inaccessible content.
- Treating every missing retained identity as deleted would silently convert ignore-policy changes into deletion proposals.

## Decision 4: Replace destination recursion with exact managed probes

Tree discovery will stop recursively enumerating destination content. For each managed identity in canonical order, it will open the destination root and traverse only the exact relative components using no-follow directory operations. Opened safe ancestor descriptors and evidence may be reused across targets. A probe has exactly three outcomes:

1. The target is absent from the first missing component onward.
2. The leaf is a supported node of the expected kind.
3. An ancestor or leaf is blocking because it is a link, non-directory ancestor, unsupported node, nested mount, wrong kind, unreadable, or otherwise unsafe.

If the destination root is absent, all paired targets are absent while existing missing-parent evidence remains available to planning. Probes must not fall back to unchecked path-based metadata calls and must not collect root-wide child-name evidence. Arbitrary destination changes therefore cannot cause records or two-pass drift, while changes to a paired target or required ancestor still do.

Rejected alternatives:

- Skipping only the project subtree while scanning the rest of the destination remains unbounded and makes unrelated permissions, special nodes, size, and churn operational inputs.
- Probing retained targets only in observation can collapse unsupported nodes into absence and bypass discovery's ancestry and two-pass safety evidence.
- Retaining root `child_names` evidence would continue to make unrelated destination churn stale an otherwise safe operation.

## Decision 5: Scope APFS and name-compatibility checks to managed identities

Name-comparison grouping will operate on current eligible and retained active identities, excluding identities currently undergoing ignored retirement. The existing nearest-existing-destination capability qualification remains. Comparison keys are derived from raw relative names; proving an exact on-disk spelling must use opened target identity rather than enumerating unrelated siblings.

This preserves alias and case-folding protection among entries Grip may own while honoring the prohibition on inspecting never-managed destination content.

Rejected alternative: enumerating siblings only for name checks still exposes unrelated destination names and makes performance depend on destination breadth.

## Decision 6: Keep persistence schemas unchanged and strengthen rebinding

Project descriptor V2 continues to store user-authored relative or portable intent, and state V4 continues to store accepted mapping identities and baselines. Containment and member relations are recomputed from the selected machine's resolved project, source, and destination paths. Binding digest changes already trigger rebinding assessment.

Before rebinding fingerprints a retained identity, it will derive the new `P` and check the identity's `R`. A newly unsafe retained identity produces a deterministic `recursive_member_topology` blocker and prevents probing through the unsafe path. Evidence is retained rather than pruned.

Rejected alternative: migrating stored state with resolved containment data would be non-portable and would not remove the need to recompute after every path binding change.

## Decision 7: Add typed blockers without changing the CLI

A recursive managed member will report stable reason `recursive_member_topology` with the mapping source, relative member, resolved source path, resolved destination path, and relation `equal`, `ancestor`, or `descendant`. Existing record and result models may gain optional typed detail, but command syntax, selector rules, machine envelope versions, ordinary or forced authority, and deletion authorization remain unchanged.

Human output will identify explicit `.gripignore` as the remediation when the operator intentionally wants the member outside ownership. It will not suggest `--force`, because force must not override ownership topology.

Rejected alternative: reusing the registry-level `recursive_topology` ownership conflict would describe the wrong subject and omit the member-level relationship needed for remediation.

## Decision 8: Revalidate the whole selected mapping before mutation

Addition must complete member validation before fence creation, descriptor publication, baseline publication, or endpoint mutation. Push, pull, sync, force, and delete keep their existing plan, lock, reinspection, per-action revalidation, verification, and publication pipeline. In addition, the entire selected mapping is topology-checked before its first payload action so a newly introduced unsafe sibling cannot be hidden by an exact-entry selector. Operations on unrelated mappings remain independent.

This uses the existing bounded revalidation model and avoids broad filesystem locking. A change to a paired target, required ancestor, resolved topology, or managed membership stops the affected operation before publication.

Rejected alternative: action-only revalidation is too narrow because ownership safety is a mapping invariant rather than an exact-entry property.

## Decision 9: Extend the existing performance harness

The ignored release-mode performance acceptance test will add a contained-source fixture with 100 managed members and 10,000 unrelated destination entries. It will run 100 warm read-only status samples, assert p95 at or below one second on the supported release platform, and verify that unrelated entries produce no output or access-dependent failures.

This measurement directly exercises the feature's scaling boundary. No cache, persistent index, parallelism, or new benchmark dependency is justified before evidence shows exact probing is insufficient.
