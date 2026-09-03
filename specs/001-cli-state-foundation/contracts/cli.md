# CLI Contract: Feature 001

This contract defines the complete Feature 001 command surface and output boundaries. Later features may add commands and result categories without changing these meanings.

## Command grammar

```text
grip [--output <human|json>] [-v|--verbose]... <COMMAND>

COMMAND := validate | version
```

`--output` is global and defaults to `human`. `--verbose` enables diagnostic detail and may be repeated for greater detail. Application results use stdout; enabled diagnostics use stderr.

The parser also provides conventional text-only `-h`/`--help` and `-V`/`--version` displays on stdout with exit `0`. These metadata displays are outside the Result Envelope contract and never resolve, create, or validate the Grip Home.

## Application commands

### `grip validate`

Resolve and validate the selected Grip Home, `config.toml`, and optional `state/state.json`. The command is read-only: it does not create a missing home, registry, state directory, state file, lock, staging file, backup, baseline, or payload entry.

An absent selected Grip Home or required `config.toml` returns `invalid_configuration` and exit `10`. An empty or relative `GRIP_HOME` has the same result. Grip never falls back to another root and never consults `/etc/grip/` or another machine-wide configuration location.

Successful human output summarizes the exact selected root, registry validity, and state condition. Successful JSON `details` contains:

```json
{
  "grip_home": "/example/absolute/grip-home",
  "registry": "valid",
  "state": "uninitialized"
}
```

`state` is `uninitialized` or `valid`. Errors use the applicable category and may add safe path and document-kind fields without exposing document contents.

### `grip version`

Report the Grip application version without requiring a Grip Home. JSON `details` contains one stable field:

```json
{
  "version": "0.1.0"
}
```

## Machine-readable result

Every application-command result requested with `--output json` is exactly one JSON object on stdout followed by a newline:

```json
{
  "schema_version": 1,
  "status": "ok",
  "code": "ok",
  "message": "Grip configuration and state are valid",
  "details": {}
}
```

No diagnostic or parser prose may share stdout with this object. When a syntactically valid `--output json` request accompanies an unknown command, missing command, or incompatible argument, Grip returns an `invalid_usage` envelope and exit `2`. If the output option itself is malformed so no valid mode was selected, Grip uses conventional parser error text on stderr and exit `2`.

## Exit-code contract

| Exit | Symbol | Meaning |
|-----:|--------|---------|
| 0 | `ok` | Command completed successfully |
| 2 | `invalid_usage` | Invalid command or arguments |
| 10 | `invalid_configuration` | Invalid Grip Home policy or registry |
| 11 | `unsupported_schema` | Unsupported registry or state version |
| 12 | `corrupt_state` | Malformed, inconsistent, or integrity-failed state |
| 20 | `operational_failure` | Permission, I/O, output, or other runtime failure |

Exit `13` and symbolic code `state_contention` are reserved for the first public command that publishes Grip-owned state. Feature 001's internal state-publication API reports contention to its caller, but neither `validate` nor `version` acquires the publication lock or emits this public result.

If stdout fails, Grip exits `20`; because the requested result channel is unavailable, it may emit one concise best-effort notice to stderr. This is the sole case where an application failure cannot guarantee a complete result in the selected mode.

## Non-mutation contract

Help, version, validation, parser failure, and rendering failure do not create or modify the Grip Home or anything outside it. Feature 001 never inspects or mutates mapped payload paths, version-control state, `/etc/grip/`, or privileged locations.
