# Implementation Plan: Knowledge Ingestion

**Branch**: `feature/time-machine-knowledge-ingestion` | **Date**: 2026-08-26 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/002-knowledge-ingestion/spec.md`

## Summary

Tighten the existing prompt-defined ingestion workflow into a bounded prepare-then-commit transaction: validate and normalize one curated source, treat its contents as untrusted evidence, derive durable cited items, select only indexed topic pages within configured limits, preserve conflicts, and synchronize pages, registry, and index only after all planned changes are valid.

## Technical Context

**Language/Version**: Markdown command definitions and YAML 1.2-compatible extension configuration

**Primary Dependencies**: Spec Kit `>=0.2.0`; initialized wiki schema and registries; host-provided file and optional URL reading

**Storage**: Plain Markdown schema, source registry, index, and typed topic pages

**Testing**: Static prompt-contract checks plus disposable-project scenarios in `quickstart.md`

**Target Platform**: Any local development environment and AI integration supported by Spec Kit

**Project Type**: Spec Kit documentation extension

**Performance Goals**: Read one source once, load the index once, and touch no more pages than the configured cap

**Constraints**: No source mutation; no instruction execution from source content; project containment for local sources; citation completeness; visible conflicts; bounded context; no partial state on validation or source-read failure

**Scale/Scope**: One source identity and at most the configured number of affected topic pages per invocation

## Constitution Check

The placeholder constitution defines no enforceable gates. The design applies the feature’s explicit safety contract:

- **Trust boundary**: Source text is evidence, never instruction.
- **Provenance**: Every changed claim maps to a stable source ID.
- **Bounded work**: Index-first selection and hard page limits prevent repository-wide loading.
- **Preservation**: Conflicts and immutable inputs remain visible and unchanged.
- **Atomicity**: Validate the intended registry, page, and index changes before committing them.

**Pre-design gate**: PASS.

**Post-design gate**: PASS — no new runtime, service, or abstraction is introduced.

## Project Structure

### Documentation (this feature)

```text
specs/002-knowledge-ingestion/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── wiki-ingest-command.md
└── checklists/
    └── requirements.md
```

### Source Code (repository root)

```text
commands/
└── speckit.wiki.ingest.md

extension.yml
README.md
```

**Structure Decision**: Keep ingestion in one declarative command and update only its directly related public documentation. Shared configuration remains owned by the foundation feature.

## Complexity Tracking

No constitution violations or additional abstractions require justification.
