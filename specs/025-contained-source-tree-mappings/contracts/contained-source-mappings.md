# Contract: Contained-Source Tree Mappings

This contract defines the externally observable admission, inspection, blocking, and compatibility behavior for Feature 025. It adds no command or persisted schema.

## Root Admission

| Mapping kind | Resolved relationship | Result |
|---|---|---|
| Tree | Disjoint roots | Existing behavior |
| Tree | Source is a strict descendant of destination | Conditionally admitted; managed members must pass member-topology validation |
| Tree | Equal roots | Reject as `recursive_topology` |
| Tree | Destination is beneath source | Reject as `recursive_topology` |
| File | Disjoint endpoints | Existing behavior |
| File | Any equal or containment relationship | Reject under existing topology rules |

The exception does not alter duplicate, source/source, destination/destination, project-metadata, or cross-mapping ownership checks. `--force` does not alter this matrix.

## Member Admission

For an admitted contained tree, let `P` be the non-empty relative path from the destination root to the source root and `R` be a managed member's relative destination.

The tree root is a mapping anchor rather than a payload member. Its empty relative path is not assessed as `R` and does not grant ownership of the destination root.

| Relationship of `R` to `P` | Meaning | Result |
|---|---|---|
| Equal | Member destination equals the source root | Block |
| Ancestor | Member destination contains the source root | Block |
| Descendant | Member destination is inside the source root | Block |
| Component-disjoint | Member destination cannot reach the source root | Admit |
| Excluded by effective source policy before ownership | Member owns no destination | Out of scope unless retained accepted evidence requires ignored retirement |

Comparisons are component-aware and preserve raw path identity. Textual prefix similarity alone never establishes containment.

## Managed Inspection Scope

For every active tree mapping, ordinary add, list, status, diff, push, pull, sync, force, and delete planning may inspect destination payload only for:

- Current non-ignored source members.
- Retained accepted identities belonging to the active mapping.
- Exact destination ancestors required to reach those identities safely.
- Existing root and capability evidence required by filesystem safety contracts.

Ordinary inspection must not enumerate arbitrary never-managed destination content. Such content produces no discovery record, classification, baseline, selection candidate, drift evidence, or payload action.

A retained accepted identity whose source is absent is probed exactly so existing source-deletion, destination-change, conflict, converged-deletion, and forced-direction behavior remains available. An accepted identity covered by current ignore policy retains existing ignored-retirement semantics.

## Exact Probe Outcomes

Each managed destination probe returns one of these outcomes:

| Outcome | Contract |
|---|---|
| Absent | The destination root or first required component is missing; existing absence and missing-parent planning applies |
| Supported | Every required ancestor is safe and the exact leaf has an allowed kind and evidence |
| Blocking | A required ancestor or leaf is a link, wrong kind, unsupported node, nested mount, inaccessible, incompatible, or otherwise unsafe under existing rules |

Grip must never follow a link, continue beyond a missing component, collapse a blocking node into absence, or fall back to an unchecked path traversal. Unrelated sibling content is not an input to the probe.

## Recursive Member Diagnostics

An unsafe current or retained member reports a stable blocker with:

```json
{
  "reason": "recursive_member_topology",
  "mapping_source": "home",
  "relative_member": "dotfiles/home",
  "source_path": "<resolved source member>",
  "destination_path": "<resolved destination member>",
  "relation": "equal"
}
```

`relation` is one of `equal`, `ancestor`, or `descendant`. Human output identifies the same mapping and member and explains that explicit `.gripignore` policy can exclude intentionally unmanaged source content. It must not suggest force as an override.

The shape is illustrative of required semantic fields; implementation may embed them in the existing diagnostic envelope without changing that envelope's version.

## Addition and Non-Mutation

`grip add` validates the complete prospective registry, source policy, managed identity set, member topology, and paired destination evidence before publishing descriptor or accepted state. It never changes endpoint payload. Existing equivalent-peer, differing-peer, source-only, collision, and publication behavior remains in force after the new validation gate.

An unsafe member blocks before fence creation, descriptor publication, baseline publication, or payload action. A rejected add leaves mapping intent, accepted state, and both endpoints unchanged.

## Selection and Mutation

Recursive-member safety is a mapping invariant. Before the first payload action for a selected contained mapping, Grip revalidates the entire selected mapping even when the command selected one exact entry. Existing per-action revalidation then remains in force.

An unsafe sibling cannot be bypassed by an exact selector or force. An unsafe member in an unrelated unselected mapping does not block the selected mapping. Any relevant membership, root-topology, target, or ancestor drift stops the affected operation before an unsafe action or state publication.

Ordinary and forced synchronization authority, direction, deletion authorization, conflict handling, supported node types, dry-run behavior, and atomic publication semantics are unchanged.

## Rebinding

When project or home binding changes, Grip resolves roots and assesses every retained identity against the new containment-relative path before probing its payload fingerprint. A safe disjoint-to-contained, contained-to-disjoint, or changed-contained binding may continue through existing rebinding checks. A retained identity that becomes equal to, an ancestor of, or a descendant of the new `P` blocks rebinding and mutation with `recursive_member_topology`.

Blocking does not delete the mapping or accepted evidence and does not probe through the unsafe path.

## Compatibility

- Command names and arguments are unchanged.
- Selector interpretation is unchanged.
- Project descriptor V2 and accepted state V4 are unchanged.
- Existing machine-readable envelope versions are unchanged.
- Existing root-level `recursive_topology` remains the reason for prohibited root relationships.
- Existing pure classification and planning support for destination-only unmanaged records may remain for file, retained-retirement, or untracked-state cases, but arbitrary never-managed tree destination entries no longer create such records.
- Features 002 and 003 remain historical; this contract flows their changed behavior forward through Feature 025.
