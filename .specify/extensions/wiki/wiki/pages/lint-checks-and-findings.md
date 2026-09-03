---
title: Lint checks and findings
type: reference
sources:
  - S004
updated: 2026-08-26
---

# Lint checks and findings

Wiki lint defines six named checks with deterministic, evidenced findings. (S004)

- **Index drift** identifies unindexed pages and index entries targeting missing pages. (S004)
- **Links** identifies missing relative page targets and citation references to unknown source IDs. (S004)
- **Orphans** identifies pages with no incoming link from another page; index membership does not count as a page link. (S004)
- **Contradictions** identifies unresolved conflict markers and incompatible claims only among pages connected by a page link or shared source ID. (S004)
- **Stale** identifies age-threshold violations and pages older than a supporting source's most recent ingestion. (S004)
- **Citations** identifies uncited claims when the schema requires citations. (S004)

Every finding records its check, mechanical, structural, or semantic severity, exact affected filename or artifact, verifiable evidence, concrete user-controlled suggested action, and applied, suggested, or unresolved disposition. (S004)

Findings are deduplicated and ordered by severity, check, page, and evidence so reports remain stable and reviewable in version control. (S004)

The [wiki command lifecycle](./wiki-command-lifecycle.md) invokes the [wiki health-check pipeline](./wiki-health-check-pipeline.md), which produces these findings before consulting the [lint repair boundary](./lint-repair-boundary.md). (S004)
