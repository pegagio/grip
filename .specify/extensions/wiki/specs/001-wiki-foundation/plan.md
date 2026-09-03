# Implementation Plan: Wiki Foundation

**Branch**: `feature/time-machine-wiki-foundation` | **Date**: 2026-08-26 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/001-wiki-foundation/spec.md`

## Summary

Reconstruct and tighten the existing wiki-initialization contract so a maintainer can create the schema, empty index, and empty source registry once; safely repeat initialization; and resolve documented configuration precedence without allowing the configured state directory to escape the project boundary. The implementation remains a declarative Spec Kit command backed by the existing extension manifest and configuration template.

## Technical Context

**Language/Version**: Markdown command definitions and YAML 1.2-compatible extension configuration

**Primary Dependencies**: Spec Kit `>=0.2.0`; an AI integration capable of executing Spec Kit prompt commands

**Storage**: Plain Markdown files in a configurable repository-relative wiki directory

**Testing**: Static contract inspection plus disposable-project acceptance scenarios defined in `quickstart.md`

**Target Platform**: Any local development environment and AI integration supported by Spec Kit

**Project Type**: Spec Kit documentation extension

**Performance Goals**: Complete initialization in one invocation while reading only configuration and the resolved schema-presence check

**Constraints**: No external service, network access, executable runtime, or third-party library; never overwrite an initialized wiki; never write outside the project root; preserve user-authored Markdown verbatim except for an explicitly appended scope item

**Scale/Scope**: One initialization command, one shared configuration template, one extension manifest entry, and three foundation artifacts per consuming project

## Constitution Check

The repository constitution is still an unratified placeholder and defines no enforceable gates. The plan therefore applies the repository instructions and the feature’s explicit safety requirements directly:

- **Small scope**: Changes stay within the initialization command and its directly related documentation or validation artifacts.
- **Preservation**: Existing wiki content is never regenerated, truncated, or normalized.
- **Project containment**: Configured state paths are resolved and checked before any write.
- **No new dependency**: The prompt-defined architecture remains unchanged.

**Pre-design gate**: PASS — no constitution violation or unjustified complexity.

**Post-design gate**: PASS — the command contract, data model, and validation guide preserve the same bounded prompt-only architecture.

## Project Structure

### Documentation (this feature)

```text
specs/001-wiki-foundation/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── wiki-init-command.md
└── checklists/
    └── requirements.md
```

### Source Code (repository root)

```text
commands/
└── speckit.wiki.init.md

config-template.yml
extension.yml
README.md
```

**Structure Decision**: Keep the feature implemented as one declarative command. `config-template.yml` and `extension.yml` remain the shared public configuration and registration surfaces; no executable source module is introduced.

## Complexity Tracking

No constitution violations or additional abstractions require justification.
