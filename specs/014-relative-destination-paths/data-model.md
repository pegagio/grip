# Data Model: Relative Destination Paths

This feature retains the existing mapping model while adding a project-relative form to a destination declaration.

## Mapping declaration

| Field | Meaning | Validation and persistence |
|---|---|---|
| `kind` | Whether the mapping is one file or a source-defined tree. | Existing mapping kind rules remain unchanged. |
| `source` | Source declaration relative to the Grip project. | Existing strict, normalized source contract remains unchanged. |
| `destination` | User-authored target declaration. | Preserve exact accepted UTF-8 spelling: absolute, `~`, `~/…`, or a non-empty project-relative path. |

The destination declaration is part of portable mapping identity. Distinct lexical declarations remain distinct descriptor values even when they resolve to the same operational endpoint.

## Destination declaration forms

| Form | Example | Persisted declaration | Operational base |
|---|---|---|---|
| Absolute | `/opt/grip-dst/app` | Exact input | The declaration itself |
| Home | `~` | Exact input | Selected home |
| Home-relative | `~/deploy/../app` | Exact input | Selected home |
| Project-relative | `../grip-dst/./app/` | Exact input | Selected Grip project root |

All forms derive a separate lexically normalized absolute operational path before existing filesystem inspection. This derivation does not rewrite the declaration or follow symbolic links.

## Resolved mapping

A resolved mapping combines the portable declaration with source and destination operational endpoints and their inspection evidence. Existing lifecycle behavior remains unchanged:

1. Parse and retain the portable declaration.
2. Resolve the source from the selected project root and the destination from its declared base.
3. Inspect both operational endpoints and validate complete-registry ownership and topology.
4. Publish only the validated portable declarations and retain them through add, remove, state binding, and rebinding.

Equivalent resolved endpoints remain subject to existing duplicate, equal-endpoint, and recursive-topology failures regardless of their declared spelling.
