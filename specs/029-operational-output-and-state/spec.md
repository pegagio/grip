# Feature Specification: Operational Output and State Rebinding

**Feature Branch**: `029-operational-output-and-state`

**Created**: 2026-09-16

**Status**: Implemented

**Input**: User description: "Make push counts, aggregate-force blockers, external diff output, verbose diff explanations, and project-local state rebinding accurately explain the operation without treating diff-tool settings as mapping changes."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read focused external diff output (Priority: P1)

An operator running `grip diff PATH` sees only the configured comparison program's standard output. With verbosity, Grip writes a readable explanation separately, including actual managed property values.

**Why this priority**: A selected external diff is useful only when Grip does not obscure the comparison program's output.

**Independent Test**: Run an isolated selected diff with and without `-v`; verify standard output belongs solely to the program and verbose explanation is on standard error.

**Acceptance Scenarios**:

1. **Given** an eligible selected human diff, **When** the operator runs `grip diff PATH`, **Then** Grip launches the configured program and writes no inspection header or completion line to standard output.
2. **Given** an eligible selected human diff with `-v`, **When** the operator runs `grip diff -v PATH`, **Then** Grip writes a property-oriented inspection explanation to standard error before launching the program.
3. **Given** source metadata differs from the accepted baseline, **When** verbose inspection is requested, **Then** Grip shows each differing managed property with aligned source, baseline, and destination values.
4. **Given** an unmanaged compatibility note, **When** verbose inspection is requested, **Then** Grip omits it rather than presenting it as an actionable difference.

### User Story 2 - Understand synchronization results (Priority: P2)

An operator can tell how many payload files a tree push changed and why a no-selector forced push cannot proceed.

**Why this priority**: Mutation transcripts must describe user payload work, not internal directory setup or unexplained counts.

**Independent Test**: Push an isolated tree mapping and block an isolated aggregate forced push; verify payload-file count and blocker reasons with paths.

**Acceptance Scenarios**:

1. **Given** a tree push that creates directories and copies files, **When** it succeeds, **Then** its count reports copied payload files and not directory setup actions.
2. **Given** a no-selector `grip push --force` blocked by safety conditions, **When** Grip reports the failure, **Then** it identifies the aggregate operation and lists each available blocker reason with its affected managed path or path pair.

### User Story 3 - Change a diff profile without blocking push (Priority: P3)

An operator can edit a project diff-tool profile and push a pending source change without a rebinding failure solely because the profile changed.

**Why this priority**: Diff-tool preferences do not alter mapping intent or accepted payload evidence.

**Independent Test**: In an isolated bound project, change source content and append a valid `[diff]` and `[difftool]` profile, then run `grip push` and verify a bound state afterward.

**Acceptance Scenarios**:

1. **Given** unchanged project root, destination home, and resolved mapping set, **When** only output-oriented descriptor settings change, **Then** Grip treats state as eligible for authorized publication rather than rejecting mutation for payload drift.
2. **Given** a mapping, resolved endpoint, project root, or destination home changes, **When** a mutation is requested, **Then** Grip retains existing rebinding inspection and safety blockers.

### Edge Cases

- A comparison program exits nonzero: Grip preserves its exit result without a standard-output footer.
- A selected comparison has no managed-property difference: verbose inspection says `Differences: none`.
- Endpoint capabilities match: verbose output identifies endpoint roles without repeating long paths or implying a difference.
- A tree operation creates empty directories only: its human file count remains zero.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: For an eligible selected human external diff, Grip MUST leave standard output entirely to the launched comparison program and MUST NOT append a Grip completion footer.
- **FR-002**: With `-v` or `--verbose` on an eligible selected human external diff, Grip MUST emit its inspection explanation to standard error without changing external-program arguments, standard output, or exit behavior.
- **FR-003**: Verbose inspection MUST explain the classification in plain language and show only managed differing properties in groups containing aligned `source`, `baseline`, and `destination` values.
- **FR-004**: Verbose inspection MUST omit compatibility notes for attributes Grip leaves unmanaged and MUST refer to filesystem capabilities by endpoint role rather than repeating endpoint paths.
- **FR-005**: Human push and preview counts for tree mappings MUST count file-level payload actions and MUST NOT represent directory setup actions as pushed files.
- **FR-006**: A blocked no-selector forced push MUST identify itself as an aggregate forced push and render each available blocker reason with its managed affected path or path pair, followed by status guidance.
- **FR-007**: Project-local rebinding MUST ignore descriptor-only changes when project root, destination home, and resolved mapping digest are unchanged.
- **FR-008**: A descriptor change that changes the resolved mapping digest, project root, or destination home MUST retain existing state-rebinding inspection and mutation-blocking behavior.
- **FR-009**: This feature MUST preserve mapping ownership, source-defined tree membership, external-diff invocation safety, JSON schemas, baseline publication safety, and forced-operation authority.
- **FR-010**: README and product documentation MUST describe the external-diff output boundary, verbose inspection, payload-file counts, aggregate blocker detail, and mapping-affecting state rebinding boundary.

## Key Entities

- **Verbose diff inspection**: Grip-owned diagnostic explanation emitted only on standard error for a verbose selected external diff.
- **Payload-file action**: A synchronization action that changes a managed file's payload, distinct from internal directory setup.
- **Aggregate blocker**: A safety condition that prevents a no-selector forced push, with a reason and affected managed path evidence.
- **Project binding**: The machine-local relationship between accepted state, selected project root, destination home, raw descriptor, and resolved mapping set.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of isolated selected external-diff scenarios, the non-verbose command writes no Grip-owned text to standard output and returns the comparison program's exit result unchanged.
- **SC-002**: In 100% of isolated verbose metadata-difference scenarios, standard error identifies changed managed properties and renders source, baseline, and destination values for each.
- **SC-003**: In 100% of isolated tree-push scenarios containing directory setup and file copy actions, the human summary count equals the number of payload files copied.
- **SC-004**: In 100% of isolated blocked aggregate-force scenarios with path evidence, the human transcript names the aggregate operation and renders every blocker reason with its affected path evidence.
- **SC-005**: In 100% of isolated bound-project scenarios where only a valid diff profile changes, a pending source push succeeds; mapping, project-root, or destination-home changes retain existing rebinding checks.

## Assumptions

- The existing selected external-diff process, literal-argument boundary, and exit-code semantics remain authoritative.
- A resolved mapping digest captures mapping declarations and resolved endpoints; non-mapping settings such as diff profiles do not alter payload ownership or topology.
- Human-readable diagnostics may evolve without changing the JSON contract.
