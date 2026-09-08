# Feature Specification: Metadata and Filesystem Contract Completion

**Feature Branch**: `009-metadata-filesystem-contract`

**Created**: 2026-09-07

**Status**: Complete

**Input**: User description: "Specify roadmap entry 009, Metadata and Filesystem Contract Completion, using the roadmap and wiki-backed governing decisions and constraints."

## Clarifications

### Session 2026-09-07

- Q: Which operating-system and filesystem combinations must pass before Feature 009 can be accepted? → A: Current macOS on APFS only; other Unix platforms and filesystems are deferred.
- Q: How should Grip decide which macOS extended attributes belong to the synchronized metadata contract? → A: Synchronize a documented allowlist; report and block on unknown attributes.
- Q: When an older baseline lacks newly supported metadata and the two current copies differ in those fields, how should the user establish the expanded baseline? → A: Treat the mismatch as a metadata migration conflict and use the existing whole-entry source-wins or destination-wins resolution workflow.
- Q: How should Grip handle known macOS security and provenance attributes, such as download quarantine metadata? → A: Report explicitly excluded attributes but do not synchronize them, include them in equality, or block because of them.
- Q: Should modification time independently count as a synchronization change for both files and directories? → A: Modification time defines equality and independently triggers synchronization for both files and directories.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Synchronize the Promised Metadata Contract (Priority: P1)

As a local user, I can inspect, preview, and synchronize supported file and directory metadata with the same conflict, recovery, verification, and baseline guarantees already applied to file content.

**Why this priority**: Completing the metadata contract is the feature's primary user outcome and closes a known gap between byte synchronization and faithful filesystem-state synchronization.

**Independent Test**: In isolated roots, create regular files and directories that differ only in each supported metadata field; inspect, preview, and synchronize them in both directions; verify classification, exact plans, preservation, verification, and accepted baseline publication.

**Acceptance Scenarios**:

1. **Given** an accepted regular file whose content is unchanged but whose supported metadata changed only at the source, **When** the user inspects and pushes it, **Then** Grip reports a source-side metadata change, previews the exact transition, reproduces every supported field at the destination, verifies the result, and publishes the accepted metadata.
2. **Given** an accepted directory whose supported metadata changed only at the destination, **When** the user pulls it, **Then** Grip applies and verifies the directory metadata independently of its children after all child actions complete.
3. **Given** both sides changed the same supported metadata to the same final values, **When** the user inspects the entry, **Then** Grip reports convergence rather than a conflict.
4. **Given** both sides changed any supported content or metadata to different final states, **When** the user inspects or synchronizes the entry, **Then** Grip treats the complete entry as conflicted and does not combine fields from opposing sides.
5. **Given** a requested metadata transition is unauthorized or cannot be reproduced and verified, **When** Grip completes preflight, **Then** it reports the exact field and reason and blocks the complete mutating invocation before its first action.
6. **Given** an older baseline lacks newly supported metadata and the current copies differ in those fields, **When** the user inspects and explicitly resolves the metadata migration conflict, **Then** Grip preserves the losing complete entry, makes both sides equal to the selected complete source or destination state, verifies the result, and publishes an expanded baseline.

---

### User Story 2 - Understand Filesystem Compatibility Before Mutation (Priority: P2)

As a local user, I can see whether the source and destination filesystems can represent every required node and metadata transition before Grip changes any payload.

**Why this priority**: Cross-filesystem and permission differences are predictable sources of silent metadata loss unless they are made explicit before mutation.

**Independent Test**: Exercise mappings across documented capability combinations, including unavailable attributes, unauthorized ownership, protected metadata, different case behavior, and different normalization behavior; verify precise read-only reports and complete preflight blocking.

**Acceptance Scenarios**:

