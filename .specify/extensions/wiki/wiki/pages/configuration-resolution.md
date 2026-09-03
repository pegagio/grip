---
title: Configuration resolution
type: concept
sources:
  - S002
updated: 2026-08-26
---

# Configuration resolution

Each wiki configuration field resolves independently in descending authority: invocation override, `SPECKIT_WIKI_*` environment override, saved project configuration, then extension default. Missing higher-authority values fall through without changing how other fields resolve. (S002)

Numeric page, word, query, and context limits must be positive whole numbers; the staleness threshold must be a non-negative whole number; citation policy must be Boolean; auto-fix must be `none` or `index-and-links`; and page types must be a non-empty list of unique names. (S002)

The configured wiki directory is resolved canonically against the project root before any initialization read or write. The project root itself and descendants are allowed, while a resolved location outside that boundary is rejected without changing either the project or the outside target. (S002)

This ordering preserves stable project defaults, permits automation-specific environment settings, and ensures explicit one-run intent wins without sacrificing repository containment. (S002)

Configuration selects the [wiki foundation state](./wiki-foundation-state.md), supplies policy to the [wiki command lifecycle](./wiki-command-lifecycle.md), and enforces the initialization boundaries described by [bounded and auditable maintenance](./bounded-and-auditable-maintenance.md). (S002)
