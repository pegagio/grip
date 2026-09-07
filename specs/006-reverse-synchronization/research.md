# Research: Reverse Synchronization

This research resolves the technical choices needed to plan Feature 006 against the verified Feature 005 implementation. All choices preserve the current Rust toolchain, dependencies, State V2 authority, and filesystem safety boundary.

## Table of Contents

- [Shared mutation architecture](#shared-mutation-architecture)
- [Direction and path roles](#direction-and-path-roles)
- [Pull classification policy](#pull-classification-policy)
- [Source-parent handling](#source-parent-handling)
- [Filesystem staging and recovery](#filesystem-staging-and-recovery)
- [Operation records and failure types](#operation-records-and-failure-types)
- [Public result compatibility](#public-result-compatibility)
- [Baseline publication](#baseline-publication)
- [Testing and performance](#testing-and-performance)

## Shared mutation architecture

**Decision**: Extract the existing push model, pure planner primitives, descriptor-oriented filesystem functions, recovery publication, checkpointed execution, and typed failure/result projection into an internal direction-neutral `mutation` module. Keep command-specific push and pull adapters responsible only for direction policy and orchestration. Preserve compatibility re-exports under `push` where current tests or internal consumers rely on those paths during the refactor.

**Rationale**: The current push implementation already encodes the complete safety transaction required by pull, but its types and functions are named and validated specifically for push. Copying the pipeline would duplicate lock ordering, revalidation, checkpoint, recovery, baseline, and result-delivery rules and invite divergence before Feature 007 combines directions. A narrow shared core represents real reuse now, while thin adapters preserve explicit product commands.

**Alternatives considered**:

- Copy `src/push` to `src/pull`: smaller initial edits but duplicates the most safety-sensitive code and test matrix.
- Add pull branches throughout `src/push`: fewer file moves but makes mapping direction implicit inside push-named APIs and worsens future maintenance.
- Build the full Feature 007 bidirectional scheduler now: premature scope expansion beyond reverse synchronization.

## Direction and path roles

**Decision**: Introduce a closed `MutationDirection` with `Push` and `Pull`. Keep canonical `source_path` and `destination_path` fields because they describe mapping roles. Derive internal `origin` and `target` roles from direction: push reads source and replaces destination; pull reads destination and replaces source. Store both expected mapping-side states in the plan so direction cannot erase classification evidence.

**Rationale**: Source and destination remain authoritative terms for membership, selection, and diagnostics even when data flows backward. Origin and target are execution roles only. This separation prevents a pull implementation from accidentally treating destination discovery as membership authority.

**Alternatives considered**:

- Swap source and destination fields for pull: simplifies a local executor call but corrupts stable identity and output meaning.
- Store only origin and target: loses the mapping-side contract needed for selectors, classification, and future sync.

## Pull classification policy

**Decision**: Use one explicit direction table over all 18 inherited classifications. For pull, only nonblocking `destination_only_change` with accepted managed identity becomes `action`. `destination_only_unmanaged` becomes a reported `no_action` entry with reason `unmanaged_destination`; its presence alone is not a blocker. Every inherited record already marked blocking remains blocked. All other nonblocking classifications are reported as no-action.

**Rationale**: This is the exact feature boundary: reverse an unambiguous change to an established entry without importing destination-defined membership, accepting converged evidence, resolving conflict, or executing deletion. A total table is auditable and prevents accidental actionability from catch-all logic.

**Alternatives considered**:

- Infer actionability from classification direction metadata alone: too broad because deletions and conflicts also have direction.
- Filter unmanaged destination records out of the pull plan: conflicts with the accepted clarification requiring each item to be reported.
- Treat every unmanaged destination item as a blocker: weakens normal overlay behavior and blocks unrelated eligible actions.

## Source-parent handling

**Decision**: Pull creates no source entry and no synthetic source parent. The planner requires the established source and every accepted source ancestry component to remain present, supported, and no-follow safe. Absence or changed ancestry retains the inherited deletion/unsafe classification and blocks or produces the corresponding non-action; it is never converted into replacement work.

**Rationale**: A destination-to-source replacement is eligible only while the source remains the accepted side of an established identity. Recreating a missing source would reverse a source-side deletion and therefore enter Feature 008's separately authorized deletion semantics.

**Alternatives considered**:

- Recreate missing source parents from mapping topology: implicitly overrides deletion intent and expands pull into restoration.
- Add pull-specific parent actions: unnecessary because every eligible pull target already exists.

## Filesystem staging and recovery

**Decision**: Generalize filesystem helpers around `origin` and `target` without weakening descriptor checks. For pull, open and stream the destination origin, create an exclusive sibling staging file beside the source target, apply the planned supported mode, sync and verify staging, preserve and verify the prior source in the existing operation-local recovery layout, revalidate both sides, atomically replace the source, sync its parent, and verify the source against expected destination state.

**Rationale**: Staging beside the target retains the same-filesystem rename guarantee. Preserving the target before replacement is symmetric and reuses the proven private recovery contract. Descriptor-relative no-follow access is required on both mapping sides because pull reverses transfer direction, not trust boundaries.

**Alternatives considered**:

- Stage under the Grip home: may cross filesystems and lose atomic target publication.
- Copy directly over the source: exposes partial content and defeats verified recovery.
- Rename the destination into place: mutates the destination and violates synchronization semantics.

## Operation records and failure types

**Decision**: Preserve Partitioned Operation Record V1 and its directory layout. Extend the allowed operation identity to `push | pull`, allocate direction-prefixed opaque IDs, and initialize records from the shared mutation plan. Generalize push-specific failure and side-effect names to mutation failures carrying direction, while retaining stable external categories and compatibility constructors where useful during migration.

**Rationale**: The persisted envelope already has an `operation` field and generic checkpoint milestones. Allowing one additional closed value is sufficient; a new schema or parallel pull history would add migration and reader complexity without new evidence. Direction-bound plan identity prevents a semantically equivalent byte transfer in the other direction from sharing an identity.

**Alternatives considered**:

- Create Operation Record V2: unnecessary because the existing shape supports operation discrimination.
- Store pull records in a separate directory: duplicates lifecycle and future inspection rules.
- Keep `PushFailed` as the only failure type and overload it for pull: misleading and likely to produce wrong messages.

## Public result compatibility

**Decision**: Keep `ResultEnvelopeV1` and the established mutation details fields. Add `direction` to shared mutation results, with `push` for existing push results and `pull` for new pull results; retain `operation` as the invoked command. Entry and action projections continue to expose mapping-role source and destination paths. Human rendering selects verbs and target labels from direction but carries the same semantic counts, blockers, milestones, recovery availability, and baseline outcome.

**Rationale**: The clarification requires a shared versioned schema with `direction: pull`. The top-level envelope remains compatible, and the flexible `details` object can add a stable field without creating a second output family. Emitting direction for both commands makes the shared contract total and gives Feature 007 a stable discriminator.

**Alternatives considered**:

- Use `operation` as the only direction field: contradicts the accepted explicit `direction` contract.
- Define a pull-specific envelope version: fragments automation for otherwise symmetric operations.
- Rename source/destination fields to origin/target publicly: breaks the mapping vocabulary and existing consumers.

## Baseline publication

**Decision**: Reuse the existing locked accepted-state publisher after all pull actions verify and a final complete observation matches the plan. Update only actioned identities to the verified complete destination state now visible at both sides; preserve selected no-actions, out-of-scope records, and pending-retirement records. A no-op pull publishes no generation.

**Rationale**: State V2 already records mapping identities and complete supported states without direction. Direction changes how convergence is reached, not what accepted convergence means.

**Alternatives considered**:

- Add direction to baseline records: direction is operation history, not accepted-state identity.
- Publish after each action: falsely accepts partial multi-entry work.
- Accept converged two-sided changes during pull: changes the established explicit baseline-acceptance boundary.

## Testing and performance

**Decision**: Convert shared push tests only where required by the extraction, retain all push behavioral regressions, and add pull-focused planner, CLI, filesystem, recovery, contention, failure, operation-record, and output-delivery tests. Extend the ignored release harness with 10,000 established entries containing synchronized and destination-only changes, measuring 100 deterministic dry-run and pure execution-plan samples against the two-second p95 threshold.

**Rationale**: Shared code needs bidirectional invariant tests while command-specific acceptance still needs independent evidence. Reusing the existing representative fixture enables comparison without adding caches or parallelism before measurements justify them.

**Alternatives considered**:

- Mirror every push test file byte-for-byte: duplicates coverage without testing the direction table directly.
- Rely on shared unit tests only: misses source-side publication, recovery, and CLI projection failures.
- Add caching or parallel planning now: unsupported by current measurements and constitutionally disproportionate.
