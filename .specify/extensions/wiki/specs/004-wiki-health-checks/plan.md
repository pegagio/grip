# Implementation Plan: Wiki Health Checks

**Branch**: `feature/time-machine-wiki-health` | **Date**: 2026-08-26 | **Spec**: [spec.md](./spec.md)

## Summary

Refine lint into a bounded analyze-then-apply workflow: validate scope, treat wiki text as untrusted data, run six evidence-producing checks, prepare only allowlisted deterministic repairs, validate the full fix set, apply it coherently, replace the derived report, and recommend one highest-value action without changing semantic content.

## Technical Context

**Language/Version**: Markdown command definitions and YAML 1.2-compatible configuration

**Primary Dependencies**: Spec Kit `>=0.2.0`; wiki schema, index, registry, and pages

**Storage**: Plain Markdown pages, index, registry, and replaceable lint report

**Testing**: Static contract checks and disposable fixtures in `quickstart.md`

**Target Platform**: Any AI integration supported by Spec Kit

**Project Type**: Spec Kit documentation extension

**Performance Goals**: Contradiction and page-scoped work stays within linked or shared-source neighborhoods

**Constraints**: Semantic content is report-only; auto-fixes are allowlisted; all wiki text is untrusted; partial fix sets are forbidden

**Scale/Scope**: Six checks across full, named-check, or page-neighborhood scope

## Constitution Check

The placeholder constitution has no enforceable gates. The design preserves user authority over semantics, deterministic repair boundaries, bounded comparisons, and verifiable evidence.

**Pre-design gate**: PASS.

**Post-design gate**: PASS — no runtime or dependency change.

## Project Structure

```text
specs/004-wiki-health-checks/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/wiki-lint-command.md
└── checklists/requirements.md

commands/speckit.wiki.lint.md
config-template.yml
extension.yml
README.md
```

**Structure Decision**: Keep lint as one declarative command with an explicit repair allowlist and derived report contract.

## Complexity Tracking

No violations or new abstractions.
