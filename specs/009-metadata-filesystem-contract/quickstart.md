# Quickstart: Metadata and Filesystem Contract Completion

This quickstart is the implementation and acceptance walkthrough for Feature 009. It uses disposable roots and a disposable `GRIP_HOME`; it must never target a developer's real files. Exact CLI setup syntax should follow the repository's existing test helpers and command contracts.

## Prerequisites

- Current macOS running on APFS
- Rust toolchain installed through the repository's `mise.toml`
- A debug or release `grip` binary built from Feature 009
- Permission to create isolated test roots and, for the full matrix, both case-insensitive and case-sensitive APFS test volumes or images
- Explicit approval before adding a direct `libc` dependency to `Cargo.toml`

Record the host evidence before acceptance: macOS version/build, Darwin kernel, APFS bundle version, filesystem type, mount flags, volume capability masks, binary revision, and build profile.

## 1. Create an Isolated Workspace

```bash
fixture_root="$(mktemp -d /private/tmp/grip-009.XXXXXX)"
export GRIP_HOME="$fixture_root/grip-home"
mkdir -p "$fixture_root/source/tree" "$fixture_root/destination/tree" "$GRIP_HOME"
```

Register mappings using the existing Grip mapping workflow. Keep every path beneath `fixture_root` and verify `grip --output=json mapping inspect` before continuing.

## 2. Verify a File Metadata-Only Push

Create identical content on both sides, accept the initial state, then change one supported source field at a time: mode, UID/GID when an authorized transition is available, nanosecond mtime, each allowlisted xattr, an ordered ACL, and each supported BSD flag.

For each field:

```bash
grip --output=json status
grip --output=json diff
grip --output=json push --dry-run
grip --output=json push
grip --output=json status
```

Verify that the first status identifies exactly the changed dimension, dry-run names the complete transition without modifying either root, execution reproduces and verifies the complete state, and final status is synchronized with a new State V3 generation.

## 3. Verify a Directory Metadata Pull

Create a nested directory tree, accept it, then change supported metadata only on a destination directory. Change a child during the same scenario so execution must finalize directory metadata after descendant actions.

```bash
grip --output=json pull --dry-run
grip --output=json pull
grip --output=json status
```

Confirm the plan orders child actions before directory finalizers and finalizers deepest-first. Verify the selected directory's mtime and every other supported field after execution.

## 4. Verify Convergence and Complete-Entry Conflict

Apply the same metadata change independently to both copies and verify `converged_two_sided_change`. Then accept that state, change content or metadata differently on each side, and verify `divergent_conflict`.

Resolve once in each direction:

```bash
grip --output=json resolve path/to/entry --source --dry-run
grip --output=json resolve path/to/entry --source
grip --output=json resolve path/to/entry --destination --dry-run
```

Confirm that a winner always supplies content plus every supported metadata field. No result may combine source content with destination metadata or vice versa.

## 5. Verify State V2 Migration

Load a valid strict State V2 fixture under the disposable `GRIP_HOME`.

- With equal complete current copies, run status and confirm `metadata_migration_ready`; confirm no file under `GRIP_HOME` changes. Run the existing explicit `grip baseline accept` workflow and verify State V3 publication.
- Restore the V2 fixture, make the copies differ only in metadata, and confirm `metadata_migration_conflict`. Verify baseline acceptance is rejected. Resolve with `--source`, then repeat with `--destination`, and verify the losing complete entry is recoverable before State V3 publishes.

## 6. Verify Xattr Policy

Exercise each allowlisted xattr with absent, empty, small, and representative large values. Include a resource fork. Verify exact byte equality through length and SHA-256 evidence and through post-apply reads.

Add each excluded attribute and confirm it is named in inspection but does not change equality or block. Add an arbitrary attribute not on either list and confirm status identifies it as unknown and every mutating dry-run is blocked before action creation.

## 7. Verify ACL Ordering and BSD Flags

Create ACLs containing multiple allow and deny ACEs plus inheritance flags. Verify equivalent permission-bit enumeration does not create a difference, while reordered ACEs do. Round-trip the ACL in both directions and confirm principal UUIDs and sequence.

Exercise `UF_NODUMP`, `UF_IMMUTABLE`, `UF_APPEND`, and `UF_HIDDEN` on files and directories, plus `UF_OPAQUE` on directories. Confirm immutable and append clearing is visible in preflight and final flags are applied last. Confirm protected, synthetic, excluded, and unknown flag differences block rather than being cleared or copied.

