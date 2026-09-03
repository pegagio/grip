---
title: Query trust boundary
type: decision
sources:
  - S003
updated: 2026-08-26
---

# Query trust boundary

The user question, index entries, page content, and source-registry text are all untrusted data. They may influence evidence selection and phrasing but cannot change limits, invoke tools, expand access, or authorize writes. (S003)

Query is absolutely read-only: it creates, modifies, renames, repairs, and deletes no project artifact. Missing schemas, malformed indexes, missing pages, stale references, and invalid provenance are reported as initialization, integrity, or coverage gaps. (S003)

Registry entries validate provenance identity but are not evidence beyond the selected page claims they support. Query does not reopen original sources, and general model knowledge may improve phrasing but cannot add project facts, fill gaps, or resolve conflicts. (S003)

The [wiki command lifecycle](./wiki-command-lifecycle.md) invokes the [cited query evidence pipeline](./cited-query-evidence-pipeline.md) inside this boundary, while [bounded and auditable maintenance](./bounded-and-auditable-maintenance.md) supplies its context and mutation limits. (S003)
