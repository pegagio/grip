# Research: Baselines, Classification, and Status

This research resolves Feature 004 implementation choices without expanding into payload copying, conflict resolution, deletion execution, automatic state recovery, multiple selectors, or final metadata fidelity.

## Table of Contents

- [Observation and classification boundary](#observation-and-classification-boundary)
- [Supported fingerprint](#supported-fingerprint)
- [Safe file hashing](#safe-file-hashing)
- [Complete stable inspection](#complete-stable-inspection)
- [State schema evolution](#state-schema-evolution)
- [Baseline identity and ordering](#baseline-identity-and-ordering)
- [Classification matrix](#classification-matrix)
- [Selection and pending retirement](#selection-and-pending-retirement)
- [Baseline acceptance transaction](#baseline-acceptance-transaction)
- [Public command and result contracts](#public-command-and-result-contracts)
- [Errors and exit behavior](#errors-and-exit-behavior)
- [Testing and performance](#testing-and-performance)

## Observation and classification boundary

**Decision**: Introduce a shared internal observation layer that returns stable evidence for source and destination nodes, membership policy, registry identity, and supported fingerprints. Preserve `mapping inspect` as a projection over that layer. Join observations and persisted baselines by structured entry identity, then pass only normalized optional supported states into a pure exhaustive classifier.

**Rationale**: Feature 003's public discovery inventory intentionally discards its internal evidence and emits separate records that cannot safely be classified by parsing rendered categories. A shared observation snapshot keeps filesystem access, policy, classification, and presentation independently testable.

**Alternatives considered**: Reopening display paths after discovery loses raw-byte identity and descriptor ancestry. Parsing `DiscoveryRecord` categories cannot reliably merge ignored and destination-only records. Adding classification directly to traversal couples filesystem behavior to the full state matrix.

## Supported fingerprint

**Decision**: Represent an ordinary file as node kind `file`, SHA-256 content digest, byte length, and Unix permission mode masked to `0o7777`. Represent an eligible directory by node kind only. Retain current modification time as diagnostic evidence but exclude it from equality, changed dimensions, accepted baselines, direction, and conflicts.

**Rationale**: The chosen fields resolve the first metadata contract exactly as specified. Length helps users and validation but cannot substitute for content. Excluding mtime makes no-op acceptance and reproducible equality independent of timestamp-only changes. Independent directory metadata and broader Unix/macOS attributes remain Feature 009.

**Alternatives considered**: Mtime-based equality can miss edits or invent drift after equivalent copies. Persisting mtime as accepted equality evidence contradicts the clarified contract. Including ownership, xattrs, ACLs, flags, or directory mode now would pull Feature 009 forward.

## Safe file hashing

**Decision**: Use the existing `sha2` dependency to stream bytes through a fixed buffer from a descriptor opened with read-only, close-on-exec, no-follow, and nonblocking flags. Bind the descriptor to prior non-following metadata, require an ordinary single-link non-sparse file, and compare descriptor metadata before and after hashing. Use descriptor-relative opens for tree members and an equivalent no-follow root open for file mappings.

**Rationale**: Streaming bounds memory, while descriptor identity and pre/post evidence reject substitution or in-place change without following links or opening unsupported nodes. Existing `rustix` primitives and SHA-256 are sufficient.

**Alternatives considered**: `std::fs::read` loads complete files and reopens pathnames without ancestry binding. Trusting size and mtime is not content equality. A new hashing or filesystem dependency adds no capability.

## Complete stable inspection

**Decision**: Extend Feature 003's sequential two-pass design so both complete passes include fingerprints. Return a classification result only when registry bytes and identity, directory membership, policy bytes, node evidence, and supported fingerprints are equivalent. Separate durable registry ownership validation from current source-presence inspection so missing mapped roots can reach deletion classification while canonical ownership and topology validation remain complete.

**Rationale**: Two complete passes preserve the existing stale-evidence boundary and make content changes visible. The current registry loader requires sources to exist, which would reject a deleted file mapping or tree root before Feature 004 could classify it.

**Alternatives considered**: One hash plus final metadata cannot detect same-size content changes with restored timestamps. Keeping current source-presence validation in registry loading makes required deletion states unreachable. Broad locks or snapshots violate the product boundary.

## State schema evolution

**Decision**: Keep strict State Envelope V1 unchanged. Add a strict State Envelope V2 and a version-neutral `AcceptedState` domain model. Read valid V1 generation `N` as an empty-baseline predecessor; publish the first semantic update as V2 generation `N+1`. Missing state is uninitialized and first publication is V2 generation `0`. V2 integrity covers deterministic compact serialization of `{schema_version:2,payload}` and excludes the integrity member.

**Rationale**: Feature 001 explicitly froze V1's canonical integrity input and directed future versions to define their own. Reading V1 avoids invalidating state the current binary can already create, while a V2 writer makes the new wire shape explicit and rejects unknown fields.

**Alternatives considered**: Adding fields under schema version 1 breaks its strict schema and digest contract. Rejecting V1 would make the application's own valid predecessor state unsupported. Eager migration on read would violate read-only commands.

## Baseline identity and ordering

**Decision**: Key each baseline with a complete mapping snapshot—kind, canonical source root, canonical destination root—and exact relative path bytes encoded losslessly. Store baseline records as a canonically sorted unique array. Sort keys by canonical source, mapping kind and destination tie-breakers, then raw relative bytes. Validate absolute canonical UTF-8 roots, safe component-relative bytes, unique keys, supported-state field applicability, and canonical ordering during decode.

**Rationale**: Source alone cannot distinguish removed and re-added intent with another destination or kind, nor resolve destination selectors for retained untracked evidence. Structured keys avoid delimiter ambiguity and preserve deterministic integrity bytes.

**Alternatives considered**: A source-only key can attach old evidence to new mapping intent. Concatenated strings create escaping ambiguities. A JSON object keyed by encoded paths obscures ordering and validation.

## Classification matrix

**Decision**: Build the union of current active observations and persisted baseline keys, then use one pure classifier for every no-baseline and baseline-present combination. Comparisons produce optional changed-dimension sets for source-to-baseline, destination-to-baseline, and source-to-destination; `null` means unavailable evidence and an empty set means a valid equivalent comparison. Unsupported managed evidence and unsafe paired collisions override ordinary equality as blockers.

**Rationale**: A pure total function makes the required classifications, direction, attention, blocking, and diff dimensions exhaustively testable. The union prevents deleted, newly ignored, or untracked baseline records from disappearing.

**Alternatives considered**: Command-specific classifiers invite status/check/diff drift. Comparing only current sides cannot distinguish one-sided changes or converged edits. Treating unavailable comparison as no difference creates false equivalence.

## Selection and pending retirement

**Decision**: Normalize every optional selector into path space plus a current mapping snapshot and raw relative component prefix, or a retained baseline identity when current mapping ownership is absent. Selection includes current and baseline-only records under the component boundary. A baseline excluded by current source policy is `newly_ignored_pending_retirement`; a baseline whose complete mapping snapshot is absent is `untracked_pending_retirement` and its old payload paths are not reopened. Destination-only and ignored entries without baselines remain informational and unmanaged.

**Rationale**: Including baseline-only records prevents scoped commands from silently hiding deletions or retirement. Not reopening removed mapping paths respects the current ownership boundary while retaining evidence for Feature 008.

**Alternatives considered**: Current-discovery-only selection loses deleted and retired evidence. Reusing a removed mapping to inspect payloads would restore ownership the user explicitly removed. Treating removal as corrupt state contradicts clarification.

## Baseline acceptance transaction

**Decision**: Build a complete deterministic candidate from stable inspection, clone the full accepted baseline map, replace only selected acceptable records, and preserve every out-of-scope or pending-retirement record. Acquire bounded locks in global order: registry publication lock, then state publication lock. Under both locks, reload and compare expected registry and state, repeat selected observation and fingerprint validation, rebuild the candidate, revalidate the registry immediately before rename, and then retain recovery, stage, verify, atomically rename, and sync. Generation advances exactly once for a semantic update. An already-current request still locks and revalidates but publishes no recovery, staging file, generation, or accepted-state bytes.

**Rationale**: Copy-on-write prevents scoped data loss. Expected-snapshot checking and final revalidation prevent stale acceptance while coordinating only Grip-owned metadata. Validating a no-op before reporting avoids claiming current evidence after a concurrent change.

**Alternatives considered**: Publishing selected records alone discards unrelated history. State-only locking races mapping updates. Skipping validation for no-op makes its success untrustworthy. Payload locks or automatic rollback add disproportionate complexity.

## Public command and result contracts

**Decision**: Add top-level `status`, `check`, and `diff` commands plus nested `baseline accept`, all sharing one optional selector with `--destination` path-space choice. Preserve the Result Envelope V1 outer shape and use typed classification/baseline DTOs inside `details`. All stable count keys are present. Unavailable comparisons serialize as `null`; valid no-change comparisons serialize as empty arrays. Human output renders the same typed domain values.

**Rationale**: Shared grammar and typed projections keep four commands consistent without changing the established outer automation envelope. `mapping inspect` remains the membership-only command from Feature 003.

**Alternatives considered**: Separate selector parsers can drift. Generic untyped maps make required fields easy to omit. Replacing the outer envelope would break established consumers without need.

## Errors and exit behavior

**Decision**: Add `AttentionRequired` with status `ok`, code `attention_required`, and exit `1`; only completed `check` uses it. Activate reserved state contention with code `state_contention` and exit `13` for the public state publisher. Status and diff return `ok`/0 after complete inspection. Baseline-ineligible evidence is invalid configuration/10 with complete corrective records; unsupported state schema is 11, corrupt state is 12, stale evidence and operational/publication failures are 20. Post-rename sync failure must report visible publication with durability unconfirmed.

**Rationale**: Automation can distinguish attention from failed inspection, and the existing reserved contention category now has a public owner. Truthful post-rename reporting preserves recovery semantics.

**Alternatives considered**: Treating attention as error status conflates completed classification with failure. Returning the first ineligible record hides the rest. Reporting post-rename sync failure as unchanged is false.

## Testing and performance

**Decision**: Add exhaustive table-driven pure classification tests; isolated content, mode, timestamp, deletion, ignore, unsupported, selector, state-version, scoped acceptance, contention, recovery, and fault integration tests; typed human/JSON parity tests; and deterministic barriers at file read, pass, locked revalidation, and publication boundaries. Extend the release harness with exactly 100 warm JSON status runs over 10,000 equivalent paired files and accepted baselines, requiring byte-equivalent output and p95 at or below two seconds. Measure sequential behavior before considering optimization.

**Rationale**: Pure tests prove matrix completeness while real temporary filesystems prove no-follow, hashing, mode, state, and publication behavior. The existing harness supplies comparable evidence without another benchmark framework.

**Alternatives considered**: Mock-only coverage cannot prove filesystem safety. Timing `check` complicates a harness that treats nonzero as process failure. Adding caches or parallelism before measurement violates proportional rigor.
