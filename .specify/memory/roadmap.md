<!--
SYNC IMPACT REPORT
==================
Version change: 1.8.0 -> 1.8.1
Bump rationale: Clarify that Feature 022 also covers ordinary re-addition after successful removal, which must not retain a stale ownership conflict or require force.

Changes this revision:
  - Added planned Feature 022, Force Mapping Replacement
  - Recorded the reported equal-destination ownership collision and the requested explicit force-add capability
  - Added the removed-mapping re-addition invariant: a successfully removed conflicting mapping cannot require force on a later add

Specs affected: 022
Open questions added/resolved: Clarified Q-21

Notes: Direct active-user authorization on 2026-09-14. The reported `grip add target/release/grip ~/.local/bin/grip` collision is an exact equal-destination conflict with an existing file mapping. A successful removal of that exact mapping must allow a later add without force; removal of a different mapping does not. Feature 022 is roadmap-only; its specification must settle the replacement contract before implementation.
-->

# Grip — Spec Roadmap

Living, non-binding map of the specs planned for Grip. It is **not a commitment to order or scope** — it captures the spec-specific discussion, decisions, technology choices, outcomes, and constraints surfaced during constitution and product-definition work so they are not lost before the spec that needs them is written. Specs are scoped and clarified when they are actually started. Foundations: the project [constitution](constitution.md) and [product definition](../../docs/product-definition.md).

Status legend (lifecycle): **undecided** · **needs-info** · **planned** · **specced** · **in-progress** · **implemented** · **verified** · **deferred** · **abandoned**.

---

## Vision & End States

The initial roadmap is complete when Grip delivers the full local product described by the product definition within the limits imposed by the constitution.

- A user can declare exact file mappings and source-defined tree mappings, inspect their managed namespace, and trust Grip to leave unrelated destination content untouched.
- Grip can classify current source and destination state against an accepted baseline, propagate unambiguous changes in either direction, and require an explicit complete-side choice for conflicts.
- Every mutation supports deterministic preview, bounded revalidation, staged replacement, verification, and truthful baseline publication. Normal operations preserve conflict safety; an explicit force direction selects one complete endpoint state, including absence, for one exact entry.
- The resulting CLI is responsive for representative local trees, precise for humans and automation, tested against real filesystem behavior in isolated roots, and deliberately limited to its supported macOS and Unix contracts.
- A user can initialize a portable Grip project, commit its mapping intent, clone it on another machine, and operate only within the explicitly selected or enclosing project while machine-local operational state remains outside committed project content.

## Constraints & Decisions

These constraints apply across the ledger. Each is grounded in the active user-approved product direction, the [constitution](constitution.md), or [product definition](../../docs/product-definition.md).

- **C-01 — Local per-user boundary:** Grip is a local CLI operating on paths visible to one invoking user. Remote transport, daemons, privileged services, and multi-user coordination are outside the initial product.
- **C-02 — Explicit selective ownership:** A canonical source path identifies each mapping. File mappings own exact pairs; tree mappings own only eligible source-defined relative paths. Destination-only content outside that namespace remains unmanaged.
- **C-03 — Baseline-informed synchronization:** Grip compares current source, current destination, and the last accepted baseline. Source and destination define discovery and command direction, not a permanently authoritative editing side.
- **C-04 — Proportional rigor:** Designs must address demonstrated local failure modes with the simplest maintainable mechanism. Long-lived tree locks, watchers, persistent inode identity, kernel integration, and snapshot-isolation machinery require explicit evidence that simpler inspection, revalidation, and atomic publication are insufficient.
- **C-05 — Validate, revalidate, verify:** Mutations require a complete deterministic plan, relevant pre-action revalidation, staged publication where supported, verification, and baseline publication only after successful acceptance. Git or another operator-selected system owns history and recovery; Grip does not provide a competing recovery interface.
- **C-06 — Bounded concurrency:** External edits are handled through evidence capture, revalidation, and explicit drift errors. Only short-lived coordination around Grip-owned registry or state publication may be introduced without a separately justified governance exception.
- **C-07 — Explicit mutation semantics:** `push`, `pull`, and `sync` mutate by default; `-n` and `--dry-run` do not mutate. Ordinary operations block on conflicts or known unsafe conditions before the first mutation. `push --force` and `pull --force` may select the source or destination complete state, respectively, including absence, for one exact managed entry.
- **C-08 — Responsive by evidence:** Common inspection and planning workflows must remain responsive. Traversal, hashing, caching, parallelism, and indexing decisions are driven by representative measurements rather than speculative optimization.
- **C-09 — Allowlisted filesystem contract:** Ordinary regular files and directories form the initial payload boundary. Unsupported nodes or unreproducible metadata transitions are reported precisely and never silently coerced; macOS and Unix contracts may precede broader portability.
- **C-10 — Merge-bounded persistence:** Before merge, feature artifacts and implementation form one mutable, reviewable unit with accepted discoveries flowed back. Merge into the designated integration branch freezes the feature directory semantically, and later behavioral changes flow forward through new features.
- **C-11 — Separated interfaces:** CLI parsing and presentation remain separate from Grip-owned domain behavior. Human output, machine-readable output, and diagnostics are distinct interfaces, with deterministic forms where automation depends on them.
- **C-12 — Project-scoped operation:** Every project-dependent command operates against exactly one Grip project. `grip init [PATH]` initializes `PATH`, or the current directory when omitted. Other commands accept a global `--project PATH` selector or discover the project by walking from the current directory toward the filesystem root. Failure to find exactly one valid project fails without mutation.
- **C-13 — Portable intent, local state:** The Grip project contains a version-controllable mapping document. Mapping sources are relative to the project root. Its original restriction of destinations to a portable, user-relative representation is superseded by C-14. Baselines, locks, and resolved machine paths remain machine-local and outside committed project content; they support synchronization safety rather than user-facing history or recovery.
- **C-14 — Explicit destination forms:** A destination may be an absolute path, `~`, a path beginning with `~/`, or a path relative to the selected Grip project root. Grip preserves every accepted spelling in mapping intent and resolves it only for validation and use; home-relative forms use the selected home and project-relative forms use the selected project root. Source paths remain relative to the Grip project root and are persisted relative to that root.

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