1. **Given** both endpoints can represent and reproduce all required supported fields, **When** the user previews an operation, **Then** Grip reports an eligible deterministic plan without claiming broader filesystem support.
2. **Given** the target cannot represent a required supported field, **When** the user inspects or previews the entry, **Then** Grip identifies the endpoint, field, required value, and unavailable capability without mutating or silently omitting the field.
3. **Given** the invoking user lacks authority to apply a required owner, group, access-control, attribute, or flag transition, **When** preflight runs, **Then** Grip blocks before mutation and does not invoke privilege escalation or substitute a different value.
4. **Given** a target filesystem treats two selected path components as equivalent by case or Unicode behavior, **When** those components would create ambiguous ownership or publication, **Then** Grip reports every colliding identity and blocks the scope before mutation.
5. **Given** a managed entry contains an extended attribute on Grip's documented security and provenance exclusion list, **When** Grip inspects or previews the entry, **Then** it reports the excluded attribute without including it in equality, synchronizing it, or blocking mutation.
6. **Given** a managed entry contains an extended attribute on neither the allowlist nor the exclusion list, **When** Grip inspects or previews the entry, **Then** it identifies the unknown attribute and blocks mutation rather than copying or silently discarding it.

---

### User Story 3 - Reject Unsupported Nodes Without Side Effects (Priority: P3)

As a local user, I receive precise, non-following reports for unsupported nodes and filesystem boundaries, while unrelated destination-only content remains unmanaged and untouched.

**Why this priority**: The final product contract must close edge cases without broadening Grip's ownership or exposing users to link, device, mount, or storage-representation surprises.

**Independent Test**: Place each unsupported node or boundary at a source member, managed destination, ignored source path, and destination-only unmanaged path; inspect and attempt mutation; verify classification, blocking scope, non-following behavior, and zero unintended access.

**Acceptance Scenarios**:

1. **Given** a symbolic link appears at a non-ignored source member or managed destination, **When** Grip inspects it, **Then** Grip reports the link itself as unsupported without following or opening its target and blocks mutation in the selected scope.
2. **Given** a hard-linked file, sparse file, socket, FIFO, device, whiteout, unknown node, or nested mount appears in the managed namespace, **When** Grip inspects or plans, **Then** it reports the exact detected condition and blocks mutation before the first action.
3. **Given** an unsupported node exists only at an unmanaged destination path, **When** Grip inspects or synchronizes the mapping, **Then** it reports or preserves the node as destination-only unmanaged content and never opens, changes, or removes it.
4. **Given** an unsupported source entry is excluded by valid source policy before it enters managed membership, **When** Grip inspects the mapping, **Then** it does not treat the excluded payload as a managed blocker.

---

### User Story 4 - Qualify the Complete Initial Product (Priority: P4)

As a maintainer, I can verify the complete initial Grip behavior on current macOS with APFS and representative workloads without introducing complexity unsupported by evidence.

**Why this priority**: Feature 009 is the closing roadmap slice, so its value includes proving that the integrated product meets its correctness, safety, observability, and responsiveness promises.

**Independent Test**: Run the documented end-to-end acceptance matrix on current macOS with APFS and representative local trees; verify every promised classification and mutation path plus published performance results.

**Acceptance Scenarios**:

1. **Given** current macOS on APFS, **When** the complete acceptance suite runs, **Then** every supported field and node behavior is exercised in both inspection and applicable mutation directions using isolated roots.
2. **Given** representative mapped trees, **When** status, preview, and synchronization qualification runs, **Then** the documented percentile thresholds are met or the feature reports the measured shortfall without adding unproven caches, broad locks, watchers, or persistent indexes.
3. **Given** human and machine-readable output for the same result, **When** paired contract tests compare them, **Then** both communicate equivalent classifications, field-level differences, capability blockers, action outcomes, verification, recovery, and accepted-state authority while diagnostics remain separate.

### Edge Cases

