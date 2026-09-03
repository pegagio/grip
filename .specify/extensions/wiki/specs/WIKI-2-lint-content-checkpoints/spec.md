# Feature Specification: Lint Content Checkpoints

**Feature Branch**: None

**Diagram Issue**: `WIKI-2`

**Created**: 2026-08-26

**Status**: Draft

**Input**: User description: "Modify the lint command so that it not only compares modification timestamps, but a content hash, like an md5 or sha-256. It doesn't need to be cryptographically secure - just something to checkpoint a file state."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Detect Content Changes Hidden by Timestamps (Priority: P1)

A project maintainer runs wiki lint and receives a stale finding when a checkpointed file's contents changed even though its modification timestamp or structural updated date did not.

**Why this priority**: Timestamp-only freshness can miss restored files, copied files, clock anomalies, and edits that preserve metadata, leaving maintainers with a false sense that knowledge is current.

**Independent Test**: Establish a lint checkpoint, change a checkpointed file while preserving its prior timestamp, rerun lint, and verify that the content change produces one evidence-backed stale finding.

**Acceptance Scenarios**:

1. **Given** a repository-local registered source with an existing checkpoint, **When** its content changes while its timestamp remains unchanged, **Then** the stale check reports that the source content changed and recommends refreshing the source.
2. **Given** an indexed wiki page with an existing checkpoint, **When** its content changes without advancing its structural updated date, **Then** the stale check reports the mismatch without rewriting the page.
3. **Given** a checkpointed file whose timestamp changes but whose content fingerprint remains the same, **When** lint runs, **Then** lint does not claim that the file's content changed solely because of the timestamp.
4. **Given** a checkpointed file whose timestamp and content fingerprint are unchanged, **When** lint runs, **Then** no content-change finding is produced for that file.

---

### User Story 2 - Establish and Advance Reliable Checkpoints (Priority: P2)

A project maintainer can run lint repeatedly and have each successful run establish a deterministic baseline for the exact file state that was evaluated.

**Why this priority**: Content comparison is useful only when checkpoints are durable, scoped, reviewable, and updated after the lint result accurately describes the final state.

**Independent Test**: Run lint against a clean fixture with no checkpoint, rerun unchanged, apply an allowed mechanical repair, and verify baseline creation, stable comparison, and checkpoint advancement to the repaired final bytes.

**Acceptance Scenarios**:

1. **Given** a valid wiki with no prior checkpoint artifact, **When** lint succeeds, **Then** lint records an initial baseline and does not report the absence of history as a content change.
2. **Given** a page-scoped or check-scoped lint run, **When** lint succeeds, **Then** only checkpoint entries examined by that scope are advanced and out-of-scope entries remain unchanged.
3. **Given** an allowed mechanical repair during lint, **When** the run succeeds, **Then** the checkpoint records the final repaired file state rather than the pre-repair state.
4. **Given** identical files and scope across repeated runs, **When** lint succeeds again, **Then** checkpoint ordering and fingerprints remain stable.

---

### User Story 3 - Preserve Lint Safety Boundaries (Priority: P3)

A project maintainer gains content-aware freshness without allowing lint to crawl unrelated files, fetch remote sources, modify source content, or leave partially updated checkpoint state.

**Why this priority**: Broader file reads and new persistent state must not weaken the wiki's containment, untrusted-text, bounded-scope, or atomic-write guarantees.

**Independent Test**: Exercise escaping symlinks, URL sources, oversized directories, malformed checkpoint data, concurrent file changes, and failed writes; verify bounded reads, explicit unknown results, no remote fetches, and complete rollback.

**Acceptance Scenarios**:

1. **Given** a registered local source that resolves outside the repository, **When** lint evaluates freshness, **Then** it rejects content checkpointing for that source before reading bytes and reports verifiable containment evidence.
2. **Given** a registered URL source, **When** lint runs without a separately authorized fetch workflow, **Then** it does not fetch the URL and continues to use available structural dates for that source.
3. **Given** a file that changes while its fingerprint is being established, **When** lint cannot prove one stable observed state, **Then** it reports the checkpoint as unknown and preserves the prior checkpoint entry.
4. **Given** a failed repair, report write, or checkpoint write, **When** lint stops, **Then** all mechanical changes, the prior lint report, and the prior checkpoint artifact are restored together.

