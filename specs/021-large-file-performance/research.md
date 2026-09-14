# Research: Large-File Operation Performance

## Decision: Extend the existing release performance harness

**Decision**: Extend `tests/performance_acceptance.rs` rather than create a benchmark framework.

**Rationale**: The existing ignored harness already builds and invokes `target/release/grip`, uses an isolated temporary project, clears ambient environment, measures p50/p95/maximum, and asserts deterministic process results. `mise run performance` is the established entry point. The feature extends it with distinct `add` and `status` approximately 19 MB unequal-file cases, 100 warm samples, and a one-second p95 requirement.

**Alternatives considered**:

- Add a separate benchmark tool or dependency: rejected because the existing harness already exercises the real release CLI safely and deterministically.
- Time ordinary unit tests: rejected because they neither invoke the release CLI nor provide a representative p95 distribution.

## Decision: Measure duplicate complete-file observation first

**Decision**: Capture before-and-after distributions for the two feature workflows and attribute elapsed work before treating any optimization as complete.

**Rationale**: File fingerprints stream full content through SHA-256 in bounded chunks and retain descriptor and metadata evidence. For a normal file mapping, `observation::inspect` and `observation::inspect_candidate` each perform two stable-observation passes. Within each pass, the current file-mapping path and discovery-record path can fingerprint the same source and destination again. Static inspection identifies this duplication as the likely large-file cost, but the target hardware benchmark remains the acceptance evidence.

**Alternatives considered**:

- Assume hashing is the bottleneck and change it immediately: rejected because the feature requires a demonstrated cause and preserves full content identity.
- Attribute the cost to classification: rejected because classification compares captured states and does not stream file content.

### Measured baseline (2026-09-14)

`mise run performance` exercised the local macOS ARM64 release binary with 100 warm samples for each isolated 19 MiB unequal-file workflow. `grip add` measured p50 `1.073838958s`, p95 `1.098413458s`, and maximum `1.107355292s`; it exceeded the one-second p95 target. `grip status` measured p50 `345.325208ms`, p95 `347.845667ms`, and maximum `373.55925ms`; it met the target.

The over-target `add` workflow performs stable candidate observation and then the fenced post-publication reinspection. Within each observation pass, the file-mapping pre-pass and its eligible discovery record independently stream the same source and destination files for complete SHA-256 and metadata evidence. Removing only that same-pass duplication is therefore the measured, bounded remedy; the separate stable passes and fenced reinspection remain required.

### Measured result after same-pass observation deduplication (2026-09-14)

The same isolated 100-run release measurement after the change recorded `grip add` p50 `565.837584ms`, p95 `573.005834ms`, and maximum `595.28875ms`; `grip status` recorded p50 `175.742792ms`, p95 `177.207917ms`, and maximum `178.189292ms`. Both workflows meet the one-second p95 target.

The implementation skips direct file-mapping observation only when the same pass contains that mapping's eligible discovery record. Discovery-absent, unsupported, and non-file records retain direct-observation fallback behavior. Each retained observation pass still performs complete descriptor-bound SHA-256 and metadata evidence, and the independent second pass plus fenced `add` reinspection remain unchanged.

### Final acceptance rerun (2026-09-14)

The documented `mise run performance` gate passed after the one-second assertion was enabled. Its 100-run large-file distributions recorded `grip add` p50 `567.853291ms`, p95 `573.669375ms`, and maximum `603.771792ms`; `grip status` p50 `176.468916ms`, p95 `184.788375ms`, and maximum `205.370875ms`.

## Decision: Select observation deduplication only after the baseline confirms its causality

**Decision**: Record the baseline distribution first. Only if it shows an over-target workflow and confirms duplicate observation as a material cause, make normal file mappings represented by a discovery record reuse that pass's complete source/destination observation instead of separately observing the same endpoints in the file-mapping pre-pass. Retain a fallback for file mappings absent from discovery records. Keep the first and second observation passes intact.

**Rationale**: This eliminates redundant work while preserving the existing model: each retained pass still opens each regular file without following it, validates supported-node properties, streams and hashes content, observes supported metadata, and checks identity and timestamps after reading. The second pass still detects unstable observation evidence before classification or publication.

**Alternatives considered**:

- Reuse a prior command's fingerprint or add persistent state: rejected because it complicates invalidation and weakens fresh filesystem evidence.
- Replace content fingerprints with length or modification time: rejected because equal metadata cannot establish equal content and would weaken baseline/classification correctness.
- Remove the second pass: rejected because it weakens stale-observation detection for concurrent external edits.

## Decision: Reject unsafe performance runs explicitly

**Decision**: Require the release binary and an isolated temporary fixture before timing either workflow. If either precondition cannot be satisfied, fail the harness with a specific diagnostic and never substitute an operator-managed path.

**Rationale**: A performance result is useful only when it is reproducible and isolated. An explicit preflight failure is safer and more actionable than attempting an alternative location or treating an incomplete workload as an accepted sample.

**Alternatives considered**:

- Fall back to the current project or home directory: rejected because it could inspect or mutate operator-managed state.
- Treat fixture setup failure as a skipped successful performance result without a reason: rejected because it obscures why no acceptance evidence was produced.

## Decision: Preserve fenced add reinspection and initial comparison semantics

**Decision**: Do not reduce the post-publication reinspection that proves a fenced unequal add still corresponds to the candidate state, and do not change the destination-derived initial comparison state.

**Rationale**: An unequal `add` publishes coupled descriptor and state evidence under a short-lived fence. The subsequent check detects destination drift before state publication completes. This safety boundary is distinct from redundant per-pass file observation.

**Alternatives considered**:

- Skip post-publication reinspection for speed: rejected because it would weaken the add drift boundary.
- Make an unequal add copy a payload to avoid a later comparison: rejected because `add` is intentionally non-mutating for endpoint payloads.

## Decision: Make performance evidence semantic as well as temporal

**Decision**: Every timed workflow asserts its existing process result, and complementary checks prove payload, mapping-publication, initial-state, and classification semantics before and after the observation change.

**Rationale**: A faster command is not acceptable if it silently changes what mapping was recorded, which baseline state was published, or what `status` reports.

**Alternatives considered**:

- Assert elapsed time only: rejected because it would not catch a shortcut that omits required observation or changes classification.
- Assert stdout only: rejected because descriptor/state publication and JSON classification are also part of the command contract.
