# Implementation Plan: Contained Tree Mapping Overrides

**Branch**: `030-contained-tree-overrides` | **Date**: 2026-09-16 | **Spec**: [spec.md](./spec.md)

## Summary

Permit one narrow exact-file reservation inside a contained home tree while rejecting a current or later duplicate tree member. Correct human add rendering so only success results display mapping rows.

## Technical Context

**Language/Version**: Rust 1.98, edition 2024

**Storage**: Existing Descriptor V2 and State V4; no schema changes

**Testing**: Isolated mapping topology and CLI contract integration tests

## Constitution Check

| Principle | Plan response | Status |
|---|---|---|
| II. Explicit Ownership and Least Surprise | Permit only an exact reserved leaf and reject any current duplicate tree member. | PASS |
| III. Validate, Revalidate, and Verify | Validate candidates and loaded registries before operations; reject without publication. | PASS |
| V. Fast, Observable, and Testable | Use bounded path checks and isolated regression fixtures. | PASS |
| Merge-Bounded Persistence | Flow forward from Features 025 and 026 without rewriting verified artifacts. | PASS |

## Design

1. Narrow static ownership validation to permit an exact-file destination below a contained tree destination only when source namespaces are disjoint.
2. Validate the corresponding tree-source leaf on candidate construction and registry load; its presence is a duplicate ownership blocker.
3. Keep all other overlap and recursion rejections unchanged.
4. Render concise mapping success output only for successful outcomes.

## Validation Strategy

Add valid and invalid exact-override fixtures, a later-member registry-load regression, and human rejected-add output coverage. Run targeted topology and mapping CLI tests, formatting, Clippy, and artifact analysis before implementation.
