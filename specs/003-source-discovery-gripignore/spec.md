# Feature Specification: Source Discovery and Gripignore

**Feature Branch**: `003-source-discovery-gripignore`

**Created**: 2026-09-04

**Status**: Complete

**Input**: User description: "Specify roadmap entry 003, Source Discovery and Gripignore, using the roadmap and wiki-backed governing decisions and constraints."

## Clarifications

### Session 2026-09-04

- Q: What should the read-only command for inspecting entries within existing mappings be called? → A: `grip mapping inspect [SOURCE]`
- Q: When inspection completes successfully but finds unsupported or unsafe entries, should the command still exit successfully? → A: Exit `0` with blocking findings in the complete inventory.

### Session 2026-09-05

- Q: If a `.gripignore` file cannot be read or parsed, should Grip fail the entire inspection instead of returning a partial inventory? → A: Fail the entire inspection with no inventory.
- Q: How should Grip handle a source entry whose relative path contains bytes that are not valid UTF-8? → A: Report a blocking unsupported entry with lossless raw-byte identity.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect the Managed Namespace (Priority: P1)

As a local user, I can inspect a tree mapping and receive a deterministic inventory of the source entries eligible for management so I know exactly what Grip recognizes before any baseline or synchronization exists.

**Why this priority**: Source-defined membership is the foundation for every later status and synchronization operation, and it must be observable before Grip is allowed to mutate payloads.

**Independent Test**: In isolated source and destination trees, request discovery for a valid tree mapping and verify that every eligible regular file and directory appears once under its source-relative identity, including empty directories, while no registry, state, baseline, or payload path changes.

**Acceptance Scenarios**:

1. **Given** a valid tree mapping containing nested ordinary files and directories, **When** the user runs `grip mapping inspect [SOURCE]`, **Then** Grip reports every eligible entry once in deterministic source-relative order with its source and paired destination path.
2. **Given** an eligible empty source directory, **When** the user requests discovery, **Then** Grip includes the directory as an eligible managed entry without claiming that its metadata has been compared or accepted.
3. **Given** a valid file mapping alongside tree mappings, **When** discovery covers all mappings, **Then** Grip reports the exact file-mapping entry without recursively traversing either of its parent directories.
4. **Given** any successful or failed discovery request, **When** the operation finishes, **Then** the registry, machine-owned state, baselines, source, and destination remain byte-for-byte and metadata unchanged.

---

### User Story 2 - Apply Predictable Gripignore Policy (Priority: P2)

As a user, I can place `.gripignore` policy at a tree root or in nested source directories so excluded content stays outside the managed namespace and later, more specific rules behave like familiar Gitignore rules.

**Why this priority**: Tree mappings are usable only when users can exclude caches, generated content, and other source entries without maintaining a separate per-file manifest.

**Independent Test**: Build an isolated source tree with root and nested `.gripignore` files covering comments, escaping, directory-only patterns, anchored patterns, wildcards, recursive wildcards, negation, and excluded-parent behavior, then compare discovery with a fixed expected inventory.

**Acceptance Scenarios**:

1. **Given** root and nested `.gripignore` files with overlapping patterns, **When** an entry is discovered, **Then** Grip evaluates applicable rules from the mapping root toward the entry and applies the last matching rule.
2. **Given** a negated pattern for an entry whose parent remains traversable, **When** discovery runs, **Then** the entry is re-included; if an ancestor directory itself is excluded, descendant rules do not re-include entries beneath that untraversed ancestor.
3. **Given** `.gitignore`, `.ignore`, global ignore settings, hidden entries, or destination-side `.gripignore` files, **When** discovery runs, **Then** none of them changes source membership.
4. **Given** a source-side `.gripignore` file, **When** discovery runs, **Then** Grip uses it as policy and never reports it as payload, even if another rule would otherwise re-include it.

---

### User Story 3 - See Unmanaged and Unsupported Boundaries (Priority: P3)

As a user, I can distinguish destination-only content from unsafe source or destination collisions so unrelated destination entries remain visibly unmanaged and unsupported filesystem nodes cannot be mistaken for safe managed payloads.

**Why this priority**: A trustworthy overlay tool must explain both what it owns and what it deliberately refuses to own without following or opening risky filesystem nodes.

**Independent Test**: Populate isolated trees with destination-only entries, symbolic links, hard-linked files, sparse files, special nodes available on the test platform, and a nested mount fixture where supported, then verify exact non-following classifications and zero mutation.

**Acceptance Scenarios**:

1. **Given** a destination entry whose relative path has no eligible source entry and no previously managed evidence, **When** discovery runs, **Then** Grip reports it as destination-only and unmanaged without applying source ignore rules to it.
2. **Given** a symbolic link or other unsupported node at a non-ignored source path, **When** discovery reaches it, **Then** Grip reports the exact path and detected reason as blocking evidence without following, opening, hashing, or traversing the node.
3. **Given** an unsupported destination node at the paired path of an eligible source entry, **When** discovery runs, **Then** Grip reports an unsafe managed-path collision; the same node at a destination-only path remains unmanaged and non-blocking.
4. **Given** a nested filesystem boundary beneath a source tree root, **When** discovery reaches that boundary, **Then** Grip reports it as unsupported and does not traverse into it.

