<!--
SYNC IMPACT REPORT
==================
Version change: 1.0.11 -> 1.0.12
Bump rationale: Record verified completion of Feature 006.

Changes this revision:
  - Transitioned Feature 006 from in-progress to verified
  - Recorded the clean post-implementation debrief as verification evidence

Specs affected: 006
Open questions added/resolved: none

Notes: Feature 006 is verified; Feature 007 now has verified dependencies 005 and 006.
-->

# Grip — Spec Roadmap

Living, non-binding map of the specs planned for Grip. It is **not a commitment to order or scope** — it captures the spec-specific discussion, decisions, technology choices, outcomes, and constraints surfaced during constitution and product-definition work so they are not lost before the spec that needs them is written. Specs are scoped and clarified when they are actually started. Foundations: the project [constitution](constitution.md) and [product definition](../../docs/product-definition.md).

Status legend (lifecycle): **undecided** · **needs-info** · **planned** · **specced** · **in-progress** · **implemented** · **verified** · **deferred** · **abandoned**.

---

## Vision & End States

The initial roadmap is complete when Grip delivers the full local product described by the product definition within the limits imposed by the constitution.

- A user can declare exact file mappings and source-defined tree mappings, inspect their managed namespace, and trust Grip to leave unrelated destination content untouched.
- Grip can classify current source and destination state against an accepted baseline, propagate unambiguous changes in either direction, and require an explicit complete-side choice for conflicts.
- Every mutation supports deterministic preview, bounded revalidation, staged replacement, verification, recovery evidence, and truthful baseline publication; deletion receives separate authorization.
- The resulting CLI is responsive for representative local trees, precise for humans and automation, tested against real filesystem behavior in isolated roots, and deliberately limited to its supported macOS and Unix contracts.

## Constraints & Decisions

These constraints apply across the ledger. Each is grounded in the active user-approved product direction, the [constitution](constitution.md), or [product definition](../../docs/product-definition.md).

- **C-01 — Local per-user boundary:** Grip is a local CLI operating on paths visible to one invoking user. Remote transport, daemons, privileged services, and multi-user coordination are outside the initial product.
- **C-02 — Explicit selective ownership:** A canonical source path identifies each mapping. File mappings own exact pairs; tree mappings own only eligible source-defined relative paths. Destination-only content outside that namespace remains unmanaged.
- **C-03 — Baseline-informed synchronization:** Grip compares current source, current destination, and the last accepted baseline. Source and destination define discovery and command direction, not a permanently authoritative editing side.
- **C-04 — Proportional rigor:** Designs must address demonstrated local failure modes with the simplest maintainable mechanism. Long-lived tree locks, watchers, persistent inode identity, kernel integration, and snapshot-isolation machinery require explicit evidence that simpler revalidation and recovery are insufficient.
- **C-05 — Validate, revalidate, recover:** Mutations require a complete deterministic plan, relevant pre-action revalidation, staged publication where supported, recovery preservation, verification, and baseline publication only after successful acceptance.
- **C-06 — Bounded concurrency:** External edits are handled through evidence capture, revalidation, and explicit drift errors. Only short-lived coordination around Grip-owned registry or state publication may be introduced without a separately justified governance exception.
- **C-07 — Explicit mutation semantics:** `push`, `pull`, and `sync` mutate by default; `-n` and `--dry-run` do not mutate. Deletion requires additional explicit authorization, and conflicts or known unsafe conditions block before the first mutation.
- **C-08 — Responsive by evidence:** Common inspection and planning workflows must remain responsive. Traversal, hashing, caching, parallelism, and indexing decisions are driven by representative measurements rather than speculative optimization.
- **C-09 — Allowlisted filesystem contract:** Ordinary regular files and directories form the initial payload boundary. Unsupported nodes or unreproducible metadata transitions are reported precisely and never silently coerced; macOS and Unix contracts may precede broader portability.
- **C-10 — Merge-bounded persistence:** Before merge, feature artifacts and implementation form one mutable, reviewable unit with accepted discoveries flowed back. Merge into the designated integration branch freezes the feature directory semantically, and later behavioral changes flow forward through new features.
- **C-11 — Separated interfaces:** CLI parsing and presentation remain separate from Grip-owned domain behavior. Human output, machine-readable output, and diagnostics are distinct interfaces, with deterministic forms where automation depends on them.

