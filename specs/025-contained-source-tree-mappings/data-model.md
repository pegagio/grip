# Data Model: Contained-Source Tree Mappings

The feature adds runtime evidence and relationships, not new persisted schemas. Project descriptor V2 and accepted state V4 remain the durable authorities.

## Resolved Root Topology

Represents the relationship between one mapping's resolved roots on the selected machine.

| Field | Meaning |
|---|---|
| Mapping kind | `file` or `tree` |
| Source root | Canonical resolved source anchor |
| Destination root | Canonical resolved destination anchor |
| Relation | `disjoint`, `source_strictly_beneath_destination`, `equal`, or `destination_beneath_source` |
| Containment-relative path | Non-empty `P` from destination root to source root when the source is strictly beneath the destination |
| Admission | Ordinary, conditionally allowed contained tree, or prohibited |

Validation rules:

- Only `tree` plus `source_strictly_beneath_destination` enters conditional member validation.
- Equal roots, destination beneath source, and every contained file mapping remain prohibited.
- Complete-registry source/source, destination/destination, duplicate, and cross-mapping checks remain independent and authoritative.
- `P` is component-aware runtime evidence and is never persisted.

## Managed Identity Set

Represents every relative member that Grip currently owns or must retain enough evidence to retire safely for an active tree mapping.

| Field | Meaning |
|---|---|
| Mapping identity | Active mapping to which the member belongs |
| Relative identity `R` | Raw relative path bytes, used as the deterministic key |
| Current source kind | Supported current source kind, if present |
| Accepted kind | Retained baseline kind, if present |
| Membership | `active` or `ignored` |
| Origin | Current source, retained accepted state, or both |

Invariants:

- The set is the union of current non-ignored source members and retained accepted identities for the active mapping.
- The tree root's empty relative path is an anchor, not a managed member, and never enters the set.
- Canonical ordering is deterministic and does not depend on destination directory enumeration.
- Never-managed destination content cannot create an entry.
- An accepted identity beneath an effective ignored prefix remains represented as ignored retirement evidence without opening the ignored subtree.
- A source-absent accepted identity remains active unless ordinary ignore-retirement rules apply.

## Member Topology Assessment

Represents whether one managed identity is safe under a contained mapping.

| Field | Meaning |
|---|---|
| Mapping source | Portable mapping source identifier |
| Relative member `R` | Managed identity under assessment |
| Containment path `P` | Runtime path locating the source beneath the destination |
| Resolved member source | Source root joined with `R` |
| Resolved member destination | Destination root joined with `R` |
| Relation | `disjoint`, `equal`, `ancestor`, or `descendant` relative to `P` |
| Result | Safe or blocking |

Validation rules:

- `R == P` is blocking because the paired destination equals the source root.
- `R` being an ancestor of `P` is blocking because the paired destination contains the source root.
- `R` being a descendant of `P` is blocking because the paired destination is inside the source root.
- A component-disjoint `R` is safe.
- Ignored never-accepted members are outside assessment because they own no destination.
- Retained identities are assessed even when their source is absent.

## Ignored Prefix Coverage

Represents how source policy applies to retained identities hidden by a pruned source walk.

| Field | Meaning |
|---|---|
| Mapping identity | Active tree mapping |
| Ignored relative prefix | Directory or entry excluded by effective `.gripignore` policy |
| Covered retained identities | Accepted identities equal to or beneath the prefix |
| Evidence | Applicable ignore-rule provenance already used by discovery |

The structure is ephemeral. It marks only known retained identities and does not fabricate or enumerate unknown descendants.

## Scoped Destination Evidence

Represents an exact no-follow destination probe for a managed identity.

| Field | Meaning |
|---|---|
| Managed identity | Entry from the managed identity set |
| Required ancestors | Exact destination-relative directory components needed to reach the leaf |
| Expected kind | Current source kind or retained accepted kind |
| Outcome | `absent`, `supported`, or `blocking` |
| Leaf evidence | Existing node evidence for the exact target when present |
| Ancestor evidence | Metadata needed to prove safe traversal and detect drift |
| Blocking reason | Existing unsupported, wrong-kind, inaccessible, mount, link, or ancestry reason when applicable |

State rules:

- The first missing component makes the target absent; Grip does not inspect deeper components.
- Every opened component is checked without following links.
- Safe opened ancestors may be cached within one pass and reused for canonically adjacent targets.
- Unsupported or unsafe ancestors and leaves remain blockers rather than being collapsed into absence.
- No sibling list or root-wide child-name evidence is collected.

## Recursive Member Blocker

Represents deterministic human and machine diagnostics for an unsafe managed identity.

| Field | Meaning |
|---|---|
| Reason | Stable value `recursive_member_topology` |
| Mapping source | Portable mapping source identifier |
| Relative member | Unsafe managed identity |
| Source path | Resolved member source path |
| Destination path | Resolved member destination path |
| Relation | `equal`, `ancestor`, or `descendant` |
| Remediation | Exclude intentionally unmanaged content through ordinary explicit `.gripignore` policy or change the layout |

The blocker is additive to existing result structures. It does not create a new command, selector, force mode, or machine envelope version.

## Rebinding Assessment

Extends existing machine-binding assessment with runtime member topology.

| Field | Meaning |
|---|---|
| Prior binding | Existing state V4 binding evidence |
| Current binding | Newly resolved project, source, and destination evidence |
| Root topology | Current resolved root relationship and optional `P` |
| Retained assessments | Member topology assessments for every retained identity |
| Result | Rebind eligible or blocked with deterministic findings |

A blocking retained assessment stops rebinding and mutation before fingerprint probing through the unsafe path. Accepted evidence remains intact.

## State Transitions

### Mapping addition

`proposed` → `root-valid` → `source-policy-discovered` → `managed-set-built` → `members-safe` → `candidate-observed` → `descriptor-and-baseline-published`

Any prohibited root, recursive member, unsupported paired path, inconsistent two-pass evidence, or existing candidate-add blocker transitions to `rejected` before publication or payload mutation.

### Read-only inspection

`active mapping` → `source-policy-discovered` → `managed-set-built` → `members-assessed` → `exact destinations probed` → `stable evidence classified`

Never-managed destination content has no transition because it never enters the managed identity set.

### Mutation or deletion

`stable classified evidence` → `plan` → `selected mapping revalidated under lock` → `action evidence revalidated` → `atomic action` → `verified` → `accepted state published`

Topology or evidence drift transitions to `blocked` before the first unsafe action. Partial execution retains the existing precise result and publication rules.

### Rebinding

`binding changed` → `roots resolved` → `retained members topology-checked` → `retained payload evidence checked` → `rebind eligible`

Any unsafe retained identity transitions to `blocked` without discarding state.