### Edge Cases

- The registry is invalid, changes during inspection, or contains a mapping whose accepted canonical roots no longer resolve to the same filesystem evidence.
- A root or nested `.gripignore` is unreadable, changes while being evaluated, contains invalid text, or has a final line without a newline.
- Ignore patterns contain escaped leading `#` or `!`, trailing spaces, directory-only suffixes, root anchoring, character ranges, `**`, or a negation beneath an excluded parent.
- An ignored directory contains an unsupported node or another `.gripignore`; discovery must not inspect entries below a directory that policy excludes from traversal.
- A source entry has a non-UTF-8 relative path, multiple hard links, sparse allocation, or changes node kind while discovery is running.
- A directory is empty, becomes empty during discovery, or is replaced while its children are being enumerated.
- A destination-only directory contains a large or unsupported subtree; inspection must not turn unmanaged content into managed membership or follow unsafe nodes.
- File and tree mappings coexist, and deterministic ordering must remain stable across mapping and entry boundaries.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST expose the read-only discovery operation as `grip mapping inspect [SOURCE]`; omitting `SOURCE` MUST inspect all accepted mappings, while providing it MUST select one mapping by its canonical source identity.
- **FR-002**: Discovery MUST validate the complete accepted registry before inspecting payload paths and MUST reject discovery if any mapping is structurally invalid.
- **FR-003**: A file mapping MUST contribute exactly its mapped source entry when the source remains an ordinary regular file; discovery MUST NOT traverse either parent directory.
- **FR-004**: A tree mapping MUST derive its candidate managed namespace dynamically from its source on every discovery operation without requiring or publishing a per-entry manifest.
- **FR-005**: Every discovered tree entry MUST have a source-relative identity and a paired destination path formed from that same relative identity.
- **FR-006**: Discovery MUST include eligible ordinary regular files and ordinary directories, including empty directories. Directory content membership is in scope; comparison or synchronization of directory metadata is deferred.
- **FR-007**: Discovery results MUST distinguish eligible managed entries, ignored source paths, destination-only unmanaged paths, unsupported source entries, and unsafe nodes occupying the paired destination path of an eligible source entry. A complete inventory MUST exit `0` even when it contains blocking findings and MUST expose a structured blocking count.
- **FR-008**: Eligible managed entries and blocking findings MUST be returned in deterministic mapping and source-relative path order. Repeated discovery over unchanged evidence MUST produce equivalent human and machine-readable results.
- **FR-009**: A source-side `.gripignore` MUST apply to its containing directory and descendants. Patterns without an explicit broader scope MUST be interpreted relative to the directory containing that policy file.
- **FR-010**: Applicable `.gripignore` rules MUST be evaluated from the mapping root toward the entry, with the last matching rule determining inclusion or exclusion.
- **FR-011**: `.gripignore` matching MUST support Gitignore-compatible blank lines, comments, escaping, path separators, directory-only patterns, anchored patterns, `*`, `?`, character ranges, recursive `**` forms, and `!` negation.
- **FR-012**: A negated rule MAY re-include an entry only while its ancestors remain traversable. When a directory itself is excluded, discovery MUST NOT traverse it and descendant rules MUST NOT re-include entries beneath it.
- **FR-013**: Discovery MUST use only source-side `.gripignore` files as ignore authority. It MUST NOT consult `.gitignore`, `.ignore`, repository excludes, global ignore settings, destination-side ignore files, or implicit hidden-file filtering.
- **FR-014**: Every source-side `.gripignore` file MUST be treated as policy and excluded from managed payload in the initial product. No pattern or user option in this feature may opt it into synchronization.
- **FR-015**: A newly ignored path for which no accepted baseline exists MUST be reported as ignored and unmanaged. Retirement of a previously managed entry that later becomes ignored remains outside this feature and MUST NOT be inferred or executed.
- **FR-016**: Discovery MUST inspect filesystem nodes without following symbolic links. A non-ignored symbolic link MUST be classified as unsupported; Grip MUST NOT read, hash, open, copy, or traverse its target.
- **FR-017**: Non-ignored hard-linked files, sparse files, sockets, FIFOs, device nodes, whiteouts, unknown special nodes, nested mount boundaries, and source entries whose relative names are not valid UTF-8 MUST be reported by exact path and detected reason as unsupported. A non-UTF-8 source-relative name MUST be a blocking finding, MUST retain its exact raw-byte identity in machine output, and MUST NOT prevent the otherwise complete inventory from being returned. Ignored entries MUST be excluded before any operation that could open or hash their payload.
- **FR-018**: Discovery MUST remain on the filesystem containing each tree mapping's source root. It MUST NOT traverse a nested mount boundary.
- **FR-019**: A destination path with no eligible source-relative counterpart and no prior managed evidence MUST remain destination-only and unmanaged regardless of its node kind. Discovery MUST NOT follow or open unsupported destination-only nodes.
- **FR-020**: An unsupported node occupying the paired destination path of an eligible source entry MUST be reported as an unsafe collision without following, opening, hashing, copying, or altering that node.
- **FR-021**: If registry, root, directory, ignore-policy, or entry evidence changes in a way that could make the inventory inconsistent during discovery, Grip MUST return an explicit stale-evidence result rather than present the inventory as complete.
- **FR-022**: Discovery MUST preserve the established human and machine-readable result boundaries, provide stable structured categories and safe paths, and keep diagnostics separate from application results.
- **FR-023**: Discovery MUST NOT create or modify registry data, machine-owned state, baselines, locks, staging files, recovery evidence, source entries, destination entries, ignore files, or version-control state.
- **FR-024**: This feature MUST NOT compare content or metadata equality, classify synchronization drift, establish baselines, copy payloads, delete or retire entries, synchronize `.gripignore`, follow links, add multiple selectors, or introduce background traversal, caches, persistent indexes, or broad filesystem locks.
- **FR-025**: Filesystem behavior and ignore matching MUST be covered through isolated temporary-root conformance tests that never inspect or mutate the developer's real files or Grip home.
- **FR-026**: If any applicable `.gripignore` file cannot be read as a supported regular UTF-8 policy document or its rules cannot be parsed, Grip MUST fail the entire discovery request with a precise policy error. It MUST NOT return a partial inventory or treat the unavailable policy as empty.