## 8. Verify Capabilities and Unsupported Nodes

Create isolated fixtures for:

- Unauthorized UID and GID transitions
- Unavailable ACL or xattr application capability
- A symbolic link whose target would reveal unintended following
- A multiply linked regular file
- A file with the authoritative sparse extended flag
- A socket, FIFO, character device where safely available, and unknown node fixture
- A nested mount below a tree root
- An explicit mapping root placed on another APFS volume

For every managed blocker, run status and a mutating dry-run and verify precise endpoint, path, property, evidence state, required behavior, and reason. Confirm no unsupported payload is opened, copied, preserved as an ordinary file, or changed. Repeat representative nodes as destination-only unmanaged content and confirm they remain untouched and nonblocking.

## 9. Verify APFS Name Compatibility

Run the collision matrix on both case-sensitive and case-insensitive APFS targets. Include case-only distinct names and canonically equivalent Unicode spellings. Confirm exact path bytes remain unchanged in output and state.

Distinct managed identities that alias on the target must be reported together and block. A qualification fixture that does not match the implementation's comparator must report unavailable comparison capability rather than accepting the mapping.

## 10. Verify Drift, Recovery, and Output Parity

Inject changes after planning but before each metadata application step: node replacement, ownership change, xattr change, ACL change, flag change, link-count change, sparse transition, mount change, and accepted-generation change. Verify the action stops with precise partial-failure evidence and never publishes successful accepted state.

Inject failures after recovery preservation and after publication milestones. Restore from Recovery Metadata V2 and verify exact content plus complete metadata. Confirm Recovery Metadata V1 remains readable under its original contract.

For every result fixture, compare human and JSON renderings and confirm they communicate equivalent classifications, dimensions, findings, actions, recovery authority, verification, durability, and accepted-state authority.

## 11. Run Validation

```bash
mise run validate
```

Run macOS/APFS-specific ignored tests explicitly using the test names established during implementation. Record environmental skips as unsupported or unavailable cases, never as passes.

## 12. Run the Performance Qualification

Build the documented 10,000-entry representative tree and run the extended ignored harness in release mode. Capture 100 status samples and 100 dry-run samples after any documented warm-up, and publish p50, p95, maximum, fixture composition, host evidence, and binary revision.

```bash
mise run performance
```

Acceptance requires at least 95 of 100 invocations in each read-only workflow to finish within two seconds. Measure a representative synchronization workload separately. If the threshold is missed, profile the sequential implementation and report the shortfall before proposing caches, parallelism, or persistent indexes.

## Qualification Evidence

The Feature 009 qualification was run on macOS 26.6.2 build 25G83, Darwin 25.6.0 (`RELEASE_ARM64_T6030`), APFS bundle 2811.160.7, and arm64. The case-insensitive qualification root reported APFS mount flags `76583040` and volume capability masks `[576460748035419871, 144115183809163207, 0, 0]`. The release binary was built from the uncommitted Feature 009 working tree whose base revision was `fd98266127587b2fd1cff35eeb9bdfefaa8fa09d`; the base revision alone therefore does not identify the qualified source delta.

The qualified complete-state contract covers regular-file content plus file and directory node kind, full permission mode, numeric UID/GID, nanosecond mtime, ordered extended ACLs, and the synchronized xattrs `com.apple.FinderInfo`, `com.apple.ResourceFork`, and `com.apple.TextEncoding`. Supported user BSD flags are `UF_NODUMP`, `UF_IMMUTABLE`, `UF_APPEND`, and `UF_HIDDEN` for files and directories, plus `UF_OPAQUE` for directories. The exercised APFS endpoints reproduced nanosecond mtime values exactly. Security, provenance, volatile, and system-maintained xattrs remain named diagnostics; attributes outside both explicit lists block.

The explicit APFS matrix used a disposable case-sensitive APFS disk image alongside the host's case-insensitive APFS temporary volume. Both product matrices passed the core lifecycle plus mapping and baseline flows, one-sided and converged classification, conflict resolution, deletion and recovery, retirement, contention, injected partial failure, State V2 migration, capability blocking, and unsupported-node handling. The cross-volume complete-state transfer and case-only collision cases also passed. The cross-volume fixture required its source group to match the destination volume's observed group; without that authorization-compatible setup, Grip correctly blocked the transition instead of substituting ownership or elevating privileges.

The explicit qualification commands were:

