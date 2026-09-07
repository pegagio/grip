# Research: Authorized Deletion and Retirement

This research resolves Feature 008's implementation choices against the verified classification, mutation, recovery, operation-record, registry-publication, and State V2 boundaries. No technical clarification remains open.

## Table of Contents

- [Command-owned workflow boundaries](#command-owned-workflow-boundaries)
- [Deletion planning and filesystem removal](#deletion-planning-and-filesystem-removal)
- [Retirement and force authorization](#retirement-and-force-authorization)
- [Recovery inventory and stable references](#recovery-inventory-and-stable-references)
- [Recovery provenance and legacy compatibility](#recovery-provenance-and-legacy-compatibility)
- [Payload restoration](#payload-restoration)
- [Registry and accepted-state restoration](#registry-and-accepted-state-restoration)
- [Cleanup and tombstones](#cleanup-and-tombstones)
- [Operation records and coordination](#operation-records-and-coordination)
- [Public results](#public-results)
- [Testing and performance](#testing-and-performance)

## Command-owned workflow boundaries

**Decision**: Add focused `delete`, `retire`, and public `recovery` modules. Reuse common observation, classification, path policy, mutation lock, state/registry publication, result envelope, and operation-component publication primitives, but keep each workflow's planner and executor typed to its own state machine.

**Rationale**: The existing shared mutation executor transfers supported state from an origin to a target. Deletion is child-first removal, retirement changes accepted state only, restore may replace payload or an authority document, and cleanup removes private bytes while publishing a tombstone. Forcing these into transfer direction or transfer action kinds would blur invariants and validation.

**Alternatives considered**:

- Add every workflow to `mutation::execution`: rejected because its direction, staging, final-observation, and baseline assumptions do not describe retirement or cleanup.
- Duplicate all filesystem and publication code: rejected because descriptor safety, atomic state/registry publication, mutation coordination, and result delivery are genuine shared invariants.
- Add a new crate or dependency: rejected because the single-crate architecture already contains the required primitives.

## Deletion planning and filesystem removal

**Decision**: Build a pure deletion plan from accepted identities and complete classified observations. One explicit authority side maps to removal of the opposite peer. Order managed actions by mapping identity, relative-path component depth descending, and raw relative bytes ascending; encode child-to-parent dependencies. Before a directory action is admitted, inspect its complete target subtree and block the plan if removal would include an unmanaged, differently classified, unsupported, mounted, unsafe, or incompletely observed descendant.

Remove files descriptor-relatively without following links. Remove a directory only after its planned managed children verify absent and a fresh descriptor-bound enumeration proves it empty. Never use recursive path deletion.

**Rationale**: Existing discovery exposes unmanaged destination children, unsupported nodes, and raw-byte ordering, while existing `EntryIdentity` provides canonical source-relative identity. A child-first plan makes partial outcomes and recovery bindings deterministic. Empty-directory removal after fresh enumeration closes the race between preflight and removal without claiming snapshot isolation.

**Alternatives considered**:

- `remove_dir_all`: rejected because a concurrent or previously unmanaged descendant could be erased.
- Parent-first removal with recursive fallback: rejected because it hides action ownership and cannot preserve each entry independently.
- Treat directory deletion as Feature 009 metadata work: rejected because Feature 008 explicitly owns converged and directional deletion; only expanded metadata fidelity remains deferred.

## Retirement and force authorization

**Decision**: Create a pure retirement plan over `newly_ignored_pending_retirement`, `untracked_pending_retirement`, and `converged_deletion`. Require exactly one path selector or `--all`. Converged deletion and equivalent surviving copies retire normally. Differing surviving copies yield `force_required` and block unless `--force`; force authorizes only loss of comparison history. Active, unsupported, unsafe, or incomplete evidence blocks regardless of force.

Extend observation of ignored and untracked accepted identities to capture safe live source and destination evidence without changing their retained membership classification. Execute an actionful plan under the mutation lock by rebuilding the same plan, journaling it, removing only selected baseline records from a cloned accepted state, and publishing one State V2 generation.

**Rationale**: Current classification labels ignored and untracked identities before comparing their surviving copies. The additional observation is required to distinguish ordinary retirement from forced evidence discard. Keeping force narrow prevents it from becoming a bypass for ownership or path safety.

**Alternatives considered**:

- Let a bare command select all: rejected during clarification as too easy to invoke accidentally.
- Block differing copies forever: rejected because deliberately ending management must remain possible.
- Make `--force` suppress all blockers: rejected because it would weaken filesystem and evidence safety.

## Recovery inventory and stable references

**Decision**: Build a read-only adapter over the three existing stores rather than introduce a central mutable index:

- operation payload recovery beneath `state/operations/<operation-id>/recovery/<action-index>/`;
- content-addressed registry recovery beneath `state/recovery/registry/sha256-<digest>/`;
- generation-addressed accepted-state recovery beneath `state/recovery/generation-<generation>/`.

Expose typed opaque references such as `payload:<operation-id>:<action-index>`, `registry:sha256:<digest>`, `state:generation:<generation>:sha256:<digest>`, and `operation:<operation-id>`. Parse a closed grammar, then resolve only validated components through private descriptor-safe access. Enumerate in kind then canonical identity order. `list` reads metadata only; `show` may verify specifically selected bytes but never prints payload contents by default.

**Rationale**: A central catalog would become a second authority requiring transactional updates across independent recovery writers. Existing storage keys already provide stable internal identities; a typed public reference avoids exposing or freezing private paths.

**Alternatives considered**:

- Public recovery paths: rejected because they enlarge traversal risk and create a permanent layout contract.
- A persistent recovery database or index: rejected pending evidence that deterministic enumeration misses the performance target.

## Recovery provenance and legacy compatibility

**Decision**: Add a strict integrity-protected Recovery Manifest V1 sidecar for every newly retained payload, registry, or accepted-state document. It records kind, opaque reference, creation time, origin operation or transition, original target and side where applicable, prior document/state digest, expected post-action evidence, byte count, and restore target. Cleanup status is not mutable in the manifest; it is represented by a separate integrity-protected cleanup tombstone.

Preserve existing Recovery Metadata V1, registry recovery files, accepted-state generations, and operation records byte-for-byte. Inventory synthesizes a conservative legacy projection when no manifest exists. Legacy entries remain listable and inspectable, but restore is eligible only when existing records independently prove every required origin and post-action compatibility fact. Missing proof yields `legacy_provenance_incomplete`, never a guessed binding or migration rewrite.

**Rationale**: Current payload metadata does not contain creation time, original side, expected post-action target, or cleanup state; registry and state backups have still less cross-transition provenance. Sidecars make future recovery safe without rewriting immutable evidence. A separate tombstone distinguishes completed cleanup from missing or unexpectedly removed bytes.

**Alternatives considered**:

- Rewrite historical metadata into a new schema: rejected because retained operation evidence is immutable.
- Mutate `payload_present` during cleanup: rejected because a crash between byte removal and metadata replacement becomes ambiguous and erases the original fact.
- Restore legacy entries from path naming alone: rejected because layout is not sufficient authority for current ownership or post-action state.

## Payload restoration

**Decision**: Restore one exact verified payload entry only to its manifest-bound original target. Require integrity-valid manifest, plan and checkpoint provenance, safe current mapping ownership, accepted membership, no-follow ancestry, verified recovery bytes matching prior supported state, and a proven originating post-action expectation. Permit an absent target. Permit an occupied target only when it exactly matches the originating operation's verified post-action state; preserve that displaced state in the new restore operation before replacement.

Restore directory trees as individually recovered managed entries in parent-before-child restoration order. A directory recovery record represents the directory node and supported mode, not an archive. Restoration changes payload only and never accepts a baseline automatically.

**Rationale**: Exact post-action equality prevents recovery from overwriting later user edits. Individual entries reuse the allowlisted file/directory model and avoid archive extraction, traversal, and new format risks.

**Alternatives considered**:

- Restore to a caller-selected path: rejected because recovery authority is bound to the original managed identity.
- Overwrite any occupied target after confirmation: rejected because the specification permits only exact post-action replacement.
- Add a tar/archive format for directories: rejected as unnecessary and riskier than typed per-entry recovery.

## Registry and accepted-state restoration

**Decision**: Restore an exact recovered registry only after verifying its digest, strict schema, complete ownership graph, endpoint path policy, recorded transition provenance, and compatibility with live accepted-state and payload evidence. Restore an exact recovered accepted-state document only after strict decode, integrity and generation validation, recorded transition provenance, live-registry validation, safe paths, and complete live payload observation. Legitimate payload drift need not equal the historical baseline, but identities and supported observations must remain safe and representable.

A valid current target must equal the manifest's recorded post-transition document; a newer or different valid authority blocks. A missing or corrupt target being repaired need not validate, but the recovered candidate and every remaining authority must. Reuse existing sibling staging, verification, atomic rename, parent sync, and visibility/durability reporting. Restore exact bytes rather than synthesizing a document or generation.

**Rationale**: This fulfills bounded recovery from missing or corrupt authority without reconstructing ownership from guesses. Exact publication preserves the historical document's integrity domain, while compatibility checks prevent stale authority from being revived into a changed namespace.

**Alternatives considered**:

- Require the broken target to validate: rejected during clarification because it makes repair impossible.
- Derive mappings or baselines from current files: rejected because payload observation cannot establish user intent or accepted history.
- Publish the recovery as a newly synthesized generation: rejected because it changes the recovered evidence and hides which authority was restored.

## Cleanup and tombstones

**Decision**: Resolve, deduplicate, strictly validate, and canonically sort every exact cleanup reference before coordination. Preview remains lock-free. Execution acquires the mutation lock, rebuilds the same plan, initializes an operation record, and processes each entry in order. For each entry, checkpoint intent, unlink only descriptor-bound recoverable bytes, sync the containing directory, verify absence, atomically publish an integrity-protected immutable cleanup tombstone, then checkpoint completion. Stop at the first failure and distinguish completed, failed, and unattempted references.

Cleanup never deletes the manifest, tombstone, accepted authority, or operation provenance. Operation references are inspectable but are not cleanup targets in this feature. Recovery bytes referenced by a currently active lock owner are blocked. If interruption occurs after unlink but before tombstone publication, later inspection reports `cleanup_incomplete` and requires a fresh explicit cleanup retry to publish the tombstone.

**Rationale**: Retained metadata keeps operation results truthful after sensitive bytes are removed. Byte absence without a tombstone must not be misreported as successful cleanup. Exact references and a closed private namespace keep cleanup bounded.

**Alternatives considered**:

- Remove manifests and operation records with the payload: rejected because it destroys auditability and provenance.
- Claim multi-reference atomic cleanup: rejected because filesystem deletion across directories has no such guarantee.
- Automatic age or size pruning: explicitly outside the feature.

## Operation records and coordination

**Decision**: Preserve Partitioned Operation Record V1 layout and strict envelope mechanics. Generalize initialization around a typed operation-plan projection containing operation, plan ID, serialized immutable plan, and action count; admit `delete`, `retire`, `recovery_restore`, and `recovery_remove` without forcing their plans into `MutationPlan`. Keep command-specific semantic validators.

All actionful workflows use `.mutation.lock`. Registry restore additionally acquires `.registry.lock`; accepted-state publication or restore acquires `state/state.lock`, preserving `mutation -> registry -> state`. Read-only inventory, previews, blocked plans, and semantic no-ops remain lock-free. After lock acquisition, rebuild and compare the complete plan before creating an operation record.

**Rationale**: Existing operation publication already supplies private partitioned evidence and result-delivery finalization, but its initializer currently accepts only transfer plans. A small generic projection preserves journal mechanics while keeping plan semantics explicit.

**Alternatives considered**:

- Reuse stale operation records as active ownership: rejected because the advisory lock is the ownership signal.
- Resume or roll back interrupted operations automatically: excluded by the specification.
- Give each command a separate lock: rejected because it would permit conflicting writers to interleave.

## Public results

**Decision**: Preserve Result Envelope V1 and existing exit categories. Add typed details for deletion authority, retirement reason and force requirement, recovery kind/reference, origin, bound side/path, integrity, availability or cleaned state, byte count, eligibility/blockers, ordered action outcomes, visibility, verification, durability, operation record, and accepted-state authority. Human output derives from the same types; diagnostics remain on stderr.

**Rationale**: The existing result boundary already separates stable machine evidence from presentation and diagnostics. Additive typed details avoid new outer schemas or free-form maps.

**Alternatives considered**:

- Print private recovery paths: rejected as a storage-layout and traversal hazard.
- Add command-specific result envelopes or exit codes: rejected because existing categories already distinguish usage, blocking, schema, corruption, contention, and operational failure.

## Testing and performance

**Decision**: Add pure policy/reference/plan tests and isolated CLI, filesystem, recovery, contention, failure, state-authority, tombstone, and output tests. Use typed test-only fault hooks at preservation, unlink, absence verification, final observation, state/registry publication, recovery byte removal, directory sync, tombstone publication, and result delivery. Extend the ignored release harness to 10,000 deletion classifications and 10,000 recovery entries, measuring 100 warm JSON previews/inventories with p95 at most two seconds.

**Rationale**: The highest risks are child-first ownership, crash-visible cleanup, provenance validation, and authority restoration. Deterministic fault seams test them without timing sleeps or production backdoors. Measurement must precede any cache or index.

**Alternatives considered**:

- Timing-dependent race tests: rejected as flaky.
- End-to-end tests alone: rejected because pure policy and strict schema validation need exhaustive matrices.
- Add caching or parallel enumeration before measurement: constitutionally disproportionate.
