# Quickstart: Simplify Status Output

Use this guide to validate Feature 015 after implementation. It exercises the default human status contract in [contracts/status-output.md](contracts/status-output.md), while keeping the existing machine-readable result intact.

## Prerequisites

- Run automated tests with isolated temporary project and metadata roots; do not use a personal project or home directory.
- Use mappings that can produce synchronized, source-only, conflicting, and metadata-blocked records.
- Run commands from the repository root.

## Validation Scenarios

1. Create a scope with two synchronized entries and run `grip status`. Confirm one concise summary reports two current entries and no action needed, without itemized paths or technical diagnostics.
2. Create one source-only change, one destination-only change, one conflict, and one matching pair that needs a baseline alongside a synchronized entry. Run `grip status`. Confirm the summary categories add to five and that nonempty sections use `->`, `<-`, `<->`, and `>-<` respectively, with source and destination paths on every row.
3. Exercise nonblocking non-directional deletion-convergence and metadata-migration-ready records. Confirm both are shown under `Needs baseline` with `>-<`, without asserting that their payloads match or suggesting a copy direction.
4. Create a blocking unknown metadata condition and an excluded metadata attribute that requires no action. Run `grip status`. Confirm the blocking entry is visible with a plain-language explanation, while the informational attribute and raw reason names are absent.
5. Select an empty path scope. Confirm `grip status PATH` reports no managed entries rather than a synchronized result.
6. Repeat the mixed and metadata scenarios with `grip status -o json` and `grip status -e`. Confirm the JSON record values remain unchanged and the documented exit behavior is unchanged.
7. Run `grip diff` for a mixed scope. Confirm the feature has not replaced its existing detailed inspection output with the concise status view.

## Automated Validation

Run the focused output contracts while developing, then the complete repository gate:

```bash
cargo test --test classification_cli_contract
cargo test --test metadata_cli_contract
cargo test --test product_acceptance
mise run validate
```

The focused suites cover status summaries, actionable ordering, metadata suppression and blocking visibility, JSON compatibility, exit behavior, and the existing detailed inspection boundary. The full validation task checks formatting, linting, the complete test suite, and the release build.
