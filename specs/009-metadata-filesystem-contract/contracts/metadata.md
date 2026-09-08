# Metadata Contract

This contract defines the complete equality state promised for ordinary regular files and directories on qualified macOS/APFS endpoints.

## Equality Fields

| Field | File | Directory | Comparison |
|---|---:|---:|---|
| Node kind | Yes | Yes | Exact supported kind |
| Content | Yes | No | SHA-256 and byte length |
| Full permission mode | Yes | Yes | Exact four-octal-digit value |
| Numeric UID | Yes | Yes | Exact unsigned identity |
| Numeric GID | Yes | Yes | Exact unsigned identity |
| Modification time | Yes | Yes | Exact signed seconds and nanoseconds |
| Allowlisted xattrs | Yes | Yes | Exact name set and exact value bytes |
| Extended ACL | Yes | Yes | Exact canonical ACEs in semantic sequence |
| Supported BSD flags | Yes | Yes | Exact allowlisted set |

Access time, birth time, change time, physical allocation, clones, compression, block layout, and excluded xattrs do not define equality.

## Extended Attributes

The synchronized allowlist is exact:

- `com.apple.FinderInfo`
- `com.apple.ResourceFork`
- `com.apple.TextEncoding`

The reported, nonblocking exclusion list is exact:

- `com.apple.quarantine`
- `com.apple.provenance`
- `com.apple.macl`
- `com.apple.metadata:kMDItemWhereFroms`
- `com.apple.metadata:kMDItemDownloadedDate`
- `com.apple.lastuseddate#PS`
- `com.apple.root.installed`

An attribute on neither list is `unknown` and blocks mutation for the selected scope. Grip must not copy, delete, or silently ignore an unknown attribute. Public output reports names, lengths, and digests for synchronized values; it does not expose raw bytes.

## Access-Control Lists

ACL equality preserves ACE sequence. Each ACE compares principal UUID, allow-or-deny kind, permission set, and inheritance/entry flag set. Permission and flag enumeration is canonicalized within an ACE. ACEs themselves are never sorted. Display names do not define equality.

No extended ACL has one stable `absent` state. Unavailable, unreadable, or unsupported ACL evidence never compares equal to absence.

## BSD Flags

Supported equality flags are:

- Files and directories: `UF_NODUMP`, `UF_IMMUTABLE`, `UF_APPEND`, `UF_HIDDEN`
- Directories only: `UF_OPAQUE`

`UF_COMPRESSED`, `UF_TRACKED`, `UF_DATAVAULT`, all `SF_*` flags, synthetic/read-only bits, and unknown bits are not reproduced. A required difference involving one of these values is a blocking capability finding.

## Ownership and Authorization

UID and GID are numeric. An already-equal value requires no assignment. Grip never invokes `sudo` or substitutes another identity. Preflight must prove the invoking process can perform every required change; otherwise the entire selected mutating invocation is blocked before its first action.

## Modification Time

Mtime independently triggers synchronization for files and directories. Qualified APFS endpoints must round-trip signed seconds and nanoseconds exactly. A coarser or inconclusive target must report the required value, observed precision, and blocking reason rather than round silently.

## Complete-Entry Rule

Every differing field is reported separately, but the content and all supported metadata form one indivisible conflict value. Push, pull, sync, and resolve must never combine fields from opposing sides.
