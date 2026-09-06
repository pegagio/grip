---
title: Configuration and state
type: component
sources: [S001, S004, S005, S007]
updated: 2026-09-06
---

# Configuration and state

Grip separates durable user intent from machine-owned operational evidence. The registry holds mappings, paired paths, kinds, and options; state holds discovered members, fingerprints, baselines, pending retirement information, and backup references. Users do not maintain the derived member inventory manually. (S001)

The default per-user root for both registry and state is `~/.grip/`. A non-empty absolute `GRIP_HOME` selects an alternate exact root; relative or empty values are invalid. `/etc/grip/` is not implicitly merged as a registry or state location, and any future system-wide policy would require a distinct precedence model. (S001)

Configuration must carry an explicit schema version and reject unknown fields, unsupported versions, malformed options, path traversal, and duplicate or overlapping ownership. Machine-owned state should be written atomically and include enough versioning and integrity information to detect incompatible or corrupt data. (S001)

The implemented v1 `config.toml` registry carries `schema_version = 1` and canonical file or tree mapping tuples; an empty `mappings` array remains valid. Grip-owned `state/state.json` is optional, and `validate` reports an absent state document as uninitialized without creating it. (S004)

The accepted registry must be a current-user-owned, non-symlink regular file without group or other write permission; mapping mutations also require owner write permission and preserve the exact accepted mode. Writers use a stable owner-only `.registry.lock`, retain exact prior bytes in content-addressed recovery generations, verify a complete staged candidate, and atomically replace `config.toml`. These artifacts contain no synchronization state and mapping commands do not create `state/state.json`. (S004)

Baseline records contain fingerprints and accepted supported metadata, not historical copies of file contents. Recovery copies belong to the operation backup namespace and are a separate concern from baseline comparison. (S001)

Feature 004 publishes integrity-checked State Envelope V2 generations atomically at `<GRIP_HOME>/state/state.json` and retains exact prior bytes as immutable recovery evidence. The state directory must be a current-user-owned, non-symlink directory with exact owner-only mode. State V1 remains readable as an empty-baseline predecessor, while corrupt, unsupported, or stale evidence produces stable non-success results rather than guessed recovery. (S004)

## Related pages

- [Grip product model](./grip-product-model.md)
- [Baseline classification and status](./baseline-classification-and-status.md)
- [Command-line and path selection](./command-line-and-path-selection.md)
- [Mapping registry publication](./mapping-registry-publication.md)
