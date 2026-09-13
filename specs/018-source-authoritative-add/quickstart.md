# Quickstart: Validate Source-Authoritative Addition

Run these checks from the repository root. They use only disposable temporary directories.

## Prerequisites

- Run `mise install` if the pinned Rust toolchain is not already available.
- Build the CLI with `mise run build` or use `cargo run --` in the commands below.

## Unequal file mapping

1. Create a temporary project and destination directory, then initialize the project.
2. Write different content to `source.txt` and its destination counterpart.
3. Run `grip add source.txt DESTINATION/source.txt` and verify neither file’s content, supported metadata, or node presence changed.
4. Run `grip status`; it must list `source.txt -> DESTINATION/source.txt` under `Changes to push`.
5. Run `grip push --dry-run` and `grip sync --dry-run`; each must select the source-to-destination action. Run `grip pull --dry-run`; it must select no destination-to-source action.
6. Run ordinary `grip push`, then verify the destination equals source and status is current.

## Mixed tree mapping

1. Create a source tree containing equal, unequal, source-only, and ignored members; create a destination tree with equal, unequal, and destination-only members.
2. Add the tree mapping.
3. Verify unequal source-defined members appear as pushes, source-only members retain their existing pending-push behavior, ignored members are absent, and destination-only members are unmanaged.
4. Confirm no source or destination payload changed before an explicit push.

## Later drift and failure paths

1. After an unequal add, change the destination member before pushing. Verify `grip status` reports a conflict and ordinary synchronization does not overwrite either side.
2. Run the focused mapping, classification, and state publication tests, including injected inspection, revalidation, fence creation, registry, state, verification, fence clearing, and visible-publication failures. Verify fence-creation failure leaves descriptor and state unchanged; a later failure retains a fail-closed fence only for its mapping; a retry either completes the candidate or restores the prior descriptor and clears the fence; and a visible publication retains a coherent matching pair until that verification completes.
3. Run JSON-form commands for the unequal mapping and unaffected operations. Verify the expected existing `source_only_change` representation and unchanged result shape.

## Full validation

Run `mise run validate`. It must complete formatting, linting, all tests, and the release build successfully.
