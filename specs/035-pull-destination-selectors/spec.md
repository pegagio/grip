# Feature Specification: Pull Destination Selectors

**Created**: 2026-09-17
**Status**: Complete

## Requirements

- **FR-001**: `grip pull PATH` MUST interpret `PATH` in destination space by default.
- **FR-002**: `grip pull -s PATH` and `grip pull --source PATH` MUST interpret `PATH` in source space.
- **FR-003**: `grip pull --force DESTINATION` MUST select exactly one destination entry and make that destination state authoritative for the corresponding source entry.
- **FR-004**: Human conflict guidance for the destination-winning choice MUST use the executable default form `grip pull --force DESTINATION`.
- **FR-005**: Ordinary pull MUST remain conservative: a source-only change, including a permission-only change, is not overwritten without `--force`.

## Success Criteria

- A destination path accepted by `pull` and the corresponding source path accepted by `pull --source` resolve to the same managed entry.
- A source permission drift can be restored from an unchanged destination by `pull --force DESTINATION`.

## Scope

This feature flows forward from Features 011, 019, and 034. It changes pull selector interpretation and permits exact destination-winning force to restore an unchanged destination state over source-only drift; pull direction, aggregate force scope, and timestamp policy remain unchanged.

## Governance

This feature is governed by Constitution Principles II (explicit ownership), III (validated forced direction), V (isolated filesystem regression coverage), and the merge-bounded flow-back model. It adds no persistent state, framework, concurrency mechanism, or broader force scope.
