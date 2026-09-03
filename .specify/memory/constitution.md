<!--
SYNC IMPACT REPORT
==================
Version change: unratified template -> 1.0.0
Bump rationale: Initial ratification of project governance.

Modified principles:
  - None; the previous document was an unratified placeholder scaffold.

Added principles:
  - I. Proportional Rigor for a Local Tool
  - II. Explicit Ownership and Least Surprise
  - III. Validate, Revalidate, and Recover
  - IV. Bounded Concurrency
  - V. Fast, Observable, and Testable

Added sections:
  - Product Boundaries and Engineering Constraints
  - Spec Evolution and Merge-Bounded Persistence
  - Concrete governance rules

Removed sections:
  - Placeholder-only scaffold content

Follow-up TODOs: None
-->

# Grip Constitution

## Core Principles

### I. Proportional Rigor for a Local Tool

Grip MUST use the simplest design that protects a local user's managed files and accepted state. Engineering rigor MUST be proportional to demonstrated local failure modes; complexity MUST NOT be justified by hypothetical distributed-system conditions outside the product boundary. Long-lived filesystem locks, background watchers, persistent inode identity, kernel integration, distributed coordination, and snapshot-isolation machinery MUST NOT be introduced unless a feature specification demonstrates why inspection, revalidation, atomic publication, and recovery cannot meet the requirement. Any proposal for such machinery MUST identify the concrete failure it prevents, the simpler alternatives considered, and its operational cost.

**Rationale:** Grip is a local per-user file tool, not critical infrastructure. Trust comes from clear boundaries and recoverable behavior, not from simulating guarantees the local filesystem cannot provide or the product does not need.

### II. Explicit Ownership and Least Surprise

Grip MUST mutate only entries owned by an explicit mapping and selected operation. Source-side discovery MAY admit new eligible entries to a proposed plan, but destination-only content outside the managed namespace MUST remain unmanaged and untouched. Dry runs MUST be non-mutating, deletion MUST require additional explicit authorization, and unsupported or ambiguous entries MUST block affected mutation rather than be coerced into a supported shape. Grip MUST NOT elevate privileges, invoke version-control operations, or silently weaken ownership or metadata guarantees.

**Rationale:** A selective overlay synchronizer is useful only when users can predict exactly what it owns and what it will leave alone.

### III. Validate, Revalidate, and Recover

Every mutating operation MUST derive a deterministic plan from a complete validated inspection of its requested scope. Immediately before applying an action, Grip MUST revalidate the evidence whose change could make that action unsafe; detected drift MUST stop the affected operation before publication. Replacements MUST be staged and published atomically where the supported filesystem permits. Replaced or deleted entries MUST be retained in a recovery namespace, results MUST be verified, and a new baseline MUST be published only for a successfully completed accepted operation. Partial failure MUST be reported precisely and MUST NOT be represented as successful convergence.

**Rationale:** Grip does not need perfect filesystem snapshot isolation. It needs to detect when its evidence is stale, stop safely, and preserve enough state for recovery.

### IV. Bounded Concurrency

Grip MUST assume that users and other processes can change mapped paths between inspection and mutation. It MUST handle this through narrow evidence capture, pre-action revalidation, safe publication, and explicit drift errors rather than attempts to lock entire source or destination trees. A short-lived, per-user lock MAY protect Grip-owned registry or state publication when concurrent Grip processes could corrupt that data; such a lock MUST have bounded scope, clear ownership, actionable contention reporting, and stale-lock recovery. Concurrency mechanisms MUST NOT depend on inode identity remaining stable across replacement or on preventing unrelated applications from editing user files.

**Rationale:** Local concurrent edits are real, but broad locking is neither portable nor necessary. Bounded coordination around Grip-owned state plus filesystem revalidation protects the useful boundary.

### V. Fast, Observable, and Testable

Common read-only and planning workflows MUST remain responsive on representative local file trees. Implementations MUST avoid redundant traversal, hashing, metadata reads, and serialization within an operation. Performance work MUST be driven by measurements of representative workloads; caches, parallelism, and additional indexing MUST NOT be added without evidence that their benefit outweighs invalidation and correctness complexity. Human output, machine-readable output, and diagnostic logs MUST remain distinct and deterministic where automation depends on them.

