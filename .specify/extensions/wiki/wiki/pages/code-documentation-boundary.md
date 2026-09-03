---
title: Code documentation boundary
type: decision
sources:
  - S001
updated: 2026-08-26
---

# Code documentation boundary

The boundary between generated code documentation and an LLM Wiki is whether knowledge can be derived again from the repository. (S001)

OpenWiki targets code documentation such as module behavior and component connections that an agent can regenerate when the code changes. (S001)

The LLM Wiki preserves knowledge that regeneration cannot recover, including why an alternative was rejected, which external constraint shaped a design, and what an outage taught. (S001)

The approaches can compose: a generated OpenWiki page may become an ingest source when a code-level fact deserves a permanent, cited home alongside the decision it influenced. (S001)

This boundary narrows the purpose of the [repository-oriented wiki](./repository-oriented-wiki.md) while preserving the accumulation goal of the [LLM Wiki model](./llm-wiki-model.md). (S001)
