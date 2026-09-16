# Quickstart: Verify Aggregate Forced Push

Use an isolated temporary project and never a real Grip home or managed tree.

## Build and Initialize

```bash
mise exec -- cargo build
grip init --source "$TMPDIR/grip-source" --destination "$TMPDIR/grip-destination"
```

Create at least three managed entries. Arrange one divergent entry, one source-present entry with a missing destination peer, and one already-converged entry. If supported by the existing complete-state contract, include a source-absent mapped entry in its own focused case.

## Dry Run

```bash
grip push --force --dry-run
```

Confirm that the report names the complete selected-project scope and source-winning changes. Compare payload trees and accepted-state records before and after; they must be unchanged.

## Execute

```bash
grip push --force
```

Confirm divergent and missing-peer destinations now match their source-complete states, unchanged entries remain unchanged, and each completed entry has accepted evidence.

## Partial Failure

Use an existing focused test fault or controlled external drift to fail a deterministically later entry after an earlier entry completes. Confirm:

- Earlier verified entries retain accepted evidence.
- The aggregate result is failed.
- The failed entry is not accepted.
- Later entries are reported as unattempted and remain unchanged.

## Compatibility and Safety

Verify that `grip push <selector> --force` remains exact-entry only. Verify a destination-leaf symbolic link and unsafe symlink ancestry block aggregate force, and that an aggregate preflight blocker causes no destination or accepted-state mutation.

## Automated Validation

Run focused tests while implementing, then the full suite:

```bash
cargo test --test push_cli_contract
cargo test --test push_planning
cargo test --test push_filesystem_integration
cargo test --test push_failure_integration
cargo test --test operation_record_integration
cargo test
mise run validate
```

Also run the existing performance acceptance coverage when the aggregate planner is finalized, and compare its traversal or hashing measurements with the pre-feature baseline.
