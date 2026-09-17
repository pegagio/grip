# Implementation Plan: Current Status Entries

**Branch**: `032-current-status-entries` | **Date**: 2026-09-16 | **Spec**: [spec.md](./spec.md)

## Summary

Retain current status records during human grouping and render them through the existing status-section helper using `=`. Keep the rendering change isolated from classification, JSON serialization, and exit behavior.

## Technical Context

**Language/Version**: Rust 1.98, edition 2024

**Storage**: None

**Testing**: Result-renderer unit tests and classification CLI contract tests

## Constitution Check

| Principle | Plan response | Status |
|---|---|---|
| II. Explicit Ownership and Least Surprise | Show the existing owned record identities without expanding discovery or ownership. | PASS |
| III. Validate, Revalidate, and Verify | Presentation-only change leaves inspection and mutation evidence unchanged. | PASS |
| V. Fast, Observable, and Testable | Reuse deterministic rendering and focused tests with no additional filesystem work. | PASS |
| Merge-Bounded Persistence | Flow forward from Feature 015 and Feature 029 without revising their history. | PASS |

## Design

1. Group non-attention records as current records rather than discarding them after counting.
2. Render `Current:` before action sections with the existing declared-source display fallback and destination display.
3. Leave machine output and action grouping untouched.

## Validation Strategy

Update mixed and clean human-status expectations, verify JSON remains unchanged, and run focused CLI tests, full validation, and artifact analysis.