## Planned Specs

The following specifications form the approved path from a read-only foundation to the complete initial product. Every entry is planned; unresolved details are retained in its notes and in Open Questions for clarification when that specification begins.

### 001 — CLI, Configuration, and State Foundation  [status: verified]

- **Spec dir:** `specs/001-cli-state-foundation`
- **Description:** Establish the Rust application, command boundary, per-user Grip home, versioned configuration and state envelopes, and initial human and machine output contracts without changing mapped payloads.
- **Outcome:** A runnable, testable `grip` CLI can locate and validate its per-user root, parse a versioned empty or minimal configuration, report structured errors, and exercise its output and exit-code boundaries without mutating user files.
- **Scope (in):** Stable Rust and Rust 2024 project skeleton; thin CLI boundary; exact absolute `GRIP_HOME` override; default `~/.grip/`; separation of user-authored registry and machine-owned state; schema-version rejection; atomic publication primitives for Grip-owned state; initial human, machine, and diagnostic channels; conventional formatting, lint, test, and release-build checks.
- **Scope (out):** Mapping lifecycle, source traversal, baseline capture, payload copying, background services, remote access, and premature caching or parallel execution.
- **Depends on:** none
- **Governed by:** C-01, C-04, C-06, C-08, C-10, C-11
- **Addresses:** `docs/product-definition.md` — Command-Line Experience, Configuration and State, Implementation Direction
- **Notes:** The spec must settle Q-01, Q-02, and the foundation portion of Q-12. `clap`, Serde, `thiserror`, and `tracing` are candidates, not predetermined dependencies.

### 002 — Mapping Registry and Ownership Validation  [status: verified]

- **Spec dir:** `specs/002-mapping-registry-ownership`
- **Description:** Define file and tree mapping intent, canonical source-path identity, lifecycle commands, and complete ownership validation before discovery or mutation.
- **Outcome:** Users can add, inspect, and remove non-overlapping mapping intent, and Grip rejects ambiguous, escaping, equal, nested, or self-recursive topologies before they can own filesystem entries.
- **Scope (in):** File and tree mapping schemas; canonical path resolution; source-path identity without user-assigned IDs; complete tuple persistence; mapping selection; overlap and traversal rejection; atomic registry updates; non-destructive untracking; CLI contracts for mapping lifecycle.
- **Scope (out):** Recursive member discovery, `.gripignore`, initial payload synchronization, baseline creation, copying, deletion, and remapping existing mappings to new roots.
- **Depends on:** 001
- **Governed by:** C-02, C-04, C-05, C-06, C-10
- **Addresses:** `docs/product-definition.md` — Mapping Model, Tracking and Untracking, Configuration and State
- **Notes:** The spec resolves Q-03, Q-04, and Q-09. Tracking records intent only and does not cause an unreviewed bulk payload mutation. Verification evidence: `specs/002-mapping-registry-ownership/roadmap-reviews/debrief-20260904T171506Z.md` (`PROCEED`, no findings).

### 003 — Source Discovery and Gripignore  [status: verified]

