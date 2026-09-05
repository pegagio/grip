# Research: Source Discovery and Gripignore

This research resolves Feature 003 implementation choices without expanding into baselines, synchronization classification, or mutation.

## Table of Contents

- [Public discovery surface](#public-discovery-surface)
- [Gitignore-compatible policy engine](#gitignore-compatible-policy-engine)
- [Descriptor-relative traversal](#descriptor-relative-traversal)
- [Filesystem classification](#filesystem-classification)
- [Inventory construction](#inventory-construction)
- [Stale-evidence detection](#stale-evidence-detection)
- [Errors and output](#errors-and-output)
- [Testing and performance](#testing-and-performance)

## Public discovery surface

**Decision**: Add `grip mapping inspect [SOURCE]`. With no selector it covers every accepted mapping; with one source selector it resolves the same canonical mapping identity as `mapping show`. It does not accept a destination selector, nested-entry selector, or more than one selector. The `inspect` verb distinguishes this ephemeral, read-only inventory from `mapping add`, which persists mapping intent, and `mapping list`, which lists registered mappings.

**Rationale**: Discovery inspects membership owned by mappings, so it fits the established mapping command family without prematurely claiming the `status`, `check`, or `diff` classification semantics owned by Feature 004. The optional source preserves the feature's all-mappings and one-mapping flows and reuses an accepted identity contract.

**Alternatives considered**: A top-level `discover` command would widen the top-level product vocabulary for a capability tied directly to mapping ownership. Extending `mapping show` would overload mapping intent with a potentially large dynamic inventory. Introducing `status` early would conflate membership with baseline-informed synchronization state.

## Gitignore-compatible policy engine

**Decision**: Add `ignore = "0.4.33"` and use `ignore::gitignore::{GitignoreBuilder, Gitignore}` only as the pattern engine. Grip reads each exact `.gripignore` through its no-follow filesystem adapter, requires a regular UTF-8 policy file, strips an initial UTF-8 BOM, accepts CRLF and a final unterminated line, adds each line with its policy path, and rejects any read, parse, or build error instead of accepting partial rules. A matcher stack follows the directory ancestry; the deepest non-neutral match wins, while each matcher already applies its own last-rule precedence. Grip structurally excludes every `.gripignore` before payload matching.

**Rationale**: The crate implements the required Gitignore grammar and has an MSRV below Grip's Rust 1.98 contract. Matching-only use prevents accidental activation of `.gitignore`, `.ignore`, parent repository, global exclude, or hidden filtering and lets Grip report ignored entries explicitly. See the [`GitignoreBuilder` documentation](https://docs.rs/ignore/0.4.33/ignore/gitignore/struct.GitignoreBuilder.html), [crate metadata](https://docs.rs/crate/ignore/0.4.33/source/Cargo.toml), and [Git's normative ignore rules](https://git-scm.com/docs/gitignore).

**Alternatives considered**: `WalkBuilder` can disable standard filters and add a custom filename, but its public walker drops ignored entries before client callbacks and therefore cannot produce the required ignored-path inventory; its traversal also obscures Grip's evidence boundary. A custom pattern parser avoids a dependency but creates a large compatibility and maintenance burden for no product advantage. `IncrementalIgnore` warns against use for directory traversal and adds caching that complicates stale-policy evidence.

## Descriptor-relative traversal

**Decision**: Implement a sequential depth-first adapter with the existing `rustix` `fs` capability. Open child directories relative to their parent descriptors with read-only, close-on-exec, directory, and no-follow flags; enumerate with `rustix::fs::Dir`; and inspect each child relative to that descriptor without following links. Retain only the current ancestry descriptor stack and sort raw child-name bytes before processing.

**Rationale**: Existing pathname-based endpoint validation safely establishes mapping roots, while descriptor-relative child access prevents an intermediate directory replacement from redirecting subsequent inspection through a symlink. The approach adds no dependency, keeps resource use bounded by depth, and remains smaller than a general filesystem abstraction. See [`openat`](https://docs.rs/rustix/1.1.4/rustix/fs/fn.openat.html), [`statat`](https://docs.rs/rustix/1.1.4/rustix/fs/fn.statat.html), and [`Dir`](https://docs.rs/rustix/1.1.4/rustix/fs/struct.Dir.html).

**Alternatives considered**: `std::fs::read_dir` plus `symlink_metadata` protects only the final component and permits parent swaps between calls. Holding every descriptor until completion risks descriptor exhaustion. A watcher, broad tree lock, or filesystem snapshot exceeds the local read-only boundary and still would not provide a portable product guarantee.

## Filesystem classification

**Decision**: After ignore evaluation, classify from non-following metadata in this order: symbolic link; socket/FIFO/character device/block device/whiteout/unknown special node; directory on a device different from the source root; regular file with link count greater than one; regular file whose reported allocated blocks are fewer than its logical size requires; otherwise ordinary file or directory. Report non-UTF-8 names as unsupported with an escaped display and raw-byte hex identity. Never open, read, hash, or traverse a node after it is classified as unsupported.

**Rationale**: Ordered allowlist classification ensures objects that may look file-like cannot enter the eligible set. Unix metadata exposes device, mode/type, link count, size, allocated 512-byte blocks, and timestamps without payload reads. macOS/BSD whiteout classification is conditional; unsupported platforms map unrecognized types to `unknown_special`. See Rust [`MetadataExt`](https://doc.rust-lang.org/std/os/unix/fs/trait.MetadataExt.html), Unix [`OsStrExt`](https://doc.rust-lang.org/std/os/unix/ffi/trait.OsStrExt.html), and Apple's [`stat(2)` contract](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/stat.2.html).

**Alternatives considered**: Opening files and using hole-seeking calls may distinguish sparse allocation more precisely but violates the metadata-only inspection boundary and is less portable. Treating every regular-file mode as supported would silently admit hard-linked or sparse representations that later copying cannot preserve. Exercising real device or whiteout creation in ordinary tests would require inappropriate privileges.

## Inventory construction

**Decision**: Build one complete in-memory inventory ordered by canonical mapping source and source-relative raw bytes. File mappings produce one eligible record without parent traversal. Tree mappings walk source membership and policy, then inspect each paired destination without following it. A separate non-following destination walk records paths absent from the eligible-source set as `destination_only`; it prunes destination symlinks, special nodes, and nested mounts after recording them. Ignored files are recorded individually; an ignored directory is recorded as the exclusion root and its unseen descendants are not fabricated. The inventory is never persisted.

**Rationale**: This preserves source-defined ownership while making both exclusions and overlay content visible. A set keyed by mapping plus relative bytes makes the eligible/destination-only distinction deterministic without introducing baseline meaning. Pruned-subtree reporting is truthful about what was inspected.

**Alternatives considered**: Reporting only eligible entries would miss explicit roadmap outcomes. Recursing through ignored or unsupported directories would violate policy and safety. Persisting the inventory would introduce invalidation, recovery, and schema obligations before Feature 004 needs durable state.

## Stale-evidence detection

**Decision**: Capture ephemeral evidence for registry bytes/identity, nodes, directories, and policy files, then perform the same complete pass again before returning. Node evidence includes side, mapping identity, relative raw bytes, device, inode, raw mode, link count, size, allocated blocks, modification/change timestamps, and for directories the sorted child-name bytes. Policy evidence also includes exact bytes and a digest. Return results only when records and evidence match; otherwise fail with `operational_failure` and `stale_discovery_evidence`. Evidence is never serialized or treated as persistent path identity.

**Rationale**: Two-pass comparison detects changes that could make a purportedly complete inventory inconsistent without claiming snapshot isolation or blocking unrelated applications. Reusing the accepted registry snapshot and exposing a read-only revalidation helper avoids taking the publication lock.

**Alternatives considered**: A single pass cannot know that earlier directories or policies changed. Per-entry immediate restats miss changes after the recheck. Persistent inode indexes, watchers, or tree locks add lifecycle and correctness complexity forbidden without demonstrated need.

## Errors and output

**Decision**: A complete inventory returns the existing top-level `ok` result and exit `0` even when records include blockers; `blocking_count` makes that state explicit. Invalid registry, selector, malformed/non-UTF-8 policy, or structurally invalid evidence uses `invalid_configuration`/10. Unsupported registry schemas retain exit 11. Read/stat/enumeration failures, unreadable policy, and stale discovery evidence use `operational_failure`/20. JSON uses stable record categories and path objects with escaped display plus optional raw hex; human output is deterministic and diagnostics remain on stderr.

**Rationale**: Unsupported entries are discovered facts, not failures to inspect. Keeping the established broad exit categories prevents scripts from depending on prose while preserving room for Feature 004 to define attention-oriented status exits.

**Alternatives considered**: Returning exit 10 for every unsupported record would prevent users from receiving one complete inventory. Adding a new attention exit now would preempt Feature 004's automation contract. Lossy-only paths could collapse distinct non-UTF-8 names.

## Testing and performance

**Decision**: Use pure unit tests for matching precedence, ordering, safe-path encoding, and raw-mode classification. Use isolated temporary-root integration tests for nested policy, authority poisoning, destination overlay, symlinks, hard links, sparse files, FIFO/socket fixtures, zero mutation, and CLI envelopes. Use synthetic classification tests for privileged device and whiteout types; conditionally skip real sparse/mount fixtures when the platform cannot prove the precondition. Add a test-only pass barrier for deterministic stale-add/remove/replace/policy-change cases. Extend the ignored release harness to measure exactly 100 warm two-pass discoveries over a documented 10,000-entry fixture.

**Rationale**: This proves behavior on real supported filesystems without touching developer data or requiring elevated privileges. The required measurement establishes whether sequential traversal is already sufficient before considering parallelism or caching.

**Alternatives considered**: Mock-only tests cannot prove no-follow or filesystem metadata behavior. Mandatory privileged fixtures would make ordinary validation unreliable. Adding a benchmark framework or parallel walker before measuring the real path would violate proportional rigor.
