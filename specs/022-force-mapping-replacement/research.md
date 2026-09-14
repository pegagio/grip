# Research: Force Mapping Replacement

This research resolves the implementation choices for Feature 022 using the existing local CLI architecture and its accepted descriptor/state publication model.

## CLI Authority

**Decision**: Add `force: bool` to `cli::AddArgs` using the existing `-f`/`--force` clap convention, with `false` preserving ordinary add behavior.

**Rationale**: The requested interface is an explicit modifier of `grip add`, not a distinct lifecycle operation. The CLI already uses this flag spelling for explicitly forced directional operations. Leaving the default path unchanged preserves scripts and existing conflict behavior.

**Alternatives considered**:

- A separate `replace` command was rejected because it duplicates add’s endpoint validation and makes the mapping lifecycle harder to understand.
- An interactive confirmation was rejected because the explicit flag is the specified confirmation and a prompt would make automation non-deterministic.

## Replacement Selection and Ownership Validation

**Decision**: After normal endpoint validation and under the existing metadata mutation lock, force mode will locate exactly one distinct active `File` mapping whose canonical destination equals the requested mapping’s canonical destination. It will remove only that portable declaration, append the requested declaration, resolve the complete candidate descriptor, and run the existing full ownership validation.

**Rationale**: `ResolvedRegistry::new` and `mapping::validate_ownership` already enforce duplicate-source, source-overlap, destination-overlap, tree, and nested constraints for the complete registry. Constructing a valid replacement candidate preserves those guarantees; bypassing validation would turn `--force` into a broad ownership override. Canonical comparison permits alternate destination spellings that resolve to the same endpoint while retaining the accepted declaration spelling.

**Alternatives considered**:

- Removing every destination conflict was rejected because it could silently retire multiple mappings.
- Removing a mapping before candidate validation was rejected because a later validation error could lose accepted membership.
- Treating tree or nested conflicts as replaceable was rejected because Feature 022 deliberately authorizes only one equal-destination file mapping.

## Descriptor and State V4 Publication

**Decision**: Generalize the existing add publication fence into a bounded descriptor/state transition fence usable by forced replacement and exact removal. Build the complete candidate descriptor and descriptor-bound State V4 bytes before descriptor publication; persist a fence containing both prior and candidate pairs; revalidate and publish the descriptor, then publish and verify the candidate state, clearing the fence only after the pair is authoritative. Retry or recovery must complete the candidate pair or restore the prior pair, never report a mixed pair as success.

**Rationale**: State V4 rejects a baseline whose mapping is absent from the descriptor. Current add already uses a fence for an unequal-endpoint descriptor/state transition. Current remove publishes the descriptor and then prunes baselines, which can expose stale state if its second publication fails. A single recoverable transition lets forced replacement retire the displaced evidence and makes exact removal satisfy the re-add postcondition even across failure or drift.

**Alternatives considered**:

- Publishing the replacement descriptor and then calling the existing remove/add baseline helpers was rejected because it can leave an orphaned baseline or bind state to the prior descriptor.
- Adding a new transaction service or database was rejected as disproportionate to a local two-file publication problem.
- Relying on a broad filesystem lock was rejected because the constitution requires bounded coordination only for Grip-owned metadata.

## Candidate Baseline Policy

**Decision**: For a forced replacement, remove every accepted baseline identity belonging to the displaced resolved mapping from the in-memory candidate state, observe only the requested mapping against the candidate registry, and apply `baseline::build_for_add` for that mapping before State V4 encoding. Publish a descriptor-bound state candidate even when the endpoint shape would normally take add’s non-fenced path.

**Rationale**: `build_for_add` carries unrelated baselines forward, so it cannot by itself retire displaced evidence. Rebinding the whole candidate state to the replacement descriptor maintains State V4’s declaration invariant. The new mapping retains the established initial-comparison behavior for equal, unequal, source-only, and destination-only endpoints.

**Alternatives considered**:

- Reusing the add-only candidate unchanged was rejected because it preserves baselines for a mapping no longer declared.
- Dropping every baseline was rejected because it discards unrelated accepted evidence.
- Copying a payload to create a baseline was rejected because add is declarative and must not mutate either endpoint.

## Exact Removal Semantics

**Decision**: Preserve source-exact removal selection. The removal transition will filter only the selected declaration and only baseline identities for that resolved mapping, then use the same descriptor/state transition safety model. An unrelated removal will not clear another mapping’s destination ownership or evidence.

**Rationale**: Existing remove identifies mappings by resolved source and prunes baselines by that source. Therefore removing `target/debug/mise` correctly cannot remove an active `target/debug/grip -> ~/.local/bin/grip` mapping. The requested ordinary add succeeds only after removal of the exact active owner.

**Alternatives considered**:

- Clearing state by destination after any removal was rejected because it could erase evidence for unrelated mappings.
- Treating a different removal as implicit force authorization was rejected because it conflicts with explicit ownership and least surprise.

## Result and Recovery Contract

**Decision**: Preserve the existing ordinary add result shape. For a forced success, return `operation: "add"`, the new mapping in `mapping`, and the retired mapping in `replaced_mapping`, with a `Mapping replaced:` human rendering that identifies both. Persist or deterministically derive replacement context for fenced retry so completed recovery returns the same replacement result rather than an ordinary `Mapped:` result.

**Rationale**: Automation needs a stable machine-readable distinction and operators need to see exactly what changed. The current fence tracks only the requested mapping, so retry context must be extended or recovered from its prior/candidate descriptors.

**Alternatives considered**:

- Reusing `Mapped:` for forced success was rejected because it conceals the retired mapping.
- Exposing only a prose message was rejected because JSON consumers need structured replacement information.

## Test Strategy

**Decision**: Add isolated temporary-root coverage in the CLI contract suite, extend topology coverage for retained rejection boundaries, and adapt existing fence fault tests for replacement/removal recovery.

**Rationale**: The project’s filesystem behavior and accepted state are observable only through integration-style fixtures. Existing suites already cover add-time baselines, exact removal/re-add, topology conflicts, payload non-mutation, and fenced recovery.

**Alternatives considered**:

- Unit-only coverage was rejected because it cannot verify descriptor, State V4, fence, and endpoint payload interactions.
- Tests against actual user paths were rejected by the constitution and existing test isolation practice.
