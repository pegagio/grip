# Research: Initial-Match Baseline Synchronization

## Decision: Extend ordinary sync rather than add a baseline command

`sync` is the established state-changing command that can combine payload actions with acceptance-only work. The product intentionally exposes no public `baseline` or `accept` command, so the new behavior belongs in selected sync.

**Rationale**: An exact tree child is already selectable by `sync`, and the operator has explicitly requested synchronization. This keeps the CLI flat and avoids a second way to mutate accepted state.

**Alternatives considered**:

- Add `grip baseline accept`: rejected because the current public command contract intentionally excludes this command family.
- Recreate the ancestor tree mapping: rejected because it discards accepted state for the entire mapping and is too broad for one child.
- Automatically accept during `status` or `diff`: rejected because inspection commands must remain read-only.

## Decision: Treat an eligible initial match as acceptance-only in sync planning

An `InitialMatch` already denotes complete equivalent source and destination evidence without an accepted baseline. Sync planning will classify it as `AcceptOnly`, alongside an existing converged two-sided change, only when the requested selector resolves to exactly one managed entry.

**Rationale**: Existing execution initializes accepted identities from acceptance-only entries, revalidates under the mutation lock, performs final inspection, and publishes the accepted candidate without a payload action. The baseline candidate already accepts `InitialMatch` evidence.

**Alternatives considered**:

- Add a new disposition: rejected because `AcceptOnly` already models the required no-payload state transition.
- Treat all initial states as acceptance-only: rejected because unequal, absent, destination-only, unsupported, and unsafe evidence must retain current behavior.

## Decision: Preserve exact selector and dry-run semantics

The change applies only when ordinary sync resolves the request to exactly one managed entry. A selected child beneath a tree mapping remains one exact entry; unselected siblings, no-selector sync, a tree mapping root, and a multi-entry subtree are not accepted. Dry-run builds the same plan but never executes or publishes state.

**Rationale**: This preserves explicit ownership and makes the proposed baseline scope visible before mutation.

**Alternatives considered**:

- Accept the whole ancestor tree when one child is selected: rejected because it broadens mutation scope beyond the request.
- Make status auto-repair stale or missing baseline state: rejected because it hides a state transition in a read-only command.