### 007 — Bidirectional Synchronization and Conflict Resolution  [status: verified]

- **Spec dir:** `specs/007-bidirectional-sync-conflicts`
- **Description:** Combine both mutation directions into one preflighted `sync` plan and implement explicit whole-entry conflict resolution.
- **Outcome:** `sync` propagates all unambiguous one-sided changes in scope, blocks the entire action before mutation when conflicts exist, and lets the user select the complete source or destination state as the winner after fresh inspection and recovery preservation.
- **Scope (in):** Unified bidirectional planning; complete-scope conflict preflight; converged two-sided edits; source-wins and destination-wins resolution; losing-side backup; post-resolution verification and baseline publication; drift detection between report and resolution.
- **Scope (out):** Automatic field or content merging, two-way interactive merge tooling, deletion, multiple selectors, and selective continuation around known conflicts.
- **Depends on:** 005, 006
- **Governed by:** C-03, C-04, C-05, C-06, C-07, C-10
- **Addresses:** `docs/product-definition.md` — Conflicts, Direction, Incremental Delivery Milestone 5
- **Notes:** The complete supported entry state wins; content and metadata differences are not combined from opposing sides. Verification evidence: `specs/007-bidirectional-sync-conflicts/roadmap-reviews/debrief-20260907T144944Z.md` (`PROCEED WITH UPDATES`, no Must-Address findings).

### 008 — Authorized Deletion and Retirement  [status: verified]

- **Spec dir:** `specs/008-authorized-deletion-retirement`
- **Description:** Add explicitly authorized directional deletion, converged deletion handling, and deliberate retirement of managed entries that become ignored or untracked.
- **Outcome:** Users can preview and authorize deletion or retirement without Grip inferring destructive intent from absence or ignore changes, and can inspect and deliberately remove retained recovery material under a defined lifecycle.
- **Scope (in):** Source-side and destination-side deletion classification execution; additional authorization; delete/change blocking; converged deletion; newly ignored pending-retirement state; explicit retirement command and transition; deletion backups; backup inspection and cleanup; baseline retirement; bounded recovery from missing or inconsistent state where evidence permits.
- **Scope (out):** Implicit deletion during ordinary sync, pruning destination-only content, filesystem snapshots, automatic cleanup without policy, and destructive state reconstruction from guesses.
- **Depends on:** 007
- **Governed by:** C-02, C-04, C-05, C-06, C-07, C-10
- **Addresses:** `docs/product-definition.md` — Deletions, Tracking and Untracking, Safety and Recovery, Incremental Delivery Milestone 5
- **Notes:** The specification settles the retirement portion of Q-06 and completes Q-11. Recovery actions requiring an authoritative side remain explicit. Verification evidence: `specs/008-authorized-deletion-retirement/roadmap-reviews/debrief-20260907T190356Z.md` (`PROCEED`, no findings).

### 009 — Metadata and Filesystem Contract Completion  [status: verified]

