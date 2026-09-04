---
title: Command-line and path selection
type: component
sources: [S001, S004]
updated: 2026-09-04
---

# Command-line and path selection

The product definition selected the prospective executable name `grip` while leaving exact mapping syntax for later specification. Human-readable output and stable machine-readable output are separate interfaces, and automation needs a defined exit-code contract. (S001)

Feature 001 implements `grip version` and the read-only `grip validate`. Both application commands support human output by default and JSON through `--output json`; repeated `-v` or `--verbose` flags enable redacted diagnostics on stderr. Conventional `--help` and `--version` displays do not access the Grip home. (S004)

The implemented public exit contract is `0`/`ok`, `2`/`invalid_usage`, `10`/`invalid_configuration`, `11`/`unsupported_schema`, `12`/`corrupt_state`, and `20`/`operational_failure`. State-publication contention remains internal because Feature 001 exposes no public state-publishing command. (S004)

Feature 002 resolves the mapping lifecycle as `grip mapping add file SOURCE DESTINATION`, `grip mapping add tree SOURCE DESTINATION`, `grip mapping list`, `grip mapping show SOURCE`, and `grip mapping remove SOURCE`. Sources identify mappings by canonical absolute path; list output is canonically ordered, and removal changes registry intent without deleting payload data. (S004)

Mapping sources must exist and match their declared kind. Destinations may be absent when their nearest existing ancestor is safe; paths must be absolute UTF-8 without parent traversal, and final symbolic-link endpoints are rejected. Complete-registry ownership validation rejects duplicate, overlapping, nested, equal, and cross-recursive mappings. (S004)

`status`, `check`, and `diff` are always read-only. `push`, `pull`, and `sync` mutate by default; `-n` and `--dry-run` preview their deterministic plans. Conflict resolution requires an explicit complete-side choice, provisionally expressed as `resolve PATH --source` or `resolve PATH --destination`. (S001)

The initial command contract permits one optional path selector. Selectors use the source path space by default, while `--destination` selects destination-path interpretation; `--` terminates option parsing so paths beginning with a hyphen can be selected safely. Complete registry validation still applies even when an action is scoped to one mapping or subtree. (S001)

Multiple path selectors, interactive two-way text reconciliation, and metadata-only remapping are potential future expansions rather than initial command commitments. A future remap would update registry and state only after separately performed filesystem movement and verified baseline continuity. (S001)

## Related pages

- [Grip product model](./grip-product-model.md)
- [Local development workflows](./local-development-workflows.md)
