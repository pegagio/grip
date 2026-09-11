# Quickstart: Destination Path Forms

Use this guide to validate Feature 012 after implementation. It exercises the public CLI and refers to [contracts/destination-paths.md](contracts/destination-paths.md) for the complete path contract.

## Prerequisites

- Run from an initialized Grip project containing `README.md`.
- Use isolated temporary project, home, and destination roots for automated tests.
- Ensure at least one endpoint exists for each `add` scenario, as required by the existing mapping contract.

## Validation Scenarios

1. Run `grip add README.md /opt/grip-dst/README.md` against an existing isolated absolute destination. Confirm the command records a mapping without copying payloads and the descriptor preserves the absolute spelling.
2. Run `grip add README.md '~/Working/../grip-dst//README.md'` against an isolated home-relative destination. Confirm the command succeeds and the descriptor retains the quoted lexical spelling exactly.
3. Run `grip add README.md '~'` with a valid home endpoint. Confirm standalone `~` remains accepted.
4. Run `grip add README.md grip-dst/README.md`. Confirm the command fails, reports allowed destination forms, and leaves descriptor bytes unchanged.
5. Reload accepted mappings with `grip list`, inspect them with `grip status`, and select them in destination space where applicable. Confirm each command resolves the declaration for operation while displaying the stored declaration text.
6. Attempt to add overlapping mappings whose distinct declarations resolve to the same destination namespace, such as `~/a/../b` and `~/b`. Confirm existing ownership validation blocks the ambiguity.

## Automated Validation

Run focused tests while developing, then the complete repository gate:

```bash
cargo test --test portable_mapping_model
cargo test --test portable_mapping_integration
cargo test --test mapping_cli_contract
cargo test --test registry_integration
mise run validate
```

The focused suites cover parsing, persistence, reloading, CLI behavior, and registry serialization. The full validation task checks formatting, linting, tests, and the release build.