- **Spec dir:** `specs/009-metadata-filesystem-contract`
- **Description:** Expand and finalize the supported macOS and Unix metadata contract, close filesystem edge cases, and verify the complete initial product against representative correctness and performance workloads.
- **Outcome:** Grip detects, copies, compares, and verifies every promised metadata field when permitted; reports unsupported or unauthorized transitions precisely; enforces its supported-node boundary; and demonstrates the complete product-definition behavior without introducing disproportionate machinery.
- **Scope (in):** Final metadata allowlist; permission mode; user and group identity representation; modification-time semantics; extended attributes; ACLs; supported BSD flags; directory metadata; final symbolic-link disposition; case and Unicode behavior; cross-filesystem capability reporting; hard-link and sparse-file detection; special-node and mount-boundary enforcement; end-to-end classification and mutation suites; representative performance acceptance.
- **Scope (out):** Automatic privilege escalation, universal cross-platform fidelity, preserving unsupported physical storage representations, remote synchronization, daemons, background watchers, and the deferred future expansions listed below.
- **Depends on:** 004, 008
- **Governed by:** C-01, C-04, C-05, C-06, C-08, C-09, C-10, C-11
- **Addresses:** `docs/product-definition.md` — Content and Metadata, Supported Node Boundary, Testing Direction, Incremental Delivery Milestone 6
- **Notes:** The spec completes Q-06, Q-07, and Q-08. Representative macOS/APFS correctness and performance qualification passed without persistent caches, broad locks, parallel execution, or inode-tracking schemes. Verification evidence: `specs/009-metadata-filesystem-contract/roadmap-reviews/debrief-20260908T135643Z.md` (`PROCEED`, no findings).

### 010 — Project-Scoped Initialization and Portable Mappings  [status: verified]

- **Spec dir:** `specs/010-project-scoped-initialization`
- **Description:** Replace the global per-user mapping registry with initialized Grip projects whose portable mapping intent is rooted in a source directory and whose commands resolve an explicit or enclosing project.
- **Outcome:** A user can initialize a directory, commit its mapping configuration to version control, clone it on another machine, and run Grip commands scoped exclusively to that project without embedding machine-specific absolute source or destination paths.
- **Scope (in):** `grip init [PATH]`; the “Grip project” term; project-root metadata discovery by ancestor walking; a global `--project PATH` selector for project-dependent commands; project-scoped command execution; relative source mappings; portable user-relative destinations; committed intent separated from machine-local operational state; complete replacement of the global mapping model; documentation, CLI, storage, state-binding, selection, validation, recovery, and test updates.
- **Scope (out):** Backward compatibility, migration, dual-read behavior, continued support for a global mapping registry, Git automation, remote synchronization, and sharing machine-owned state.
- **Depends on:** 009
- **Governed by:** C-01, C-02, C-03, C-04, C-05, C-06, C-10, C-11, C-12, C-13
- **Addresses:** `docs/product-definition.md` — Core Concepts, Mapping Model, Command-Line Experience, Configuration and State
- **Notes:** This is an intentional pre-release product correction. Verified Features 001–009 remain historical evidence, but their global registry, absolute mapping, and per-user command-scope decisions are superseded where Feature 010 conflicts with them. Verification evidence: `specs/010-project-scoped-initialization/roadmap-reviews/debrief-20260909T193508Z.md` (`PROCEED`, no findings).

### 011 — Git-Inspired Command Hierarchy  [status: verified]

- **Spec dir:** `specs/011-git-command-hierarchy`
- **Description:** Replace Grip's broad, nested command surface with a flat, Git-inspired synchronization interface whose public verbs describe the user's deployment workflow rather than internal state-management mechanics.
- **Outcome:** Users can initialize or select a project; add, list, and remove mappings; inspect status and differences; and push, pull, or safely synchronize changes using a concise, consistent command contract. Human text is the default output and JSON is an explicit automation format. Git or another operator-selected system remains responsible for history and recovery.
- **Scope (in):** The exact public hierarchy `init`, `version`, `add`, `list`, `remove`, `status`, `diff`, `push`, `pull`, and `sync`; global `-p|--project`, `-o|--output json`, and repeatable `-v|--verbose`; automatic file-or-directory mapping classification for `add`; filtered `list [SOURCE]`; `status -e|--exit-code`; consistent `-d|--destination` selector interpretation; `-n|--dry-run`; and exact-entry `-f|--force` for directional conflict resolution and accepted one-sided absence. Human output is the default. `.gripignore` directly defines exclusions inside tree mappings, while baseline data remains internal synchronization evidence.
- **Scope (out):** A nested `mapping` command family; `show`, `check`, `validate`, `fsck`, `resolve`, `delete`, `retire`, `accept`, untracking, or recovery commands; a Grip-managed history or recovery interface; automatic conflict merging; implicit forced resolution; backward-compatible command aliases; additional output formats beyond JSON; and implementation planning or code changes.
- **Depends on:** 010
- **Governed by:** C-01, C-02, C-03, C-04, C-05, C-07, C-10, C-11, C-12, C-13
- **Addresses:** Direct active-user decisions from the command-hierarchy conversation on 2026-09-09; `docs/product-definition.md` — Command-Line Experience, Synchronization Model, Tracking and Untracking, Safety and Recovery
- **Notes:** This is an intentional pre-release interface replacement. The specification is `specs/011-git-command-hierarchy/spec.md`. It must remove superseded options and commands throughout the repository rather than preserve compatibility. Ordinary `status` validates Grip-owned metadata as a prerequisite; `-e` changes only the exit result for valid attention findings. `push -f` and `pull -f` require a single exact managed entry and make the source or destination state, respectively, the complete winner. Removing an ignore rule later must treat a newly reintroduced entry as newly discovered rather than silently revive obsolete baseline history. Direct active-user authorization is the governing provenance. Verification evidence: `specs/011-git-command-hierarchy/roadmap-reviews/debrief-20260910T153132Z.md` (`PROCEED`, no findings).

