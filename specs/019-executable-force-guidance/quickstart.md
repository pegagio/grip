# Quickstart: Verify Executable Force-Resolution Guidance

Use this focused sequence to prove that every suggested force command is executable and aggregate conflicts receive a safe inspection step.

## Focused CLI contracts

Run the status and directional-mutation contract suites while implementing the feature.

```bash
cargo test --test classification_cli_contract
cargo test --test push_cli_contract
cargo test --test pull_cli_contract
cargo test --test sync_cli_contract
```

The fixtures should prove both sides of the contract:

- An exact file conflict prints both force commands, and their `--dry-run` forms pass the existing exact-entry selector validation.
- An aggregate tree-root conflict prints `Run: grip diff SOURCE`, prints no force command, and preserves the rejection of an attempted tree-root force resolution.
- A blocked mutation presents the same guidance as status for its corresponding conflict and does not cross-associate guidance when more than one blocker exists.

## Full validation

Before handoff, run the repository validation workflow.

```bash
mise run validate
```

Review `git diff --check` and manually confirm that JSON output and normal `grip diff` output are unchanged by the feature.