- A metadata field is readable on one endpoint but unreadable, absent, protected, or unsupported on the other.
- Numeric owner or group identity exists on one endpoint but cannot be assigned by the invoking user on the other.
- Extended attributes include empty values, large values, resource-fork data, protected names, or attributes that change during inspection.
- Access-control entries coexist with permission mode changes, expose permissions or flags in different enumeration order within the same entry, or change semantic access-control entry sequence.
- BSD flags include both user-changeable and privileged values, including flags that prevent a later content or metadata replacement.
- A directory's modification time changes as a consequence of applying child actions before its own metadata is finalized.
- A path is distinct on one filesystem but aliases another selected path by case folding or Unicode normalization on the target filesystem.
- An entry changes node kind, metadata, link count, sparse allocation, filesystem identity, or target ancestry after inspection but before mutation.
- A regular file becomes multiply hard-linked or sparse after staging or recovery preservation.
- A mount appears beneath a selected tree after discovery begins, or a mapping root itself resides on a different filesystem from its peer.
- Metadata publication succeeds but final verification, directory durability, operation recording, or accepted-state publication fails.
- An older baseline lacks fields introduced by this feature.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Grip MUST define the final initial-release equality contract for ordinary regular files as node kind, byte content, full Unix permission mode, numeric user identity, numeric group identity, modification time at the platform's reproducible precision, every extended attribute in the documented macOS allowlist, the effective extended access-control list when present, and supported user-changeable BSD flags when exposed by the platform.
- **FR-002**: Grip MUST define the final initial-release equality contract for ordinary directories as node kind, full Unix permission mode, numeric user identity, numeric group identity, modification time at the platform's reproducible precision, every extended attribute in the documented macOS allowlist, the effective extended access-control list when present, and supported user-changeable BSD flags. Directory metadata MUST be represented independently from child membership and child state.
- **FR-003**: Modification time MUST be accepted synchronization data for both regular files and directories, participate independently in equality and conflict classification, trigger synchronization when it is the only changed supported field, and be restored and verified at the finest precision both endpoints can reproduce consistently. Access time, creation or birth time, change time, and other volatile or system-maintained timestamps MUST remain diagnostic or unsupported rather than equality-defining.
- **FR-004**: Owner and group identity MUST be represented and compared by the numeric identities reported by the filesystem. Names MAY be displayed as contextual labels but MUST NOT replace numeric identity or make equality depend on mutable account-directory resolution.
- **FR-005**: Grip MUST NOT elevate privileges, invoke `sudo`, silently substitute the invoking user's owner or group, or report synchronization when a required ownership transition cannot be applied and verified. An already-matching identity requires no assignment.
- **FR-006**: Grip MUST define and publish both an explicit allowlist of synchronized macOS extended-attribute names and an explicit exclusion list limited to known security, provenance, volatile, or system-maintained attributes. Allowlisted attributes MUST use exact names and byte values in comparison, preservation, application, and verification. Excluded attributes MUST be reported by name during inspection but MUST NOT participate in equality, be synchronized, or block solely because they are present. Any attribute on neither list MUST be reported as unknown and MUST block mutation for the selected scope; Grip MUST NOT copy or silently discard it. Resource-fork data MAY be included only through an explicitly allowlisted attribute and then follows the same preservation and verification contract.
- **FR-007**: Access-control comparison MUST use a deterministic canonical representation that preserves access-control entry sequence because macOS evaluates entries in order. Grip MUST canonicalize only unordered permission and inheritance-flag sets within each entry so irrelevant bit-enumeration order is not treated as a change. Permission mode and extended access-control state MUST both be reported when both contribute to the entry's effective contract.
- **FR-008**: BSD-flag support MUST be allowlist-based and limited to flags that an ordinary invoking user can inspect, reproduce, and verify on the documented platform. Required protected, privileged, unknown, or unchangeable flag transitions MUST block mutation with the exact flag and reason.
- **FR-009**: Every supported metadata field MUST participate consistently in snapshot evidence, accepted baselines, three-way classification, diff and status results, deterministic plans, revalidation, copying or application, recovery preservation, final verification, and accepted-state publication.
- **FR-010**: A difference in any supported field MUST be reported at field level, while conflict selection and resolution MUST continue to treat content plus all supported metadata as one complete entry. Grip MUST NOT merge content or metadata fields from opposing sides.
- **FR-011**: Metadata-only changes MUST be eligible for push, pull, sync, and explicit conflict resolution under the established direction, selection, dry-run, authorization, recovery, failure, and baseline rules.
- **FR-012**: Grip MUST apply child entry actions before finalizing a managed directory's independent metadata, with deeper directories finalized before their parents, so child mutation does not invalidate the intended final directory state.
- **FR-013**: Before mutation, Grip MUST determine whether each endpoint can inspect, represent, apply, and verify every required supported node and metadata transition in the complete selected scope. Unknown, unavailable, unauthorized, or lossy transitions MUST block the complete invocation before its first mutation.
- **FR-014**: Capability and authorization results MUST identify the endpoint, affected path, field or node property, required value or behavior, detected capability, and reason the transition is eligible or blocked. Grip MUST NOT claim universal fidelity from support observed on a different platform or filesystem.
- **FR-015**: Immediately before each action, Grip MUST revalidate every content, metadata, node, ancestry, filesystem, ownership, capability, and accepted-state observation whose change could alter the action or its safety. Detected drift MUST stop execution under the established partial-failure contract.
- **FR-016**: Recovery evidence for replacement or deletion MUST preserve the complete supported content and metadata state needed to reproduce and verify the prior entry. If any supported state cannot be preserved, verified, or bound to the action, mutation MUST NOT remove or replace the original entry.
- **FR-017**: The initial release MUST support ordinary regular files and ordinary directories only. Symbolic links MUST remain unsupported link objects and MUST never be followed implicitly, dereferenced as payload, recreated, or treated as their targets.
- **FR-018**: Grip MUST detect regular files with multiple hard links and sparse allocation and report them as unsupported without reading, copying, replacing, backing up, or recreating them as ordinary payload.
- **FR-019**: Grip MUST identify sockets, FIFOs, character devices, block devices, whiteouts, unknown special nodes, and nested mount boundaries through non-following inspection and MUST never open, hash, copy, back up, recreate, or follow them as payload.
- **FR-020**: A non-ignored unsupported source node, unsupported node at a managed destination, unsupported replacement of a previously managed entry, or nested mount inside a managed tree MUST block mutation in the selected scope. An ignored source node excluded before membership and a destination-only unsupported node outside managed ownership MUST remain untouched and nonblocking.
- **FR-021**: Path identity MUST preserve the exact component bytes supplied by the filesystem; Grip MUST NOT normalize Unicode, fold case, or silently rename components to manufacture compatibility.
- **FR-022**: Grip MUST detect when distinct managed identities in the selected registry or plan resolve to equivalent names on an endpoint because of case or Unicode behavior. Every such collision MUST be reported with all involved source identities and MUST block inspection acceptance and mutation for that mapping or selected scope.
- **FR-023**: Case and Unicode compatibility detection MUST use observed endpoint behavior or documented filesystem capability evidence rather than assumptions based only on operating-system name. Inconclusive behavior MUST be reported as unavailable capability rather than guessed.
- **FR-024**: Crossing filesystems between mapping peers MAY be supported only when both endpoints can satisfy the full requested logical content and metadata contract. Grip MUST preserve no promise about hard-link topology, sparse allocation, clones, compression, block layout, or other physical storage representation.
- **FR-025**: A mapping root located on a mounted filesystem MAY be used as its own explicit mapping root, but traversal beneath a tree mapping MUST remain on the source root's filesystem and MUST block at a nested mount boundary.
- **FR-026**: Baseline and machine-output schemas MUST distinguish absent, unavailable, unsupported, unreadable, unauthorized, and observed values so missing evidence cannot compare equal to a concrete value or disappear from automation output.
- **FR-027**: Existing accepted baselines that predate this final metadata schema MUST NOT be silently interpreted as containing newly required fields. Complete read-only compatibility inspection MUST classify equivalent current copies as eligible for the existing state-only `grip baseline accept` workflow and differing current copies as a metadata migration conflict. A metadata migration conflict MUST be eligible only for the existing whole-entry `grip resolve PATH --source|--destination` workflow with its explicit winner, losing-side recovery, revalidation, verification, and baseline-publication guarantees. Until one path completes, other mutation for that entry MUST remain blocked.
- **FR-028**: Dry runs MUST render the same complete ordered content, metadata, capability, recovery, and publication plan as an equivalent mutating invocation over unchanged evidence and MUST NOT mutate payloads, registry state, baselines, recovery evidence, or coordination state.
- **FR-029**: Human output and versioned machine-readable output MUST communicate equivalent node classifications, field-level differences, endpoint capabilities, blockers, planned and completed actions, drift, recovery evidence, verification, durability, and accepted-state authority. Diagnostic logs MUST remain separate.
- **FR-030**: The complete initial-product acceptance suite MUST cover every classification and mutation direction from Features 001 through 009, including conflict, deletion, retirement, recovery, partial failure, contention, unsupported nodes, metadata-only changes, capability mismatch, and baseline migration, using isolated temporary roots only.
- **FR-031**: Initial-release platform qualification MUST pass on current macOS with APFS and MUST document the exact macOS and APFS versions exercised, supported fields and flags, unavailable capabilities, precision limits, and any tests skipped because the environment cannot expose a required condition. Other Unix platforms and filesystems are outside Feature 009 acceptance.
- **FR-032**: Representative performance qualification MUST measure status, dry-run planning, and one synchronization workload over at least 10,000 managed entries containing both files and directories plus representative metadata. At least 95 of 100 status and dry-run invocations MUST complete within two seconds on a documented representative local workstation.
- **FR-033**: Caches, parallel traversal or execution, persistent indexes, background services, long-lived payload locks, inode-identity schemes, or snapshot-isolation machinery MUST NOT be introduced unless the representative measurements fail a success criterion and the plan documents the concrete failure, simpler alternatives, and correctness and maintenance costs.
- **FR-034**: Grip MUST NOT silently discard metadata, follow symbolic links, cross nested mounts, coerce unsupported nodes, prune destination-only content, elevate privileges, invoke version-control operations, or claim synchronized state before complete verification and accepted baseline publication.

