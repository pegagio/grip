# Feature Specification: Simplify Status Output

**Feature Branch**: `015-simplify-status-output`

**Created**: 2026-09-11

**Status**: Complete

**Input**: User description: "The output from `grip status` is overwhelming. Users need a simple, Git-like default view that makes it easy to distinguish what differs from what is the same and shows only important information."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Identify work that needs attention (Priority: P1)

An operator runs `grip status` and can immediately see only the managed entries that differ or cannot be synchronized safely, along with a plain-language explanation of each item's significance.

**Why this priority**: Status is the primary pre-action check. Operators must be able to recognize conflicts and changes without parsing internal comparison evidence.

**Independent Test**: Create a project containing one synchronized entry, one source-side change, and one conflicting entry. Run `grip status` and confirm that the output identifies the changed and conflicting paths in clear language, while omitting per-endpoint capability and low-level comparison detail.

**Acceptance Scenarios**:

1. **Given** a selected scope with a conflicting managed entry, **When** the operator runs `grip status`, **Then** the conflict appears in a `Conflicts` section as `SOURCE <-> DESTINATION`, identifying both competing paths without choosing a winner.
2. **Given** a selected scope with a one-sided managed change, **When** the operator runs `grip status`, **Then** the output lists the affected source and destination paths under `Changes to push` as `SOURCE -> DESTINATION` or under `Changes to pull` as `SOURCE <- DESTINATION`, as appropriate.
3. **Given** the same scope also contains synchronized entries, **When** the operator runs `grip status`, **Then** those entries do not produce individual detail lines.
4. **Given** an entry needs non-directional baseline or reconciliation attention, including matching endpoints that need a baseline established or refreshed, **When** the operator runs `grip status`, **Then** the output lists the pair under `Needs baseline` as `SOURCE >-< DESTINATION` without presenting a payload-copy direction.

---

### User Story 2 - Confirm that a scope is current (Priority: P2)

An operator checking a project or selected path can tell at a glance whether there is any work to do, without mistaking internal validation information for a change.

**Why this priority**: A concise clean result makes `grip status` useful as a routine confidence check, like `git status`.

**Independent Test**: Run `grip status` against a scope whose managed entries are all synchronized and verify that it reports a concise clean result with no itemized technical evidence.

**Acceptance Scenarios**:

1. **Given** a selected scope whose managed entries are all synchronized, **When** the operator runs `grip status`, **Then** the output clearly says that no synchronization work is needed and provides a count of entries checked.
2. **Given** a selected scope with both synchronized and actionable entries, **When** the operator runs `grip status`, **Then** the summary distinguishes the number that are current from the number needing attention.
3. **Given** an empty selected scope, **When** the operator runs `grip status`, **Then** the output clearly reports that no managed entries were found without presenting it as an error or a clean synchronized entry.

---

### User Story 3 - Preserve operational safety and automation contracts (Priority: P3)

An operator receives concise human output without losing safety-critical warnings or changing scripts that consume the machine-readable status result.

**Why this priority**: Simplification must improve comprehension without hiding blockers or changing established automation behavior.

**Independent Test**: Compare human and machine-readable status results for a scope with a safety-blocking compatibility or unsupported-entry finding. Verify that the human view reports the blocker in plain language and that the machine-readable result retains its detailed structure and values.

**Acceptance Scenarios**:

1. **Given** a managed entry blocked by an unsupported state or compatibility requirement, **When** the operator runs `grip status`, **Then** the human output identifies the affected path, explains the blocker in plain language, and does not omit it because it is technical.
2. **Given** a status result containing an informational finding that requires no operator action, **When** the operator runs `grip status`, **Then** the default human output omits that finding.
3. **Given** an existing script that requests machine-readable status output, **When** the feature is delivered, **Then** the script receives the same status records, classifications, and diagnostic detail as before.
4. **Given** a valid status result with entries requiring attention, **When** the operator adds the existing exit-status option, **Then** its exit behavior remains unchanged.

### Edge Cases

