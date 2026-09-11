# Data Model: Current-Directory Status Paths

This feature adds transient command context only. It does not alter on-disk state, mappings, baselines, or serialized JSON schemas.

## Invocation working directory

The command captures the process current working directory once for project-aware execution.

| Field | Purpose | Persistence |
|---|---|---|
| `working_directory` | Base for human source-path presentation and source-side `push` selector resolution | In-memory only |

The working directory may be nested within, alongside, or outside an explicitly selected project.

## Human status projection

The default human status renderer receives a projection separate from the serialized classification details.

| Field | Purpose | Persistence |
|---|---|---|
| Raw source path | Authoritative source path used to calculate a relative display | In-memory only |
| Source display path | Git-style human rendering relative to `working_directory` | In-memory only |
| Existing destination path | Canonical destination display retained from the classification record | Existing JSON and human output behavior |

This separation ensures that the canonical `SafePath` values in `CommandOutcome.details` remain the JSON source of truth.

## CWD-relative push selector

A source-side `push` selector becomes a candidate path through these conceptual states:

| State | Rule |
|---|---|
| CLI text | The user supplies a relative source selector to `push`; absolute source selectors retain their existing rejection |
| Candidate source path | Relative input is resolved against `working_directory` |
| Lexical boundary | Normalized candidate must be within the selected project root |
| Canonical boundary | Existing candidate paths must resolve within the selected project root, including through symlinks |
| Selected source path | Only a contained candidate proceeds to the existing selection and mutation workflow |

Destination selector parsing and all non-push selector parsing retain the existing portable project-relative model.