### Key Entities

- **Supported Entry State**: One ordinary file or directory's complete equality-defining node, content, and metadata evidence, including explicit unavailable or unsupported observations.
- **Metadata Field Observation**: A field name, endpoint, observed value or bounded absence reason, precision or canonicalization rule, and evidence needed for revalidation.
- **Endpoint Capability Profile**: Operation-scoped evidence describing what one concrete filesystem endpoint can inspect, represent, apply, and verify for the selected entries; it is not a universal platform promise.
- **Compatibility Finding**: A precise eligible, unavailable, unsupported, unauthorized, lossy, or ambiguous result binding an endpoint capability to a required transition.
- **Filesystem Identity Collision**: Two or more distinct managed path identities that an endpoint would treat as the same name because of case or Unicode behavior.
- **Expanded Baseline**: The first accepted generation whose entry records contain the complete final metadata schema and explicit evidence states.
- **Platform Qualification Record**: Reproducible evidence naming the exercised operating system, filesystem, supported contract, capability gaps, correctness results, and representative performance measurements.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every supported metadata field is exercised independently for regular files and, where applicable, directories in both synchronization directions; 100% of successful fixtures reproduce and verify the exact supported state and publish the same accepted state.
- **SC-002**: In 100% of fixtures where one required field is unavailable, unauthorized, unsupported, or lossy, Grip identifies the exact field and endpoint and starts zero mutation actions.
- **SC-003**: In 100% of paired metadata-only classification fixtures, including file-only and directory-only modification-time changes, Grip distinguishes unchanged, one-sided, converged, and divergent states consistently with the complete-entry conflict contract.
- **SC-004**: In 100% of directory fixtures, child actions complete before final directory metadata, and the verified final directory state matches the selected authority including empty directories.
- **SC-005**: Every symbolic link, hard-linked file, sparse file, special node, and nested mount fixture is detected without following or opening the unsupported payload; managed occurrences block and unmanaged destination-only occurrences remain untouched.
- **SC-006**: Every documented case-folding or Unicode-equivalence collision fixture identifies all colliding managed identities before mutation, while non-colliding exact path identities remain unchanged rather than normalized or renamed.
- **SC-007**: Across injected drift and failure at every metadata preservation, application, verification, recovery, and state-publication phase, Grip reports completed, failed, and unattempted effects, retains recovery evidence, and never publishes a falsely accepted baseline.
- **SC-008**: In 100% of pre-feature baseline fixtures, equivalent current copies can publish an expanded baseline only through explicit state-only baseline acceptance, differing copies can publish one only through explicit whole-entry source-wins or destination-wins resolution, and unresolved fixtures remain mutation-blocked; none silently infer newly required metadata.
- **SC-009**: The complete end-to-end suite passes on current macOS with APFS, with the exact tested versions recorded and unsupported or unavailable environmental cases documented rather than omitted or represented as passes.
- **SC-010**: At least 95 of 100 status and dry-run invocations over the documented 10,000-entry representative tree complete within two seconds on the documented workstation, and any optimization beyond sequential inspection is tied to measured evidence.
- **SC-011**: Human and machine-readable paired tests communicate equivalent field differences, capabilities, blockers, actions, verification, recovery, durability, and accepted-state authority in 100% of result fixtures.
- **SC-012**: A user can determine from read-only results whether an entry is fully synchronized, precisely which supported fields differ, whether the target can reproduce them, and what corrective choice is available without inspecting Grip's internal state files.

