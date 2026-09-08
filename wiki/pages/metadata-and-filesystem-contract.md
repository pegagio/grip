---
title: Metadata and filesystem contract
type: component
sources: [S012]
updated: 2026-09-08
---

# Metadata and filesystem contract

Feature 009 completes Grip's initial equality contract for ordinary files and directories on current macOS with APFS. File equality includes node kind, content digest and length, full permission mode, numeric UID and GID, nanosecond modification time, allowlisted extended attributes, ordered extended ACLs, and supported user BSD flags. Directories use the same metadata fields without file content, and their metadata is independent of child membership. (S012)

The synchronized xattr allowlist is `com.apple.FinderInfo`, `com.apple.ResourceFork`, and `com.apple.TextEncoding`. Known security, provenance, volatile, and system-maintained attributes are named diagnostics but do not define equality. An attribute on neither list is unknown and blocks the selected mutation scope without being copied, discarded, or exposed as raw bytes. (S012)

ACL comparison preserves access-control entry order while canonicalizing only unordered permissions and flags inside each entry. Supported BSD equality flags are `UF_NODUMP`, `UF_IMMUTABLE`, `UF_APPEND`, and `UF_HIDDEN`, plus `UF_OPAQUE` for directories. Protected, privileged, synthetic, or unknown flags block a required transition rather than being cleared opportunistically. (S012)

Metadata-only changes use the same source-only, destination-only, converged, and divergent classifications as content. Content and all supported metadata remain one indivisible conflict value: resolution selects one complete source or destination state and never merges fields from opposing sides. Child actions finish before directory metadata finalizers, which run deepest-first. (S012)

State Envelope V3 persists the complete accepted state. Existing V2 baselines remain explicitly incomplete: equal current copies require `baseline accept`, differing copies require whole-entry `resolve`, read-only inspection never rewrites authority, and failed migration leaves V2 authoritative. Recovery Metadata V2 binds complete prior metadata and any preserved payload to its operation and action before replacement, deletion, or metadata-only mutation can remove the old logical state. (S012)

Preflight records concrete endpoint filesystem identity, mount flags, case behavior, mtime precision, and metadata operation capabilities. Grip preserves exact path bytes and blocks case or canonical-Unicode collisions instead of normalizing or renaming. Symbolic links, multiple hard links, sparse files, special nodes, and nested mounts remain unsupported and are inspected without following or opening them as payload. (S012)

The final qualification passed on documented macOS/APFS case-sensitive and case-insensitive roots. A measured operation-local capability cache and elimination of redundant metadata reads and payload hashing brought all 100-sample status and dry-run p95 measurements below two seconds without a persistent index, background service, broad lock, or parallel execution. (S012)

## Related pages

- [Filesystem support boundaries](./filesystem-support-boundaries.md)
- [Baseline classification and status](./baseline-classification-and-status.md)
- [Synchronization and conflicts](./synchronization-and-conflicts.md)
- [Deletion, retirement, and recovery](./deletion-retirement-and-recovery.md)
- [Local development workflows](./local-development-workflows.md)
