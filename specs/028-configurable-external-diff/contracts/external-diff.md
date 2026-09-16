# External Diff Contract

## Configuration Shape

`~/.grip/config.toml` is optional machine-wide tool preference. The selected project keeps `schema_version = 2` and mappings, adding the same optional profile sections.

```toml
# ~/.grip/config.toml
[diff]
tool = "visual"

[difftool.visual]
program = "/Applications/Visual Diff.app/Contents/MacOS/visual-diff"
args = ["--wait", "--label=source value"]
```

```toml
# selected-project/.grip/config.toml
schema_version = 2
mappings = []

[diff]
tool = "visual"

[difftool.visual]
args = ["--project-mode"]
```

The project inherits `program` and replaces the global arguments with exactly `--project-mode`.

## Selection and Invocation

| Invocation or condition | Required behavior |
|---|---|
| Non-empty `GRIP_EXTERNAL_DIFF` | Execute it with `SOURCE`, `DESTINATION`; ignore both profile layers and their arguments. |
| Project-selected named tool | Resolve from merged project-over-global fields. |
| Global selected named tool | Resolve from global definition. |
| No selection | Execute `diff SOURCE DESTINATION`. |
| Named tool | Execute `PROGRAM ARG_1 ... ARG_N SOURCE DESTINATION` as separate arguments. |
| Human `grip diff PATH` or `--destination PATH` with exact safe pair | Render detailed inspection and launch one child. |
| Human `grip diff` without PATH | Existing broad detailed inspection; no global-profile load or child. |
| `grip --output json diff ...` | Existing JSON result; no child. |

Grip never evaluates a shell, expands variables, redirects I/O, globs paths, or interpolates tokens. Shell-looking values remain literal; a wrapper executable is required for environment-specific extra arguments.

## Safety and Completion

Global fields merge first and matching project fields replace them. A missing global file is normal. Profile content is safely current-user-owned/no-follow read. An invalid relevant profile, empty program, unavailable executable, or launch failure reports a precise Grip diagnostic with no data changes.

Existing selector, ownership, classification, and no-follow checks remain authoritative. Before spawn both direct endpoints must be present supported files or directories. Missing, unsafe, unsupported, or ineligible endpoints do not launch a process.

| Child outcome | Required Grip behavior |
|---|---|
| Normal exit `N` | Report completion and exit `N` unchanged, including nonzero diff results. |
| Unix signal `S` | Identify `S` and exit `128 + S`. |
| Preflight/configuration/launch failure | Grip-owned diagnostic and existing error behavior; no child status claimed. |

The child inherits terminal I/O. Grip does not change JSON schema or route an already-started child's result through static result categories.
