# Classification Contract

This contract defines the pure three-way decision boundary shared by `status`, `check`, `diff`, and baseline acceptance.

## Inputs

For each Entry Identity, the classifier receives current membership, optional supported source state, optional supported destination state, optional accepted baseline state, and any unsupported or unsafe evidence. It performs no filesystem or state access.

Equality compares node kind, file content digest, and file permission mode. File length supports evidence but never replaces digest equality. Directory equality is node-kind presence only. Mtime and other deferred metadata do not participate.

## No-baseline classifications

| Source | Destination | Classification | Direction | Attention | Blocking |
|---|---|---|---|---|---|
| supported | absent | `source_addition` | source to destination | yes | no |
| supported | equal supported | `initial_match` | none | yes until accepted | no |
| supported | different supported | `initial_collision` | none | yes | yes |
| absent | present without managed evidence | `destination_only_unmanaged` | none | no | no |

Ignored source evidence without a baseline remains outside managed membership. Unsupported managed source or unsafe paired destination evidence uses the override rules below.

## Baseline-present classifications

| Source relative to baseline | Destination relative to baseline | Classification | Direction | Attention | Blocking |
|---|---|---|---|---|---|
| equal | equal | `synchronized` | none | no | no |
| changed | equal | `source_only_change` | source to destination | yes | no |
| equal | changed | `destination_only_change` | destination to source | yes | no |
| changed | same changed state | `converged_two_sided_change` | none | yes until accepted | no |
| changed | different changed state | `divergent_conflict` | none | yes | yes |
| absent | equal | `source_side_deletion` | source to destination | yes | no |
| equal | absent | `destination_side_deletion` | destination to source | yes | no |
| absent | changed | `delete_change_conflict` | none | yes | yes |
| changed | absent | `change_delete_conflict` | none | yes | yes |
| absent | absent | `converged_deletion` | none | yes | no |
| excluded by current source policy | any | `newly_ignored_pending_retirement` | none | yes | no |
| complete mapping snapshot absent | not inspected | `untracked_pending_retirement` | none | yes | no |

If both sides are present and their complete supported states are equal, classify `synchronized` when both equal baseline and `converged_two_sided_change` when both differ from baseline. Opposing changes in different supported dimensions remain divergent unless complete current states converge.

## Safety overrides

| Evidence | Classification | Attention | Blocking |
|---|---|---|---|
| unsupported active source | `unsupported_managed` | yes | yes |
| unsupported or wrong-kind node at paired managed destination | `unsafe_collision` | yes | yes |
| unsupported destination-only node without baseline | `destination_only_unmanaged` | no | no |

An override retains every safely available comparison but never invents a Supported State for the unsupported side. Stable reasons reuse Feature 003 node and collision reasons.

## Changed dimensions

Each available comparison returns an ordered subset of `node_kind`, `content`, and `permission_mode`. The fixed order is the order shown. A comparison is:

- `null` when either side is absent or unsupported;
- `[]` when both supported states are equal;
- a non-empty array when supported states differ.

Every record contains source-to-baseline, destination-to-baseline, and source-to-destination comparison fields. `diff` displays all three; status and check may use compact human presentation but machine output remains equivalent.

## Determinism and completeness

The classified universe is the union of current observation identities and accepted baseline identities. Current mappings that no longer have source roots still participate so root deletion is classifiable. Baseline-only mapping snapshots become untracked pending retirement and their former payload paths are not reopened.

Records sort by canonical source, mapping kind and destination tie-breakers, decoded raw relative bytes, then stable classification order. Counts, attention count, and blocking count derive only after sorting. Commands return no partial classification when registry, state, policy, hashing, traversal, or stability validation fails.

## Baseline eligibility

`baseline accept` may accept only selected active records whose source and destination have complete equivalent Supported State. `initial_match`, `converged_two_sided_change`, and already-synchronized records satisfy equality; all additions, collisions, one-sided changes, conflicts, deletions, ignored or untracked pending retirement, unsupported evidence, and unsafe collisions reject the complete request.

The candidate clones the full accepted map and replaces only selected eligible records. Out-of-scope and pending-retirement records remain byte-semantically unchanged. Exact semantic equality with accepted baselines is an `already_current` no-op after locked revalidation.