### Key Entities

- **Discovery Request**: A read-only request covering all accepted mappings or one mapping selected by canonical source identity.
- **Discovered Entry**: An eligible ordinary file or directory identified by its mapping and source-relative path, with its paired destination path.
- **Gripignore Policy**: A source-side `.gripignore` document whose directory establishes the scope of its ordered inclusion and exclusion rules; the document itself is never managed payload in the initial product.
- **Ignored Path**: A source path excluded from candidate membership by the applicable Gripignore policy.
- **Destination-Only Entry**: A destination path with no eligible source-relative counterpart and no accepted managed evidence; it remains outside Grip ownership.
- **Unsupported Entry**: A non-ignored source node or nested boundary outside the allowlisted discovery contract.
- **Unsafe Destination Collision**: An unsupported node occupying the paired destination path of an eligible source entry.
- **Discovery Inventory**: The deterministic, non-persisted result containing eligible, ignored, unmanaged, unsupported, and unsafe-collision records for one validated inspection.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A conformance suite covering root and nested policy, comments, escaping, anchoring, wildcards, recursive wildcards, negation, excluded parents, directory-only rules, and precedence produces the expected managed inventory in 100% of cases.
- **SC-002**: Across acceptance fixtures containing eligible files, eligible empty directories, ignored paths, destination-only entries, symbolic links, hard links, sparse files, available special nodes, and nested filesystem boundaries, Grip classifies every path according to this specification without following or opening unsupported nodes.
- **SC-003**: Successful, stale, invalid-registry, unreadable-policy, unsupported-entry, and output-failure tests produce zero changes to registry data, machine-owned state, baselines, source and destination payloads, policy files, and version-control state.
- **SC-004**: For an unchanged representative tree containing 10,000 entries and nested ignore policy, 100 repeated discovery runs produce equivalent ordered results and at least 95 runs finish within two seconds on a documented representative local workstation.
- **SC-005**: A user can identify all eligible, ignored, destination-only, and blocking paths for a representative mapping from one discovery result without consulting internal state or manually enumerating the tree.

## Assumptions

- Feature 002 is verified at commit `9e346045df453959ed6389b14c8b72edc5221b11`; its canonical mapping identity, complete-registry validation, path-safety, registry, result-envelope, and non-mutation contracts remain authoritative.
- The roadmap's `in-progress` state records the user's explicit start of Feature 003; the checkout is detached, so no Git branch is created by this specification workflow.
- Gitignore compatibility means the behavior enumerated in FR-009 through FR-013 and verified by a fixed conformance corpus. Selecting a supporting library or a particular implementation reference belongs in planning and must not weaken that behavior.
- `.gripignore` is always source-side policy rather than payload in the initial product. An explicit opt-in to synchronize it would be a later behavior change, not an option hidden inside this feature.
- Eligible ordinary directories, including empty directories, enter the discovered namespace now; independent directory metadata equality and synchronization remain deferred to Features 004 and 009.
- Symbolic links are unsupported discovery entries in this feature and are never followed. Feature 009 may define a later explicit link-object contract through a new forward-flowing behavioral change.
- Destination-only classification in this feature expresses ownership membership only. It does not use baseline evidence or decide synchronization state, which remain Feature 004 responsibilities.