- **Spec dir:** `specs/003-source-discovery-gripignore`
- **Description:** Discover the managed namespace of tree mappings from the source while applying root and nested `.gripignore` policy and preserving destination-only content as unmanaged.
- **Outcome:** Grip produces a deterministic, read-only inventory of eligible managed entries, ignored paths, destination-only paths, and blocking unsupported source entries without requiring a per-file manifest.
- **Scope (in):** Recursive source traversal; source-relative identity; root and nested `.gripignore`; Gitignore precedence, negation, and traversal behavior; explicit disabling of unrelated ignore sources; destination-only exclusion; non-following node inspection; nested mount detection; conformance and boundary tests.
- **Scope (out):** Payload mutation, baseline comparison, implicit synchronization of `.gripignore`, silent retirement of newly ignored managed entries, and following symbolic-link targets.
- **Depends on:** 002
- **Governed by:** C-02, C-04, C-08, C-09, C-10
- **Addresses:** `docs/product-definition.md` — Tree Discovery and Gripignore, Supported Node Boundary
- **Notes:** The specification resolves Q-05 with an enumerated Gitignore-compatible contract and policy-only `.gripignore`, includes empty directories for the discovery portion of Q-06 while deferring retirement and independent directory metadata, and treats symbolic links as unsupported non-followed nodes for the discovery portion of Q-07. Verification evidence: `specs/003-source-discovery-gripignore/roadmap-reviews/debrief-20260905T164650Z.md` (`PROCEED`, no findings).

### 004 — Baselines, Classification, and Status  [status: verified]

- **Spec dir:** `specs/004-baselines-classification-status`
- **Description:** Capture current state and an accepted baseline, implement the complete three-way entry-classification model, and expose deterministic read-only inspection commands.
- **Outcome:** `status`, `check`, and `diff` can distinguish synchronization, additions, initial collisions, unmanaged destination entries, one-sided drift, converged edits, divergent conflicts, deletions, unsafe collisions, and pending retirement without changing payloads.
- **Scope (in):** Fingerprints; first supported metadata set; versioned and integrity-checked baseline state; explicit classification types; deterministic planning model; one optional source-path selector; `--destination` interpretation; `--` option termination; human and machine results; automation-oriented exit behavior; corrupt or missing state detection.
- **Scope (out):** Copying, conflict resolution, deletion execution, automatic state recovery, multiple path selectors, and universal metadata fidelity.
- **Depends on:** 003
- **Governed by:** C-03, C-04, C-08, C-09, C-10, C-11
- **Addresses:** `docs/product-definition.md` — Synchronization Model, Content and Metadata, Command-Line Experience
- **Notes:** The spec must settle Q-08 and complete Q-12. Every source/destination/baseline classification requires behavioral coverage before mutation work begins. Verification evidence: `specs/004-baselines-classification-status/roadmap-reviews/debrief-20260906T133558Z.md` (`PROCEED`, no findings).

### 005 — Safe Push and Recovery  [status: verified]

- **Spec dir:** `specs/005-safe-push-recovery`
- **Description:** Apply eligible source-to-destination additions and changes under Grip's deterministic planning, revalidation, staging, backup, verification, and baseline-publication contract.
- **Outcome:** `push` mutates by default and `push --dry-run` or `push -n` previews the same plan without mutation; stale evidence, conflicts, unsupported entries, or failed verification cannot produce a falsely accepted baseline.
- **Scope (in):** Complete preflight; deterministic action ordering; relevant pre-action revalidation; short-lived state-publication coordination if required; destination-parent handling; same-filesystem staging where possible; replacement and recovery namespaces; verification; partial-failure reporting; successful baseline publication; representative performance measurements.
- **Scope (out):** Destination-to-source changes, bidirectional planning, deletion, automatic privilege escalation, broad tree locks, persistent inode identity, and guaranteed filesystem snapshot isolation.
- **Depends on:** 004
- **Governed by:** C-02, C-04, C-05, C-06, C-07, C-08, C-09, C-10
- **Addresses:** `docs/product-definition.md` — Safety Model, Dry Runs, Incremental Delivery Milestone 3
- **Notes:** The specification settles Q-10 with stop-after-first-operational-failure behavior and the creation, preservation, binding, and reporting portion of Q-11; later backup inspection, cleanup, rollback, and broader state recovery remain Feature 008 work. Verification evidence: `specs/005-safe-push-recovery/roadmap-reviews/debrief-20260906T185539Z.md` (`PROCEED`, no findings).

### 006 — Reverse Synchronization  [status: verified]

