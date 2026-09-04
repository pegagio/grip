# Feature Specification: Mapping Registry and Ownership Validation

**Feature Branch**: `002-mapping-registry-ownership`

**Created**: 2026-09-03

**Status**: Complete

**Input**: User description: "Specify roadmap entry 002, Mapping Registry and Ownership Validation, using the roadmap and wiki-backed governing decisions and constraints."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Declare Mapping Intent Safely (Priority: P1)

As a local user, I can add an exact file mapping or a tree-root mapping so Grip records which source and destination paths I intend it to manage without copying, deleting, or otherwise changing payloads.

**Why this priority**: Explicit mapping intent is the ownership boundary required by every later discovery and synchronization feature.

**Independent Test**: In an isolated Grip home with isolated source and destination paths, add one file mapping and one tree mapping, inspect the resulting registry, and verify that the complete tuples and prior-registry recovery evidence are retained while all payload paths and synchronization state remain unchanged.

**Acceptance Scenarios**:

1. **Given** an unowned regular-file source and a valid destination path, **When** the user runs `grip mapping add file SOURCE DESTINATION`, **Then** Grip records one file mapping identified by the canonical source path, retains the prior accepted registry for recovery, and changes no payload or synchronization state.
2. **Given** an unowned source directory and a valid destination directory root, **When** the user runs `grip mapping add tree SOURCE DESTINATION`, **Then** Grip records one tree mapping identified by the canonical source root and performs no recursive discovery or payload mutation.
3. **Given** a mapping add request, **When** the source, destination, kind, or topology is invalid, **Then** Grip reports the complete reason and leaves the registry and all payload paths unchanged.

---

### User Story 2 - Inspect Mapping Ownership (Priority: P2)

As a user or automation author, I can list all mappings or inspect the mapping selected by a source path so I can determine exactly what Grip has been asked to own.

**Why this priority**: Ownership must be visible and deterministic before any later feature discovers members or proposes mutation.

**Independent Test**: Populate an isolated registry with file and tree mappings, invoke list and show operations in human and machine-readable modes, and verify stable ordering, complete mapping tuples, and no filesystem mutation.

**Acceptance Scenarios**:

1. **Given** a valid registry with multiple mappings, **When** the user runs `grip mapping list`, **Then** Grip reports every mapping in canonical source-path order with its kind, source, and destination.
2. **Given** a canonical source path identifying an existing mapping, **When** the user runs `grip mapping show SOURCE`, **Then** Grip reports exactly that mapping in the selected output mode.
3. **Given** a path that is not the canonical source identity of a mapping, **When** the user runs `grip mapping show PATH`, **Then** Grip reports that no mapping matches and does not infer identity from the destination or a tree member.

---

### User Story 3 - Remove Intent Without Removing Data (Priority: P3)

As a local user, I can remove a mapping by its canonical source path so Grip relinquishes ownership intent without deleting, moving, copying, or reconciling either side.

**Why this priority**: Users need a reversible registry lifecycle whose removal semantics cannot be mistaken for payload deletion.

**Independent Test**: Remove an existing mapping from an isolated registry and verify that only the registry changes, both paired paths retain their original content and metadata, and removing a missing mapping is a reported non-success rather than an implicit no-op.

**Acceptance Scenarios**:

1. **Given** an existing mapping and unchanged registry evidence, **When** the user runs `grip mapping remove SOURCE`, **Then** Grip retains the prior accepted registry for recovery, atomically removes only that mapping, and leaves source, destination, and synchronization state unchanged.
2. **Given** no mapping with the supplied canonical source identity, **When** removal is requested, **Then** Grip reports that no mapping matches and changes nothing.
3. **Given** the registry changes after Grip reads it but before publication, **When** an add or remove would otherwise publish, **Then** Grip rejects the stale update without overwriting the newer registry.

### Edge Cases

