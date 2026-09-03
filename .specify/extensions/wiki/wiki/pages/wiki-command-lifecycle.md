---
title: Wiki command lifecycle
type: component
sources:
  - S001
  - S002
  - S003
  - S004
  - S005
updated: 2026-08-26
---

# Wiki command lifecycle

The extension maps the LLM Wiki model to four ongoing maintenance commands with distinct responsibilities, while initialization is a separate foundation operation. (S001, S002)

## Initialize

`/speckit.wiki.init` resolves configuration and creates the schema, empty index, and empty source registry exactly once without creating knowledge pages. Later invocations preserve existing state, optionally append an explicitly supplied scope item, and otherwise report status. (S002)

## Ingest

`/speckit.wiki.ingest` registers one immutable source, extracts durable knowledge, performs cap-bounded page updates, and maintains the index. (S001)

Spec Kit research and plan decisions are first-class sources, and `after_plan` and `after_implement` hooks align ingestion with moments when spec-driven development produces knowledge. (S001)

## Query

`/speckit.wiki.query` loads a bounded page slice, requires citations, and reports honestly when wiki coverage is insufficient. (S001)

Query decomposes the question into material parts, ranks index entries before reading bodies, validates selected claims against registered source IDs, and emits exactly one coverage verdict. Empty input returns a bounded structural overview. (S003)

The [cited query evidence pipeline](./cited-query-evidence-pipeline.md) defines selection and provenance, [coverage verdicts](./coverage-verdicts.md) expose support and gaps, and the [query trust boundary](./query-trust-boundary.md) keeps the operation read-only. (S003)

## Lint

`/speckit.wiki.lint` performs six health checks while keeping semantic findings report-only and limiting automatic repair to mechanical drift. (S001)

Lint accepts full-wiki, named-check, or page-neighborhood scope; analyzes before preparing changes; applies only a validated fix set; replaces the derived current-run report; and recommends exactly one highest-value unresolved action. (S004)

The [wiki health-check pipeline](./wiki-health-check-pipeline.md) defines execution, [lint checks and findings](./lint-checks-and-findings.md) define diagnostics, and the [lint repair boundary](./lint-repair-boundary.md) preserves semantic authority. (S004)

## Status

`/speckit.wiki.status` rebuilds context from files so work can resume after an agent restart, compaction, or context-window overflow. (S001)

Status reads structural metadata only, renders a bounded snapshot of scope, pages, sources, freshness, and open issues, and selects exactly one evidence-backed next action without reading knowledge bodies or changing project state. (S005)

The [wiki status snapshot](./wiki-status-snapshot.md) defines the resumption view, the [status next-action policy](./status-next-action-policy.md) makes continuation deterministic, and the [status read boundary](./status-read-boundary.md) prevents hidden retrieval or repair. (S005)

The lifecycle implements the [LLM Wiki model](./llm-wiki-model.md), starts from the [wiki foundation state](./wiki-foundation-state.md), resolves policy through [configuration resolution](./configuration-resolution.md), follows the [repository-oriented design](./repository-oriented-wiki.md) and [declarative extension architecture](./declarative-extension-architecture.md), and enforces [bounded and auditable maintenance](./bounded-and-auditable-maintenance.md). (S001, S002, S003, S004, S005)