- **Spec dir:** `specs/006-reverse-synchronization`
- **Description:** Extend the proven mutation pipeline to propagate eligible destination-side changes back to the source for established managed entries.
- **Outcome:** `pull` safely applies unambiguous destination-to-source changes with the same preview, revalidation, recovery, verification, and baseline guarantees as `push`.
- **Scope (in):** Reverse action planning and execution; destination-path selection; source-parent handling; scoped conflict behavior; reverse backups and verification; symmetric human and machine reporting.
- **Scope (out):** Importing destination-only entries, bidirectional execution in one operation, automatic conflict merging, deletion, and weakening source-defined membership.
- **Depends on:** 005
- **Governed by:** C-02, C-03, C-05, C-06, C-07, C-10, C-11
- **Addresses:** `docs/product-definition.md` — Direction, Incremental Delivery Milestone 4
- **Notes:** Reverse synchronization applies only after an entry has entered the managed namespace and has accepted baseline evidence. Verification evidence: `specs/006-reverse-synchronization/roadmap-reviews/debrief-20260907T131934Z.md` (`PROCEED`, no findings).

### 007 — Bidirectional Synchronization and Conflict Resolution  [status: planned]

- **Description:** Combine both mutation directions into one preflighted `sync` plan and implement explicit whole-entry conflict resolution.
- **Outcome:** `sync` propagates all unambiguous one-sided changes in scope, blocks the entire action before mutation when conflicts exist, and lets the user select the complete source or destination state as the winner after fresh inspection and recovery preservation.
- **Scope (in):** Unified bidirectional planning; complete-scope conflict preflight; converged two-sided edits; source-wins and destination-wins resolution; losing-side backup; post-resolution verification and baseline publication; drift detection between report and resolution.
- **Scope (out):** Automatic field or content merging, two-way interactive merge tooling, deletion, multiple selectors, and selective continuation around known conflicts.
- **Depends on:** 005, 006
- **Governed by:** C-03, C-04, C-05, C-06, C-07, C-10
- **Addresses:** `docs/product-definition.md` — Conflicts, Direction, Incremental Delivery Milestone 5
- **Notes:** The complete supported entry state wins; content and metadata differences are not combined from opposing sides.

### 008 — Authorized Deletion and Retirement  [status: planned]

- **Description:** Add explicitly authorized directional deletion, converged deletion handling, and deliberate retirement of managed entries that become ignored or untracked.
- **Outcome:** Users can preview and authorize deletion or retirement without Grip inferring destructive intent from absence or ignore changes, and can inspect and deliberately remove retained recovery material under a defined lifecycle.
- **Scope (in):** Source-side and destination-side deletion classification execution; additional authorization; delete/change blocking; converged deletion; newly ignored pending-retirement state; explicit retirement command and transition; deletion backups; backup inspection and cleanup; baseline retirement; bounded recovery from missing or inconsistent state where evidence permits.
- **Scope (out):** Implicit deletion during ordinary sync, pruning destination-only content, filesystem snapshots, automatic cleanup without policy, and destructive state reconstruction from guesses.
- **Depends on:** 007
- **Governed by:** C-02, C-04, C-05, C-06, C-07, C-10
- **Addresses:** `docs/product-definition.md` — Deletions, Tracking and Untracking, Safety and Recovery, Incremental Delivery Milestone 5
- **Notes:** The spec must settle the retirement portion of Q-06 and complete Q-11. Recovery actions requiring an authoritative side must remain explicit.

### 009 — Metadata and Filesystem Contract Completion  [status: planned]

