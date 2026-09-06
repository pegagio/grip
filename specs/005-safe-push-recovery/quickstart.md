# Quickstart: Validate Safe Push and Recovery

This guide validates Feature 005 end to end in one disposable root. It proves dry-run parity, source-to-destination additions and replacements, verified recovery, deterministic ordering, first-failure stopping, baseline truth, writer contention, interrupted-record coexistence, output failure, and representative planning performance.

## Prerequisites

- macOS or a supported Unix-like system
- `mise` trusted for this checkout
- The pinned Rust toolchain installed through `mise install`

All filesystem tests and commands must use isolated Grip homes and payload roots. They must never inspect or mutate the operator's real Grip home or files.

## Build and static validation

From the repository root:

```bash
mise trust
mise install
mise exec -- cargo fmt --check
mise exec -- cargo clippy --all-targets --all-features -- -D warnings
mise exec -- cargo test
mise exec -- cargo build --release
```

## Create an isolated accepted mapping

```bash
grip_push_root="$(mktemp -d)"
mkdir -p "$grip_push_root/grip-home" "$grip_push_root/source/editor/themes" "$grip_push_root/destination/editor/themes"
printf '%s\n' 'schema_version = 1' 'mappings = []' > "$grip_push_root/grip-home/config.toml"
printf '%s\n' 'original' > "$grip_push_root/source/editor/config.txt"
cp "$grip_push_root/source/editor/config.txt" "$grip_push_root/destination/editor/config.txt"
chmod 0644 "$grip_push_root/source/editor/config.txt" "$grip_push_root/destination/editor/config.txt"
GRIP_HOME="$grip_push_root/grip-home" mise exec -- cargo run -- mapping add tree "$grip_push_root/source/editor" "$grip_push_root/destination/editor"
GRIP_HOME="$grip_push_root/grip-home" mise exec -- cargo run -- baseline accept
printf '%s\n' "$grip_push_root"
```

Inspect the printed path and confirm it is the disposable root before continuing.

## Create actionable source state

```bash
printf '%s\n' 'source replacement' > "$grip_push_root/source/editor/config.txt"
chmod 0600 "$grip_push_root/source/editor/config.txt"
printf '%s\n' 'new theme' > "$grip_push_root/source/editor/themes/new.txt"
chmod 0644 "$grip_push_root/source/editor/themes/new.txt"
```

Expected classification: `config.txt` is `source_only_change`, `themes/new.txt` is `source_addition`, and existing directory entries are no-action evidence.

## Prove dry-run non-mutation and parity

Capture source, destination, Grip-state, and metadata snapshots through the test support utility, then run both aliases:

```bash
GRIP_HOME="$grip_push_root/grip-home" mise exec -- cargo run -- --output json push --dry-run
GRIP_HOME="$grip_push_root/grip-home" mise exec -- cargo run -- --output json push -n
```

Expected outcome:

- both commands exit `0` with `result: "planned"`;
- both return the same `plan_id`, entries, actions, blockers, and counts;
- parent actions precede descendants and remaining actions follow canonical identity order;
- `operation_record` is `null` and baseline outcome is `not_attempted`;
- destination payload, accepted state, operation directories, recovery entries, and mutation lock are unchanged or absent.

## Execute and verify the push

```bash
GRIP_HOME="$grip_push_root/grip-home" mise exec -- cargo run -- --output json push
cmp "$grip_push_root/source/editor/config.txt" "$grip_push_root/destination/editor/config.txt"
cmp "$grip_push_root/source/editor/themes/new.txt" "$grip_push_root/destination/editor/themes/new.txt"
GRIP_HOME="$grip_push_root/grip-home" mise exec -- cargo run -- check
```

Expected outcome:

- push exits `0` with `result: "applied"`;
- every action is completed, visible, verified, and durable;
- the replacement has a verified recovery reference while the addition records no prior payload;
- destination file bytes and supported modes equal their sources;
- exactly one new accepted baseline generation contains the actioned identities;
- check exits `0` and reports the scope synchronized;
- unrelated destination-only content remains byte- and metadata-identical.

Decode the operation record and recovery metadata through the strict storage contract test helper. Do not edit private state by hand.

## Validate no-op behavior

```bash
GRIP_HOME="$grip_push_root/grip-home" mise exec -- cargo run -- --output json push
```

Expected outcome: `result: "no_op"`, no operation record, no recovery entry, no lock artifact creation beyond an already persistent valid lock file, and no new baseline generation.

## Validate complete preflight blocking

Create divergent changes after an accepted baseline:

```bash
printf '%s\n' 'source conflict' > "$grip_push_root/source/editor/config.txt"
printf '%s\n' 'destination conflict' > "$grip_push_root/destination/editor/config.txt"
GRIP_HOME="$grip_push_root/grip-home" mise exec -- cargo run -- --output json push
printf 'push exit=%s\n' "$?"
```

