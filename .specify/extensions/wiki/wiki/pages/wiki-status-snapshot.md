---
title: Wiki status snapshot
type: component
sources:
  - S005
updated: 2026-08-26
---

# Wiki status snapshot

Wiki status is a compact session-resumption view derived only from structural metadata, not an evidence-answering query or semantic health analysis. (S005)

## Inputs and scope

Status may read schema scope, the index, source registry, optional current lint report, and bounded active-feature metadata consisting of the feature pointer plus artifact paths, existence, and timestamps. (S005)

Empty input uses default slices, a configured page type filters only the page slice, and `full` expands every default slice by no more than three times. An unknown argument reports accepted values and stops without mutation. (S005)

## Snapshot

The default one-screen snapshot contains scope, page counts by type, bounded recent pages, source count, bounded recent sources, structural freshness, prioritized open lint issues, and exactly one [next action](./status-next-action-policy.md). (S005)

Freshness is current, stale, unknown, or invalid based only on comparable structural dates. Missing facts are `unknown`, malformed facts are `invalid`, and neither condition triggers inference or repair. (S005)

The [wiki command lifecycle](./wiki-command-lifecycle.md) invokes the snapshot within the [status read boundary](./status-read-boundary.md), while [bounded and auditable maintenance](./bounded-and-auditable-maintenance.md) keeps every view compact. (S005)