### 012 — Destination Path Forms  [status: verified]

- **Spec dir:** `specs/012-destination-path-forms`
- **Description:** Replace the destination-path restriction inherited from Features 010 and 011 with an explicit contract accepting absolute paths, `~`, and any `~/`-prefixed path, including a non-normalized spelling.
- **Outcome:** `grip add README.md ~/Working/grip-dst/README.md` and `grip add README.md /Users/pegagio/Working/grip-dst/README.md` succeed without a destination-normalization error, while a relative destination is rejected. Source paths remain relative to, and are persisted relative to, the Grip project root.
- **Scope (in):** The `add` destination parser, validation, resolution, mapping persistence, diagnostics, documentation, and isolated filesystem tests; absolute destination paths; `~/`-prefixed destination paths without a lexical-normalization prerequisite; explicit relative-destination rejection; and unchanged project-relative source-path semantics.
- **Scope (out):** Relative destination paths; changes to source-path forms; automatic destination migration; backward-compatible command aliases; remote paths; and implementation planning or code changes.
- **Depends on:** 011
- **Governed by:** C-01, C-02, C-04, C-10, C-11, C-12, C-13, C-14
- **Addresses:** Direct active-user decision on 2026-09-10, including the reported `destination must be ~ or a normalized ~/ path` diagnostic.
- **Notes:** This is an intentional pre-release interface correction. Feature 010's portable user-relative-destination restriction and Feature 011's resulting destination validation are superseded only where they conflict with C-14; both verified features remain historical evidence. Accepted destination spelling is preserved in mapping intent and resolved only for validation and use. Verification evidence: `specs/012-destination-path-forms/roadmap-reviews/debrief-20260911T045048Z.md` (`PROCEED`, no findings).

### 013 — Source Path Input Normalization  [status: implemented]

- **Spec dir:** `specs/013-source-path-input`
- **Description:** Accept ordinary project-contained relative source spellings at the CLI boundary while retaining strict normalized source declarations in project metadata.
- **Outcome:** An operator can use a spelling such as `./app/` for source-side mapping creation and selection, while Grip stores and resolves the same canonical project-relative source identity as `app` and rejects project escapes or `.grip` paths before mutation.
- **Scope (in):** Lexical source-input normalization; source-space command selection; project and reserved-metadata containment; strict persisted-declaration validation; documentation; and isolated boundary regression coverage.
- **Scope (out):** Absolute or environment-expanded source forms, destination-path changes, endpoint payload mutation during `add`, storage migration, and new commands or options.
- **Depends on:** 012
- **Governed by:** C-02, C-04, C-05, C-10, C-11, C-12, C-13, C-14
- **Addresses:** `specs/013-source-path-input/spec.md`; direct active-user authorization for this ledger backfill on 2026-09-13.
- **Notes:** Delivered in `8d70eb3` with its complete feature artifacts and validation. The feature normalizes only submitted source input; stored source declarations remain strict and canonical. It remains implemented pending a feature-specific debrief.

### 014 — Relative Destination Paths  [status: verified]

- **Spec dir:** `specs/014-relative-destination-paths`
- **Description:** Extend destination declarations to accept paths relative to the selected Grip project while retaining the exact declaration spelling.
- **Outcome:** A user can run `grip add ./app/ ../grip-dst/app/`; Grip stores `../grip-dst/app/` unchanged and resolves it from the selected project root for validation, selection, state use, and synchronization.
- **Scope (in):** Project-relative destination parsing and lexical operational resolution; exact declaration persistence; destination-space selection; descriptor reload; state binding and rebinding; declaration-preserving removal publication; ownership and topology validation; user documentation; isolated parser, integration, portability, and regression coverage.
- **Scope (out):** Current-working-directory-relative destination mode, destination migration, new flags or commands, remote syntax, and changes to the strict project-relative source declaration contract.
- **Depends on:** 012
- **Governed by:** C-01, C-02, C-04, C-05, C-06, C-10, C-11, C-12, C-13, C-14
- **Addresses:** Direct active-user decision on 2026-09-11; `docs/product-definition.md` — Product model and mappings; `README.md` — Configure portable mappings.
- **Notes:** This feature supersedes Feature 012 only where Feature 012 rejected relative destinations. Relative declarations are based on the selected project, never the process current directory, and may explicitly address an adjacent path through `..`; existing endpoint, ownership, topology, state, and publication protections remain in force. Verification evidence: `specs/014-relative-destination-paths/spec.md`, `plan.md`, `tasks.md`, and completed isolated validation through `mise run validate`.