### Edge Cases

- A missing checkpoint artifact means no baseline exists; a successful run creates one without inventing a prior content-change finding.
- An existing checkpoint artifact is malformed, duplicated, unsupported, or names a path outside the repository; lint reports it as invalid and does not silently rebuild or trust it.
- A checkpointed file was deleted or renamed; lint reports the structural condition and preserves enough prior identity evidence for a maintainer to distinguish deletion from a new file.
- Two files have the same content but different normalized paths; their checkpoint identities remain distinct.
- A repository-local directory source contains reordered entries, ignored files, generated files, binary files, symlinks, or inaccessible files; its aggregate checkpoint uses stable path ordering and the existing source inclusion rules, and reports exclusions.
- A file exceeds the configured checkpoint size or the run exceeds its file or byte budget; lint reports an unknown checkpoint for the skipped item and does not substitute partial content as a complete fingerprint.
- The fingerprint algorithm or checkpoint schema changes; existing records remain labeled with their original version and are not compared as if they used the new version.
- A hash collision occurs; the checkpoint is treated only as a practical change signal and never as proof of authenticity, trust, or security.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The stale check MUST compare both available timestamp evidence and a durable content fingerprint for checkpointable artifacts within the selected lint scope.
- **FR-002**: Checkpointable artifacts MUST include indexed wiki pages examined by the selected lint scope and registered repository-local file or directory sources relevant to that scope.
- **FR-003**: Lint MUST NOT fetch URL sources, read outside-repository sources, or broaden its scope merely to obtain a content fingerprint; those sources retain timestamp-only or unknown freshness evidence.
- **FR-004**: Each checkpoint entry MUST identify the normalized repository-relative artifact, artifact kind, fingerprint algorithm and version, last observed timestamp evidence, content fingerprint, and successful checkpoint date.
- **FR-005**: The default fingerprint MUST be deterministic for identical content and MUST change with overwhelming practical reliability when checkpointed content changes; no authentication or security property may depend on it.
- **FR-006**: File fingerprints MUST represent the complete file state evaluated by lint, not a partial sample or normalized semantic interpretation.
- **FR-007**: Directory fingerprints MUST deterministically represent included child paths and their complete file fingerprints using stable path ordering and the existing directory-source inclusion and exclusion rules.
- **FR-008**: A changed fingerprint MUST be treated as content-change evidence even when timestamps are unchanged.
- **FR-009**: A changed timestamp with an unchanged fingerprint MUST NOT by itself be reported as a content change, while existing age-threshold rules may still produce their independently supported findings.
- **FR-010**: A page fingerprint change without a corresponding structural updated-date advance MUST produce a semantic stale finding and MUST NOT authorize an automatic page rewrite.
- **FR-011**: A registered local source fingerprint change since the last successful checkpoint MUST produce a semantic stale finding that identifies the source and recommends the existing refresh or re-ingestion workflow.
- **FR-012**: The absence of a prior checkpoint MUST be reported as baseline establishment, not as evidence that content changed before the first checkpoint.
- **FR-013**: Scoped lint MUST read and update only checkpoint entries needed by the selected page, neighborhood, or named check and MUST preserve out-of-scope checkpoint entries byte-for-byte.
- **FR-014**: Lint MUST enforce declared per-run file-count, byte, and individual-file limits before checkpointing and MUST report skipped or incomplete checkpoint coverage as unknown rather than current.
- **FR-015**: Lint MUST treat checkpoint content, paths, algorithm labels, timestamps, and all checkpointed file bytes as untrusted data that cannot alter checks, invoke tools, or authorize more access.
- **FR-016**: Malformed, duplicated, unsupported, escaping, or ambiguous checkpoint entries MUST be reported and MUST NOT be used as comparison or repair authority.
- **FR-017**: Lint MUST confirm that a file remained stable for the observed read; when it cannot, it MUST report unknown status and preserve the prior checkpoint entry.
- **FR-018**: The checkpoint artifact MUST be repository-native, deterministically ordered, reviewable, and owned by lint as derived state separate from source records and semantic page content.
- **FR-019**: Checkpoint updates MUST be prepared after all checks and allowed fixes determine the final evaluated bytes, then validated and committed atomically with the mechanical fix set and lint report.
- **FR-020**: Any failed fix, report write, checkpoint validation, or checkpoint write MUST restore mechanical changes, the prior lint report, and the prior checkpoint artifact to their pre-run state.
- **FR-021**: Lint MUST never modify an original source, synthesize a source fingerprint from unavailable content, silently migrate an unsupported checkpoint, or present a content fingerprint as proof of trust or authenticity.
- **FR-022**: The lint response MUST summarize checkpoint coverage, baselines created, content changes found, unknown or skipped artifacts, and exactly one next action selected under the existing finding-priority rules.

