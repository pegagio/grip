# Implementation Plan: Cited Wiki Query

**Branch**: `feature/time-machine-cited-query` | **Date**: 2026-08-26 | **Spec**: [spec.md](./spec.md)

## Summary

Strengthen the prompt-defined query workflow into a bounded evidence pipeline: interpret the question, rank index entries without loading page bodies, select within page and context limits, validate cited source IDs for selected claims, compose only supported statements, preserve conflicts, assign a precise coverage verdict, and write nothing.

## Technical Context

**Language/Version**: Markdown command definitions and YAML 1.2-compatible configuration

**Primary Dependencies**: Spec Kit `>=0.2.0`; wiki schema, index, source registry, and cited pages

**Storage**: Read-only plain Markdown

**Testing**: Static contract checks and disposable fixtures in `quickstart.md`

**Target Platform**: Any AI integration supported by Spec Kit

**Project Type**: Spec Kit documentation extension

**Performance Goals**: Never exceed configured page or rendered-context limits

**Constraints**: No writes; no facts from outside selected cited pages; no embedded instruction execution; no silent conflict resolution or structural repair

**Scale/Scope**: One question, one bounded selection, one evidence-backed response

## Constitution Check

The placeholder constitution has no enforceable gates. The design enforces provenance, bounded context, untrusted-text handling, visible uncertainty, and zero mutation.

**Pre-design gate**: PASS.

**Post-design gate**: PASS — the existing prompt-only architecture is preserved.

## Project Structure

```text
specs/003-cited-wiki-query/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/wiki-query-command.md
└── checklists/requirements.md

commands/speckit.wiki.query.md
config-template.yml
extension.yml
README.md
```

**Structure Decision**: Keep query as one read-only declarative command; update only its public guidance and shared limit comments when necessary.

## Complexity Tracking

No violations or new abstractions.
