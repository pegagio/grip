# Data Model: Source-Authoritative Mapping Addition

## Existing durable entities

| Entity | Relevant fields | Feature use |
|---|---|---|
| `PortableMapping` | Declared kind, source spelling, destination spelling | Remains the portable mapping declaration in `.grip/config.toml`; no new fields are required. |
| `EntryIdentity` | Resolved mapping plus member-relative path | Keys an accepted state for a file mapping or each admitted tree member. |
| `SupportedEntryStateV3` | Node kind, content fingerprint, and supported metadata | Stores the complete destination comparison reference for an unequal pair. |
| `AcceptedState` | Generation and `complete_baselines` map | Receives the add-specific initial entries before serializing State V4. |
| State V4 binding | Descriptor digest and resolved-mapping digest | Must describe the candidate descriptor and matching accepted-state map as one coherent published outcome. |
| Incomplete-add publication fence | Canonical mapping identity; prior and candidate descriptor digests; prior and candidate State V4 digests | Is created and verified before descriptor/state publication. It blocks only its mapping until a verified retry completes the candidate transition or restores the prior descriptor and clears the fence. |

## Add-time candidate rules

| Observed member | Initial state entry | Resulting existing classification |
|---|---|---|
| Source and destination are complete and equal | Source complete state | `Synchronized` |
| Source and destination are complete and unequal | Destination complete state | `SourceOnlyChange` |
| Source is complete, destination is absent | None | `SourceAddition` |
| Destination-only, ignored, unsafe, or unsupported | None | Existing unmanaged or blocking result |

## State transitions

```text
unmapped endpoints
  └─ add (equal) ──────────────> descriptor + source baseline ──> current
  └─ add (unequal) ────────────> verified fence ─> descriptor + destination baseline ─> verify + clear fence ─> pending push
  └─ add (source-only) ────────> descriptor without member baseline ──> pending push

fenced unequal add
  └─ publish/verify failure ───> retain fence ─> same `grip add` revalidates ─> complete candidate or restore prior descriptor ─> verify + clear fence

pending push
  └─ ordinary push succeeds ───> destination matches source + accepted source baseline ──> current
  └─ destination changes first ─> divergent conflict
```

## Publication invariants

- An unequal newly added source-defined member is never left in a usable declaration without its destination-derived accepted state; the fence is created and verified before either descriptor or state publication.
- A fence records the exact expected prior and candidate descriptor/state digests, and it is cleared only after the active descriptor and State V4 state match that candidate.
- A visible state publication is retained with the matching descriptor, even if durability confirmation reports an error after rename.
- A fenced mapping fails closed rather than classify as an initial collision or offer a force winner. Read and mutation commands for explicitly selected unfenced mappings remain available.
- Existing accepted entries and their State V4 generation remain untouched unless the compound add publication completes a new state candidate.
- Endpoint payload bytes, supported metadata, and filesystem-node presence do not change during any add path.
