# Implementation Plan: Local Artifact Release Automation

**Branch**: `specs/024-local-artifact-release-automation` | **Date**: 2026-09-15 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/024-local-artifact-release-automation/spec.md`

## Summary

Add one `mise run release` entry point that verifies a clean local `master` candidate, runs the established release gates, creates a macOS Apple Silicon archive and matching SHA-256 checksum in `dist/`, and creates an annotated local `v<package-version>` tag only after all preparation succeeds. The implementation uses a small repository-owned shell script for the ordered, failure-safe workflow; the task does not push, upload, or contact GitHub.

## Technical Context

**Language/Version**: Rust 2024 application, Rust 1.98 toolchain; Bash-compatible local release script; Python 3.11 is already pinned by mise for metadata extraction when needed

**Primary Dependencies**: Cargo, mise, Git, macOS `tar`, and `shasum`; no new runtime or third-party dependency

**Storage**: Ignored project-local `dist/` directory for final archive/checksum files and attempt-owned temporary staging; local Git refs for the annotated tag

**Testing**: Existing `mise run validate` and `mise run performance` gates; a repository-owned shell contract test using isolated temporary Git repositories and command stubs for failure injection

**Target Platform**: macOS on Apple Silicon (`Darwin` / `arm64`); task fails before release preparation on another platform

**Project Type**: Local Rust CLI with repository-local developer tooling

**Performance Goals**: Preserve existing performance acceptance; the release wrapper adds no product-runtime work and performs one release build through the existing gates

**Constraints**: Clean attached `master` checkout; valid package version; no existing final artifact/checksum/tag for that version; no external publication action; tag must identify the captured clean `HEAD`; incomplete staging must never be advertised as a release. Required local validation may use installed build tooling; the prohibition is Git push, GitHub/API/upload, and package-registry publication rather than a blanket offline guarantee.

**Scale/Scope**: One release candidate at a time; one macOS Apple Silicon archive, checksum, and local tag per project version

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

| Principle | Plan response | Status |
|---|---|---|
| I. Proportional Rigor | Use one small local script and existing tools rather than CI, GitHub APIs, a release service, or a new dependency. | Pass |
| II. Explicit Ownership and Least Surprise | The workflow touches only ignored `dist/` outputs and an explicitly requested local tag. It does not alter Grip mappings, user payloads, or remote state. | Pass |
| III. Validate, Revalidate, and Verify | Check repository identity and collisions before work, run both existing release gates, stage artifacts privately, verify checksum/content, then recheck identity/collisions before finalization and tagging. | Pass |
| IV. Bounded Concurrency | Avoid locks and services; use attempt-owned staging and final collision rechecks so another local attempt cannot silently replace the selected release identity. | Pass |
| V. Fast, Observable, and Testable | Reuse existing validation gates and add isolated contract coverage for success and failure boundaries. Clear stage-specific diagnostics make operator failures actionable. | Pass |
| Product boundaries | Local tag creation is approved release tooling, not a Grip runtime command. No push, GitHub, registry, daemon, or remote operation is added. | Pass |
| Merge-bounded persistence | Plan, research, contract, quickstart, task list, script, tests, and README updates remain one reviewable Feature 024 change set. | Pass |

**Post-design re-check**: Pass. The design introduces no constitutional exception or durable product behavior beyond the explicitly authorized local release workflow.

## Project Structure

### Documentation (this feature)

```text
specs/024-local-artifact-release-automation/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── release-task.md
└── tasks.md              # Created by speckit-tasks
```

### Source Code (repository root)

```text
mise.toml                             # Adds the release task entry point
scripts/
└── prepare-release.sh                # Ordered local release-preparation workflow
tests/
└── release_task_contract.sh          # Isolated Git/artifact task contract coverage
README.md                             # Documents release preparation and manual publication
.gitignore                            # Already ignores dist/; no change expected
dist/                                 # Ignored generated artifacts, never committed
```

**Structure Decision**: Keep product code untouched. A repository-owned script isolates ordered shell and Git behavior from the concise mise task declaration; a shell contract test is appropriate because the interface spans Git, archive, checksum, and task-runner behavior rather than Grip's Rust runtime.

## Complexity Tracking

No constitutional violations require tracking.
