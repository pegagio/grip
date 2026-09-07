---
title: Command-line and path selection
type: component
sources: [S001, S004, S006, S007, S008, S009]
updated: 2026-09-07
---

# Command-line and path selection

Feature 001 implements `grip version` and the read-only `grip validate`. Both application commands support human output by default and JSON through `--output json`; repeated `-v` or `--verbose` flags enable redacted diagnostics on stderr. Conventional `--help` and `--version` displays do not access the Grip home. (S004)

The implemented public exit contract is `0`/`ok`, `1`/`attention_required`, `2`/`invalid_usage`, `10`/`invalid_configuration`, `11`/`unsupported_schema`, `12`/`corrupt_state`, `13`/`state_contention`, and `20`/`operational_failure`. Only `check` uses successful exit `1` for a complete result that needs attention; baseline-publication contention has its own stable error code. (S004)

Feature 002 resolves the mapping lifecycle as `grip mapping add file SOURCE DESTINATION`, `grip mapping add tree SOURCE DESTINATION`, `grip mapping list`, `grip mapping show SOURCE`, and `grip mapping remove SOURCE`. Sources identify mappings by canonical absolute path; list output is canonically ordered, and removal changes registry intent without deleting payload data. (S004)

Feature 003 adds `grip mapping inspect [SOURCE]`: omitting the source inspects every accepted mapping, while one canonical source selects a mapping. The command returns a deterministic human or JSON inventory, keeps diagnostics on stderr, and reports a successful complete inventory even when `blocking_count` is nonzero; failures to complete inspection remain nonzero errors. (S004)

The Feature 003 contract also requires complete-registry validation before either all-mapping or selected-mapping inspection. Results identify the selected mapping, classify every discovered entry, and remain deterministically ordered by canonical source and source-relative raw path bytes. (S006)

Mapping sources must exist and match their declared kind. Destinations may be absent when their nearest existing ancestor is safe; paths must be absolute UTF-8 without parent traversal, and final symbolic-link endpoints are rejected. Complete-registry ownership validation rejects duplicate, overlapping, nested, equal, and cross-recursive mappings. (S004)

`status`, `check`, and `diff` are always read-only. `push`, `pull`, and `sync` mutate by default; `-n` and `--dry-run` preview their deterministic plans. Conflict resolution requires an explicit complete-side choice, provisionally expressed as `resolve PATH --source` or `resolve PATH --destination`. (S001)

Feature 004 implements `grip status [PATH]`, `grip check [PATH]`, `grip diff [PATH]`, and `grip baseline accept [PATH]`. Each optional selector names one mapping, entry, or component-boundary subtree; `--destination` switches path space, and `--` protects dash-prefixed paths. Status returns complete classification, check distinguishes attention from failure, diff reports three available comparison dimensions without payload content, and baseline acceptance publishes evidence only for complete equivalent pairs. (S004)

The initial command contract permits one optional path selector. Selectors use the source path space by default, while `--destination` selects destination-path interpretation; `--` terminates option parsing so paths beginning with a hyphen can be selected safely. Complete registry validation still applies even when an action is scoped to one mapping or subtree. (S001)

Feature 005 implements `grip push [-n|--dry-run] [--destination] [--] [PATH]`. Execute mode mutates from source to destination; `--destination` changes only selector interpretation. Dry run emits the same complete ordered plan without taking the mutation lock or creating payload, operation, recovery, or baseline state. JSON and human results distinguish planned, blocked, no-op, applied, partial, and failed outcomes while keeping diagnostics separate. (S004) (S008)

Feature 006 implements the symmetric selector surface as `grip pull [-n|--dry-run] [--destination] [--] [PATH]`. Pull always transfers eligible destination state to an established managed source; `--destination` still changes only selector interpretation. Human and JSON results describe the same ordered plan, and shared mutation output identifies `operation: pull` and `direction: pull` while preserving mapping-role source and destination fields. (S004) (S009)

## Related pages

- [Grip product model](./grip-product-model.md)
- [Baseline classification and status](./baseline-classification-and-status.md)
- [Local development workflows](./local-development-workflows.md)
