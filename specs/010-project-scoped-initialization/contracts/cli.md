# CLI Contract: Project-Scoped Grip

## Global Form

```text
grip [--project PATH] [--output human|json] [-v...] COMMAND
```

`--project PATH` is global for parsing and may appear before or after a project-dependent subcommand. It names the exact project root. A relative value is resolved against the invocation directory. It never means “start discovery here.”

`--project` with `init` or the application `version` command is invalid usage. Clap-provided help/version, command help, and `grip version` access no project metadata. All other commands are project-dependent, including `validate`, `mapping`, `status`, `check`, `diff`, `baseline`, `push`, `pull`, `sync`, `resolve`, `delete`, `retire`, and `recovery`.

## Initialization

```text
grip init [PATH]
```

When omitted, `PATH` is the invocation directory. The target must already exist and must be the exact directory to initialize. Grip creates only `.grip/config.toml` and `.grip/.gitignore`. It does not create the target, `.grip/state/`, a Git repository, mappings, baselines, recovery, operation records, or payloads.

Success details contain the safe canonical project root and `initialization: initialized|already_initialized`. Human output reports the same root and outcome. Exact reinitialization is a no-op.

## Mapping Arguments

```text
grip [--project PATH] mapping add file SOURCE DESTINATION
grip [--project PATH] mapping add tree SOURCE DESTINATION
grip [--project PATH] mapping inspect [SOURCE]
grip [--project PATH] mapping show SOURCE
grip [--project PATH] mapping remove SOURCE
```

`SOURCE` uses project-relative syntax. Exact `.` is permitted only for a tree mapping. `DESTINATION` is exact `~` or `~/RELATIVE_PATH`. No shell, tilde-user, or environment expansion is performed.

Mapping results expose both forms without treating resolved values as portable intent:

```json
{
  "project": {"root": {"display": "/safe/project"}},
  "mapping": {
    "declared": {"kind": "tree", "source": "home/editor", "destination": "~/.config/editor"},
    "resolved": {
      "source": {"display": "/safe/project/home/editor"},
      "destination": {"display": "/safe/home/.config/editor"}
    }
  }
}
```

Safe-path objects retain the existing escaping rules.

## Selectors

Project-dependent positional `PATH` selectors use project-relative source space by default. Existing `--destination` flags switch only selector interpretation to exact `~` or `~/...` destination space; they do not reverse operation direction.

Selectors must resolve within the selected mapping and cannot select `.grip`. An absolute selector, traversal, environment expression, other-user tilde, ambiguous mapping, or symlink escape fails before payload access.

## Output and Finalization

Every outcome after successful selection includes the same retained project root in human and JSON output. The command does not rediscover the project when persisting an operation result or rendering output. Discovery failures contain candidate evidence but no false selected-project field. Version/help output has no project field.

Result Envelope V1 remains the top-level JSON contract. Project and portable/resolved mapping details are additive values within command details.

## Stable Errors

| Category | Exit | Stable reasons |
|---|---:|---|
| `invalid_usage` | 2 | Inapplicable `--project` or malformed portable argument. |
| `invalid_configuration` | 10 | `project_not_found`, `ambiguous_project`, `invalid_project_root`, `invalid_project_metadata`, `nested_project`, `project_changed`, or invalid mapping. |
| `unsupported_schema` | 11 | Descriptor or local-state schema is unsupported. |
| `corrupt_state` | 12 | Integrity, ordering, portable identity, binding, or recovery evidence is corrupt. |
| `state_contention` | 13 | A project-local writer lock is held. |
| `operational_failure` | 20 | Filesystem inspection, publication, or revalidation failed. |

Existing specific operation reasons remain where applicable. Invalid explicit selection never falls back to discovery or `GRIP_HOME`.
