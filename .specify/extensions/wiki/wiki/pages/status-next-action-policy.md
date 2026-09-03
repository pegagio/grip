---
title: Status next-action policy
type: decision
sources:
  - S005
updated: 2026-08-26
---

# Status next-action policy

Status emits exactly one continuation supported by evidence read during the current invocation. It does not present a generic menu or invent a project path. (S005)

Candidates are selected in this priority order: (S005)

1. Resolve one evidenced conflict while naming its page and source IDs without choosing a winning claim. (S005)
2. Run lint when the report is absent or older than a later source ingestion. (S005)
3. Ingest one concrete available artifact when the wiki has no pages. (S005)
4. Ingest one concrete available, un-ingested active-feature artifact. (S005)
5. Run one query grounded in recorded scope, or set or probe scope when it is unset. (S005)

Ties resolve by policy priority, newest relevant evidence, then lexical page or source identity, making repeated status runs stable against unchanged state. (S005)

The [wiki status snapshot](./wiki-status-snapshot.md) renders the selected action, the [wiki command lifecycle](./wiki-command-lifecycle.md) executes the recommended follow-up separately, and the [status read boundary](./status-read-boundary.md) prevents recommendation from becoming mutation authority. (S005)
