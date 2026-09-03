---
title: Wiki foundation state
type: component
sources:
  - S002
updated: 2026-08-26
---

# Wiki foundation state

The wiki foundation consists of one user-editable schema, one page index, and one source registry in the configured repository-relative wiki directory. Initialization creates these three artifacts together and does not create `pages/` or synthesize project knowledge. (S002)

## Schema authority

The schema defines scope, page taxonomy, citation and linking rules, conflict handling, page metadata, and maintenance workflows. Its existence is the sole initialized-state sentinel because it signals that maintainers may already own and intentionally edit the directory contents. (S002)

If the schema exists, initialization does not overwrite, regenerate, truncate, delete, or silently reconstruct any wiki artifact, even when the index or registry is missing. A repeated invocation without a new scope leaves all files unchanged and reports current status. (S002)

## Scope lifecycle

A scope statement is user-authored metadata preserved verbatim as a numbered schema item. A scope supplied during repeated initialization is appended without deduplication or rewriting, while every other existing byte remains unchanged. (S002)

## Relationships

[Configuration resolution](./configuration-resolution.md) selects and validates the foundation directory, the [declarative extension architecture](./declarative-extension-architecture.md) implements creation without a runtime, the [wiki command lifecycle](./wiki-command-lifecycle.md) grows and uses the resulting state, and [bounded and auditable maintenance](./bounded-and-auditable-maintenance.md) preserves it. (S002)