### 015 — Simplify Status Output  [status: verified]

- **Spec dir:** `specs/015-simplify-status-output`
- **Description:** Replace the verbose default human `grip status` detail with concise, path-centered grouped output.
- **Outcome:** Operators can see a selected scope's summary and its nonempty `Conflicts`, `Changes to push`, `Changes to pull`, and `Needs baseline` groups without parsing internal comparison evidence; JSON, `diff`, selection, and exit behavior remain unchanged.
- **Scope (in):** Default human status summary and deterministic grouping; direction symbols for source-to-destination, destination-to-source, conflicts, and non-directional baseline attention; plain-language safety blockers; focused renderer contracts; user documentation; and regression coverage.
- **Scope (out):** Classification, mapping selection, synchronization, mutation, conflict resolution, metadata policy, baseline semantics, JSON output, `diff` output, exit behavior, commands, storage, and migrations.
- **Depends on:** 004
- **Governed by:** C-03, C-04, C-08, C-09, C-10, C-11
- **Addresses:** `docs/product-definition.md` — Synchronization Model, Content and Metadata, Command-Line Experience; `README.md` — Inspect changes and baseline status.
- **Notes:** `->`, `<-`, `<->`, and `>-<` are presentation-only signals and do not select a conflict winner or an unsafe payload direction. Informational no-action diagnostics are omitted only from default human output; safety blockers remain visible. Direct active-user decision and completed feature artifacts are the governing provenance. Verification evidence: `specs/015-simplify-status-output/roadmap-reviews/debrief-20260911T212103Z.md` (`PROCEED`, no findings).

### 016 — Current-Directory Status Paths  [status: implemented]

- **Spec dir:** `specs/016-cwd-status-paths`
- **Description:** Render default-human status source paths relative to the invocation directory and accept ordinary displayed source selectors in a following `grip push` from that directory.
- **Outcome:** Operators can inspect a pushable source path from the project root, a nested directory, a sibling directory, or outside an explicitly selected project, then copy the displayed ordinary source selector into `grip push` without translating it manually.
- **Scope (in):** Invocation-directory-relative source display; Git-style human quoting; `grip push` source-selector resolution and containment; preserved status grammar and JSON; documentation; and isolated round-trip, escape, and regression coverage.
- **Scope (out):** Shell-safe quoting guarantees, destination-selector changes, mapping-declaration changes, JSON changes, and weakened selected-project containment.
- **Depends on:** 015
- **Governed by:** C-02, C-04, C-05, C-08, C-10, C-11, C-12, C-13
- **Addresses:** `specs/016-cwd-status-paths/spec.md`; direct active-user authorization for this ledger backfill on 2026-09-13.
- **Notes:** Delivered in `b67be25` with complete feature artifacts and validation. Source labels are a human display and immediate `push` handoff contract; destination representation and structured output remain distinct. It remains implemented pending a feature-specific debrief.

### 017 — Simplify Default Command Output  [status: implemented]

- **Spec dir:** `specs/017-simplify-command-output`
- **Description:** Make default human output for mappings, status, mutation results, and errors concise and action-oriented while preserving detailed and machine-readable interfaces.
- **Outcome:** Operators can see declared mappings, status groups, planned or completed directional changes, blocked next steps, and errors without internal plan, recovery, verification, baseline, or record-identification detail; `grip diff` and JSON remain unchanged.
- **Scope (in):** Default human mapping, status, mutation, and error presentation; concise result headings and directional rows; force-resolution guidance; documentation; and transcript-based regression coverage.
- **Scope (out):** `grip diff` presentation, JSON schemas or values, selection, classification, mutation semantics, filesystem safety, storage, and migrations.
- **Depends on:** 015, 016
- **Governed by:** C-01, C-03, C-04, C-05, C-08, C-10, C-11, C-12
- **Addresses:** `specs/017-simplify-command-output/spec.md`; direct active-user authorization for this ledger backfill on 2026-09-13.
- **Notes:** Delivered in `1b65b74` with complete feature artifacts and validation. Feature 019 supersedes only its force-guidance rendering where an aggregate selector is not executable. It remains implemented pending a feature-specific debrief.

