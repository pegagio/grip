# Data Model: Wiki Status and Resumption

## Status Scope

Default, one configured page type, or bounded `full`.

## Page Summary

| Field | Rule |
|---|---|
| Path | Exact indexed page filename |
| Type | Configured page type from the index grouping |
| Updated | Structural date when present; otherwise unknown |

## Source Summary

| Field | Rule |
|---|---|
| Source ID | Stable registered `S-id` |
| Identity | Registry identity rendered as untrusted data |
| Last ingested | Recorded date or unknown |
| Pages touched | Registry filenames, never opened by status |

## Freshness Signal

Current, stale, unknown, or invalid, based only on comparable structural dates.

## Open Issue Summary

Severity, check, page, evidence, and suggested action copied from the current lint report. Semantic issues precede structural and mechanical issues.

## Next Action Candidate

An evidence-backed command or user decision with priority, relevant evidence date, and lexical identity for deterministic selection.

## Status Snapshot

Scope, page counts, bounded page and source slices, freshness signals, bounded issue slice, and exactly one selected next action.

## Lifecycle

```text
validate scope -> read structural metadata -> derive bounded summaries -> rank evidence-backed candidates -> render one snapshot -> exit without writes
```