```bash
GRIP_APFS_CASE_INSENSITIVE_ROOT=/private/tmp \
GRIP_APFS_CASE_SENSITIVE_ROOT=/private/tmp/grip-feature-009-case-sensitive \
cargo test --test product_acceptance -- --ignored --nocapture

GRIP_APFS_CASE_INSENSITIVE_ROOT=/private/tmp \
GRIP_APFS_CASE_SENSITIVE_ROOT=/private/tmp/grip-feature-009-case-sensitive \
cargo test --test metadata_capability_integration cross_volume_profiles_prove_logical_metadata_capability_without_layout_promises -- --ignored --nocapture

GRIP_APFS_CASE_INSENSITIVE_ROOT=/private/tmp \
GRIP_APFS_CASE_SENSITIVE_ROOT=/private/tmp/grip-feature-009-case-sensitive \
cargo test --test apfs_name_compatibility_integration case_only_aliases_report_every_identity_before_destination_mutation -- --ignored --nocapture
```

Their dispositions were three of three expanded product-acceptance cases passed, one of one cross-volume capability case passed, and one of one case-only collision case passed.

The first 100-sample release run exposed a real performance miss: sync dry-run measured p50 2.001 seconds, p95 2.036 seconds, and maximum 2.170 seconds. Status remained within the gate at p95 1.965 seconds, and the other dry-run and plan distributions passed. The correction removed a redundant second payload hash used only to reconstruct the legacy compatibility fingerprint from information already present in the complete observation, retained operation-local endpoint-profile reuse, and removed duplicate state from result serialization. The final distributions below supersede this diagnostic run.

The final 100-sample release qualification passed every asserted threshold. The fixture contained 10,000 managed entries across 100 directories with representative modes, nanosecond mtimes, `com.apple.TextEncoding`, `com.apple.ResourceFork`, and `UF_NODUMP`; the mixed sync fixture included one-sided, converged, and conflicting changes.

| Workflow | p50 | p95 | Maximum |
|---|---:|---:|---:|
| Help | 3.011 ms | 4.179 ms | 9.710 ms |
| Version | 3.005 ms | 3.377 ms | 4.076 ms |
| Validate | 383.066 ms | 394.533 ms | 401.604 ms |
| Mapping list, 1,000 mappings | 381.080 ms | 394.939 ms | 401.673 ms |
| Discovery, 10,000 entries | 138.319 ms | 141.312 ms | 147.128 ms |
| Status, accepted paired 10,000 entries | 1.518 s | 1.610 s | 1.650 s |
| Push dry-run, 10,000 entries | 1.277 s | 1.304 s | 1.352 s |
| Push execute-mode plan, 10,000 entries | 1.123 s | 1.221 s | 1.353 s |
| Pull dry-run, 10,000 entries | 1.465 s | 1.525 s | 2.036 s |
| Pull execute-mode plan, 10,000 entries | 1.290 s | 1.335 s | 1.381 s |
| Sync dry-run, mixed 10,000 entries | 1.514 s | 1.540 s | 1.668 s |
| Sync execute-mode plan, mixed 10,000 entries | 1.303 s | 1.365 s | 1.600 s |
| Delete preview, 10,000 entries | 1.391 s | 1.426 s | 1.495 s |
| Recovery inventory, 10,000 entries | 1.349 s | 1.366 s | 1.375 s |

The separately measured synchronization of 10 changes within a 10,000-entry tree completed in 28.718 seconds; the specification assigns no pass/fail threshold to this actionful measurement.

Character-device, block-device, and whiteout nodes were not safely creatable by the unprivileged qualification environment. Their node-kind behavior was exercised through synthetic non-following fixtures and recorded as environmentally unavailable rather than as live filesystem passes. Protected BSD flags likewise remained an unavailable live transition; deterministic modeled findings verify that such flags are named and block rather than being copied or cleared.

Final validation passed through `mise run validate`: formatting, Clippy over all targets and features with warnings denied, the complete default test suite, documentation tests, and the release build all succeeded. The three explicit APFS groups were then rerun against the final code and passed with the counts recorded above. `git diff --check` also passed. The disposable case-sensitive APFS image was detached and its empty mountpoint and 256 MiB image file were removed after qualification.

## Cleanup

After preserving the required test report, remove only the exact disposable `fixture_root` created for this run. Do not remove or alter any real Grip home, mapping root, APFS volume, or disk image that was not created by this walkthrough.