### Key Entities

- **Checkpoint Artifact**: The repository-contained derived lint state holding deterministic checkpoint entries from the last successful applicable run.
- **Checkpoint Entry**: One normalized artifact identity with kind, algorithm/version, timestamp evidence, content fingerprint, and successful checkpoint date.
- **File State**: The exact bytes and available timestamp evidence observed as one stable unit during lint.
- **Directory State**: The deterministic aggregate of included repository-relative child paths and their complete file fingerprints.
- **Checkpoint Coverage**: Examined, baseline-established, unchanged, changed, unknown, or skipped status for an artifact in the current lint scope.
- **Content-Change Finding**: A semantic stale finding supported by a mismatch between a valid prior checkpoint and the current stable file state.

### Scope Boundaries

- This feature extends the existing `stale` lint check and does not add a new check category.
- Content checkpointing covers wiki pages examined by lint and registered repository-local file or directory sources relevant to the selected scope.
- Remote URL fetching, outside-repository sources, source registration, source refresh, semantic page edits, conflict resolution, and authenticity verification remain outside lint.
- Existing timestamp, age-threshold, citation, contradiction, link, orphan, and index-drift behavior remains in force unless content evidence makes a timestamp-only content-change conclusion more precise.
- The checkpoint artifact is new derived lint state; it does not become source provenance and cannot replace citations or the source registry.

### Dependencies

- This feature flows forward from the existing Wiki Health Checks specification and its six check categories, mutation boundary, deterministic report, and finding priority.
- It depends on existing wiki containment, source normalization, directory inclusion, lint scoping, and atomic repair behavior.
- The Refresh Wiki Source feature remains the recommended user workflow when a registered source fingerprint changes.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every fixture whose bytes change while timestamp evidence is preserved produces exactly one content-change finding for the affected checkpointed artifact.
- **SC-002**: Every fixture whose timestamp changes while bytes remain identical produces zero false content-change findings.
- **SC-003**: Repeating lint with identical scope and file states produces byte-for-byte identical checkpoint entries and finding order.
- **SC-004**: First-run fixtures create baselines for 100% of eligible in-budget artifacts and report zero historical content changes solely because no prior checkpoint existed.
- **SC-005**: Scoped-run fixtures leave 100% of out-of-scope checkpoint entries byte-for-byte unchanged.
- **SC-006**: Escaping, remote, oversized, unstable-read, malformed, and unsupported fixtures produce explicit unknown or invalid results with zero unauthorized reads or writes.
- **SC-007**: Every failed-write fixture restores the complete pre-run mechanical state, lint report, and checkpoint artifact.
- **SC-008**: A maintainer can identify changed, unchanged, baseline, unknown, and skipped checkpoint coverage plus the recommended next action from the lint response in under two minutes.

## Assumptions

- SHA-256 is the default content fingerprint because it is widely available and collision-resistant enough for practical change detection; the feature does not rely on its cryptographic security.
- “Modification timestamps” includes filesystem modification times for repository-local files and the existing structural `updated` and `Last ingested` dates where applicable.
- URL source contents are not fetched by lint; URL freshness continues to rely on existing structural evidence until an explicitly authorized workflow refreshes the source.
- A directory source uses one aggregate checkpoint while retaining enough per-child evidence to compute the aggregate deterministically and report exclusions.
- Checkpoint limits will be declared in configuration with conservative defaults during planning so repositories can bound file count, bytes read, and individual file size.