### 018 — Source-Authoritative Mapping Addition  [status: implemented]

- **Spec dir:** `specs/018-source-authoritative-add`
- **Description:** Keep `grip add` non-mutating for endpoint payloads while making a newly added unequal source-defined member immediately eligible for an ordinary push.
- **Outcome:** Adding an unequal file or source-defined tree member records the destination as its initial comparison reference, so status, push, and sync offer source-to-destination work without copying either endpoint during `add`; incomplete publication remains explicitly fenced per mapping.
- **Scope (in):** Add-time initial comparison state for unequal managed files and tree members; source-defined membership and ignore handling; mapping-scoped incomplete-add publication fencing; bounded retry or restoration; documentation; and isolated success and failure regression coverage.
- **Scope (out):** Endpoint payload mutation during `add`, destination-only ownership, changed selectors or force semantics, automatic conflict resolution, `diff` changes, JSON changes, and user-facing recovery history.
- **Depends on:** 004, 011
- **Governed by:** C-02, C-03, C-04, C-05, C-06, C-07, C-10, C-11, C-12, C-13, C-14
- **Addresses:** `specs/018-source-authoritative-add/spec.md`; direct active-user authorization for this ledger backfill on 2026-09-13.
- **Notes:** Delivered in `1b65b74` with complete feature artifacts and validation. Source authority is represented by the destination comparison reference, not by payload copying or a permanent source-wins rule. It remains implemented pending a feature-specific debrief.

### 019 — Executable Force-Resolution Guidance  [status: implemented]

- **Spec dir:** `specs/019-executable-force-guidance`
- **Description:** Display force-resolution commands only when the shown selector resolves to one exact established managed entry; give aggregate conflicts a read-only inspection step instead.
- **Outcome:** Every source-winning or destination-winning force command shown by default human status or blocked mutation output passes the existing exact-entry selector validation from the same invocation directory. Aggregate conflicts show `grip diff SOURCE` rather than a force command that cannot run.
- **Scope (in):** Exact-entry eligibility for human force guidance; aggregate inspection guidance; status and blocked mutation rendering; documentation; and exact-file, aggregate-tree, JSON, and safety regression coverage.
- **Scope (out):** Expanded force authority, aggregate conflict resolution, selector interpretation changes, classification, mutation planning, filesystem changes, JSON changes, and `grip diff` behavior changes.
- **Depends on:** 017
- **Governed by:** C-02, C-04, C-05, C-07, C-10, C-11, C-12
- **Addresses:** `specs/019-executable-force-guidance/spec.md`; direct active-user authorization for this ledger backfill on 2026-09-13.
- **Notes:** Delivered in `af6f9af` with complete feature artifacts and full validation. The existing exact-entry force boundary remains authoritative; aggregate guidance is inspection-only. It remains implemented pending a feature-specific debrief.

### 020 — Release Binary Size Investigation  [status: abandoned]

- **Spec dir:** To be assigned when specification work begins
- **Description:** Investigate the approximately 19 MB `grip` binary, identify its material size contributors, determine whether that size is justified for the supported release target, and apply evidence-backed reductions that preserve the supported CLI contract.
- **Outcome:** Maintainers can explain the release binary's size with reproducible measurements, decide whether a reduction is warranted against an explicit budget, and deliver only reductions whose compatibility, safety, and operational tradeoffs are validated.
- **Scope (in):** Reproducible release-artifact measurement; target, profile, dependency, symbol, and packaging contribution analysis; comparison of justified reduction options; selected implementation changes; release-size regression coverage; and documentation of the resulting evidence and tradeoffs.
- **Scope (out):** Feature delivery unrelated to release-artifact size; unmeasured stripping or compression changes; unsupported-target expansion; and weakening diagnostics, safety behavior, or supported CLI functionality solely to reduce bytes.
- **Depends on:** none
- **Governed by:** C-04, C-08, C-10, C-11
- **Addresses:** Direct active-user decision on 2026-09-14 reporting an approximately 19 MB `grip` binary and requesting investigation of whether it must be that large and easy reduction opportunities.
- **Notes:** Abandoned by direct active-user decision on 2026-09-14 after the user reported an approximately 4.5 MB release artifact, already below the feature's 10 MiB stretch target. No size investigation, reduction, or acceptance claim is implied by this abandonment.

### 021 — Large-File Operation Performance  [status: verified]

