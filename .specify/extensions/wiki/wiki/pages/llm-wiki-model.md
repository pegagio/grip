---
title: LLM Wiki model
type: concept
sources:
  - S001
updated: 2026-08-26
---

# LLM Wiki model

An LLM Wiki is a persistent, LLM-maintained knowledge layer that accumulates synthesized understanding instead of rediscovering it from raw documents for every question. (S001)

## Layers

- **Raw sources** are immutable documents that remain the ground truth. (S001)
- **Wiki pages** are synthesized Markdown with cross-references that form the working knowledge layer. (S001)
- **The schema** defines the wiki structure and maintenance contract that commands obey. (S001)

## Division of labor

Humans select sources, ask questions, and resolve conflicts; the LLM summarizes, maintains citations and links, and keeps related pages consistent. (S001)

## Operations

The model grows through ingest, serves cited answers through query, and receives regular health checks through lint. (S001)

This model is implemented by the [wiki command lifecycle](./wiki-command-lifecycle.md), adapted for a [repository-oriented wiki](./repository-oriented-wiki.md), protected by [bounded and auditable maintenance](./bounded-and-auditable-maintenance.md), and distinguished from regenerable material at the [code documentation boundary](./code-documentation-boundary.md). (S001)
