# Research: Aggregate Forced Push

## Decision 1: Use a dedicated aggregate operation

**Decision**: Add a typed aggregate forced-push operation rather than widening generic `Resolve`.

**Rationale**: `Resolve` is currently exact-entry oriented. The aggregate contract needs an omitted-selector source-authorized scope, complete preflight, per-entry publication, and distinct result semantics. A dedicated operation makes that authority explicit in command routing, operation records, and tests.

**Alternatives rejected**:

- Treat aggregate force as ordinary `Resolve`: would silently broaden an exact operation and obscure the authorization boundary.
- Loop through existing exact force commands: would repeat inspection, permit partial mutation before later preflight blockers, and weaken one-operation reporting.

## Decision 2: Preflight the full scope before mutation

**Decision**: Inspect and validate every selected managed entry before any destination mutation.

**Rationale**: The aggregate command is intentionally consequential. Unsupported nodes, ownership, topology, project selection, and no-follow blockers must remain blockers, not be discovered only after earlier entries changed.

**Alternatives rejected**:

- Validate lazily per entry only: could mutate early entries even though a later deterministic blocker was known at plan time.
- Let `--force` override blockers: contradicts ownership and safety principles.

## Decision 3: Express complete source state explicitly

**Decision**: The aggregate planner includes source-present changes, source-winning conflicts, missing destination peers, and source absence only where the existing complete-state contract supports it.

**Rationale**: The feature authorizes source-complete state, including absence, but does not create general implicit-deletion authority. Typed deletion planning must be part of the aggregate policy and its deterministic entry ordering.

**Alternatives rejected**:

- Exclude source absence: contradicts the stated complete-state semantics.
- Invoke the existing delete executor as an unrelated second operation: loses one aggregate preflight, ordering, and result boundary.

## Decision 4: Publish accepted state per completed entry

**Decision**: Publish a baseline only after all actions belonging to one managed entry have revalidated and verified.

**Rationale**: A directory entry can require multiple raw actions and finalizers. Publishing after each raw action would claim an incomplete entry as accepted. The constitution permits individual verified entry acceptance despite later aggregate failure.

**Alternatives rejected**:

- Publish only after the entire aggregate: loses verified earlier acceptance after a late failure.
- Publish after every raw action: violates the completed-entry boundary.

## Decision 5: Reload state after publication

**Decision**: Refresh the State V4 snapshot after each accepted entry publication before later action revalidation.

**Rationale**: Current execution retains an initial state view. Aggregate publication changes accepted evidence; later work must not validate against stale baseline information.

**Alternatives rejected**:

- Keep the original immutable snapshot: creates false drift or incorrect result interpretation.
- Add snapshot isolation or broad locking: disproportionate to the local tool boundary.

## Decision 6: Stop at the first execution failure

**Decision**: After a revalidation, execution, verification, or publication failure, do not attempt later aggregate entries.

**Rationale**: This is the clarified user boundary. Earlier verified entries remain accepted; the aggregate is failed and all failed or later entries are unaccepted.

**Alternatives rejected**:

- Continue independently: weakens the explicit aggregate failure boundary and makes recovery less predictable.

## Decision 7: Reuse existing local-tool infrastructure

**Decision**: Extend current CLI, mutation planning/execution, State V4 publication, and operation record paths without new services or persistent indexes.

**Rationale**: Existing behavior already provides the required local primitives. The feature needs explicit policy and accounting, not infrastructure.

**Alternatives rejected**:

- Add a daemon, watcher, or long-lived locks: violates proportional rigor and bounded concurrency.
