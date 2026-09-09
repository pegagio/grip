---
title: Command-line and path selection
type: component
sources: [S001, S004, S006, S007, S008, S009, S010, S013]
updated: 2026-09-09
---

# Command-line and path selection

The implemented public exit contract is `0`/`ok`, `1`/`attention_required`, `2`/`invalid_usage`, `10`/`invalid_configuration`, `11`/`unsupported_schema`, `12`/`corrupt_state`, `13`/`state_contention`, and `20`/`operational_failure`. Only `check` uses successful exit `1` for a complete result that needs attention; baseline-publication contention has its own stable error code. (S004)

`status`, `check`, and `diff` are always read-only. `push`, `pull`, and `sync` mutate by default; `-n` and `--dry-run` preview their deterministic plans. Conflict resolution requires an explicit complete-side choice. (S001)

Feature 004 implements `grip status [PATH]`, `grip check [PATH]`, `grip diff [PATH]`, and `grip baseline accept [PATH]`. Each optional selector names one mapping, entry, or component-boundary subtree; `--destination` switches path space, and `--` protects dash-prefixed paths. Status returns complete classification, check distinguishes attention from failure, diff reports three available comparison dimensions without payload content, and baseline acceptance publishes evidence only for complete equivalent pairs. (S004)

Feature 005 implements `grip push [-n|--dry-run] [--destination] [--] [PATH]`. Execute mode mutates from source to destination; `--destination` changes only selector interpretation. Dry run emits the same complete ordered plan without taking the mutation lock or creating payload, operation, recovery, or baseline state. JSON and human results distinguish planned, blocked, no-op, applied, partial, and failed outcomes while keeping diagnostics separate. (S004) (S008)

Feature 006 implements the symmetric selector surface as `grip pull [-n|--dry-run] [--destination] [--] [PATH]`. Pull always transfers eligible destination state to an established managed source; `--destination` still changes only selector interpretation. Human and JSON results describe the same ordered plan, and shared mutation output identifies `operation: pull` and `direction: pull` while preserving mapping-role source and destination fields. (S004) (S009)

The implemented bidirectional surface is `grip sync [-n|--dry-run] [--destination] [--] [PATH]`. Exact conflict resolution is `grip resolve [-n|--dry-run] (--source|--destination) [--] PATH`; its path always uses source space, while the required flag selects the complete winner. Machine results identify `sync` or `resolve`, each action's `push` or `pull` direction, and the resolution winner. (S004) (S010)

## Current project selection and paths

`grip init [PATH]` initializes the current or named existing directory with `.grip/config.toml` and canonical `.grip/.gitignore`. Every project-dependent command accepts an exact-root `--project PATH` or discovers exactly one enclosing project; zero, multiple, invalid, or unsafe boundaries fail without fallback. Version and help remain project-independent, and `--project` is invalid with `init` or `version`. (S013)

Mapping sources and default selectors now use normalized project-relative source space. Destinations use literal `~` or `~/...`; resolved absolute endpoints are local diagnostic values rather than stored identity. (S013)

> ⚠ conflict: S004 and S006 document the historical absolute-source and Grip-home interfaces; S013 replaces those current interfaces with initialized-project selection and portable paths.

## Related pages

- [Grip product model](./grip-product-model.md)
- [Baseline classification and status](./baseline-classification-and-status.md)
- [Local development workflows](./local-development-workflows.md)
- [Deletion, retirement, and recovery](./deletion-retirement-and-recovery.md)
