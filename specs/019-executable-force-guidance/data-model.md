# Data Model: Executable Force-Resolution Guidance

This model describes only presentation-time guidance. Durable accepted state, classification records, and mutation plans remain their existing models.

## Existing authoritative entities

| Entity | Existing authority | Relevant rule |
|---|---|---|
| `Selection` | `src/observation/model.rs` | A selector resolves as `Entry`, `Subtree`, `Mapping`, `All`, or unmanaged. |
| `EntryIdentity` | `src/observation/model.rs` | Identifies one managed entry within one mapping. |
| `AcceptedState.complete_baselines` | `src/state` | Establishes baseline identities used when converting a subtree or mapping selection into an exact entry. |
| `exact_resolution_selection` | `src/lib.rs` | Accepts only one exact managed entry for forced resolution. |

## Presentation-only guidance

The implementation will introduce an internal, non-serialized guidance representation associated with a displayed conflict or a blocked mutation blocker.

| Variant | Required data | Rendered result |
|---|---|---|
| `ForcePair` | Invocation-directory-relative source selector and destination selector | `Keep source: grip push --force SOURCE` plus `Keep destination: grip pull --force --destination DESTINATION` |
| `InspectDiff` | Invocation-directory-relative source selector | `Run: grip diff SOURCE` |
| `None` | Blocking technical or compatibility reason | Existing blocker explanation; no invented action command |

## Eligibility transition

```text
classified conflict + displayed source selector + accepted state
    -> resolve with existing selector rules
    -> exact_resolution_selection accepts one EntryIdentity
        -> ForcePair
    -> selector is aggregate or cannot become one entry
        -> InspectDiff
    -> technical or compatibility blocker
        -> None
```

The decision is computed separately from the serialized classification record. It does not alter classifications, accepted baselines, mutation plans, filesystem state, or JSON output.

## Validation rules

- A `ForcePair` may be emitted only if the exact source selector would be accepted by the same force-resolution path that the printed command invokes.
- The rendered source selector uses the existing invocation-directory-relative display policy, so it is copyable from `grip status` into `grip push --force` or `grip diff`.
- The destination selector uses the established destination display form accepted by `grip pull --force --destination`.
- An aggregate mapping root must not receive a `ForcePair` merely because its classification is `initial_collision` or `divergent_conflict`.
- A blocked mutation with several blockers retains the established blocker ordering and uses guidance matching each individual blocker; it must not index a force pair against the wrong blocker.
