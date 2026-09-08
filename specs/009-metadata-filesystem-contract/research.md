# Research: Metadata and Filesystem Contract Completion

Feature 009 closes Grip's initial metadata and filesystem contract. This research resolves the technical choices needed to plan implementation for current macOS on APFS without broadening the product to other Unix platforms or physical-storage fidelity.

## Table of Contents

- [Platform and Filesystem Boundary](#platform-and-filesystem-boundary)
- [Metadata Evidence Model](#metadata-evidence-model)
- [State and Recovery Schema Evolution](#state-and-recovery-schema-evolution)
- [Modification Time](#modification-time)
- [Numeric Ownership](#numeric-ownership)
- [Extended Attributes](#extended-attributes)
- [Access-Control Lists](#access-control-lists)
- [BSD Flags](#bsd-flags)
- [Unsupported Nodes and Filesystem Boundaries](#unsupported-nodes-and-filesystem-boundaries)
- [Path Identity and APFS Collisions](#path-identity-and-apfs-collisions)
- [Mutation and Directory Ordering](#mutation-and-directory-ordering)
- [Dependencies and Native Bindings](#dependencies-and-native-bindings)
- [Qualification and Performance](#qualification-and-performance)
- [Sources](#sources)

## Platform and Filesystem Boundary

**Decision**: Qualify only current macOS on concrete APFS endpoints. Record the exact macOS version and build, Darwin kernel, APFS bundle version, filesystem type, mount flags, returned attribute masks, and capability masks for every acceptance run. Test both default case-insensitive APFS and case-sensitive APFS.

**Rationale**: APFS may be case-sensitive or case-insensitive while preserving spelling, and capability behavior belongs to the mounted endpoint rather than the operating-system label. The user explicitly deferred other Unix platforms and filesystems.

**Alternatives rejected**: Treating all macOS filesystems alike would overstate fidelity. Building a general Unix abstraction now would add branches that cannot be accepted by this feature.

## Metadata Evidence Model

**Decision**: Model each field as explicit evidence with one of `observed`, `absent`, `unavailable`, `unsupported`, `unreadable`, or `unauthorized`. A complete supported entry state contains node kind, optional content fingerprint, and independent metadata: mode, numeric UID, numeric GID, mtime, allowlisted xattrs, ordered ACL, and supported BSD flags. Capability findings name the endpoint, path, field or node property, required value, observed capability, reason, and blocking status.

**Rationale**: Missing evidence is not equivalent to an absent value. Typed findings let human and machine output communicate the same safety conclusion without parsing diagnostic strings.

**Alternatives rejected**: Optional fields collapse absence and failure. Free-form blocker strings cannot support stable paired output contracts or precise tests.

## State and Recovery Schema Evolution

**Decision**: Publish State Envelope V3 for complete metadata. Strictly decode V2 as `legacy_incomplete`; never reinterpret omitted fields as absent or defaulted. When current copies are completely equal, read-only inspection reports `metadata_migration_ready` and the existing explicit `grip baseline accept` workflow may publish V3. When they differ, report `metadata_migration_conflict` and require the existing whole-entry `grip resolve PATH --source|--destination` workflow.

Publish Recovery Metadata V2 for new recovery records while retaining strict read support for Recovery Metadata V1. V2 records the complete prior logical metadata and deterministic references to preserved payload and xattr evidence. Existing Operation Record V1 and Result Envelope V1 remain versioned as-is; typed metadata details are additive within their existing extensible payloads.

**Rationale**: Adding optional fields to State V2 would silently change the meaning of already accepted evidence. An explicit version transition makes user authority visible and preserves strict history. Recovery needs its own version because the old schema cannot prove restoration of the expanded state.

**Alternatives rejected**: Automatic V2 rewrite violates read-only behavior. Treating missing metadata as defaults can publish a false baseline. Requiring a new CLI command adds surface without new user intent.

## Modification Time

**Decision**: Store mtime as signed seconds plus unsigned nanoseconds and compare both exactly on qualified APFS endpoints. Use descriptor-bound `futimens` with access time set to `UTIME_OMIT`. Retain the observed reproducible precision in endpoint evidence; a coarser or inconclusive endpoint is a capability blocker rather than a reason to round silently.

**Rationale**: Darwin `stat` exposes `st_mtimespec`, and current APFS supports nanosecond-scale inode timestamps. Integer components avoid floating-point and `SystemTime` serialization ambiguity. Mtime independently defines equality and triggers synchronization for files and directories.

**Alternatives rejected**: Whole-second truncation loses promised state. Taking the minimum precision without reporting it can make unequal values compare equal.

## Numeric Ownership

**Decision**: Compare and persist raw numeric UID and GID. Names are optional presentation context only. Preflight considers a UID transition authorized only when it is already equal for an unprivileged process. A GID transition is authorized only when already equal or when the invoking user owns the entry and the requested group is among the effective or supplementary groups. Grip never invokes privilege escalation.

Apply `fchown` before final `fchmod` because ownership changes may clear set-user-ID and set-group-ID bits.

**Rationale**: Numeric identity is the filesystem contract. Conservative authorization proof avoids discovering predictable permission failures after mutation starts.

**Alternatives rejected**: User and group names are mutable directory-service labels. Trial mutations followed by rollback violate dry-run and preflight guarantees.

## Extended Attributes

**Decision**: Synchronize exactly these names in the initial contract:

- `com.apple.FinderInfo`
- `com.apple.ResourceFork`
- `com.apple.TextEncoding`

Report but exclude exactly these known security, provenance, volatile, or system-maintained names:

- `com.apple.quarantine`
- `com.apple.provenance`
- `com.apple.macl`
- `com.apple.metadata:kMDItemWhereFroms`
- `com.apple.metadata:kMDItemDownloadedDate`
- `com.apple.lastuseddate#PS`
- `com.apple.root.installed`

Any other xattr is `unknown` and blocks the selected mutation scope. Enumeration order is irrelevant: sort names by raw bytes for deterministic reporting. Compare exact name and value bytes; accepted evidence stores byte length and SHA-256 digest, while the live operation and private recovery object retain the actual value needed for transfer or restoration. Retry bounded size/read races and fail on unstable evidence.

**Rationale**: The allowlist covers established Finder information, resource forks, and text encoding without silently claiming ownership of arbitrary application metadata. The exclusion list prevents download policy and system provenance from being propagated as synchronization data. Exact values meet the fidelity contract while digests avoid embedding potentially large resource forks in the accepted-state JSON.

**Alternatives rejected**: A denylist would silently expand Grip's behavior when new attributes appear. Prefix allowlisting such as all `com.apple.metadata:` has the same problem. Omitting resource forks would turn ordinary macOS files into blockers and leave logical content incomplete.

## Access-Control Lists

**Decision**: Represent an extended ACL as an ordered sequence of ACEs. Each ACE contains principal UUID bytes, allow-or-deny tag, permission bit set, and inheritance/entry flag set. Canonicalize ordering within the permission and flag sets, but preserve ACE sequence exactly. Preserve inherited, file-inherit, directory-inherit, limit-inherit, and only-inherit semantics. Use one stable `absent` representation when no extended ACL exists.

**Rationale**: macOS evaluates ACL entries in order. Sorting ACEs could change access behavior, while canonicalizing unordered bits removes irrelevant API enumeration noise. UUIDs are durable identity; display names are contextual only.

**Alternatives rejected**: Text from `ls` is locale- and formatting-dependent. Sorting ACEs contradicts macOS evaluation semantics. Reducing ACLs to mode bits discards promised state.

## BSD Flags

**Decision**: Synchronize these ordinary user flags:

- Files and directories: `UF_NODUMP`, `UF_IMMUTABLE`, `UF_APPEND`, `UF_HIDDEN`
- Directories only: `UF_OPAQUE`

Report but do not reproduce `UF_COMPRESSED`, `UF_TRACKED`, `UF_DATAVAULT`, every `SF_*` flag, synthetic or read-only flags, and unknown bits. A differing unsupported or protected flag blocks mutation. Preflight represents any required clearing of supported immutable or append-only flags; application restores final supported flags last.

**Rationale**: XNU distinguishes owner-changeable flags from system, protected, and synthetic state. The allowlist is explicit and covers meaningful user-controlled file behavior without claiming compression or platform-security fidelity.

**Alternatives rejected**: Copying every exposed bit would include state an ordinary process cannot safely reproduce. Supporting only cosmetic flags would omit user-controlled immutability that directly affects synchronization safety.

## Unsupported Nodes and Filesystem Boundaries

**Decision**: Use non-following, descriptor-bound inspection and reject the following before payload access: symbolic links, sockets, FIFOs, character devices, block devices, whiteouts, unknown special nodes, ordinary files with `st_nlink != 1`, files marked `EF_IS_SPARSE`, and nested mount points below a tree root. A mounted filesystem may be an explicit mapping root.

Use `getattrlist` extended flags as the sparse-file authority rather than allocation heuristics. Bind each tree traversal to the root's device/filesystem identity and use directory mount status where available. Revalidate node kind, link count, sparse status, mount status, and ancestry immediately before every action.

**Rationale**: Allocation counts can also reflect compression or clones, so `st_blocks` is not an authoritative sparse indicator. Descriptor binding and `O_NOFOLLOW` keep reports and mutations attached to the inspected object.

**Alternatives rejected**: Shell utilities are path-based and locale-sensitive. Following links or copying special nodes as regular payload violates the product boundary. Treating all allocation reduction as sparse creates false blockers.

## Path Identity and APFS Collisions

**Decision**: Preserve exact `OsStr` component bytes as managed identity and display them through the existing safe-path representation. Query APFS volume capabilities for case behavior. Detect potential canonical Unicode or case collisions using a comparator qualified against actual lookup fixtures on the exact target APFS build; block if the comparator is inconclusive or if distinct managed identities alias. Never normalize, fold, or rename the stored path.

**Rationale**: APFS preserves spelling but may compare names case-insensitively and normalization-insensitively. Apple does not expose a general syscall for comparing two prospective names, so observed conformance fixtures are part of the qualification proof.

**Alternatives rejected**: Lowercasing or NFC alone is not an APFS contract. Creating probe files during preflight would make dry runs mutating. Assuming behavior from `macOS` rather than the concrete volume misses case-sensitive APFS.

## Mutation and Directory Ordering

**Decision**: Complete all capability and authorization checks before the first mutation. For a created or replaced entry, use this order:

1. Create or stage and verify the payload under restrictive temporary permissions.
2. Apply numeric owner and group.
3. Apply the ordered extended ACL.
4. Apply allowlisted xattrs.
5. Apply the final complete mode.
6. Apply mtime.
7. Apply final supported BSD flags.
8. Flush durable state and re-read every supported field.

Child actions precede directory metadata finalizers, and directory finalizers run deepest-first. Metadata-only transitions are explicit planned actions with recovery evidence. Complete-state conflict resolution never mixes fields from opposing sides.

**Rationale**: Ownership can clear special mode bits, child changes alter directory mtime, and immutable flags can prevent later operations. This order minimizes side effects and makes each final state verifiable.

**Alternatives rejected**: Applying directory metadata during shallow creation produces immediate drift. In-place unplanned metadata changes cannot satisfy recovery or operation-record guarantees.

## Dependencies and Native Bindings

**Decision**: Continue using `rustix 1.1` for xattrs, ownership, modes, timestamps, and filesystem statistics. Implement one `cfg(target_os = "macos")` adapter for ACL, `fchflags`, and APFS capability calls. Declare the already transitive `libc 0.2` crate as a direct dependency; this choice was explicitly approved on 2026-09-07 before the dependency files changed.

**Rationale**: The adapter keeps unsafe code and Darwin constants in one auditable place. A direct dependency accurately expresses source-level use and is smaller than introducing a broad filesystem metadata crate.

**Alternatives rejected**: Shelling out weakens race binding and deterministic error handling. Distributing handwritten FFI throughout modules is harder to review. A portability framework is outside the accepted platform boundary.

## Qualification and Performance

**Decision**: Build isolated macOS/APFS tests for every supported field on files and directories, absent/present transitions, same- and cross-volume mappings, both APFS case modes, Unicode-equivalent names, State V2 migration, Recovery Metadata V1/V2, excluded and unknown xattrs, unsupported nodes, authorization, drift, recovery, and paired human/JSON results.

Extend the current ignored 10,000-entry performance harness to run 100 status and 100 dry-run samples and publish p50, p95, maximum, build mode, host evidence, and fixture composition. The acceptance threshold is p95 at or below two seconds. Measure a representative sync workload without assigning an invented threshold. Optimize only from measured profiles.

**Rationale**: The roadmap requires integrated product acceptance, not isolated API tests. Host evidence makes a platform-specific performance claim reproducible and bounded.

**Alternatives rejected**: A single warm run hides variance. Adding caches or parallelism before measuring creates invalidation and ordering risk without evidence.

## Sources

- [Apple File System FAQ](https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/APFS_Guide/FAQ/FAQ.html)
- [Apple File System Reference](https://developer.apple.com/support/apple-file-system/Apple-File-System-Reference.pdf)
- [Apple filesystem security and ACL overview](https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/FileSystemDetails/FileSystemDetails.html)
- [Apple `acl_get_file(3)`](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/acl_get_file.3.html)
- [Apple `acl_set_link(3)`](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/acl_set_link.3.html)
- [Apple `chflags(2)`](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fchflags.2.html)
- [Apple `chown(2)`](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/chown.2.html)
- [Apple `getxattr(2)`](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getxattr.2.html)
- [XNU `stat.h`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/stat.h)
- [XNU `attr.h`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/attr.h)
- [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2)
- [XNU xattr definitions](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/xattr.h)
- [Apple copyfile xattr policy](https://github.com/apple-oss-distributions/copyfile/blob/main/xattr_flags.c)
- [Rustix filesystem API](https://docs.rs/rustix/latest/rustix/fs/index.html)
- [Rust Darwin `MetadataExt`](https://doc.rust-lang.org/std/os/unix/fs/trait.MetadataExt.html)
