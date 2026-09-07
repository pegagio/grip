# Quickstart: Validate Reverse Synchronization

This guide defines runnable end-to-end evidence for Feature 006. Use only disposable temporary homes and mapping roots. The detailed interfaces are in [CLI contract](contracts/cli.md), [pull contract](contracts/pull.md), [filesystem contract](contracts/filesystem.md), and [storage contract](contracts/storage.md).

## Table of Contents

- [Prerequisites](#prerequisites)
- [Build and baseline validation](#build-and-baseline-validation)
- [Preview and apply one managed change](#preview-and-apply-one-managed-change)
- [Verify unmanaged destination reporting](#verify-unmanaged-destination-reporting)
- [Verify blocking and non-mutation](#verify-blocking-and-non-mutation)
- [Verify selectors and output parity](#verify-selectors-and-output-parity)
- [Verify recovery and partial failure](#verify-recovery-and-partial-failure)
- [Run the complete validation gates](#run-the-complete-validation-gates)
- [Recorded performance evidence](#recorded-performance-evidence)

## Prerequisites

From the repository root:

```bash
mise trust
mise install
```

Tests and manual scenarios must set `HOME` and `GRIP_HOME` to disposable directories. Never point these examples at real managed files.

## Build and baseline validation

```bash
mise run build
mise run test
```

The existing push, mapping, discovery, classification, baseline, state, and operation-record regressions must remain green after the shared mutation extraction.

## Preview and apply one managed change

Create a disposable file mapping with identical accepted source and destination files, accept its baseline, then change only the destination. The exact registry setup may use the existing test helpers or public mapping command.

```bash
grip --output json pull --dry-run
grip --output json pull
grip --output json status
```

Expected evidence:

- Dry run returns `operation: pull`, `direction: pull`, `mode: dry_run`, and one `replace_file` action.
- Dry run leaves both payloads and all Grip-owned state byte-for-byte unchanged.
- Execute mode preserves the prior source in operation-local recovery, replaces and verifies the source from destination, and publishes one accepted generation.
- Subsequent status reports the pair synchronized.
- The action retains mapping-role `source_path` and `destination_path`; direction determines which one was read and replaced.

## Verify unmanaged destination reporting

In a tree mapping with an accepted managed file, add a sibling only under the destination and change the accepted managed destination file. Preview pull.

```bash
grip --output json pull --dry-run
```

Expected evidence:

- The accepted changed entry has disposition `action`.
- The new destination-only sibling appears individually with classification `destination_only_unmanaged`, disposition `no_action`, and reason `unmanaged_destination`.
- The unmanaged item does not become an action or blocker.
- Executing the pull changes only the established source entry and never creates the unmanaged sibling under the source.

## Verify blocking and non-mutation

Create an isolated selected scope containing one eligible destination-only change plus one divergent conflict or unsupported managed collision.

```bash
grip --output json pull --dry-run
grip --output json pull
```

Expected evidence:

- Both commands report the complete blocker set and start zero actions.
- Neither command takes a mutation side effect, creates operation or recovery evidence, or publishes a baseline.
- A missing source or replaced source ancestry is reported under the inherited deletion or unsafe classification and is never recreated.

## Verify selectors and output parity

Exercise an omitted selector, a source-space mapping selector, a source-space managed-entry selector, and the corresponding destination-space forms.

```bash
grip pull --dry-run -- /tmp/grip-source/config
grip pull --dry-run --destination -- /tmp/grip-destination/config
grip --output json pull --dry-run -- /tmp/grip-source/config
```

Expected evidence:

- Equivalent source-space and destination-space selectors resolve the same Entry Identities and ordered actions. Their scopes retain the requested path space, so their scope-bound plan identities may differ.
- Human and JSON output communicate equivalent actions, unmanaged non-actions, blockers, recovery availability, and baseline outcome.
- JSON uses Result Envelope V1 and the shared mutation details with `direction: pull`.
- Dash-prefixed path components work after `--`; multiple positional paths fail with invalid usage.

## Verify recovery and partial failure

Use typed test-only fault injection at every source recovery, staging, publication, verification, final observation, baseline publication, and result-delivery boundary.

Expected evidence:

- Before source replacement, the recovery payload is private, operation-local, and verified against prior source state.
- The first action failure stops execution and leaves later actions unattempted.
- Completed source replacements remain visible and their recovery evidence remains available.
- A failed or partial operation publishes no new baseline unless state publication is explicitly reported visible with durability unconfirmed.
- A subsequent read-only status classifies actual filesystem state against the still-authoritative baseline.
- A later pull does not resume or modify a prior nonterminal operation record.
- Result-delivery failure after successful baseline publication returns exit `20` without revoking the accepted generation.

Run the focused suites after implementation:

```bash
cargo test --test pull_cli_contract
cargo test --test pull_planning
cargo test --test pull_filesystem_integration
cargo test --test pull_failure_integration
cargo test --test pull_recovery_integration
cargo test --test pull_contention_integration
cargo test --test operation_record_integration
```

## Run the complete validation gates

```bash
mise run validate
mise run performance
```

`validate` must pass formatting, Clippy with warnings denied, all default tests, and the release build. The ignored performance harness must document OS, architecture, Rust version, profile, fixture mix, and 100-run p95 results. Pull dry-run and pure execute-mode planning over 10,000 eligible managed entries must each complete within two seconds in at least 95 of 100 warm runs, without a persistent cache, background service, broad payload lock, or unmeasured parallelism.

Before implementation begins, generate tasks with `$speckit-tasks` and run `$speckit-analyze` after task generation as required by the project constitution.

## Recorded performance evidence

On 2026-09-06, `mise run performance` passed on macOS/aarch64 with Rust 1.98 in the release profile. The representative tree contained 10,000 eligible managed entries: 6,700 synchronized records and 3,300 accepted records changed only at the destination. The harness used 100 warm invocations for each measured command.

- **Pull dry-run p95**: 997.840917 ms
- **Pull execution-planning p95**: 834.299833 ms
- **Persistent acceleration**: no cache, background service, additional index, broad payload lock, or parallel execution was introduced

The complete run finished in 491.05 seconds. These results satisfy SC-008's requirement that at least 95 of 100 invocations complete within two seconds.
