---
title: Coverage verdicts
type: concept
sources:
  - S003
updated: 2026-08-26
---

# Coverage verdicts

Coverage is derived deterministically by mapping valid evidence to the material parts of a question rather than by judging the confidence of the generated prose. (S003)

- **Covered** means every material part is supported by selected evidence with valid registered provenance. (S003)
- **Partial** means some material parts are supported; the result cites those answers, identifies every exact unsupported part, and suggests a concrete likely source to ingest. (S003)
- **Uncovered** means no material part is supported; the result gives no factual project answer and suggests a concrete likely source category or path. (S003)

An unresolved conflict is reported as a conflict set containing each incompatible cited position and its provenance. Query does not select a winner or merge incompatible claims. (S003)

A provenance failure or structural integrity problem becomes a coverage gap rather than an improvised answer or silent repair. (S003)

The [wiki command lifecycle](./wiki-command-lifecycle.md) invokes the [cited query evidence pipeline](./cited-query-evidence-pipeline.md), which assigns exactly one verdict after validating selected claims. (S003)
