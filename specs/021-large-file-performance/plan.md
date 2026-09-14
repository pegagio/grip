# Implementation Plan: Large-File Operation Performance

**Branch**: `working` | **Date**: 2026-09-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/021-large-file-performance/spec.md`

## Summary

Make release-build `grip add` and `grip status` complete within one second p95 for 100 warm runs against isolated, differing approximately 19 MB source and destination regular files. First capture and record the baseline; only if it shows an over-target workflow and attributes the latency to duplicate complete-file observation, remove that duplication within an observation pass while preserving content fingerprinting, two-pass stale-evidence detection, add fencing, mapping publication, initial comparison state, and status classification. Extend the existing ignored release performance harness and focused CLI contracts without changing public commands, output, schemas, or persistent state.

## Technical Context

**Language/Version**: Rust 1.98, edition 2024

**Primary Dependencies**: clap, serde, serde_json, sha2, rustix; no new dependencies

**Storage**: Existing project descriptor and accepted baseline state; no schema or migration changes

**Testing**: Existing Rust unit, integration, and CLI-contract suites; ignored release performance acceptance harness; `mise run validate`

**Target Platform**: Local macOS ARM64 release build for performance acceptance; existing macOS and Unix filesystem contract remains unchanged

**Project Type**: Single Rust CLI

**Performance Goals**: `grip add` and `grip status` each meet p95 <= 1 second across 100 warm release-build runs against isolated differing approximately 19 MB regular-file workloads; record p50, p95, and maximum

**Constraints**: Preserve SHA-256 content identity, descriptor-bound no-follow observation, complete metadata checks, two-pass stale-evidence detection, add publication fencing, baseline semantics, and human/JSON output. The performance harness must fail with an explicit diagnostic when its release binary or isolated temporary fixture cannot be prepared; it must never fall back to an operator-managed path. Do not add persistent caching, indexing, background processes, broad locks, parallelism, schemas, or public options.

**Scale/Scope**: Two isolated workflows only: mapping creation with an existing unequal source/destination pair, and status for that established mapping. Other commands are non-regression-only and remain out of scope.

## Constitution Check

*GATE: Pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Plan compliance |
|---|---|
| I. Proportional Rigor for a Local Tool | Measure the reported workflows first, remove only demonstrated redundant work, and introduce no persistent or background mechanism. |
| II. Explicit Ownership and Least Surprise | The feature changes neither mapping ownership nor endpoint payload mutation; `add` and `status` retain their existing command contracts. |
| III. Validate, Revalidate, and Verify | Keep full fingerprints, before/after descriptor evidence, two complete observation passes, readonly revalidation, and fenced add publication checks. |
| IV. Bounded Concurrency | Do not add broad payload locking or attempt to prevent external edits; preserve existing drift errors and narrow Grip-owned publication locking. |
| V. Fast, Observable, and Testable | Extend the existing isolated release harness to 100 warm samples, record distributions, assert deterministic results, and add focused safety/semantic regression coverage. |
| Merge-Bounded Persistence | Keep measured evidence, research, design artifacts, tasks, implementation, and validation aligned in this mutable Feature 021 directory. |

**Gate result before research**: Pass. The plan proposes no constitutional exception.

**Gate result after design**: Pass. The design retains all existing content-evidence and publication safeguards; it adds only operation-local deduplication of duplicate file observations.

## Project Structure

### Documentation (this feature)

```text
specs/021-large-file-performance/
├── plan.md              # This file ($speckit-plan command output)
├── research.md          # Phase 0 output ($speckit-plan command)
├── data-model.md        # Phase 1 output ($speckit-plan command)
├── quickstart.md        # Phase 1 output ($speckit-plan command)
├── contracts/           # Phase 1 output ($speckit-plan command)
└── tasks.md             # Phase 2 output ($speckit-tasks command - NOT created by $speckit-plan)
```

### Source Code (repository root)
```text
src/
├── lib.rs                         # `add` and `status` orchestration; fenced add reinspection
├── observation/
│   ├── mod.rs                     # Stable two-pass observation and file-mapping observation ownership
│   └── fingerprint.rs             # Existing descriptor-bound full-file fingerprint authority
├── classification/
│   └── mod.rs                     # Existing classification semantics, unchanged
├── discovery/
│   └── mod.rs                     # Existing inventory records reused by observation
└── state/
    ├── add_fence.rs               # Existing fenced add publication authority, unchanged
    └── publication.rs             # Existing state publication/revalidation authority, unchanged

tests/
├── performance_acceptance.rs       # Ignored release benchmark extended with add/status large-file workloads
├── mapping_cli_contract.rs          # Add publication and unequal-endpoint semantics
├── classification_cli_contract.rs   # Status classification/output semantics
└── support/                         # Existing isolated project helpers
```

**Structure Decision**: Keep observation ownership and command orchestration in their existing modules. If the recorded baseline establishes the proposed cause, normal file mappings use the discovery-backed observation already produced in a pass instead of being fingerprinted again by a separate file-mapping pre-pass. Preserve a fallback for mappings not represented by discovery records, and retain both stable-observation passes. The harness remains in `tests/performance_acceptance.rs`; no benchmark framework, service, or persisted state is introduced.

## Phase 0: Research

Research findings are recorded in [research.md](research.md). They establish the benchmark extension, the measured duplicate-observation hypothesis, the safety invariants that cannot be weakened, and the rejection of metadata-only equality, persistent caching, parallel execution, or a new performance framework.

## Phase 1: Design

The design artifacts define operation-local benchmark evidence in [data-model.md](data-model.md), the regression and compatibility contract in [contracts/large-file-performance.md](contracts/large-file-performance.md), and the runnable validation sequence in [quickstart.md](quickstart.md).

## Complexity Tracking

No constitution violations or additional complexity require justification.
