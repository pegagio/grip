# Quickstart: Validate Simplified Default Command Output

Run these steps from the repository root after implementation. The scenarios use the isolated temporary-root fixtures already provided by the command-contract tests; they must not inspect or alter a real Grip project or home directory.

## Focused output contracts

Run the changed human-output suites:

```sh
cargo test --test mapping_cli_contract --test classification_cli_contract --test push_cli_contract --test pull_cli_contract --test sync_cli_contract
```

Verify the following outcomes against [the human-output contract](contracts/default-human-output.md):

1. Add, all-list, selected-list, and remove show declared mappings only, with the approved headings and mapping-kind behavior.
2. Push and pull previews/applies show the concise operation heading and correct arrow direction.
3. A forced push uses the same concise push result, without an extra winner or operation-evidence line.
4. A mixed sync result shows both pull and push rows under one heading with per-row arrows.
5. Status orders push, pull, conflicts, then needs-baseline; initial collisions and ordinary divergent conflicts show both concrete force-resolution choices, while technical blockers retain their existing detail.
6. No-action push, pull, and sync results say only `Nothing to …` for their public direction; baseline-only plans state that they would establish or established a baseline for the accepted-entry count.
7. Conflict-only blocked push, pull, sync, and forced-direction results show valid source-winning and destination-winning choices. Technical or mixed blocked results direct the operator to `grip status`, without raw planner IDs or baseline evidence.
8. A partial mutation failure preserves its completed-action count and directs the operator to `grip status` before retrying, without execution milestones, recovery, baseline, or operation-record evidence.
9. Parser and domain errors begin `Error:` while retaining the existing exit categories.

## Regression boundaries

Run the complete test suite:

```sh
cargo test --all --no-fail-fast
```

Confirm that JSON assertions, `grip diff` output tests, path-selector tests, dry-run non-mutation tests, forced-direction tests, and filesystem-safety tests still pass.

## Full validation

Run the repository validation gate:

```sh
mise run validate
```

The gate passes only when formatting, linting, all default tests, and the release build succeed.
