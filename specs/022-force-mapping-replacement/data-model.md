# Data Model: Force Mapping Replacement

Feature 022 changes metadata membership only. Source and destination payloads are observed for validation and initial comparison state, never copied, deleted, or altered by add or removal.

## Entities

### Force-Add Request

| Field | Meaning | Validation |
|---|---|---|
| `force` | Explicit request to replace an existing owner. | Must be true to activate replacement selection. |
| `source` | User-supplied project-relative source declaration. | Must pass existing path, endpoint, kind, and safety validation. |
| `destination` | User-supplied destination declaration. | Must pass existing destination and safety validation. |
| resolved mapping | Canonical `kind`, source, and destination identity derived from endpoint evidence. | Used for ownership comparison and fence protection. |

### Displaced Mapping

| Field | Meaning | Validation |
|---|---|---|
| portable declaration | The active descriptor entry retained for user-authored spelling. | Must be the sole entry selected for retirement. |
| resolved mapping | Canonical file mapping identity. | Must be `File`, have the requested canonical destination, and have a distinct canonical source. |
| accepted baseline identities | State V4 entries associated with the resolved mapping. | Every such identity is removed from the candidate state. |

### Replacement Candidate

| Field | Meaning | Validation |
|---|---|---|
| candidate descriptor | Current portable declarations minus the displaced declaration plus the requested declaration. | Must resolve and pass complete registry ownership validation. |
| candidate registry | Resolved ownership graph for the candidate descriptor. | Must preserve all existing source, destination, tree, and nested constraints. |
| candidate state | Current accepted state minus displaced identities, plus initial evidence for the requested mapping. | Must be encodable and bound to the candidate descriptor. |
| endpoint evidence | Durable inspection of requested endpoints. | Must be revalidated immediately before descriptor publication. |

### Descriptor/State Transition Fence

| Field | Meaning | Validation |
|---|---|---|
| operation context | Add, forced replacement, or exact removal plus the mapping identity needed for retry. | A retry may act only on the transition recorded by the fence. |
| prior pair | Accepted descriptor bytes and optional State V4 bytes with digests. | Supports restoration after a stale or failed transition. |
| candidate pair | Candidate descriptor and State V4 bytes with digests. | Supports completion and post-publication verification. |
| result context | Ordinary-add or replacement outcome details. | Enables deterministic retry output. |

The fence remains a short-lived Grip-owned metadata record, protected by the existing mutation lock; it is not a general history or recovery feature.

## Relationships

```text
Force-Add Request
        │ resolves to
        ▼
Requested Mapping ── same canonical destination ──► Displaced Mapping (exactly one)
        │                                               │
        └──────────── builds ───────────► Replacement Candidate ◄── retires baselines
                                                    │
                                                    ▼
                                      Descriptor/State Transition Fence
                                                    │
                                      publish + verify or restore
                                                    ▼
                                   Accepted descriptor and State V4 pair
```

## State Transitions

### Ordinary Add

`No requested declaration` → validate candidate → publish normal add using the existing path. Any active ownership conflict remains rejected. This transition does not select or retire another mapping.

### Forced Replacement

`Accepted old mapping + state` → select exactly one displaced mapping → construct and validate candidate descriptor/state → persist fence → publish and verify candidate pair → `Accepted new mapping + descriptor-bound state`.

If endpoint evidence, descriptor bytes, or state bytes drift during this sequence, the transition either completes its already-recorded candidate pair or restores the complete prior pair. A mixed pair is invalid and cannot be reported as success.

### Exact Removal

`Accepted selected mapping + state` → remove only its declaration and baseline identities → persist fence → publish and verify reduced descriptor/state pair → `Mapping fully absent`.

Removal of another source follows the same exact rule but cannot affect the active mapping that owns a requested destination.

## Validation Matrix

| Request condition | Ordinary add | Forced add |
|---|---|---|
| No destination ownership conflict | Add normally. | Reject because force did not identify one active mapping that it is authorized to replace. |
| Exactly one distinct equal-destination active file mapping | Reject `destination_overlap`. | Replace that one mapping after complete candidate validation. |
| Zero or multiple replaceable mappings | Normal add behavior. | Reject without descriptor, state, fence, or payload change. |
| Source overlap, tree, nested, ambiguous, unsafe, or incompatible conflict | Reject. | Reject with the existing category/detail; force does not bypass it. |
| Exact prior mapping successfully removed | Allow later ordinary add. | Not needed for the later add. |
| Different mapping removed while conflict remains active | Reject active conflict. | May replace only if the remaining conflict independently satisfies the exact replacement rule. |