- **Description:** Expand and finalize the supported macOS and Unix metadata contract, close filesystem edge cases, and verify the complete initial product against representative correctness and performance workloads.
- **Outcome:** Grip detects, copies, compares, and verifies every promised metadata field when permitted; reports unsupported or unauthorized transitions precisely; enforces its supported-node boundary; and demonstrates the complete product-definition behavior without introducing disproportionate machinery.
- **Scope (in):** Final metadata allowlist; permission mode; user and group identity representation; modification-time semantics; extended attributes; ACLs; supported BSD flags; directory metadata; final symbolic-link disposition; case and Unicode behavior; cross-filesystem capability reporting; hard-link and sparse-file detection; special-node and mount-boundary enforcement; end-to-end classification and mutation suites; representative performance acceptance.
- **Scope (out):** Automatic privilege escalation, universal cross-platform fidelity, preserving unsupported physical storage representations, remote synchronization, daemons, background watchers, and the deferred future expansions listed below.
- **Depends on:** 004, 008
- **Governed by:** C-01, C-04, C-05, C-06, C-08, C-09, C-10, C-11
- **Addresses:** `docs/product-definition.md` — Content and Metadata, Supported Node Boundary, Testing Direction, Incremental Delivery Milestone 6
- **Notes:** The spec must complete Q-06, Q-07, and Q-08. Performance success must be measured on representative local trees without speculative caches, broad locks, or inode-tracking schemes.

## Open Questions

These questions are intentionally deferred to the specification that owns the decision. They do not change the approved feature sequence or initial-product boundary.

- **Q-01 — Package and executable identity (001):** Confirm the final package identity and whether `grip` remains the executable name.
- **Q-02 — Serialization formats (001):** Select the human-managed registry and machine-owned state formats, including atomic-publication and forward-version behavior.
- **Q-03 — Mapping command syntax (002):** Define exact commands and arguments for creating, inspecting, and removing file and tree mappings.
- **Q-04 — Initial tracking behavior (002):** Decide whether tracking creates intent only or may also execute an explicitly previewed initial synchronization.
- **Q-05 — Gripignore contract (003):** Select the normative Gitignore behavior and decide whether `.gripignore` can ever be explicitly synchronized as payload.
- **Q-06 — Directory membership and retirement (003, 008, 009):** Decide how empty directories enter management, which directory metadata is independent, and the exact command and state transition for retirement.
- **Q-07 — Symbolic links (003, 009):** Decide whether links are supported as non-followed link objects in the initial product or rejected entirely.
- **Q-08 — Metadata equality (004, 009):** Select the first and final supported metadata fields, modification-time role, identity representation, and behavior when target capabilities differ.
- **Q-09 — Recursive topology (002):** Enumerate equal, nested, and otherwise recursive source/destination relationships that mapping validation must reject.
- **Q-10 — Operational failure policy (005):** Confirm stop-after-first-failure behavior and the exact reporting and recovery contract for remaining planned actions.
- **Q-11 — Backup and state recovery (005, 008):** Define backup inspection and removal plus bounded recovery when registry or baseline state is absent, corrupt, unreadable, or inconsistent.
- **Q-12 — Automation contract (001, 004):** Define stable machine-output schemas and exit codes for success, drift, conflict, invalid configuration, unsupported entries, and operational failure.
- **Q-13 — Integration branch:** Name the branch whose acceptance freezes a feature directory under the Merge-Bounded Flow-Back model.

## Cross-Cutting Notes

These notes guide specification work without prematurely resolving feature-owned questions.

- Every feature must identify its applicable constitutional principles and define isolated temporary-root tests for any filesystem behavior it introduces.
- After tasking or consequential artifact reconciliation, `/speckit.analyze` gates implementation or resumption. After implementation, `/speckit.converge` runs until gaps are reconciled or explicitly removed from scope.
- Registry validation remains complete even when an inspection or action is scoped to one mapping or subtree. Conflicts and known unsafe conditions within the selected scope block mutation before the first action.
- Performance acceptance belongs in features that introduce traversal, hashing, metadata capture, or mutation. Measurements must precede additional caching, indexing, parallelism, or coordination complexity.
- Human output, machine-readable output, and diagnostic logging remain separate throughout the roadmap; domain state and decisions must not be buried in presentation strings.
- Two-way interactive `merge`, metadata-only `remap`, multiple path selectors, network synchronization, daemons, privileged services, and multi-user coordination are deferred future expansions, not hidden requirements of specs 001–009.
- No configured ADRs were present when roadmap version 1.0.0 was created. Durable decisions that later require an ADR may add governing pointers through a roadmap amendment.

---

**Version**: 1.0.12 | **Ratified**: 2026-09-03 | **Last Amended**: 2026-09-07
