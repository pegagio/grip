# Feature Specification: Optional Modification-Time Detection

**Created**: 2026-09-17
**Status**: Implemented

## Requirements

- **FR-001**: Grip MUST ignore file and directory modification times by default.
- **FR-002**: `status`, `diff`, `push`, `pull`, and `sync` MUST accept `-m` and `--use-modification-time` to opt one operation into modification-time detection.
- **FR-003**: Opted-in push, pull, and sync MUST transfer a modification-time-only change.
- **FR-004**: Other managed metadata, including permission mode, MUST remain unchanged.

## Success Criteria

- A Git checkout that changes only file timestamps reports clean default status.
- `status -m` reports a timestamp-only change and `push -m` applies it.

## Scope

This feature flows forward from Feature 033. It changes default file timestamp treatment and adds per-operation opt-in; it does not add persistent configuration.