- **Spec dir:** `specs/021-large-file-performance`
- **Description:** Measure why Grip operations degrade on a representative approximately 19 MB file, remove demonstrated bottlenecks, and establish regression coverage for the resulting performance contract.
- **Outcome:** Operators can run the selected Grip operations on representative large files within explicit, reproducible acceptance targets without weakening classification, mutation safety, verification, or baseline correctness.
- **Scope (in):** Representative workload definition; operation-level profiling and measurement; analysis of file reading, hashing, metadata capture, comparison, staging, copying, verification, and output costs where applicable; evidence-backed implementation improvements; performance regression coverage; and documentation of measured tradeoffs.
- **Scope (out):** Speculative caching, indexing, parallelism, locking, watcher, daemon, or filesystem-integration mechanisms without evidence; changes to ownership, conflict, recovery, or baseline semantics; and unmeasured optimization of unrelated workflows.
- **Depends on:** none
- **Governed by:** C-04, C-05, C-08, C-10, C-11
- **Addresses:** Direct active-user decision on 2026-09-14 reporting slow Grip operations on an approximately 19 MB file and requesting diagnosis and correction.
- **Notes:** Verified on 2026-09-14 by the no-finding debrief at `specs/021-large-file-performance/roadmap-reviews/debrief-20260914T161151Z.md`. The representative workload is an isolated local macOS ARM64 release build with differing 19 MiB regular source and destination files; it runs `grip add` and JSON `grip status` for 100 warm samples each with a one-second p95 target. The measured cause was duplicate same-pass complete observation of discovery-backed file mappings. The operation-local deduplication retained descriptor-bound content and metadata evidence, independent stable-observation passes, fenced add reinspection, initial comparison behavior, and discovery-absent fallback. Final acceptance p95 was 573.669375 ms for `grip add` and 184.788375 ms for `grip status`. No cache, index, parallelism, lock, watcher, daemon, or baseline-semantics change was introduced.

### 022 — Force Mapping Replacement  [status: planned]

- **Spec dir:** To be assigned when specification work begins
- **Description:** Allow an operator to explicitly force `grip add` to replace an existing conflicting mapping when the requested mapping would otherwise be rejected by accepted-registry ownership validation.
- **Outcome:** An operator can deliberately replace the prior mapping in the reported equal-destination case, such as switching a managed executable destination from a debug artifact to a release artifact, while Grip preserves exact ownership validation, prevents partial registry or state publication, and leaves endpoint payloads unchanged by `add`. An operator who successfully removes that exact conflicting mapping can later add a replacement without force or stale ownership rejection.
- **Scope (in):** An explicit force-add interface; exact conflicting-mapping identification and replacement eligibility; descriptor, baseline, fence, and operation-state handling for the displaced mapping; verification that `remove` fully retires the removed mapping from every ownership-validation input before a later ordinary add; complete registry revalidation; atomic publication and failure behavior; human and JSON results; documentation; and isolated file- and tree-conflict regression coverage.
- **Scope (out):** Implicit replacement; treating removal of a different mapping as removal of the active conflict; bypassing unrelated ownership or topology conflicts; payload copying or synchronization during `add`; multi-mapping bulk replacement; changes to `push`, `pull`, or `sync` force semantics; automatic recovery or history; and any permission escalation outside the selected Grip project.
- **Depends on:** 018, 019
- **Governed by:** C-02, C-03, C-04, C-05, C-06, C-07, C-10, C-11, C-12, C-13, C-14
- **Addresses:** Direct active-user decision on 2026-09-14, supported by the reported rejected command `grip add target/release/grip ~/.local/bin/grip` against an existing `target/debug/grip` mapping to the same destination and a request that re-adding after removal of that exact mapping must not require force.
- **Notes:** The roadmap authorizes no implementation yet. The feature specification must decide the exact `add` force spelling and whether it may replace only one exact equal-destination file mapping or a broader precisely identified ownership conflict; it must also define displaced baseline and incomplete-fence treatment, the post-remove ownership invariant, conflict reporting, and user confirmation or preview requirements. The reported transcript removes `target/debug/mise` while `grip status` still shows `target/debug/grip`; that is evidence that removal of a different mapping cannot clear the active `grip` conflict.

## Open Questions

These questions are intentionally deferred to the specification that owns the decision. They do not change the approved feature sequence or initial-product boundary.

