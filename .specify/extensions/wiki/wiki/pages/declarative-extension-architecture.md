---
title: Declarative extension architecture
type: decision
sources:
  - S002
updated: 2026-08-26
---

# Declarative extension architecture

The wiki foundation remains implemented by a Markdown command, a YAML configuration template, and the extension manifest rather than an executable runtime. This preserves compatibility across Spec Kit AI integrations and matches the architecture of the other wiki commands. (S002)

The design introduces no external service, network access, third-party library, or executable source module. Static contract inspection and disposable-project scenarios validate the prompt-defined behavior. (S002)

A shell or Python initializer was rejected because it would add platform and packaging dependencies without user value. A shared command library was rejected because the extension has no runtime module system and initialization alone owns the foundation writes. (S002)

This architecture creates the [wiki foundation state](./wiki-foundation-state.md), participates in the [wiki command lifecycle](./wiki-command-lifecycle.md), and keeps the [repository-oriented wiki](./repository-oriented-wiki.md) portable and reviewable. (S002)
