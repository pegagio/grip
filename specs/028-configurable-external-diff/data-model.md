# Data Model: Configurable External Diff Program

## Configuration Documents

| Entity | Fields | Validation and ownership |
|---|---|---|
| Selected-project Descriptor V2 | `schema_version`, `mappings`, optional `diff`, optional `difftool` | Retains strict mapping rules. The optional profile fragment is preserved by mapping publication and parsed only for an eligible handoff. |
| Machine-wide profile | optional `diff`, optional `difftool` | Read only from `~/.grip/config.toml`; no mappings/state role; absent or safely owned/no-follow readable. |
| Diff selection layer | optional `tool` | Names one named tool. |
| Named tool layer | optional `program`, optional `args` | Partial definitions compose across layers. `args` distinguishes absent from explicit `[]`. |

Only `schema_version`, `mappings`, `diff`, and `difftool` are accepted at the project root. The machine-wide document accepts profile keys only. Unknown keys remain errors, while malformed profile content is deferred until external-diff resolution.

## Effective Profile

| Field | Merge rule | Effective validation |
|---|---|---|
| Tool selection | Global then project replacement | Selected name must resolve. |
| `program` | Global then project replacement | Selected tool needs a non-empty program. |
| `args` | Global then complete project replacement | Each string is one literal argument. |
| `GRIP_EXTERNAL_DIFF` | Highest non-empty value | Executable only; ignores profile arguments. |
| Default | No environment or named selection | `diff`, no arguments. |

## Comparison Handoff and Completion

| Field or state | Meaning |
|---|---|
| `source` / `destination` | Direct resolved endpoints for one exact selected managed file, directory, or tree member. |
| `program` / `arguments` | Direct executable and literal configured tokens followed by source and destination. |
| `origin` | `environment`, `named_tool`, or `default` for diagnostics. |
| `not_requested` | JSON or unselected human diff; existing inspection only. |
| `preflight_blocked` | Endpoints are not exact, present, supported, or safe; no child. |
| `configuration_failed` / `launch_failed` | Grip diagnostic; no child exit exists. |
| `exited` | Return child normal code unchanged. |
| `signaled` | Identify signal and return `128 + signal`. |

```text
requested -> inspection -> rendered result (JSON/no selector; no child)
          -> exact human selection -> endpoint validation -> profile resolution
          -> direct child -> normal code / signal-derived code
```
