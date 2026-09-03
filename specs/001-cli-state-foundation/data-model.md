# Data Model: CLI, Configuration, and State Foundation

Feature 001 introduces only the data needed to establish Grip's per-user root, strict document boundaries, publication safety, and stable result classification. Mapping and synchronization models remain outside this feature.

## Table of Contents

- [Grip Home](#grip-home)
- [Registry V1](#registry-v1)
- [State Envelope V1](#state-envelope-v1)
- [Result Envelope V1](#result-envelope-v1)
- [Result Category](#result-category)
- [Diagnostic Event](#diagnostic-event)
- [State transitions](#state-transitions)
- [Validation ordering](#validation-ordering)

## Grip Home

The Grip Home is a version-neutral domain value created only after path-policy validation.

| Field | Type | Rules |
|-------|------|-------|
| `root` | Absolute path | Exact non-empty `GRIP_HOME`, or resolved user home plus `.grip`; never canonicalized into a different reported path |
| `registry_path` | Derived path | `root/config.toml` |
| `state_directory` | Derived path | `root/state` |
| `state_path` | Derived path | `root/state/state.json` |
| `lock_path` | Derived path | `root/state/state.lock` |
| `recovery_directory` | Derived path | `root/state/recovery` |

For an existing root, the final component must be a directory rather than a symbolic link or special node, must be owned by the effective user, and must be accessible through an opened descriptor. Newly created Grip-owned state and recovery directories use mode `0700`; state, lock, staging, and recovery files use `0600`. Grip does not silently change permissions on a pre-existing user-authored root.

## Registry V1

`RegistryV1` is the strict TOML wire model for durable user intent.

| Field | Type | Rules |
|-------|------|-------|
| `schema_version` | Positive integer | Required and exactly `1` |
| `mappings` | Array | Required and empty in Feature 001 |

Unknown fields are invalid at every object level. Missing `mappings` is not silently defaulted to an empty array. A non-empty array is rejected as unsupported Feature 002 behavior rather than partially interpreted.

The version-neutral domain form is `Registry { mappings: Empty }`. It contains no serialization attributes and conveys only that the foundation configuration is valid.

## State Envelope V1

`StateEnvelopeV1` is the strict JSON wire model for machine-owned operational evidence.

| Field | Type | Rules |
|-------|------|-------|
| `schema_version` | Positive integer | Required and exactly `1` |
| `payload` | `StatePayloadV1` | Required |
| `integrity` | `IntegrityV1` | Required |

`StatePayloadV1` contains one field:

| Field | Type | Rules |
|-------|------|-------|
| `generation` | Unsigned integer | Required; begins at `0`; increments exactly once for each accepted state publication |

`IntegrityV1` contains:

| Field | Type | Rules |
|-------|------|-------|
| `algorithm` | String enum | Required and exactly `sha256` |
| `digest` | String | Exactly 64 lowercase hexadecimal characters |

The digest covers the UTF-8 bytes of the compact canonical object `{"schema_version":1,"payload":{"generation":N}}`, with keys in the shown order, no insignificant whitespace, and `N` rendered as unsigned decimal. Future state versions define their own canonical integrity input rather than changing V1.

An absent accepted state file is `Uninitialized`, not an error. A present file transitions to `Valid`, `UnsupportedSchema`, or `Corrupt`; malformed JSON, unknown fields, semantic inconsistency, and digest mismatch are all `Corrupt` after version probing distinguishes unsupported schema.

## Result Envelope V1

`ResultEnvelopeV1` is emitted for application commands in machine-readable mode.

| Field | Type | Rules |
|-------|------|-------|
| `schema_version` | Positive integer | Required and exactly `1` |
| `status` | String enum | `ok` or `error` |
| `code` | `ResultCategory` symbolic string | Must correspond to `status` and process exit code |
| `message` | String | Non-empty concise summary; stable meaning, wording not an automation key |
| `details` | Object | Always present, including as `{}`; category-specific fields only |

The envelope has no additional top-level fields. Domain results are constructed before either human or JSON rendering so both modes express the same classification.

## Result Category

One exhaustive domain enum owns the result status, symbolic code, and process exit code.

| Category | Status | Symbol | Exit | Boundary |
|----------|--------|--------|-----:|----------|
| Success | `ok` | `ok` | 0 | Command completed |
| Invalid usage | `error` | `invalid_usage` | 2 | Parser or command-shape error |
| Invalid configuration | `error` | `invalid_configuration` | 10 | Invalid `GRIP_HOME`, registry, or path policy |
| Unsupported schema | `error` | `unsupported_schema` | 11 | Known document kind with unsupported version |
| Corrupt state | `error` | `corrupt_state` | 12 | Malformed, unknown-field, inconsistent, or integrity-failed state |
| Operational failure | `error` | `operational_failure` | 20 | Permission, I/O, output, or uncategorized runtime failure |

An inaccessible path is operational failure rather than invalid configuration. A final-component symlink or wrong node type violates path policy and is invalid configuration. An advisory-lock `WouldBlock` condition is the internal `StatePublicationError::StateContention`; public exit `13` and symbolic code `state_contention` are reserved until a public state-publishing command exists.

## Diagnostic Event

A Diagnostic Event is internal troubleshooting data with a severity, stable event name, and redacted fields. It is emitted only when verbosity enables it, never changes the Result Category, and never includes full registry or state content by default. Diagnostic rendering uses a writer separate from the application result writer.

## State transitions

State validation follows this lifecycle:

```text
Absent
  └── validate ──> Uninitialized

Present
  ├── unsupported version ──> UnsupportedSchema
  ├── malformed or failed integrity ──> Corrupt
  └── valid schema and integrity ──> Valid
```

State publication follows this lifecycle:

```text
Uninitialized or Valid
  └── acquire stable lock
      └── revalidate paths and prior state
          └── if replacing, retain and verify recovery/generation-N/state.json
              └── create unique same-directory staging file
                  └── write, sync, reread, verify
                      └── rename to accepted path
                          └── sync directory ──> Valid at generation + 1
```

Failure before rename leaves the previous accepted state authoritative and removes the known staging file when ordinary cleanup is possible. A recovery-copy failure blocks replacement. A byte-identical existing recovery copy for the accepted generation is reused on retry; a conflicting copy is corrupt state. Failure at or after rename is reported as operational failure; Grip rereads the accepted path before making any later acceptance claim. An orphan staging file is never accepted state.

The stable lock file remains after release. A crashed holder releases the kernel lock; no PID, age, or file deletion is used to break it.

## Validation ordering

The application validates evidence in this order so error categories remain deterministic:

1. Parse the command shape and requested output mode.
2. Resolve and validate the Grip Home policy.
3. Require the selected root and `config.toml`; classify either absence as invalid configuration without creating it or consulting any machine-wide fallback.
4. Probe and dispatch the registry schema version, then validate the concrete registry.
5. Treat absent state as uninitialized; otherwise probe and dispatch its version, validate structure, semantics, and integrity.
6. Construct one version-neutral domain result.
7. Render the result once in the requested mode and emit optional diagnostics separately.

Publication adds lock acquisition after home validation, then repeats relevant path and accepted-state validation while holding the lock before creating the staging file.
