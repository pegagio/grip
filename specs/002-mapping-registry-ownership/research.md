# Research: Mapping Registry and Ownership Validation

This research resolves the Feature 002 implementation choices without expanding into discovery or payload synchronization.

## Table of Contents

- [Command model](#command-model)
- [Registry wire format](#registry-wire-format)
- [Path canonicalization](#path-canonicalization)
- [Ownership topology](#ownership-topology)
- [Registry publication](#registry-publication)
- [Concurrency](#concurrency)
- [Errors and output](#errors-and-output)
- [Testing and performance](#testing-and-performance)
- [Dependency boundary](#dependency-boundary)

## Command model

**Decision**: Add a `mapping` command group with `add file SOURCE DESTINATION`, `add tree SOURCE DESTINATION`, `list`, `show SOURCE`, and `remove SOURCE`. All path positions use the conventional `--` option terminator when needed. Parsing converts immediately into Grip-owned command values.

**Rationale**: The noun-first hierarchy groups one lifecycle without consuming top-level names needed by later synchronization operations. Separate file and tree variants make kind-specific source validation explicit, while list/show/remove use canonical source identity consistently. `clap` already supplies nested subcommands and operating-system-string paths.

**Alternatives considered**: Top-level `track` and `untrack` match older product prose but obscure the distinct read-only inspection operations. `mapping add --kind` makes an invalid or missing kind a loosely typed value. User-assigned IDs conflict with the accepted canonical-source identity.

## Registry wire format

**Decision**: Extend registry schema version 1 with ordered `[[mappings]]` tables containing exactly `kind`, `source`, and `destination`. Decode through strict version-specific Serde types, convert to domain mappings, sort by canonical source bytes, and publish with `toml::to_string_pretty`. Mapping updates preserve supported semantic fields but normalize comments and presentation-only ordering.

**Rationale**: This evolves the already accepted `mappings = []` document without a schema bump and keeps unknown-field rejection. Complete deterministic serialization makes repeated equivalent updates byte-identical and supports stale-evidence comparison. The existing TOML crate directly serializes typed documents through Serde ([TOML serialization](https://docs.rs/toml/latest/toml/ser/index.html), [`to_string_pretty`](https://docs.rs/toml/latest/toml/fn.to_string_pretty.html)).

**Alternatives considered**: Adding `toml_edit` solely to preserve comments introduces another direct dependency before a demonstrated requirement. Text splicing around TOML arrays is fragile. A second registry file or schema version is unnecessary because Feature 001 explicitly reserved the empty mapping collection for this evolution.

## Path canonicalization

**Decision**: Require absolute UTF-8 inputs. Lexically reject parent traversal, then use non-following metadata to reject a final source or existing destination symlink and validate the required node kind. Canonicalize the longest existing prefix, append any absent destination suffix after validating every lexical component, and retain both the submitted path for diagnostics and the canonical absolute path for identity. Immediately before registry publication, resolve and validate the relevant paths again and require the canonical identities and node kinds to match the planned candidate.

**Rationale**: Rust canonicalization returns an absolute normalized path but resolves symbolic links, while `symlink_metadata` inspects a link itself without following the final component ([`std::fs`](https://doc.rust-lang.org/stable/std/fs/), [`Metadata`](https://doc.rust-lang.org/stable/std/fs/struct.Metadata.html)). Combining them permits ordinary platform ancestry such as macOS `/var` while rejecting a mapping root that is itself a link. Revalidation makes an ancestry substitution a stale-evidence error instead of accepting a changed target.

**Alternatives considered**: Rejecting every intermediate symbolic-link component would reject common platform paths and isolated test roots. Pure lexical normalization would not collapse filesystem aliases. Descriptor-relative traversal would provide a stronger mutation boundary but is disproportionate for recording intent and can be reconsidered when payload mutation begins.

## Ownership topology

**Decision**: Represent every mapping as two namespaces with `Exact` or `Tree` extent. Compare every unordered mapping pair plus each mapping's own source/destination relation. Reject equal paths; exact-in-tree and tree containment overlaps on the same side; source-to-destination cross-overlap in either direction; and every relationship that could let future source discovery encounter a destination namespace. Return all deterministic conflicts, ordered by canonical paths, rather than stopping at the first pair.

**Rationale**: Mapping counts are small and the accepted scale is 1,000, making an auditable quadratic comparison preferable to a path index. Validating the whole graph prevents a future source addition from creating ambiguity in an unchanged registry. Collecting all conflicts supports complete preflight without exposing payload content.

**Alternatives considered**: Prefix-string comparison is incorrect at component boundaries. A trie or interval index adds invalidation and ordering complexity without measured need. Only checking the candidate against mappings misses corruption already present in the accepted registry.

## Registry publication

**Decision**: Under the registry lock, reread `config.toml`, validate its owner, final node type, permissions/accessibility, schema, and complete topology, and require its bytes to equal the inspected snapshot. A readable accepted registry must be current-user-owned with group/other write bits unset; add and remove also require its owner-write bit. Before replacement, publish and verify the exact prior bytes at `state/recovery/registry/sha256-<digest>/config.toml`, reusing an existing byte-identical generation and rejecting a conflict. Then serialize the complete candidate, exclusively create a `0600` staging file beside `config.toml`, write and sync it, reread and decode it, compare the decoded candidate, apply the exact accepted registry mode, rename it over `config.toml`, and sync the Grip-home directory where supported.

**Rationale**: Same-directory rename cannot cross mount points and replaces the accepted name as one operation ([`std::fs::rename`](https://doc.rust-lang.org/stable/std/fs/fn.rename.html)). Exclusive staging prevents collision, verification catches incomplete or incorrect serialization, and byte comparison prevents an accepted concurrent edit from being overwritten. Content-addressed immutable recovery satisfies the constitutional replacement boundary without inventing restoration policy or duplicating identical prior versions. Exact mode preservation avoids silently widening access. The guarantee is old-or-new visibility on supported local filesystems, not global transactionality or universal sudden-power-loss durability.

**Alternatives considered**: In-place truncation exposes partial TOML. A system temporary directory may cross filesystems. A timestamp-only recovery name is collision-prone and duplicates identical documents. Automatically changing an unsafe existing registry mode would violate least surprise. Restoration and cleanup commands remain deferred because recovery evidence can be retained safely without prematurely defining those workflows.

## Concurrency

**Decision**: Create or open one stable `<GRIP_HOME>/.registry.lock` regular file with mode `0600`, current-user ownership, and no mapping data. Acquire a nonblocking exclusive `std::fs::File::try_lock` for the reread/revalidation/publication section; keep the file after release and never interpret existence as contention.

**Rationale**: Standard-library file locks are released with the file descriptor and report `WouldBlock` without polling ([`File`](https://doc.rust-lang.org/stable/std/fs/struct.File.html)). A stable inode avoids split-lock races around replacement. This duplicates only the small generic safety rules of the existing state lock; tasking may extract a narrowly shared helper if doing so actually reduces duplication without coupling registry and state policy.

**Alternatives considered**: Locking `config.toml` is unsafe because atomic replacement changes the locked inode. Creating and deleting a sentinel lock requires crash recovery and can split contenders. A blocking lock hides contention. Locking the payload tree is outside the product boundary.

## Errors and output

**Decision**: Keep existing result-envelope fields and exit meanings. Syntax errors remain `invalid_usage`/2; invalid mappings, unsafe paths, not-found mapping identity, and invalid existing registry remain `invalid_configuration`/10; unsupported schema remains 11; unsafe machine-owned registry coordination nodes use `corrupt_state`/12; lock contention and I/O/publication failure use `operational_failure`/20. Add stable mapping detail objects and reason values defined in the CLI contract.

**Rationale**: Feature 001 deliberately established broad public categories that can carry structured details. New exit codes are not needed to distinguish mapping conditions because automation receives stable operation and reason fields.

**Alternatives considered**: A new exit for every topology relation would make scripts brittle. Treating a missing mapping as success would hide operator mistakes. Embedding canonical fields only in messages would force prose parsing.

## Testing and performance

**Decision**: Unit-test path relation and mapping ordering as pure behavior. Use temporary-root integration tests for real canonicalization, non-following node checks, complete-registry validation, CLI contracts, lock contention, byte-staleness, staged-publication faults, mode preservation, and zero payload/state mutation. Add an ignored release harness with 1,000 non-overlapping mappings and exactly 100 warm validation/list iterations.

**Rationale**: Filesystem behavior must be observed on actual supported filesystems, while pure relation tests provide dense combinatorial coverage. The existing test support and performance harness already record environment and isolate `GRIP_HOME`, so extending them avoids a benchmark framework.

**Alternatives considered**: Mock-only path tests cannot prove symlink or rename behavior. Running performance acceptance in every debug test would be noisy. Parallel validation is unjustified before the simple implementation is measured.

## Dependency boundary

**Decision**: Add no runtime or development dependency. Use existing `clap`, Serde, TOML, `thiserror`, `rustix`, and `tempfile`, plus Rust 1.98 standard-library path, lock, sync, and rename facilities.

**Rationale**: Every required capability already exists in the accepted dependency set. Avoiding a format-preserving editor is an explicit tradeoff documented in the specification and storage contract.

**Alternatives considered**: `toml_edit` could preserve comments but adds maintenance surface for a behavior not currently required. A path-normalization crate would not replace filesystem-specific validation and revalidation.