- A source or destination path is relative, empty, is not valid UTF-8, contains unresolved parent traversal, passes through symbolic-link ancestry, or cannot be represented canonically.
- A source path does not exist, is itself a symbolic link, or has a node kind inconsistent with the requested mapping kind.
- A destination exists with a kind inconsistent with the requested mapping; a destination parent is absent; or the destination itself does not yet exist.
- Source and destination resolve to the same path, one contains the other, or their relationship would make a tree mapping encounter its own destination during future traversal.
- A new source or destination overlaps an existing file mapping, tree root, or potential tree-member namespace.
- Two textual path spellings resolve to the same canonical source identity.
- The registry contains a duplicate, overlap, unknown field, unsupported schema version, or invalid pre-existing mapping unrelated to the selected command.
- Registry publication is interrupted, contended, or observes stale evidence.
- A path begins with a hyphen or contains spaces or non-ASCII characters.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST support exactly two mapping kinds in this feature: `file`, pairing one exact source file path with one exact destination file path; and `tree`, pairing one source directory root with one destination directory root.
- **FR-002**: Grip MUST expose `grip mapping add file SOURCE DESTINATION`, `grip mapping add tree SOURCE DESTINATION`, `grip mapping list`, `grip mapping show SOURCE`, and `grip mapping remove SOURCE` as the mapping lifecycle commands.
- **FR-003**: Adding a mapping MUST record intent only. It MUST NOT discover tree members, create a baseline, create machine-owned synchronization state, create missing payload paths, or copy, replace, move, or delete payload content. It MAY create or retain one owner-only registry-publication lock and prior-registry recovery evidence that contain no synchronization data.
- **FR-004**: The canonical absolute source path MUST be the mapping's sole user-facing identity. Grip MUST NOT create or require a separate mapping ID, and destination paths or prospective tree members MUST NOT identify a mapping for lifecycle commands.
- **FR-005**: Each stored mapping MUST retain its kind, canonical absolute source path, and canonical absolute destination path as one complete tuple.
- **FR-006**: A mapping source MUST exist when added. A file mapping source MUST be an ordinary regular file, and a tree mapping source MUST be an ordinary directory. Symbolic links and unsupported source node kinds MUST be rejected without following or opening their targets.
- **FR-007**: A mapping destination MAY be absent when added if its nearest existing ancestor is an ordinary accessible directory and canonical destination identity can be determined without following a symbolic link. An existing destination MUST be an ordinary regular file for a file mapping or an ordinary directory for a tree mapping.
- **FR-008**: Source and destination paths MUST be valid UTF-8, absolute, normalized, and resolved against existing ancestors into stable canonical identities. Intermediate symbolic-link ancestry MAY be resolved during canonicalization, but an existing final source or destination node that is itself a symbolic link MUST be rejected. Empty paths, non-UTF-8 paths, relative paths, unresolved traversal, inaccessible ancestry, ancestry substitution detected during revalidation, and canonical resolution outside the submitted path's resolved ancestry MUST be rejected.
- **FR-009**: Before every add, show, list, or remove operation, Grip MUST validate the complete registry, including mappings outside the selected command's target. An invalid existing mapping MUST block the operation with no registry or payload change.
- **FR-010**: Grip MUST reject duplicate canonical source identities and duplicate source-destination tuples.
- **FR-011**: Grip MUST reject any pair of mappings whose source namespaces overlap, whose destination namespaces overlap, or whose source and destination namespaces create ambiguous ownership of an exact path or prospective tree member.
- **FR-012**: Grip MUST reject a mapping when source and destination are equal; when either tree root contains the other; when a destination is within any source tree; when a source is within any destination tree; or when transitive relationships among mappings could cause future discovery to encounter a managed destination as source input.
- **FR-013**: Topology validation MUST evaluate the complete proposed registry rather than only the newly added tuple, and MUST report the conflicting canonical paths and relationship without exposing unrelated file content.
- **FR-014**: `mapping list` MUST order results by canonical source path and MUST include kind, source, and destination for every mapping. `mapping show` MUST return exactly one mapping selected by canonical source identity or a distinct not-found result.
- **FR-015**: `mapping remove` MUST remove only registry intent. It MUST NOT delete, copy, move, compare, or reconcile either payload path; alter synchronization state; or imply retirement of future managed members. It MAY retain the prior accepted registry as recovery evidence.
- **FR-016**: Registry updates MUST use a complete validated candidate document and an atomic publication boundary on supported filesystems. Grip MUST revalidate the previously read registry and path evidence immediately before publication and reject stale or contended updates without overwriting accepted intent.
- **FR-017**: A failed or interrupted registry update MUST leave the previously accepted registry intact and MUST NOT be reported as successful.
- **FR-018**: Mapping lifecycle results MUST support the established human and JSON output modes, keep diagnostics on the diagnostic channel, and preserve the established meanings of all Feature 001 result fields and exit codes.
- **FR-019**: Mapping validation failures MUST use stable machine-readable detail fields sufficient to identify the operation, mapping kind when supplied, relevant canonical path or paths when safely available, and failure reason without requiring automation to parse prose.
- **FR-020**: All feature tests MUST use isolated temporary Grip homes and payload roots. They MUST verify registry contents and zero unintended payload or machine-state mutation for success, validation failure, contention, and interrupted-publication scenarios.
- **FR-021**: This feature MUST NOT add recursive source discovery, `.gripignore` evaluation, initial synchronization, baseline capture, payload copying, deletion, remapping, multiple selectors, destination-selected lifecycle operations, or recovery commands.
- **FR-022**: Mapping parsing, topology and ownership validation, registry updates, and result presentation MUST remain behaviorally separable so domain outcomes do not depend on human-facing text.
- **FR-023**: A registry update MUST preserve every supported configuration field and value. The published document MAY normalize whitespace, comments, and ordering that do not carry configuration meaning, and repeated equivalent operations MUST produce the same canonical representation.
- **FR-024**: Before replacing an accepted registry, Grip MUST retain and verify its exact bytes in an immutable recovery generation identified by the registry content digest. An existing byte-identical generation MAY be reused; a conflicting or unverifiable recovery generation MUST block publication. Restoration and cleanup commands remain outside this feature.
- **FR-025**: An accepted `config.toml` MUST be a current-user-owned, non-symlink regular file. Read-only operations MAY inspect a readable registry whose group/other write bits are unset; add and remove additionally require the owner-write bit. Publication MUST preserve the accepted file's exact permission mode and MUST NOT silently change ownership or widen access.

