# Implementation Plan: Wiki Status and Resumption

**Branch**: `feature/time-machine-wiki-status` | **Date**: 2026-08-26 | **Spec**: [spec.md](./spec.md)

## Summary

Refine status into a strictly read-only session-resumption snapshot built only from structural wiki metadata. The command will validate its bounded scope, summarize indexed pages and registered sources without reading page bodies, label unavailable freshness as unknown, surface prioritized lint evidence, and select exactly one deterministic evidence-backed next action.

## Technical Context

**Language/Version**: Markdown command definitions and YAML 1.2-compatible configuration

**Primary Dependencies**: Spec Kit `>=0.2.0`; wiki schema, index, source registry, and optional lint report

**Storage**: Read-only plain Markdown structural artifacts

**Testing**: Static contract checks and disposable fixtures in `quickstart.md`

**Target Platform**: Any AI integration supported by Spec Kit

**Project Type**: Spec Kit documentation extension

**Performance Goals**: Default output fits on one screen; `full` expands each slice by at most three times

**Constraints**: Zero writes; zero page-body, feature-artifact-body, or original-source reads; active-feature inspection is metadata-only; structural text is untrusted; missing metadata is unknown; exactly one next action

**Scale/Scope**: Default, one configured page type, or bounded `full`

## Constitution Check

The placeholder constitution has no enforceable gates. The design preserves read-only operation, explicit provenance, bounded access, deterministic output, and user authority over follow-up changes.

**Pre-design gate**: PASS.

**Post-design gate**: PASS — no runtime, dependency, or persistence change.

## Project Structure

```text
specs/005-wiki-status-resumption/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/wiki-status-command.md
└── checklists/requirements.md

commands/speckit.wiki.status.md
extension.yml
README.md
```

**Structure Decision**: Keep status as one declarative command with explicit read boundaries and recommendation rules.

## Complexity Tracking

No violations or new abstractions.
