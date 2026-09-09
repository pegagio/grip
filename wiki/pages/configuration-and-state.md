---
title: Configuration and state
type: component
sources: [S001, S004, S005, S007, S008, S009, S010, S011, S013]
updated: 2026-09-09
---

# Configuration and state

Grip separates durable user intent from machine-owned operational evidence. The project descriptor holds portable mapping declarations; project-local state holds fingerprints, accepted baselines, pending retirement information, operation evidence, and recovery references. Users do not maintain the derived member inventory manually. (S001)

Configuration must carry an explicit schema version and reject unknown fields, unsupported versions, malformed options, path traversal, and duplicate or overlapping ownership. Machine-owned state should be written atomically and include enough versioning and integrity information to detect incompatible or corrupt data. (S001)

Baseline records contain fingerprints and accepted supported metadata, not historical copies of file contents. Recovery copies belong to the operation backup namespace and are a separate concern from baseline comparison. (S001)

## Current project-local layout

Feature 010 replaces the current global model with `<project>/.grip/config.toml`, canonical `.grip/.gitignore`, and lazily created `.grip/state/`. Descriptor V2 stores portable mappings; State V4 stores portable accepted identities plus a revalidatable local binding; Operation Record V2 and Recovery Manifest V2 use portable identities and project-relative private references. No compatibility reader or migration is provided because Grip is unreleased. (S001) (S013)

Read-only and dry-run commands create no state. Project writers create owner-only state directories and coordinate through bounded locks below `.grip/state/locks/`; unrelated projects never share a writer lock. Copied state remains byte-preserved but untrusted until complete in-memory rebinding succeeds, and only a successful state-writing operation persists the new binding. (S013)

The completed feature's acceptance evidence covers independent clone state, project-local contention, copied-state eligibility and blockers, project-bound recovery, strict rejection of older schemas, and zero legacy-global fallback across the existing command families. (S013)

> ⚠ conflict: S004, S007, S008, S009, S010, and S011 preserve historical global paths and V1–V3 schemas; the re-ingested S001 and Feature 010 source S013 define the current strict project-local Descriptor V2, State V4, Operation Record V2, and portable recovery authority.

## Related pages

- [Grip product model](./grip-product-model.md)
- [Baseline classification and status](./baseline-classification-and-status.md)
- [Command-line and path selection](./command-line-and-path-selection.md)
- [Mapping registry publication](./mapping-registry-publication.md)
