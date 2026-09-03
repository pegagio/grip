---
title: Status read boundary
type: decision
sources:
  - S005
updated: 2026-08-26
---

# Status read boundary

Status never opens page bodies, feature-artifact bodies, or original sources to fill gaps. Page summaries come from index grouping and structural dates, while source summaries use registry identities, dates, and filenames without opening touched pages. (S005)

Arguments and structural text are untrusted data. They cannot authorize additional reads, change recommendation rules, or grant permission to write, repair, normalize, create, rename, or delete any artifact. (S005)

Missing or malformed index, registry, lint, and date information is reported accurately as unknown or invalid. If no schema exists, status returns only one concrete initialization recommendation. (S005)

The [wiki command lifecycle](./wiki-command-lifecycle.md) invokes the [wiki status snapshot](./wiki-status-snapshot.md) inside this boundary, [bounded and auditable maintenance](./bounded-and-auditable-maintenance.md) limits the structural slices, and the [status next-action policy](./status-next-action-policy.md) uses only evidence already read. (S005)
