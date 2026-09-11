# Data Model: Source Path Input Normalization

## Source Input

This is a source-space CLI argument before Grip records or looks up a mapping.

| Attribute | Rules |
|---|---|
| `spelling` | UTF-8 relative text submitted by the operator. |
| Accepted lexical components | Names, repeated separators, current-directory components, and parent components that normalize within the selected project. |
| Normalized result | One canonical project-relative path with no repeated separators, dot components, trailing separator, or escape above the project root. |
| Reserved result | `.grip` and every descendant are rejected after normalization. |
| Invalid input | Empty, absolute, non-UTF-8, environment-expansion form, or any form that normalizes outside the project or within `.grip`. |

## Project-Relative Declaration

This is the stored source value in a portable mapping.

| Attribute | Rules |
|---|---|
| `value` | Canonical project-relative path created from a normalized source input or read from a strict descriptor. |
| Project root | Exact `.` is valid only for a tree mapping. |
| Descriptor validation | Existing noncanonical declarations are rejected; they are never normalized while loading. |
| Resolution | Joined beneath the selected project root before existing endpoint inspection. |

## Portable Mapping

| Attribute | Rules |
|---|---|
| `kind` | Existing file or tree mapping kind inferred and validated by `add`. |
| `source` | Project-Relative Declaration; used for declaration identity, lookup, and persistence. |
| `destination` | Existing destination declaration; unchanged by this feature. |
| Identity | Uses the normalized stored source, mapping kind, and destination declaration. Equivalent accepted source spellings therefore name one mapping. |

## Lifecycle

1. An operator supplies a source argument to `add` or another source-space command.
2. Grip validates UTF-8 and lexically normalizes it relative to the selected project.
3. If the normalized result is valid and outside `.grip`, Grip uses that declaration for lookup, mapping identity, or endpoint validation.
4. `add` persists only the normalized declaration after existing ownership and endpoint checks succeed.
5. Later source-space commands normalize their selector identically before selecting a mapping or scope.
6. Descriptor loading continues to accept only already-normalized declarations.
