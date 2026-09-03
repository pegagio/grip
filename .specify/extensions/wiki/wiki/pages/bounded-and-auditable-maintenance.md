---
title: Bounded and auditable maintenance
type: decision
sources:
  - S001
  - S002
  - S003
  - S004
  - S005
updated: 2026-08-26
---

# Bounded and auditable maintenance

The extension turns informal guidance into explicit, configurable caps for pages touched per ingest, words per page, pages loaded per query, and rendered context tokens. Overflow is deferred visibly rather than allowed to sprawl. (S001)

Every written claim requires a source ID, and contradictory claims remain side by side under a conflict marker until a human resolves them. Silent replacement is rejected because a shared wiki must make changes of mind reviewable. (S001)

Lint separates deterministic repairs from judgment calls: index drift and unambiguous relative link-target changes may be fixed mechanically, while contradictions, staleness, and other semantic conclusions remain report-only. Prose is not automatically rewritten. (S001, S004)

Sources remain immutable, `query` and `status` are read-only, and lint writes are limited to the index, link targets, and lint report. Commands load only the index and relevant page slices rather than the entire wiki. (S001)

Initialization adds two further safety boundaries: it canonicalizes the configured wiki directory and rejects locations outside the project before writing, and it treats an existing schema as authority rather than reconstructing missing companion artifacts. (S002)

Query narrows selected pages until both the page slice and rendered-context budget hold, validates only the registry identities required by selected claims, and never repairs malformed or stale structures encountered while answering. (S003)

Lint gathers findings before preparing fixes, validates the entire allowlisted fix set before applying it, and restores pre-run state if a fix or report write fails. Its report records actual dispositions rather than intended changes. (S004)

Status reads fixed structural artifacts and bounded metadata slices, never page or source bodies. Default output fits on one screen, page-type scope filters only the page slice, and `full` expands each slice by no more than three times. (S005)

These controls constrain the [wiki command lifecycle](./wiki-command-lifecycle.md), [wiki foundation state](./wiki-foundation-state.md), [cited query evidence pipeline](./cited-query-evidence-pipeline.md), [wiki health-check pipeline](./wiki-health-check-pipeline.md), and [wiki status snapshot](./wiki-status-snapshot.md), apply through [configuration resolution](./configuration-resolution.md), enforce the [query trust boundary](./query-trust-boundary.md), [lint repair boundary](./lint-repair-boundary.md), and [status read boundary](./status-read-boundary.md), and preserve the accumulation promised by the [LLM Wiki model](./llm-wiki-model.md) in a [repository-oriented wiki](./repository-oriented-wiki.md). (S001, S002, S003, S004, S005)
