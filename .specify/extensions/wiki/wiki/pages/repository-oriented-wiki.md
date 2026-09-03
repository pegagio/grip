---
title: Repository-oriented wiki
type: decision
sources:
  - S001
  - S002
updated: 2026-08-26
---

# Repository-oriented wiki

The extension deliberately adapts a pattern motivated by personal knowledge bases into a project wiki stored with the repository. (S001)

The wiki is plain Markdown that can be committed, reviewed in pull requests, and shared by teammates and agents. Typed pages make decisions and their rationale directly reviewable. (S001)

Spec Kit artifacts are first-class sources because research and plan decisions contain knowledge that should survive the feature that produced it. Hooks after planning and implementation provide natural ingestion points. (S001)

Status is a first-class operation because agent sessions are discontinuous; files provide the durable memory from which context is rebuilt. (S001)

The extension remains a declarative Spec Kit capability built from Markdown commands, YAML configuration, and its manifest, with no executable runtime or external service. (S002)

The design applies the [LLM Wiki model](./llm-wiki-model.md) through the [wiki command lifecycle](./wiki-command-lifecycle.md), follows the [declarative extension architecture](./declarative-extension-architecture.md), uses [bounded and auditable maintenance](./bounded-and-auditable-maintenance.md), and distinguishes accumulated knowledge from regenerable material at the [code documentation boundary](./code-documentation-boundary.md). (S001, S002)