- A selected scope may contain zero entries, only synchronized entries, only actionable entries, or a mixture; the summary must accurately reflect the selected scope in each case.
- Nonempty `Conflicts`, `Changes to push`, `Changes to pull`, and `Needs baseline` sections must be presented in a stable, easy-to-scan order; each section retains stable ordering for its entries.
- A single entry may have both a synchronization difference and informational compatibility evidence; the default view must show the actionable state without repeating information that requires no action.
- A safety blocker whose cause is technical must be translated into an operator-understandable explanation without concealing the affected path or the fact that operation is blocked.
- Paths containing spaces or unusual printable characters must remain unambiguous in human output.
- Existing validation, selector, and exit-status errors remain distinct from a successful inspection that reports entries needing attention.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The default human output of `grip status` MUST present a concise summary of the selected scope that distinguishes current entries, entries needing attention, and blocking entries.
- **FR-002**: When no entry in the selected scope requires attention or is blocked, the default human output MUST clearly state that no synchronization work is needed and report the number of managed entries checked.
- **FR-003**: The default human output MUST organize every actionable entry into nonempty `Conflicts`, `Changes to push`, `Changes to pull`, and `Needs baseline` sections, and list the source and destination paths for each entry. `Needs baseline` contains nonblocking entries with no safe payload-copy direction, including baseline establishment, refresh, deletion convergence, and metadata-migration readiness.
- **FR-004**: The default human output MUST render directional entries as `SOURCE -> DESTINATION` for `Changes to push`, `SOURCE <- DESTINATION` for `Changes to pull`, `SOURCE <-> DESTINATION` for `Conflicts`, and `SOURCE >-< DESTINATION` for `Needs baseline`; these symbols MUST NOT imply a conflict winner or a payload-copy direction where none exists. `>-<` identifies a non-directional state requiring reconciliation attention, rather than asserting that payload contents match.
- **FR-005**: The default human output MUST use stable ordering within each section, omit empty sections, and avoid command suggestions or forced-operation advice; section names and directional symbols are the only default action guidance.
- **FR-006**: The default human output MUST omit per-endpoint capability profiles, baseline-comparison dimensions, raw internal reason identifiers, and compatibility findings that require no operator action.
- **FR-007**: The default human output MUST retain safety-significant compatibility and unsupported-entry blockers, expressed in plain language that identifies the affected path and that action is blocked.
- **FR-008**: The default human output MUST make a selected scope with no managed entries distinguishable from a scope whose managed entries are synchronized.
- **FR-009**: The feature MUST preserve the existing machine-readable status result's records, classifications, diagnostic detail, and exit behavior.
- **FR-010**: The feature MUST preserve existing mapping selection semantics and must not change synchronization, mutation, conflict-resolution, metadata, or baseline behavior.
- **FR-011**: The feature MUST update user-facing documentation with representative concise status examples for a clean scope, a one-sided change, and a blocking conflict.

### Key Entities *(include if feature involves data)*

- **Status summary**: The compact statement of how many managed entries in the selected scope are current, need attention, or are blocked.
- **Actionable status entry**: A managed entry whose observed state differs in a way that needs operator attention or blocks safe operation.
- **Informational diagnostic**: Observed validation or capability information that does not require an operator action and is therefore not shown in the default human status view.
- **Safety blocker**: A condition that prevents Grip from safely operating on an affected entry and must remain visible to the operator.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative mixed-status tests containing current entries, pushable changes, pullable changes, conflicts, and entries needing a baseline, an operator can identify every affected source and destination path and its status direction from the default human output without consulting machine-readable output or internal comparison terminology.
- **SC-002**: In representative clean-status tests, the default human output contains one concise result summary and no per-entry technical diagnostic lines.
- **SC-003**: In representative mixed-status tests, the default human output contains no capability-profile lines, baseline-comparison lines, raw internal reason identifiers, or no-action informational findings.
- **SC-004**: In isolated compatibility and unsupported-entry tests, 100% of safety-blocking conditions remain visible in the default human output and identify the affected path.
- **SC-005**: All existing machine-readable status contract tests and exit-status tests continue to pass without changing their expected structured results or exit codes.

## Assumptions

- The default human view is the target of this feature; the existing machine-readable status result remains the detailed interface for automation and diagnostics.
- “Git-like” means concise, path-centered, action-oriented presentation, not adoption of Git terminology or expansion of Grip's public command surface.
- Synchronized entries are represented by the summary count rather than individual lines so attention is reserved for work that matters.
- The default status view uses `->` for a pushable source-side change, `<-` for a pullable destination-side change, `<->` for a conflict with no selected winner, and `>-<` for a non-directional, nonblocking reconciliation state such as baseline establishment, refresh, deletion convergence, or metadata-migration readiness.
- Informational findings such as unmanaged excluded attributes are omitted only when they require no operator action; safety blockers remain visible.
- This feature does not add commands, a new status mode, automatic repair, command suggestions, or new conflict-resolution behavior.
- The feature follows the constitution's observable-output boundary: it changes presentation only and does not alter filesystem inspection, ownership, validation, or mutation decisions.