Every behavior that can change payloads, registry data, or baselines MUST have automated tests using isolated temporary roots. Tests MUST cover ownership boundaries, dry-run non-mutation, classification, conflicts, concurrent drift, partial failure, recovery, unsupported nodes, and baseline publication. Tests MUST NOT inspect or mutate the developer's real files or Grip home.

**Rationale:** A responsive Grip earns daily use without trading away operator trust. Measurement and filesystem integration tests keep performance and safety claims grounded in observable behavior.

## Product Boundaries and Engineering Constraints

Grip is a local, per-user, stateful, selective, bidirectional overlay synchronizer. The initial product MUST operate only on paths visible to the invoking user and MUST preserve the distinction between user-authored mapping intent and machine-owned synchronization state. It MUST NOT require a daemon, remote transport, multi-user coordination, a privileged service, automatic conflict merging, filesystem snapshots, or automatic ownership of destination-only content.

The initial supported payload boundary MUST be explicit and allowlist-based. Unsupported node types and metadata transitions MUST be reported precisely and MUST NOT be followed, opened, copied, or silently discarded. Platform contracts MAY begin with macOS and Unix behavior rather than claiming universal cross-platform fidelity.

Features MUST keep CLI parsing and presentation separate from Grip-owned domain behavior. New frameworks, services, caches, concurrency mechanisms, or persistent indexes require a concrete capability need and an explanation of their maintenance and correctness costs. Reversible implementation choices belong in feature plans; durable product behavior belongs in specifications or this constitution.

## Spec Evolution and Merge-Bounded Persistence

The project MUST use the Merge-Bounded Flow-Back Spec Persistence Model.

- **One mutable change set**: Before a feature is merged, its `spec.md`, `plan.md`, `tasks.md`, and implementation MUST be treated as one mutable, reviewable unit.
- **Changes flow back**: Accepted discoveries MAY originate in any artifact, but their consequences MUST be applied throughout the artifact set before work proceeds from the changed direction. A change to intended behavior MUST be reflected in `spec.md`; a change to technical approach MUST be reflected in `plan.md`; and a change to the required work MUST be reflected in `tasks.md`. Lower-level artifacts and implementation MUST NOT silently contradict higher-level intent.
- **Scope requires acceptance**: Flow-back MUST NOT be used to introduce material scope without review. Independently valuable behavior, substantial scope expansion, or work requiring separate acceptance MUST be captured as a separate feature.
- **Consistency gates implementation and merge**: After tasking or consequential artifact reconciliation, the agent MUST run `/speckit.analyze` before starting or resuming implementation. After implementation, the agent MUST use `/speckit.converge` until no gaps remain. Known divergence MUST block implementation or merge until it is reconciled or explicitly removed from scope.
- **Merge freezes history**: Acceptance into the project's designated integration branch is the persistence boundary. After that merge, the feature directory MUST be treated as a semantically immutable historical record. Editorial corrections MAY improve presentation only when they do not alter meaning.
- **Later changes flow forward**: A later requirement or behavioral change MUST be expressed in a new feature directory. The new feature MUST reference any earlier feature that it amends, replaces, or depends on when that relationship is material, and MUST NOT rewrite the earlier feature to describe the new outcome retroactively.

**Rationale:** This model permits requirements and implementation knowledge to converge while a feature is being developed, makes the merged feature a coherent unit of review, and preserves an auditable sequence of accepted changes without rewriting project history.

## Governance

This constitution governs project specifications, plans, tasks, implementation, and review practices. When another project artifact conflicts with it, the constitution takes precedence until an explicit amendment resolves the conflict.

Amendments require an explicit user-approved change to this file, a Sync Impact Report, and a semantic version increment. A MAJOR increment removes or incompatibly redefines a principle or governance obligation; a MINOR increment adds a principle or materially expands governance; a PATCH increment clarifies wording without changing obligations. The original ratification date MUST remain stable, and Last Amended MUST record the date of the latest semantic change.

Every feature specification and plan MUST identify applicable constitutional principles and explain any justified exception. Reviews MUST verify proportional complexity, managed-path boundaries, mutation safety, concurrency behavior, recovery, performance evidence where relevant, isolated filesystem tests, and merge-bounded artifact consistency. Exceptions MUST be explicit, narrowly scoped, and approved before implementation or merge.

**Version**: 1.0.0 | **Ratified**: 2026-09-03 | **Last Amended**: 2026-09-03
