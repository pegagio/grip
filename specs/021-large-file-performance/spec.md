# Feature Specification: Large-File Operation Performance

**Feature Branch**: `working`

**Created**: 2026-09-14

**Status**: Complete

**Input**: User description: "Investigate why Grip operations on a large file of approximately 19 MB take a long time, and fix the demonstrated performance degradation."

## Clarifications

### Session 2026-09-14

- Q: Which large-file workflows must meet the one-second p95 target? → A: `grip add` and `grip status`.
- Q: Which build should be required to meet the p95 target for `grip add` and `grip status`? → A: Release builds only.
- Q: What endpoint state should the approximately 19 MB test file use for both `grip add` and `grip status`? → A: Source and destination both exist but have different content.
- Q: What p95 completion-time target should release-build `grip add` and `grip status` meet for the defined 19 MB workload? → A: At most 1 second.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Diagnose a Slow Large-File Operation (Priority: P1)

As a Grip operator, I want a representative large-file operation to be measured and explained so that I can understand why it is slow and trust that the improvement addresses the actual cause.

**Why this priority**: A reproducible baseline prevents a superficial optimization from masking the reported degradation.

**Independent Test**: Run the defined workload against an isolated project containing an approximately 19 MB regular file and verify that the report identifies the operation, elapsed-time distribution, relevant work performed, and demonstrated bottleneck.

**Acceptance Scenarios**:

1. **Given** an isolated project with differing approximately 19 MB source and destination regular files, **When** a maintainer runs a representative Grip operation, **Then** the result records reproducible workload conditions and timing evidence.
2. **Given** timing evidence for a slow operation, **When** a maintainer reviews the investigation, **Then** it identifies a demonstrated causal bottleneck rather than relying on an assumed optimization.

---

### User Story 2 - Complete Large-File Work Promptly (Priority: P2)

As a Grip operator, I want `grip add` and `grip status` involving an approximately 19 MB file to complete promptly so that configuring and inspecting a mapping do not interrupt my workflow.

**Why this priority**: The feature exists to make the reported workflow usable, not merely to produce a performance report.

**Independent Test**: Run release-build `grip add` and `grip status` 100 times each after warm-up in isolated projects and verify that their p95 completion times meet the stated target without changing mapping or classification results.

**Acceptance Scenarios**:

1. **Given** the representative large-file workload, **When** an operator runs `grip add`, **Then** it completes within the defined p95 target and publishes the same mapping and initial comparison state as before the improvement.
2. **Given** an established representative large-file mapping, **When** an operator runs `grip status`, **Then** it completes within the defined p95 target and returns the same classification as before the improvement.

---

### User Story 3 - Keep the Improvement Durable (Priority: P3)

As a Grip maintainer, I want regression coverage for the large-file workload so that later changes cannot silently reintroduce the same slowdown.

**Why this priority**: A one-time benchmark does not protect operators from future regressions.

**Independent Test**: Run the project performance gate with the large-file workload and verify that it reports a failing result when the defined p95 limit is exceeded.

**Acceptance Scenarios**:

1. **Given** the accepted performance behavior, **When** a later change slows `grip add` or `grip status` beyond its limit, **Then** the performance gate reports the regression.
2. **Given** a representative workload that cannot be exercised safely, **When** the performance gate runs, **Then** it fails or skips with an explicit reason rather than measuring a developer's real files.

### Edge Cases

- The source and destination are unequal when `grip add` creates the mapping; the command retains its initial comparison-state and publication behavior.
- The large file is unchanged, changed on one side, or divergent when `grip status` runs; the command retains correct classification and reporting behavior.
- The file is absent, unsupported, inaccessible, or changes during the operation; Grip retains its existing precise error and safety behavior rather than treating the condition as a performance success.
- The measured environment is materially slower or noisier than the defined representative environment; the report records the condition and does not claim an unexplained regression or improvement.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The feature MUST define reproducible isolated `grip add` and `grip status` workloads containing differing approximately 19 MB source and destination regular files and the mapping states needed to reproduce the reported degradation.
- **FR-002**: The feature MUST measure release-build `grip add` and `grip status` after warm-up across 100 runs each and record their p50 and p95 completion times, workload conditions, and mapping or classification result.
- **FR-003**: The feature MUST identify the demonstrated cause of `grip add` or `grip status` latency whose baseline p95 exceeds one second before selecting an improvement.
- **FR-004**: The feature MUST apply only improvements supported by the measured cause and record the before-and-after p50 and p95 results for each changed workflow.
- **FR-005**: Every improvement MUST preserve `grip add` mapping publication and initial comparison state, plus `grip status` selection, classification, and failure-reporting behavior.
- **FR-006**: Release-build `grip add` and `grip status` MUST each complete with a p95 of one second or less on their defined workloads after the improvement.
- **FR-007**: The feature MUST add regression coverage that exercises the representative workload without reading or mutating a developer's real files.
- **FR-008**: The feature MUST not add persistent or background operational complexity unless a separately accepted amendment records the measured need, simpler alternatives, and operational cost.

### Key Entities *(include if feature involves data)*

- **Representative large-file workload**: The isolated project, differing approximately 19 MB source and destination regular files, selected operation, and measurement conditions used for repeatable evaluation.
- **Performance sample**: One measured `grip add` or `grip status` result containing elapsed time, mapping or classification result, and workload identity.
- **Performance baseline**: The summarized pre-improvement p50 and p95 evidence for the `grip add` and `grip status` workloads.
- **Performance decision**: The causal diagnosis, selected improvement or retain decision, and before-and-after evidence for a selected workflow.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Release-build `grip add` and `grip status` each have a 100-run warm p50 and p95 baseline recorded against their isolated approximately 19 MB workloads.
- **SC-002**: Release-build `grip add` and `grip status` each complete with a p95 of one second or less after the improvement.
- **SC-003**: 100% of performance changes cite a measured causal bottleneck and record before-and-after p50 and p95 results.
- **SC-004**: All existing correctness and safety checks for the selected operations pass unchanged after the improvement.
- **SC-005**: The new large-file regression workload runs without accessing developer-owned source, destination, project, or state paths.

## Constitution Alignment

- **Principle I — Proportional Rigor for a Local Tool**: The feature chooses the simplest measured remedy and requires explicit evidence before adding higher-complexity performance mechanisms.
- **Principle III — Validate, Revalidate, and Verify**: Any change preserves deterministic planning, validation, revalidation, verification, and truthful failure reporting.
- **Principle V — Fast, Observable, and Testable**: The workload measures p50 and p95 performance on isolated temporary roots and protects the accepted behavior with regression coverage.
- **Product boundaries**: The feature does not add remote behavior, daemons, privileged services, automatic ownership, or a user-facing history or recovery interface.
- **Merge-bounded persistence**: The measured evidence, specification, plan, tasks, implementation, and validation remain one reviewable feature change set until accepted.

## Assumptions

- The approximately 19 MB source and destination files are regular managed files with different content, and the selected representative environment is a local macOS ARM64 release build; the workload records exact conditions before results are compared.
- One second p95 is the release-build acceptance target. Development-build timing is diagnostic only, and an environment-specific exception requires explicit evidence and review.
- The feature evaluates only the reported `grip add` and `grip status` workflows. Other commands remain out of scope unless a separately accepted amendment adds evidence of a shared regression.
- Any optimization that changes a public command contract, supported payload behavior, or safety guarantee is out of scope and requires a separate accepted feature.
