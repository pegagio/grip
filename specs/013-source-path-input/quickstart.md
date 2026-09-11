# Quickstart: Source Path Input Normalization

Use this guide to validate Feature 013 after implementation. It exercises the public CLI and refers to [contracts/source-paths.md](contracts/source-paths.md) for the complete input contract.

## Prerequisites

- Run from an initialized Grip project containing an `app` directory and a valid destination path.
- Use isolated temporary project, home, and destination roots for automated tests.
- Ensure at least one endpoint exists for each `add` scenario, as required by the existing mapping contract.

## Validation Scenarios

1. Run `grip add ./app/ ~/app` when `app` exists in the selected project. Confirm the command records a tree mapping without copying either endpoint and the descriptor declares `app` rather than `./app/`.
2. Run `grip list ./app/` and `grip remove ./app/`. Confirm each finds the `app` mapping.
3. Add a matching source and destination, then use `./app/` as the source-space selector for `status`, `diff`, `push --dry-run`, `pull --dry-run`, and `sync --dry-run`. Confirm each resolves the same source scope as `app`.
4. Attempt `grip add ../app ~/app`, `grip add .grip ~/app`, and `grip add nested/../.grip ~/app`. Confirm each fails before a descriptor, state, or payload change.
5. Attempt a relative destination such as `grip add ./app/ ../destination`. Confirm the source is accepted but the destination remains rejected under its existing contract.
6. Load a descriptor containing a noncanonical stored source such as `./app`. Confirm normal descriptor validation rejects it rather than rewriting it.

## Automated Validation

Run focused tests while developing, then the complete repository gate:

```bash
cargo test --test portable_mapping_model
cargo test --test portable_mapping_integration
cargo test --test project_reserved_metadata_integration
cargo test --test mapping_cli_contract
mise run validate
```

The focused suites cover lexical normalization, strict descriptor parsing, persistence, source-space selection, and reserved-path rejection. The full validation task checks formatting, linting, tests, and the release build.
