# Research: Source-Authoritative Mapping Addition

## Decision 1: Use the destination complete state as the accepted baseline for unequal pairs

**Decision**: At add time, record the destination’s `SupportedEntryStateV3` for an active source-defined member when both endpoints are present and unequal.

**Rationale**: Existing `classify_complete_with_baseline` classifies `(source, destination, destination-baseline)` as `SourceOnlyChange`. That is already a source-to-destination action for normal `push` and `sync`, while `pull` selects no action. If the destination later changes, the same classifier produces `DivergentConflict`; if it independently becomes identical to source, it produces the existing converged-two-sided result.

**Alternatives considered**:

- Record the source as baseline: this makes the destination appear changed and incorrectly offers a pull.
- Add a new classification: unnecessary because the present three-way state model already represents the intended meaning.

## Decision 2: Build add-specific initial state rather than changing normal baseline acceptance

**Decision**: Add a builder dedicated to mapping addition. For each safe, active, source-defined member it records source state when endpoints are equal, destination state when endpoints are unequal, and no state when the destination is absent. It ignores destination-only and ignored members.

**Rationale**: The existing general builder accepts only equivalent records. A mixed tree would otherwise reject the complete set when it contains an unequal or source-only member. The new builder scopes the policy to first-add initialization and leaves ordinary synchronization acceptance untouched.

**Alternatives considered**:

- Broaden `baseline::build`: risks changing accepted-baseline semantics for synchronization and reconciliation.
- Store an explicit absent-destination baseline: unnecessary; source-only entries already classify as a safe pending push.

## Decision 3: Coordinate descriptor and initial-state publication for add

**Decision**: Use a dedicated mutation-locked add publication path that preflights the candidate descriptor and initial state, then creates and verifies a mapping-scoped fence before publishing either one. The fence binds the canonical mapping identity plus the expected prior and candidate descriptor/state digests. Publish the candidate descriptor, publish the matching State V4 candidate, verify both against the fence, and only then clear it. If any later step fails, retain the fence. A retry of the same `grip add` revalidates the fenced transition and either completes the matching candidate or restores the prior descriptor before clearing the fence. A fence creation or verification failure leaves descriptor and state untouched.

**Rationale**: State V4 binds to active descriptor bytes and cannot be safely published first. The current independent sequence can expose a new unequal mapping without the required reference. Creating the durable, exact fence first means even its own failed write cannot expose a descriptor/state transition without a known guard. The fence makes retry deterministic without becoming a recovery subsystem or history store, and its mapping identity permits explicitly selected unrelated mappings to continue safely.

**Alternatives considered**:

- Publish state before the descriptor: state binding would refer to a descriptor that is not yet active.
- Create the fence after a descriptor or state failure: its own creation can fail after a descriptor becomes visible, recreating the unsafe gap.
- Leave an unconfirmed transition as an initial collision: incorrectly offers a force winner choice for a failed add rather than failing closed.
- Leave the current sequence: violates FR-007 for an unequal mapping after a state failure.

## Decision 4: Preserve existing classifiers, planners, and public output formats

**Decision**: Do not change classifications, directions, mutation planning, selector behavior, default add output, `diff`, or JSON schema.

**Rationale**: The new baseline produces an existing `source_only_change` record, which already renders under `Changes to push` and is already actionable by normal push and sync. This keeps the change narrow and preserves Feature 017’s output contracts.

**Alternatives considered**:

- Introduce an `initial_source_authoritative` classification: duplicates current behavior and expands CLI and JSON compatibility risk.

## Decision 5: Validate with isolated file, tree, drift, and fault fixtures

**Decision**: Extend current mapping CLI tests for file and re-add behavior; add mixed-tree and publication-fault coverage; retain the existing complete-state classification matrix as direct state-model regression coverage.

**Rationale**: The behavior crosses descriptor publication, state binding, classification, planning, and filesystem safety. Isolated temporary roots satisfy the constitution and prevent dependence on real user files.

**Alternatives considered**:

- Unit-test only the baseline builder: insufficient to establish non-mutation, dry-run direction, state binding, and publication coherence.
