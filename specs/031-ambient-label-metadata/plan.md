# Implementation Plan: Ambient Label Metadata and Add Diagnostics

**Branch**: `031-ambient-label-metadata` | **Date**: 2026-09-16 | **Spec**: [spec.md](./spec.md)

## Summary

Treat the one demonstrated opaque macOS metadata-label name family as unmanaged while retaining the allowlisted metadata boundary. Render baseline-admission failures as concise managed-entry blockers rather than a generic evidence message.

## Technical Context

**Language/Version**: Rust 1.98, edition 2024

**Storage**: Existing in-memory observation and accepted baseline state; no schema change

**Testing**: Metadata filesystem and CLI-contract integration tests

## Constitution Check

| Principle | Plan response | Status |
|---|---|---|
| II. Explicit Ownership and Least Surprise | Exclude only the named ambient label family and retain blocking behavior for all other unknown metadata. | PASS |
| III. Validate, Revalidate, and Verify | Keep unknown metadata as an admission blocker; diagnostics expose names but never values. | PASS |
| V. Fast, Observable, and Testable | Reuse existing xattr classification and deterministic result rendering with isolated fixture tests. | PASS |
| Merge-Bounded Persistence | Flow forward from Feature 030 without changing its historical artifact. | PASS |

## Design

1. Extend xattr policy with a strict byte-prefix test for `com.apple.metadata:kMDLabel_` and require a nonempty suffix.
2. Reuse the existing excluded-xattr evidence path so labels are neither fingerprinted nor transferred.
3. Add a dedicated human renderer for `baseline_not_acceptable` records that lists blocking unknown-xattr findings by affected endpoint and path.
4. Preserve JSON records and their existing value-free representation.

## Validation Strategy

Add policy, successful-admission, retained-unknown-blocker, and human-rendering tests. Run focused metadata and mapping CLI tests, complete Rust tests, formatting, Clippy, and artifact analysis.