- **Q-01 — Package and executable identity (001):** Confirm the final package identity and whether `grip` remains the executable name.
- **Q-02 — Serialization formats (001):** Select the human-managed registry and machine-owned state formats, including atomic-publication and forward-version behavior.
- **Q-03 — Mapping command syntax (002):** Superseded by Feature 011. Its flat `add`, `list [SOURCE]`, and `remove` contract replaces the historical nested mapping syntax.
- **Q-04 — Initial tracking behavior (002):** Decide whether tracking creates intent only or may also execute an explicitly previewed initial synchronization.
- **Q-05 — Gripignore contract (003):** Select the normative Gitignore behavior and decide whether `.gripignore` can ever be explicitly synchronized as payload.
- **Q-06 — Directory membership and retirement (003, 008, 009):** Directory membership and metadata questions remain historical Feature 003/009 evidence. The retirement-command portion is superseded by Feature 011: `.gripignore` directly defines exclusions, and excluded entries do not retain separate user-managed tracking state.
- **Q-07 — Symbolic links (003, 009):** Decide whether links are supported as non-followed link objects in the initial product or rejected entirely.
- **Q-08 — Metadata equality (004, 009):** Select the first and final supported metadata fields, modification-time role, identity representation, and behavior when target capabilities differ.
- **Q-09 — Recursive topology (002):** Enumerate equal, nested, and otherwise recursive source/destination relationships that mapping validation must reject.
- **Q-10 — Operational failure policy (005):** The historical stop-after-first-failure evidence remains available in Feature 005. Its Grip-managed recovery portion is superseded by Feature 011 and Constitution 2.0.0; failures remain precisely reported without creating a Grip history or recovery interface.
- **Q-11 — Backup and state recovery (005, 008):** Superseded by Feature 011 and Constitution 2.0.0. Git or another operator-selected system owns user-facing history and recovery; Grip retains only internal synchronization evidence.
- **Q-12 — Automation contract (001, 004):** Define stable machine-output schemas and exit codes for success, drift, conflict, invalid configuration, unsupported entries, and operational failure.
- **Q-13 — Integration branch:** Name the branch whose acceptance freezes a feature directory under the Merge-Bounded Flow-Back model.
- **Q-14 — Project descriptor and destination notation (010):** Resolved by Features 012 and 014 and C-14. Destinations may be absolute, `~`, `~/`-prefixed, or project-relative; accepted spelling is preserved in mapping intent and resolved only for validation and use. Sources remain project-relative.
- **Q-15 — Machine-local project identity (010):** Define how machine-local state is keyed to a project without creating collisions between multiple clones.
- **Q-16 — Initialization and discovery boundaries (010):** Define initialization idempotency, nested-project discovery, and behavior when an explicit project conflicts with an enclosing project.
- **Q-17 — Release target and package configuration (020):** No longer applicable. Feature 020 was abandoned by direct active-user decision on 2026-09-14 after the reported release artifact was already below the stretch target.
- **Q-18 — Binary-size acceptance budget (020):** No longer applicable. Feature 020 was abandoned by direct active-user decision on 2026-09-14 after the reported release artifact was already below the stretch target.
- **Q-19 — Representative large-file workload (021):** Resolved by verified Feature 021. The representative workload is an isolated local macOS ARM64 release build with differing 19 MiB regular source and destination files, exercising `grip add` and JSON `grip status` for 100 warm samples each.
- **Q-20 — Large-file performance acceptance (021):** Resolved by verified Feature 021. Both representative operations have a one-second p95 release-build acceptance threshold, enforced by the isolated performance gate; final recorded p95 values were 573.669375 ms for `grip add` and 184.788375 ms for `grip status`.
- **Q-21 — Force-add replacement contract (022):** Define the `grip add` force spelling; exact replacement eligibility across equal-destination, source-overlap, and tree conflicts; whether preview or confirmation is required; how the displaced mapping's baseline, incomplete fence, operation evidence, and state are retired or preserved without partial publication; and the proof that removal of the exact conflicting mapping clears every ownership-validation input before a later ordinary add.

## Cross-Cutting Notes

These notes guide specification work without prematurely resolving feature-owned questions.

- Every feature must identify its applicable constitutional principles and define isolated temporary-root tests for any filesystem behavior it introduces.
- After tasking or consequential artifact reconciliation, `/speckit.analyze` gates implementation or resumption. After implementation, `/speckit.converge` runs until gaps are reconciled or explicitly removed from scope.
- Registry validation remains complete even when an inspection or action is scoped to one mapping or subtree. Conflicts and known unsafe conditions within the selected scope block mutation before the first action.
- Performance acceptance belongs in features that introduce traversal, hashing, metadata capture, or mutation. Measurements must precede additional caching, indexing, parallelism, or coordination complexity.
- Human output, machine-readable output, and diagnostic logging remain separate throughout the roadmap; domain state and decisions must not be buried in presentation strings.
- Two-way interactive `merge`, metadata-only `remap`, multiple path selectors, network synchronization, daemons, privileged services, and multi-user coordination are deferred future expansions, not hidden requirements of specs 001–009.
- No configured ADRs were present when roadmap version 1.0.0 was created. Durable decisions that later require an ADR may add governing pointers through a roadmap amendment.
- Feature 010 is authorized to replace the global `~/.grip/` mapping registry, absolute stored mapping paths, `GRIP_HOME`-selected command scope, and any other per-user global mapping behavior established by Features 001–009. No backward-compatible or dual-read behavior is required because Grip has not been released.

---

**Version**: 1.8.1 | **Ratified**: 2026-09-03 | **Last Amended**: 2026-09-14