Expected outcome: exit `10`, `completion: "blocked"`, every discovered blocker present, zero started actions, no operation record or recovery entry, no baseline change, and both conflicting payloads untouched.

Use the planning suite to cover every inherited classification and mixed scopes with more than one blocker.

## Validate partial failure and recovery evidence

Run the deterministic fault suites rather than inducing unsafe failures manually:

```bash
mise exec -- cargo test --test push_failure_integration
mise exec -- cargo test --test push_recovery_integration
```

Expected coverage includes failure before and after each durable milestone. For every case, assert completed, failed, and unattempted actions; publication visibility; verification; durability; recovery references; exact accepted baseline bytes and generation; and operation-record last-known state.

A failure after one completed action must preserve that destination and its recovery evidence, stop before later actions, and leave the prior baseline authoritative. Automatic rollback must never occur.

## Validate writer coordination

```bash
mise exec -- cargo test --test push_contention_integration
mise exec -- cargo test --test baseline_integration
mise exec -- cargo test --test mapping_registry_integration
```

Expected outcome: push, mapping publication, and changed baseline acceptance obey `mutation → registry → state`; active contention exits `13` with safe owner details; unlocked stale metadata is overwritten after advisory acquisition; malformed or unsafe lock nodes fail closed; dry runs and no-ops do not contend.

## Validate interrupted-operation coexistence

Use the recovery suite to leave a valid nonterminal operation record and release the advisory lock. A subsequent push must:

1. preserve the prior journal and recovery bytes exactly;
2. perform a fresh complete inspection and build a new plan;
3. create a distinct operation ID only if it is about to mutate;
4. never resume, complete, or roll back the interrupted operation.

Corrupt operation or recovery evidence must fail closed instead of being ignored or repaired.

## Validate output failure

Run the failing-writer case in the CLI contract suite:

```bash
mise exec -- cargo test --test push_cli_contract
```

Expected outcome: if payload verification and baseline publication succeeded before output failed, the published baseline remains authoritative, process exit is `20`, stderr contains the concise output diagnostic, and the operation summary records `result_delivery: "failed"` when that best-effort checkpoint succeeds.

## Run focused suites

```bash
mise exec -- cargo test --test push_planning
mise exec -- cargo test --test push_cli_contract
mise exec -- cargo test --test push_filesystem_integration
mise exec -- cargo test --test push_failure_integration
mise exec -- cargo test --test push_recovery_integration
mise exec -- cargo test --test push_contention_integration
mise exec -- cargo test --test baseline_integration
mise exec -- cargo test --test state_integration
```

## Measure representative performance

```bash
mise exec -- cargo build --release
mise exec -- cargo test --release --test performance_acceptance -- --ignored --nocapture
```

The Feature 005 harness creates exactly 10,000 eligible entries: 100 directories, 3,300 source additions, 3,300 source-only replacements, and 3,300 no-action files. It warms and measures exactly 100 JSON dry-run planning invocations plus 100 internal execute-mode planning invocations stopped before coordination or mutation, verifies equivalent semantic plans, byte-identical repeated dry-run output, and unchanged payload/state snapshots, records environment, mix, and both planning distributions, and requires at least 95 dry runs within two seconds.

No cache, persistent index, parallel traversal, background service, or new dependency may be introduced unless the harness fails and profiling identifies a concrete bottleneck.

## Measured acceptance results

The disposable-root scenarios were executed on 2026-09-06 through the focused suites and the repository-owned `mise run validate` and `mise run performance` workflows. All fixtures used temporary roots outside the operator's real Grip home.

- **Build and static validation**: Formatting, Clippy with warnings denied, the complete default test suite, documentation tests, and the optimized release build passed.
- **Planning and execution**: Dry-run parity, all classification dispositions, dependency ordering, additions, replacements, private recovery, explicit parent creation, scoped baseline publication, no-op behavior, and complete blocking passed.
- **Failure and recovery**: Pre- and post-side-effect fault cases, first-failure stopping, action checkpoint state, baseline non-publication, pre-rename and post-rename State V2 faults, interrupted-record coexistence, corrupt component rejection, and output-delivery failure finalization passed.
- **Coordination and regressions**: Mutation-lock ownership and stale metadata, mapping and baseline participation, State V1/V2, registry, discovery, classification, non-UTF-8 identity, and unsupported-node regressions passed.
- **Representative performance**: On macOS ARM64 with Rust 1.98 in the release profile, p95 was 948.758083 ms for 100 JSON dry runs and 833.376917 ms for 100 internal execute-mode planning passes over the same deterministic 10,000-entry mixed plan. The dry-run p95 satisfied the two-second gate.

## Cleanup

After confirming the printed path is the disposable root created above, remove only that exact directory using the environment's normal recoverable cleanup workflow. Never adapt cleanup to a broad path, an empty variable, a real home directory, or the repository root.
