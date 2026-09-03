---
title: Configuration and state
type: component
sources: [S001, S004]
updated: 2026-09-03
---

# Configuration and state

Grip separates durable user intent from machine-owned operational evidence. The registry holds mappings, paired paths, kinds, and options; state holds discovered members, fingerprints, baselines, pending retirement information, and backup references. Users do not maintain the derived member inventory manually. (S001)

The default per-user root for both registry and state is `~/.grip/`. A non-empty absolute `GRIP_HOME` selects an alternate exact root; relative or empty values are invalid. `/etc/grip/` is not implicitly merged as a registry or state location, and any future system-wide policy would require a distinct precedence model. (S001)

Configuration must carry an explicit schema version and reject unknown fields, unsupported versions, malformed options, path traversal, and duplicate or overlapping ownership. Machine-owned state should be written atomically and include enough versioning and integrity information to detect incompatible or corrupt data. (S001)

The implemented minimal v1 registry is `config.toml` with `schema_version = 1` and an empty `mappings` array. Grip-owned `state/state.json` is optional; `validate` reports an absent state document as uninitialized and does not create it. (S004)

Baseline records contain fingerprints and accepted supported metadata, not historical copies of file contents. Recovery copies belong to the operation backup namespace and are a separate concern from baseline comparison. (S001)

## Related pages

- [Grip product model](./grip-product-model.md)
- [Command-line and path selection](./command-line-and-path-selection.md)
