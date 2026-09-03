---
title: Wiki health-check pipeline
type: component
sources:
  - S004
updated: 2026-08-26
---

# Wiki health-check pipeline

Wiki lint is a bounded analyze-then-apply workflow that produces verifiable findings, performs only allowlisted repairs, and preserves human authority over semantic content. (S004)

## Scope

Lint accepts an empty scope for all checks, one declared check name, or one page plus directly linked or shared-source neighbors. Unknown scopes stop without mutation, and contradiction comparisons never extend beyond linked or shared-source page pairs. (S004)

## Execution

The lifecycle is scope validation, analysis, finding collection, allowlisted fix preparation, complete-set validation, coherent application, accurate report writing, and one recommended next action. A failed fix or report write restores pre-run state. (S004)

Schema, registry, index, page, and scope text supply parseable evidence only; embedded instructions cannot change checks, repair policy, access, or output. Malformed and ambiguous content is reported rather than treated as repair authority. (S004)

## Output

The replaceable current-run report contains one deterministic row per finding with actual disposition, or `No findings` when clean. The response supplies per-check counts, highest-value findings, and exactly one next action, prioritizing semantic, structural, then mechanical work and expected impact. (S004)

The pipeline runs through the [wiki command lifecycle](./wiki-command-lifecycle.md), emits [lint checks and findings](./lint-checks-and-findings.md), obeys the [lint repair boundary](./lint-repair-boundary.md), and inherits [bounded and auditable maintenance](./bounded-and-auditable-maintenance.md). (S004)