### Key Entities

- **Mapping**: A complete user-authored ownership-intent tuple consisting of kind, canonical source path, and canonical destination path.
- **File Mapping**: A mapping that owns one exact prospective source-destination file pair and no surrounding namespace.
- **Tree Mapping**: A mapping whose roots define a prospective source-relative namespace; member discovery is deferred to Feature 003.
- **Canonical Source Identity**: The normalized absolute source path by which users select one mapping and by which Grip detects duplicate identity.
- **Registry Candidate**: The complete proposed registry after applying one add or remove operation and before atomic publication.
- **Ownership Conflict**: A duplicate, overlap, containment, or recursive relationship that makes exact or prospective managed ownership ambiguous or unsafe.
- **Registry Recovery Generation**: An immutable, verified copy of the exact previously accepted `config.toml` bytes stored by content digest before replacement.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In acceptance tests, users can add valid file and tree mapping intent and then retrieve the exact kind, canonical source, and canonical destination in both output modes in 100% of cases.
- **SC-002**: A conformance suite covering equal paths, source/destination containment in both directions, duplicate identities, source overlap, destination overlap, cross-mapping recursion, symbolic links, traversal, and incompatible node kinds rejects every unsafe topology before registry publication.
- **SC-003**: Every successful or rejected add and remove test produces zero payload-content, payload-metadata, synchronization-state, baseline, and version-control changes; the only permitted non-registry artifacts are the owner-only registry-publication lock and verified prior-registry recovery generations.
- **SC-004**: Successful replacement tests retain a verified exact recovery generation before publication, while interrupted, stale, contended, or recovery-failure tests preserve the prior accepted registry in 100% of cases and never report the rejected candidate as accepted.
- **SC-005**: For registries containing up to 1,000 mappings, list output is deterministic across 100 repeated runs and complete ownership validation finishes within one second on a representative local workstation in at least 95 runs.
- **SC-006**: A user following the documented workflow can add, inspect, and remove one mapping on the first attempt in under two minutes without needing to understand machine-owned state.

## Assumptions

- Feature 001 is accepted and verified at commit `c144da9`; its Grip-home, registry schema, result-envelope, exit-code, diagnostic, owner-only-permission, and atomic state-publication contracts remain authoritative.
- The roadmap's `in-progress` state is the canonical lifecycle equivalent of the requested “implementing” state.
- Tracking records intent only. This resolves roadmap question Q-04 conservatively and keeps all payload mutation in later features where plans, baselines, recovery, and dry-run semantics exist.
- Exact lifecycle syntax is resolved by the `mapping add`, `mapping list`, `mapping show`, and `mapping remove` command family in FR-002. This resolves Q-03 without introducing shorthand aliases.
- An absent destination is permitted because mapping intent may precede creation of the paired target, but the existing ancestor chain must be safe and canonicalizable.
- Recursive topology is rejected across both source and destination namespaces, including transitive cross-mapping relationships. This resolves Q-09 in favor of a conservative ownership boundary.
- Registry lifecycle commands publish a deterministic canonical TOML document. Comments and presentation-only ordering are not retained as configuration data; this avoids adding a format-preserving editor dependency before a demonstrated need exists.
- Registry recovery is bounded to immutable content-addressed prior documents. This feature creates and verifies recovery evidence but does not add restoration, listing, retention-policy, or cleanup commands.
- Filesystem case sensitivity and Unicode-equivalence behavior remain governed by the host filesystem's canonical path resolution until Feature 009 defines the final portability contract.
