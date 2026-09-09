# Data Model: Project-Scoped Initialization and Portable Mappings

Feature 010 separates portable, version-controlled intent from resolved filesystem evidence and machine-local operational state. Absolute paths may appear in local binding or diagnostic evidence, but never as portable mapping identity or restoration authority.

## Table of Contents

- [Project Models](#project-models)
- [Portable Mapping Models](#portable-mapping-models)
- [Descriptor Model](#descriptor-model)
- [Durable Entry Identity](#durable-entry-identity)
- [Local State Model](#local-state-model)
- [Operation and Recovery Models](#operation-and-recovery-models)
- [Initialization Models](#initialization-models)
- [State Transitions](#state-transitions)
- [Validation Rules](#validation-rules)

## Project Models

### ProjectContext

`ProjectContext` is the one command-scoped authority value.

| Field | Type | Contract |
|---|---|---|
| `root` | Canonical absolute path | Existing current-user-owned ordinary directory and source root. |
| `metadata_dir` | Derived path | Exactly `<root>/.grip`; safe non-symlink directory. |
| `descriptor_path` | Derived path | Exactly `<root>/.grip/config.toml`; accepted Descriptor V2. |
| `ignore_path` | Derived path | Exactly `<root>/.grip/.gitignore`; canonical safe file. |
| `state_dir` | Derived path | Exactly `<root>/.grip/state`; optional until a writer creates it. |
| `user_home` | Canonical absolute path | Safe invoking-user home used to resolve destinations. |
| `evidence` | `ProjectEvidence` | Root and metadata identities plus accepted descriptor bytes and digest. |

### ProjectSelection

| Variant | Input | Rule |
|---|---|---|
| `Explicit` | `--project PATH` | Input itself is the exact root; no ancestor search or fallback. |
| `Discovered` | Invocation directory | Inspect all ancestors and accept exactly one valid boundary. |
| `Independent` | Version/help | No project or home discovery. |

## Portable Mapping Models

### ProjectRelativePath

A normalized UTF-8 path containing ordinary relative components. Exact `.` is permitted only for a tree mapping root. Empty, absolute, parent, repeated, trailing, and other dot components are invalid. `.grip` is forbidden as the first payload component.

### HomeRelativePath

A normalized UTF-8 declaration containing exact `~` or `~/` followed by ordinary relative components. It forbids `~user`, absolute syntax, environment expressions, parent, dot, repeated, and trailing components.

### PortableMapping

| Field | Type | Contract |
|---|---|---|
| `kind` | `file | tree` | Existing mapping extent. |
| `source` | `ProjectRelativePath` | Durable source identity under the project root. |
| `destination` | `HomeRelativePath` | Durable destination identity under the user home. |

The canonical identity tuple is `(source, kind, destination)`. Source remains the primary lookup key, while the complete tuple binds accepted and recovery evidence.

### ResolvedMapping

| Field | Type | Contract |
|---|---|---|
| `declaration` | `PortableMapping` | The only durable identity. |
| `source` | Canonical absolute path | Resolved beneath the project root. |
| `destination` | Canonical or prospective absolute path | Resolved beneath the selected home. |
| `source_evidence` | `PathEvidence` | Non-following ancestry and endpoint proof. |
| `destination_evidence` | `PathEvidence` | Non-following ancestry and endpoint proof. |

Encoding always uses `declaration`, never resolved absolute values.

## Descriptor Model

### ProjectDescriptorV2

| Field | Type | Validation |
|---|---|---|
| `schema_version` | Integer | Must equal `2`. |
| `mappings` | Ordered list of `PortableMapping` | Strict complete collection with no unknown fields. |

The empty descriptor is:

```toml
schema_version = 2
mappings = []
```

Encoding is deterministic. Loading validates grammar, resolves all mappings against one context, and applies complete ownership/topology validation.

### DescriptorSnapshot

Contains the decoded descriptor, exact bytes, file identity and mode, SHA-256 digest, resolved mappings, and path evidence. A writer revalidates all fields before replacement.

## Durable Entry Identity

### EntryIdentityV4

| Field | Type | Contract |
|---|---|---|
| `mapping` | `PortableMapping` | Complete portable tuple. |
| `relative_path_hex` | Hex bytes | Lossless entry-relative path; empty identifies the mapping root. |

Path construction first matches the portable identity to one accepted descriptor mapping, then appends validated relative bytes to the current resolved endpoint. No persisted absolute prefix is authoritative.

## Local State Model

### ProjectBindingV1

| Field | Type | Meaning |
|---|---|---|
| `project_root` | Safe absolute-path evidence | Publication-time root; diagnostic and relocation detector. |
| `user_home` | Safe absolute-path evidence | Publication-time home; diagnostic and relocation detector. |
| `descriptor_digest` | SHA-256 | Accepted descriptor bytes at publication. |
| `resolved_mapping_digest` | SHA-256 | Portable identities plus resolved endpoint evidence. |

The binding is not project identity and never authorizes prefix substitution.

### StateEnvelopeV4

State V4 retains integrity and generation ordering and contains `binding`, ordered `BaselineRecordV4` values, and explicit pending-retirement records. Each baseline contains `EntryIdentityV4` and the existing complete `SupportedEntryStateV3`. State versions 1–3 are unsupported.

### StateTrust

| State | Meaning | Eligible behavior |
|---|---|---|
| `uninitialized` | No state file. | Existing uninitialized read behavior; no external reconstruction. |
| `bound` | Binding and evidence validate. | Normal project operation. |
| `rebind_eligible` | Binding differs, but complete fresh evidence matches. | Read-only report or authorized writer; final state publication records current binding. |
| `rebind_blocked` | Evidence is stale, incomplete, incompatible, ambiguous, corrupt, or contradictory. | Diagnostics only; acceptance and mutation blocked. |

## Operation and Recovery Models

`OperationRecordV2` retains partitioned integrity but binds actions to portable identity, relative bytes, endpoint role, and expected evidence. `MutationRecoveryV2`, `RecoveryManifestV2`, and `RecoveryMetadataV3` do the same. Private payload references remain relative to the selected state root. Resolved display paths may be diagnostic but never replay authority.

Descriptor recovery identifies the selected descriptor role rather than an absolute target. Restoration resolves through the current context, then revalidates descriptor, binding, endpoint, and expected current state.

## Initialization Models

### InitializationPlan

| Field | Type | Meaning |
|---|---|---|
| `target` | Canonical absolute path | Existing selected directory. |
| `target_evidence` | Filesystem identity | Ownership, type, mode, access, and ancestry proof. |
| `staging_name` | Private component | Unique no-follow directory within the target. |
| `descriptor_bytes` | Bytes | Canonical empty Descriptor V2. |
| `ignore_bytes` | Bytes | Exact `/state/\n`. |

`InitializationResult` is `initialized` or `already_initialized`. Failure removes only private staging nodes created by that invocation.

## State Transitions

### Initialization

```text
uninitialized target
  -> validate target and ancestry
  -> stage and sync descriptor plus .gitignore
  -> revalidate target and absence
  -> atomic no-replace publication
  -> initialized project
```

Existing exact metadata transitions directly to `already_initialized`. Unsafe, nested, partial, or non-equivalent metadata fails with no accepted change.

### Project Operation

```text
invocation
  -> exact explicit selection or exhaustive discovery
  -> one ProjectContext
  -> descriptor and mapping resolution
  -> state binding and accepted-evidence validation
  -> plan
  -> lock and context revalidation
  -> existing recover-apply-verify-publish flow
```

### Copied-State Rebinding

```text
retained State V4 with binding mismatch
  -> retain bytes as untrusted
  -> resolve portable identities in current context
  -> reobserve accepted entries and recovery targets
  -> complete match: rebind_eligible
  -> successful state writer: publish current binding

incomplete or contradictory match
  -> rebind_blocked
  -> no acceptance or payload mutation
```

## Validation Rules

- One command has at most one context and never rediscovers it for result delivery.
- Descriptor V2 contains no ID, absolute mapping path, resolved path, state, lock, operation, or recovery value.
- `.grip/.gitignore` has exact bytes `/state/\n` and a safe node and mode.
- Initialization, read-only commands, and dry runs never create `.grip/state/`.
- Every state, lock, staging, operation, and recovery path is below the selected state root.
- Durable identity is portable; resolved paths never authorize later restoration or mutation.
- A project-root tree structurally excludes `.grip`.
- Binding mismatch requires complete fresh validation and never prefix substitution.
- Legacy descriptor and state schemas are rejected rather than migrated.
- Separate projects never share local locks or state through a global lookup.
