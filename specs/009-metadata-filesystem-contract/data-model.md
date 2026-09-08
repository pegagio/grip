# Data Model: Metadata and Filesystem Contract Completion

Feature 009 expands Grip's accepted entry state and introduces explicit capability evidence. The model keeps filesystem observation separate from durable fingerprints: live evidence may hold bytes required for a pending action, while accepted state stores deterministic equality evidence.

## Table of Contents

- [Core Evidence Types](#core-evidence-types)
- [Supported Entry State](#supported-entry-state)
- [Metadata Values](#metadata-values)
- [Capability and Compatibility](#capability-and-compatibility)
- [Classification and Planning](#classification-and-planning)
- [Persistence Models](#persistence-models)
- [State Transitions](#state-transitions)
- [Validation Rules](#validation-rules)

## Core Evidence Types

### Evidence

`Evidence<T>` distinguishes a concrete value from every reason a value is not available.

| State | Value | Meaning |
|---|---|---|
| `observed` | `T` | The endpoint returned a validated concrete value. |
| `absent` | None | The field is supported and definitively not present. |
| `unavailable` | Optional reason | The endpoint cannot provide conclusive evidence for this operation. |
| `unsupported` | Optional reason | The endpoint does not implement the required field or behavior. |
| `unreadable` | Error category | The field exists or may exist but could not be inspected. |
| `unauthorized` | Required transition and reason | Inspection may succeed, but the invoking user cannot prove the required change is permitted. |

Only `observed` and, for presence-optional fields, `absent` can participate in an eligible equality comparison. Other states produce structured findings and block mutation when the field belongs to the selected managed scope.

### EntryIdentity

The existing identity remains the canonical source-path identity:

- Mapping snapshot: kind, absolute source root, absolute destination root
- Relative path: exact filesystem bytes

No normalized or case-folded spelling is stored as identity. Endpoint-specific comparison keys are ephemeral capability evidence.

## Supported Entry State

### SupportedEntryStateV3

| Field | Type | Rules |
|---|---|---|
| `node_kind` | `file | directory` | No other node kind is accepted. |
| `content` | `ContentFingerprint?` | Required for a file and forbidden for a directory. |
| `metadata` | `MetadataState` | Required for both files and directories. |

`ContentFingerprint` retains lowercase SHA-256 and byte length.

### MetadataState

| Field | Type | Equality behavior |
|---|---|---|
| `permission_mode` | Four-octal-digit string | Exact full Unix mode bits promised by Grip. |
| `uid` | Unsigned numeric ID | Exact numeric equality. |
| `gid` | Unsigned numeric ID | Exact numeric equality. |
| `modified_time` | `ModificationTime` | Exact seconds and nanoseconds. |
| `extended_attributes` | Sorted map of `XattrFingerprint` | Exact name set, length, and digest. |
| `acl` | `AclState` | Exact ordered ACE sequence or stable absence. |
| `bsd_flags` | `BsdFlagSet` | Exact supported allowlisted set. |

## Metadata Values

### ModificationTime

| Field | Type | Constraint |
|---|---|---|
| `seconds` | Signed 64-bit integer | Unix epoch seconds reported by Darwin. |
| `nanoseconds` | Unsigned 32-bit integer | `0..999999999`. |
| `precision_nanoseconds` | Positive unsigned integer | Operation-scoped capability evidence; qualified APFS expects `1`. |

The accepted equality value is seconds plus nanoseconds. Precision is endpoint evidence and must prove that the accepted value can be reproduced.

### XattrFingerprint

| Field | Type | Constraint |
|---|---|---|
| `name` | Byte-safe name | Must be on the exact synchronized allowlist. |
| `length` | Unsigned 64-bit integer | Exact value byte count. |
| `algorithm` | String | `sha256`. |
| `digest` | String | Lowercase 64-character hexadecimal. |

Live `XattrValue` additionally carries exact bytes for a pending transfer. Recovery payloads preserve exact bytes on the private recovery object and bind them to the fingerprint.

### AclState

`AclState` is either `absent` or an ordered list of `AccessControlEntry` values.

| ACE field | Type | Constraint |
|---|---|---|
| `principal_uuid` | 16 bytes | Durable identity; name is not equality data. |
| `kind` | `allow | deny` | Exact tag. |
| `permissions` | Sorted unique enum set | Known macOS permission bits only. |
| `flags` | Sorted unique enum set | Preserves inherited and inheritance semantics. |

ACE order is never sorted or discarded. Permission and flag sets within an ACE are canonicalized.

### BsdFlagSet

The stable enum values are `nodump`, `immutable`, `append`, `hidden`, and `opaque`. `opaque` validates only for directories. The set is sorted and unique for serialization. Protected, synthetic, excluded, and unknown bits are findings rather than accepted values.

## Capability and Compatibility

### EndpointCapabilityProfile

An operation-scoped profile records:

- Endpoint role: `source` or `destination`
- Root path in safe-path form
- Filesystem type and device/filesystem identity
- Mount flags and returned attribute masks
- Case-sensitive and case-preserving capability evidence
- Unicode-comparison qualification identifier
- Supported inspection, application, and verification operations per metadata dimension
- Mtime precision
- Host qualification record reference

Profiles are observations, not persisted promises. They must be revalidated where a changed value could affect action safety.

### CompatibilityFinding

| Field | Type | Meaning |
|---|---|---|
| `endpoint` | Endpoint role | Where the finding applies. |
| `path` | Safe path | Affected entry. |
| `field` | Stable enum | Node, ownership, mtime, xattr name, ACL, flag, mount, case, or Unicode. |
| `required` | Typed or safe display value | State or behavior Grip needs. |
| `evidence_state` | Evidence state | Observed failure category. |
| `reason` | Stable reason code | Machine-readable cause. |
| `message` | String | Human explanation without secret metadata bytes. |
| `blocking` | Boolean | Whether the selected operation may mutate. |

### FilesystemIdentityCollision

Contains the endpoint, comparison behavior evidence, and every colliding managed `EntryIdentity`. It blocks mapping acceptance and selected mutation. It never replaces an identity with a normalized spelling.

### PlatformQualificationRecord

Records macOS version/build, Darwin kernel, APFS bundle version, filesystem type, mount flags, volume capability masks, binary build profile, test matrix version, and performance host description. It is acceptance evidence, not runtime state.

## Classification and Planning

### ChangedDimension

Extend the stable dimension enum with:

- `permission_mode`
- `owner`
- `group`
- `modification_time`
- `extended_attribute` with the safe attribute name
- `access_control_list`
- `bsd_flags`

The existing node-kind and content dimensions remain. Differences are reported independently, but classification and resolution operate on the complete entry state.

### MigrationClassification

| Value | Condition | Authorized next step |
|---|---|---|
| `metadata_migration_ready` | Baseline is V2 and complete current source equals destination. | Explicit `grip baseline accept`. |
| `metadata_migration_conflict` | Baseline is V2 and current source differs from destination in any complete-state dimension. | Explicit whole-entry `grip resolve PATH --source` or `--destination`. |

Read-only commands do not publish State V3.

### MetadataTransition

Contains entry identity, direction, before state, after state, changed dimensions, required flag-clearing preconditions, capability proofs, and revalidation evidence. It never contains fields selected from different winners.

### Mutation Actions

Add these action kinds to the existing ordered plan:

- `ApplyMetadata`: applies a complete metadata transition to an existing file when content and node kind are unchanged.
- `FinalizeDirectoryMetadata`: applies a directory's independent final state after every descendant action.

File add and replacement actions gain complete metadata application steps. All actions keep existing indexes, dependencies, status, milestones, recovery reference, and verification evidence.

## Persistence Models

### State Envelope V3

State V3 retains envelope integrity, generation, registry binding, and the canonical baseline map. Every baseline value is `SupportedEntryStateV3`. V2 remains strictly decodable only as legacy-incomplete accepted evidence.

### Recovery Metadata V2

Recovery V2 retains operation/action binding and adds:

- Original complete supported state
- Preserved payload reference when applicable
- Per-xattr length and digest references
- ACL and BSD-flag evidence
- Preservation and verification status per field
- Private recovery-object security state, recorded separately from original mode and ownership

Recovery V1 remains readable and restorable under its original contract; it cannot authorize a new Feature 009 destructive mutation.

### Operation Record V1 and Result Envelope V1

Their schema versions remain stable. Existing JSON value/detail extension points carry the new typed action state, findings, dimensions, and verification. Strict envelope and integrity behavior do not change.

## State Transitions

### Accepted-State Migration

```text
State V2 + equal complete current copies
  -> metadata_migration_ready
  -> explicit baseline accept
  -> verified State V3 publication

State V2 + unequal complete current copies
  -> metadata_migration_conflict
  -> explicit complete-entry winner
  -> preserve losing entry
  -> apply and verify winner on both sides
  -> verified State V3 publication
```

### Metadata Mutation

```text
observed
  -> complete read-only capability and authorization preflight
  -> deterministic planned transition
  -> action-bound revalidation
  -> complete recovery preservation
  -> metadata application in side-effect-aware order
  -> full-state verification and durability
  -> accepted State V3 publication
```

Any blocker stops before mutation. Drift after mutation begins enters the existing partial-failure result with recovery authority and never publishes a successful baseline.

### Directory Finalization

```text
create or validate parent directory
  -> execute all descendant payload and metadata actions
  -> finalize deepest selected directory
  -> finalize each parent toward the mapping root
  -> verify complete selected scope
```

## Validation Rules

- A file requires content and complete metadata; a directory forbids content and requires complete metadata.
- Accepted xattrs must be allowlisted, unique by exact name bytes, sorted deterministically, and carry valid SHA-256 fingerprints.
- Unknown xattrs cannot enter accepted state.
- Excluded xattrs appear only in diagnostic findings.
- ACL principal UUIDs must be 16 bytes, known bits must be unique, and ACE order must round-trip unchanged.
- `opaque` is invalid for files.
- Nanoseconds must be below one billion; qualified precision must reproduce the exact accepted value.
- An endpoint capability profile must prove every required inspect, apply, and verify step before a plan is eligible.
- A collision must contain at least two distinct managed identities.
- State V2 cannot be converted to State V3 without explicit user authority and equal verified final copies.
- Recovery evidence must bind to the exact action and prior complete state before destructive publication.
