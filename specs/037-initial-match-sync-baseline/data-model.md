# Data Model: Initial-Match Baseline Synchronization

This feature reuses existing entities and persistence formats. It changes only the permitted transition for an eligible selected record.

| Entity | Relevant existing fields | Feature rule |
| --- | --- | --- |
| Classification record | Identity, classification, complete source/destination evidence, blocking flag | An unblocked `InitialMatch` with complete equivalent evidence is eligible for acceptance-only sync. |
| Mutation entry | Identity, disposition, actions | An eligible initial match uses the existing acceptance-only disposition and has no payload actions only when the request resolves to that single entry. |
| Mutation plan | Selected scope, acceptance identities, actions, blockers | The selected initial-match identity appears in acceptance identities; no sibling or unrelated identity is added. |
| Accepted state | Complete baselines keyed by managed identity | A successful operation adds or refreshes only the selected eligible identity using its verified equivalent state. |

## State Transition

```text
InitialMatch + exact single-entry selected sync
  -> acceptance-only plan
  -> lock-held revalidation and final equivalent observation
  -> accepted baseline published
  -> Synchronized/current status
```

Any non-equivalent, incomplete, blocked, unsafe, ignored, or unmanaged state does not take this transition.
