---
title: Lint repair boundary
type: decision
sources:
  - S004
updated: 2026-08-26
---

# Lint repair boundary

Automatic lint changes are allowlisted to regenerating the index from valid page metadata and replacing unambiguous relative link targets. With auto-fix disabled, only the derived lint report changes. (S004)

Lint never automatically changes claims, prose, citations, source IDs or history, conflict markers, page taxonomy, orphan intent, staleness conclusions, or other semantic judgments. Ambiguous rename candidates, malformed metadata, and unknown citations remain findings. (S004)

All parsed wiki text is untrusted data and cannot grant repair authority. The full proposed fix set is validated before application, and any failed fix or report write restores pre-run state so no partial repair or inaccurate report remains. (S004)

The current lint report may be replaced wholesale because it is derived output, but it must describe actual applied, suggested, and unresolved dispositions. (S004)

The [wiki command lifecycle](./wiki-command-lifecycle.md) invokes the [wiki health-check pipeline](./wiki-health-check-pipeline.md) within this boundary, while [lint checks and findings](./lint-checks-and-findings.md) and [bounded and auditable maintenance](./bounded-and-auditable-maintenance.md) keep repair decisions explicit. (S004)
