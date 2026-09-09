# Research: Project-Scoped Initialization and Portable Mappings

Feature 010 replaces the global Grip-home model with initialized source projects. This research covers the current CLI dispatch, registry publication, absolute mapping model, State V3, operation and recovery formats, shared test fixtures, current product documentation, and the clarified specification.

## Table of Contents

- [Project Context and Command Dispatch](#project-context-and-command-dispatch)
- [Initialization and Metadata Safety](#initialization-and-metadata-safety)
- [Project Discovery](#project-discovery)
- [Portable Paths and Mappings](#portable-paths-and-mappings)
- [Descriptor Schema](#descriptor-schema)
- [Local State and Concurrency](#local-state-and-concurrency)
- [Durable Identity and Schema Cutover](#durable-identity-and-schema-cutover)
- [Copied-State Rebinding](#copied-state-rebinding)
- [Reserved Metadata](#reserved-metadata)
- [Results, Documentation, and Tests](#results-documentation-and-tests)
- [Performance](#performance)

## Project Context and Command Dispatch

**Decision**: Introduce one `ProjectContext` containing the canonical project root, `.grip` metadata directory, descriptor path, state root, canonical invoking-user home, and revalidation evidence. Resolve it once in central dispatch and pass it through every project-dependent subsystem and result finalizer.

**Rationale**: The current `GripHome` is selected independently by handlers and again after execution for operation-result delivery. Independent rediscovery can select different authority after working-directory or metadata drift. A project also needs both its source root and destination-home binding.

**Alternatives rejected**: Repointing `GripHome` at `.grip` preserves misleading global terminology and encourages callers to infer the project root. Resolving inside each subsystem creates inconsistent fallback and revalidation behavior. Retaining `GRIP_HOME` as an alias violates the clean cutover.

## Initialization and Metadata Safety

**Decision**: `grip init [PATH]` accepts only an existing, current-user-owned, accessible ordinary directory whose final component is not a symlink and which is not beneath another Grip project. It stages complete metadata in a private unique directory inside the target, creates and syncs Descriptor V2 and `.gitignore`, then atomically renames the staging directory to `.grip` without replacement and syncs the target.

The canonical `.gitignore` bytes are exactly `/state/\n`. Existing complete metadata with the canonical descriptor and exact ignore file is a no-op. Any partial layout, additional ignore content, unsafe node, unsupported schema, or non-equivalent bytes blocks without repair or replacement. A concurrent winner is reloaded and is either equivalent or a conflict.

**Rationale**: Staging makes the visible transition complete without creating state. Exact Grip-owned metadata avoids implementing a Git-ignore semantic merger. Ordinary clone modes remain valid: metadata directories may be `0755` and committed files `0644` when current-user-owned, non-symlink, and not group/world writable. State retains strict `0700` directory and `0600` file rules.

**Alternatives rejected**: Direct two-file writes expose partial initialization. Repair risks overwriting unrelated content. Owner-only committed metadata would reject ordinary clones. Git operations exceed Grip's responsibility.

## Project Discovery

**Decision**: Explicit `--project PATH` resolves relative input against the invocation directory, requires that exact existing directory to contain valid metadata, and never walks upward or falls back. Implicit discovery canonicalizes the invocation directory and inspects every ancestor through the filesystem root.

Any ancestor containing `.grip` is a candidate boundary. Unsafe, incomplete, malformed, or unsupported candidate metadata blocks rather than being skipped. Zero candidates produces `project_not_found`; more than one valid candidate produces `ambiguous_project`; exactly one valid candidate is selected. Identity and descriptor-byte evidence are retained for mutation revalidation.

**Rationale**: Exhaustive inspection implements the deliberate ambiguity rule and prevents a broken inner boundary from silently redirecting a command outward.

**Alternatives rejected**: Nearest-project selection contradicts the specification. Considering only valid descriptors could cross malformed metadata. Allowing an explicit descendant turns authority selection into another discovery mode.

## Portable Paths and Mappings

**Decision**: Separate `PortableMapping` from `ResolvedMapping`. A portable source is a normalized project-relative path. A portable destination is exactly `~` or normalized `~/RELATIVE_PATH`. Runtime resolution binds both to the selected project and canonical home, captures non-following ancestry evidence, and then applies ownership checks to absolute endpoints.

A tree mapping may use exact `.` as its source; file mappings may not. All other empty, dot, parent, repeated, or trailing components are invalid. Absolute paths, `${HOME}`, all environment expressions, `~user`, and non-UTF-8 descriptor strings are invalid. Selectors use the same source or destination grammar.

**Rationale**: Portable values are stable across clones. Resolved values are filesystem observations, not durable intent. Distinct types prevent accidental serialization of absolute paths.

**Alternatives rejected**: One type for storage, runtime, and durable identity caused the current coupling. Storing both forms as identity remains location-dependent. Shell expansion introduces ambiguous environment-dependent configuration.

## Descriptor Schema

**Decision**: `.grip/config.toml` uses strict Descriptor V2 with `schema_version = 2` and deterministic portable mappings ordered by source, kind, and destination. Descriptor V1 is unsupported at the project path.

**Rationale**: The grammar and authority boundary are incompatible with Registry V1. A version bump prevents an absolute registry from being reinterpreted. The unreleased product does not need migration code.

**Alternatives rejected**: Reusing version 1 silently changes field meaning. Dual reading preserves the model being removed.

## Local State and Concurrency

**Decision**: Every mutable artifact lives below `.grip/state/`: accepted state, locks, staging, operations, and recovery. State is created lazily by an actual writer. Read-only and dry-run commands create nothing and take no writer lock.

One nonblocking project mutation lock serializes config, state, recovery, operation, and payload writers. Narrower registry and state publication locks retain their existing roles and deterministic order after the mutation lock. Config staging occurs under `.grip/state/staging/` on the same filesystem as the descriptor.

**Rationale**: The ignored state tree becomes the complete local-instance boundary, prevents mutable files from being committed, and allows unrelated projects to operate concurrently.

**Alternatives rejected**: Locks beside config violate the committed/local separation. A global lock recreates cross-project contention. Eager state creation violates init and read-only contracts.

## Durable Identity and Schema Cutover

**Decision**: Durable `EntryIdentity` contains the portable mapping tuple plus lossless entry-relative bytes. Filesystem paths are derived only through the current context. State Envelope V4, Operation Record V2, Mutation Recovery V2, Recovery Manifest V2, and Recovery Metadata V3 replace every current format whose identity or restoration authority contains absolute mapping roots or targets. Result Envelope V1 remains stable because its details are presentational.

**Rationale**: State V2/V3 and Recovery Metadata V2 embed absolute `MappingSnapshot` values; operation plans and recovery manifests also carry resolved targets. Reinterpretation would be a silent semantic schema change. A project ID would not protect a filesystem copy because ID and state copy together.

**Alternatives rejected**: Prefix substitution cannot prove retained meaning. Keeping old readers creates prohibited compatibility behavior. Storing absolute and portable identity together makes equality location-dependent.

## Copied-State Rebinding

**Decision**: State V4 records a local binding containing the prior canonical project root, canonical home, accepted descriptor digest, and resolved mapping-set digest. A mismatch marks retained state untrusted; it does not trigger path rewriting.

Before state can authorize acceptance or mutation, Grip revalidates project and descriptor evidence, resolves every portable mapping, matches persisted identities to the descriptor or explicit pending-retirement evidence, reobserves all accepted entries required by the command, validates recovery and operation targets, and compares the result with accepted state. Read-only and dry-run commands report eligibility without writing. A successful state-writing operation records the current binding in its final publication. Missing, ambiguous, stale, incomplete, incompatible, or contradictory evidence blocks.

**Rationale**: The binding detects relocation while portable identity permits safe re-resolution. Delayed publication preserves read-only behavior; complete validation still occurs before payload mutation.

**Alternatives rejected**: Config digest alone misses home and recovery changes. Rewriting during status violates read-only guarantees. Discarding copied state loses useful evidence.

## Reserved Metadata

**Decision**: `.grip` is structurally reserved. Mapping validation rejects file mappings at or below it; traversal for project-root tree mapping prunes it before `.gripignore`; selectors and revalidation cannot reintroduce it. Topology checks reject destination or cross-mapping overlap with project metadata or state.

**Rationale**: Ignore policy is not a security boundary. Structural exclusion guarantees metadata never becomes payload while retaining the broad project-root tree use case.

**Alternatives rejected**: A default ignore can be weakened by policy handling. Forbidding safe root-tree mappings unnecessarily limits dotfiles layouts.

## Results, Documentation, and Tests

**Decision**: Decorate every outcome after successful selection with one safe project root in human and JSON details. Use that same context for result delivery. Add stable project and rebinding reasons within existing categories and exit codes.

Update runtime code, `README.md`, `docs/product-definition.md`, current wiki claims through ingestion, and active tests. Preserve Features 001–009 and their reviews. Replace the shared test helper with isolated project/home builders, cwd-based and explicit invocation helpers, portable fixtures, and a populated legacy `GRIP_HOME` trap.

**Rationale**: Fifty-three of sixty-two test files currently depend on global-home helpers or fixtures, so centralized conversion reduces hidden compatibility assumptions. Historical artifacts remain evidence, not current instructions.

**Alternatives rejected**: Free-form project errors weaken automation. Rediscovery for output can select another project. Blind repository-wide replacement would corrupt historical provenance.

## Performance

**Decision**: Extend the ignored release harness with a 100-level ancestor fixture and 100 samples recording p50, p95, maximum, build profile, revision, and host evidence. The target is p95 at or below one second. Add no persistent cache, index, parallel walk, or background service unless measurement shows a miss.

**Rationale**: Discovery is bounded by path depth and expected to be cheap. Measurement preserves evidence-before-optimization.

**Alternatives rejected**: Persistent indexes require invalidation before the simple walk has demonstrated a problem.