## Assumptions

- Features 001 through 008 are verified, and their mapping ownership, discovery, baseline classification, selection, mutation, conflict, deletion, retirement, recovery, coordination, operation-record, result, and publication contracts remain authoritative.
- The final initial-release contract prioritizes faithful logical file and directory state over preserving physical storage representation. Hard-link topology, sparse allocation, clones, compression, and block layout remain outside the product promise.
- Symbolic links are rejected as payload in the initial release. Supporting links as value-preserving link objects would require a later feature with explicit identity, safety, conflict, recovery, and cross-platform semantics.
- Numeric user and group identities are stable equality values within the documented local Unix/macOS contract; friendly names are presentation-only. Grip does not solve cross-host identity translation because remote synchronization is outside scope.
- Modification time is synchronized metadata at mutually reproducible precision. Other system-maintained timestamps remain diagnostic because ordinary user processes cannot set or preserve them reliably.
- Directory metadata is independent state and is finalized after child actions to avoid child mutations invalidating the selected directory state.
- Endpoint capability evidence is scoped to the concrete operation and paths. It may be reused within that operation only while its revalidation assumptions remain true.
- Current macOS with APFS is the sole required initial-release qualification environment. Other Unix platforms and filesystems are deferred and MUST NOT be presented as qualified by Feature 009.
- The wiki query provides Partial coverage: it establishes Feature 009's eligibility, safety and ownership boundaries, existing mode/content equality, unsupported-node handling, proportional-complexity rule, and deferred question set, while leaving the final field, link, identity, time, directory, case, Unicode, and capability contracts for this specification.
- This checkout has no executable `before_specify` or `after_specify` hooks, so creating the feature artifacts does not create or switch a Git branch or dispatch another command.
