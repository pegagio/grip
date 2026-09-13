# Feature Specification: Simplify Default Command Output

**Feature Branch**: `017-simplify-command-output`

**Created**: 2026-09-12

**Status**: Complete

**Input**: User description: "Compare `updated-output.md` with actual command output and specify the required default human-output changes. Leave `grip diff` unchanged. Remove baseline-generation evidence from ordinary blocked push and pull output. Audit and remediate remaining internal execution detail in default human mutation output."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Act on a concise mutation result (Priority: P1)

An operator previews or completes a push, pull, synchronization, or forced push and can immediately see what changed and in which direction without parsing implementation-state evidence.

**Why this priority**: Mutation commands are the direct follow-up to status. Their default output must confirm the user’s intended file operation rather than expose internal execution details.

**Independent Test**: Create source-side, destination-side, mixed-direction, and divergent changes; preview and run each applicable command; then compare the human transcript with the approved output examples.

**Acceptance Scenarios**:

1. **Given** one or more pushable files, **When** an operator runs `grip push --dry-run`, **Then** the output begins `Would push N file(s):` and lists each file as `SOURCE -> DESTINATION` without plan, baseline, recovery, or capability details.
2. **Given** one or more pushable files, including a forced source-winning conflict, **When** an operator runs `grip push`, **Then** the output begins `Pushed N file(s):` and lists each completed file as `SOURCE -> DESTINATION` without a separate winner, verification, baseline, or operation-record line.
3. **Given** one or more pullable files, **When** an operator previews or runs `grip pull`, **Then** the output begins `Would pull N file(s):` or `Pulled N file(s):` and lists each file as `SOURCE <- DESTINATION`.
4. **Given** a synchronization scope containing both pushable and pullable files, **When** an operator previews or runs `grip sync`, **Then** the output begins `Would synchronize N file(s):` or `Synchronized N file(s):` and lists every affected file in its actual direction.

---

### User Story 2 - Read mapping and status output without repeated detail (Priority: P2)

An operator can view or change mappings and inspect synchronization state without repeated resolved paths or explanatory lines that do not change the next safe action.

**Why this priority**: Mapping and status commands are frequent orientation tools; concise rows make large mapping sets readable.

**Independent Test**: Add, list, select, remove, and inspect mappings in a fixture with push, pull, and conflict states; verify the transcript matches the approved examples and contains no omitted internal detail.

**Acceptance Scenarios**:

1. **Given** a valid new mapping, **When** an operator runs `grip add`, **Then** the output is `Mapped:` followed by one declared `SOURCE -> DESTINATION` row and no resolved-endpoint row.
2. **Given** several mappings, **When** an operator runs `grip list`, **Then** the output is `N mapping(s):` followed by one declared source-to-destination row for each mapping and no resolved-endpoint rows.
3. **Given** one selected mapping, **When** an operator runs `grip list SOURCE` or `grip remove SOURCE`, **Then** the output uses the approved heading and one mapping-kind-prefixed declared row, without a resolved-endpoint row.
4. **Given** a status scope with pushable, pullable, and force-resolvable conflict files, **When** an operator runs `grip status`, **Then** nonempty sections appear in this order: `Changes to push`, `Changes to pull`, `Conflicts`, then `Needs baseline`; each initial collision or ordinary divergent conflict includes its `SOURCE <-> DESTINATION` row followed by explicit source-winning and destination-winning force commands.

---

### User Story 3 - Receive concise terminal mutation results and clear errors without changing contracts (Priority: P3)

An operator can tell whether a requested mutation had nothing to do, is blocked, or stopped partway through without parsing raw planner reasons, baseline state, lifecycle evidence, or generated record identifiers.

**Why this priority**: A concise success transcript is insufficient if common no-action, blocked, and interrupted paths revert to internal implementation detail instead of explaining the next safe operator action.

**Independent Test**: Exercise no-action, force-resolvable blocked, technical blocked, and partial-failure mutation outcomes, plus parser and invalid-selector errors; compare their human transcript, exit category, JSON result, and unchanged `diff` behavior before and after the feature.

**Acceptance Scenarios**:

1. **Given** an invalid human-invoked command, **When** Grip displays its error, **Then** the first error line begins `Error:` while retaining the existing actionable message, usage information where available, and exit category.
2. **Given** a push or pull is blocked solely by an initial collision or ordinary divergent conflict, **When** Grip displays the default human result, **Then** it includes concrete source-winning and destination-winning force commands for each affected conflict, omits baseline-generation evidence, and leaves the blocked outcome, exit category, and JSON result unchanged.
3. **Given** a `grip diff` invocation, **When** the feature is delivered, **Then** its default human output is unchanged.
4. **Given** a script requesting JSON from any command, **When** the feature is delivered, **Then** it receives the existing structured output and exit behavior unchanged.
5. **Given** a push, pull, or sync has no actionable work, **When** Grip displays the default human result, **Then** it states `Nothing to push.`, `Nothing to pull.`, or `Nothing to synchronize.` without plan, baseline, or operation-record detail.
6. **Given** push, pull, sync, or a forced directional resolution is blocked only by initial collisions or ordinary divergent conflicts, **When** Grip displays the default human result, **Then** it names the public requested operation, displays each `SOURCE <-> DESTINATION` conflict row with the two valid force choices, and omits raw planner reason codes, winner fields, baseline authority, and operation-record detail.
7. **Given** a mutation is blocked for a technical or mixed reason, **When** Grip displays the default human result, **Then** it names the public requested operation and directs the operator to `grip status` for current detailed safety information without exposing raw blocker identifiers, baseline authority, or operation-record detail.
8. **Given** a mutation stops after execution begins, **When** Grip displays the default human result, **Then** it reports the existing completed-action count and directs the operator to run `grip status` before retrying, without visibility, verification, durability, raw recovery, baseline, or operation-record detail.
9. **Given** a push, pull, or sync has no copy actions but can establish an initial baseline, **When** Grip previews or completes that accepted baseline operation, **Then** it states `Would establish a baseline for N file(s).` or `Established a baseline for N file(s).` without internal execution evidence.

### Edge Cases

- A mutation result may contain both source-to-destination and destination-to-source actions; each output row must show its own direction rather than inherit a direction from the command heading.
- A forced push resolves a divergent conflict, but its concise output must not add a separate conflict-winner line because the source-to-destination row communicates the result.
- Conflict guidance must identify which endpoint each force command keeps and must use the displayed source or destination selector accepted by that command.
- Initial collisions and ordinary divergent conflicts are force-resolvable; technical compatibility, unsupported-state, deletion, and other safety blockers retain their existing detail without invented force guidance.
- A status scope may have only one nonempty actionable section; its ordering rule must not create empty headings.
- Safety-significant status blockers other than an ordinary divergent conflict remain subject to Feature 015’s visibility requirement.
- Help output, verbose diagnostics, and output outside the public mutation, mapping, status, and ordinary error paths remain unchanged by this feature.
- Baseline authority remains available to JSON and detailed diagnostics but is not ordinary blocked-command output.
- The changed error prefix must not alter exit categories, selector validation, or whether a command mutates files.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Default human output for a push preview with one or more actions MUST use `Would push N file(s):` and render each affected file as `SOURCE -> DESTINATION`.
- **FR-002**: Default human output for a completed push with one or more actions, including a forced push, MUST use `Pushed N file(s):` and render each affected file as `SOURCE -> DESTINATION`.
- **FR-003**: Default human output for a pull preview or completed pull with one or more actions MUST use `Would pull N file(s):` or `Pulled N file(s):` and render each affected file as `SOURCE <- DESTINATION`.
- **FR-004**: Default human output for a synchronization preview or completed synchronization with one or more actions MUST use `Would synchronize N file(s):` or `Synchronized N file(s):` and render every affected file using its actual source-to-destination or destination-to-source direction.
- **FR-005**: The changed successful mutation outputs MUST omit internal action names such as `replace_file`, recovery state, visibility/verification/durability fields, baseline-generation statements, and generated operation-record identifiers.
- **FR-006**: Default human `grip add` output MUST be `Mapped:` followed by one declared `SOURCE -> DESTINATION` row, with no mapping-kind prefix or resolved-endpoint row.
- **FR-007**: Default human `grip list` output for an unselected scope MUST be `N mapping(s):` followed by one declared `SOURCE -> DESTINATION` row per mapping, with no mapping-kind prefixes or resolved-endpoint rows.
- **FR-008**: Default human `grip list SOURCE` output MUST be `1 mapping(s):` followed by one mapping-kind-prefixed declared source-to-destination row, with no resolved-endpoint row.
- **FR-009**: Default human `grip remove SOURCE` output MUST be `Mapping removed:` followed by one mapping-kind-prefixed declared source-to-destination row, with no resolved-endpoint row.
- **FR-010**: This feature amends Feature 015’s default-human status section order: nonempty sections MUST appear as `Changes to push`, `Changes to pull`, `Conflicts`, then `Needs baseline`; stable ordering within a section remains unchanged.
- **FR-011**: For each initial collision or ordinary divergent conflict, default human `grip status` output MUST render the `SOURCE <-> DESTINATION` row followed by `Keep source: grip push --force SOURCE` and `Keep destination: grip pull --force --destination DESTINATION`. The guidance MUST replace the generic explanatory blocked line and MUST not change selection or force semantics.
- **FR-012**: Default human error output for command-parser and Grip domain errors MUST begin its first error line with `Error:` while retaining the existing error meaning, usage text where applicable, and exit category.
- **FR-013**: Default human `grip diff` output, help output, verbose diagnostics, and default output for unshown no-action outcomes MUST remain unchanged.
- **FR-014**: This feature MUST preserve every command’s JSON schema, JSON values, and JSON exit behavior.
- **FR-015**: This feature MUST preserve command syntax, mapping declarations, selection semantics, project and path validation, classification, synchronization, mutation, conflict-resolution, metadata, baseline, and filesystem-safety behavior.
- **FR-016**: User-facing documentation MUST show the concise mapping, status, mutation-preview, and completed-mutation output forms introduced by this feature and identify `grip diff` as the unchanged detailed diagnostic view.
- **FR-017**: Default human blocked push and pull results caused solely by one or more initial collisions or ordinary divergent conflicts MUST include the same concrete source-winning and destination-winning force commands for each affected conflict, while retaining their existing blocked category, exit behavior, and safety details.
- **FR-018**: Default human blocked push and pull results MUST omit baseline outcome and generation statements. This presentation change MUST NOT remove baseline fields from JSON or diagnostic output, or alter baseline authority.
- **FR-019**: Default human no-action results for `push`, `pull`, and `sync` MUST be exactly `Nothing to push.`, `Nothing to pull.`, or `Nothing to synchronize.` according to the public direction. They MUST omit plan, baseline, and operation-record details.
- **FR-020**: Default human blocked results for `push`, `pull`, `sync`, and a forced directional resolution MUST use the public operation name (`Push`, `Pull`, or `Sync`), rather than the internal `Resolution` operation name. A conflict-only result MUST render each affected `SOURCE <-> DESTINATION` row and its two valid force choices; a technical or mixed result MUST direct the operator to `Run: grip status` for current detailed safety information. These paths MUST omit raw blocker identifiers, winner fields, baseline statements, and operation-record details.
- **FR-021**: Default human failed results for `push`, `pull`, `sync`, and a forced directional resolution MUST retain the existing operation name and completed-action count, append `Run: grip status before retrying.`, and omit action milestones, recovery state, baseline statements, and operation-record details.
- **FR-022**: The default human output changes in FR-019 through FR-021 MUST be presentation-only. Existing JSON values, exit categories, mutation/baseline behavior, operation evidence, and detailed `diff` output MUST remain unchanged.
- **FR-023**: A default human planned or applied mutation with zero copy actions and one or more accepted baseline entries MUST use `Would establish a baseline for N file(s).` or `Established a baseline for N file(s).`, where `N` is the number of accepted entries. It MUST omit internal execution evidence while preserving existing JSON and baseline behavior.

### Key Entities *(include if feature involves data)*

- **Declared mapping row**: The portable source-to-destination mapping representation shown to an operator, without an additional resolved-path row.
- **Concise mutation row**: A human-readable result row that shows an affected file and its actual copy direction without execution-engine evidence.
- **Force-resolvable conflict**: An initial collision or ordinary divergent conflict for one managed file, represented by the existing `<->` directional row and two explicit force-resolution choices.
- **Human output contract**: The default textual presentation intended for operators, distinct from JSON and verbose diagnostic output.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative push, pull, forced-push, and mixed-direction synchronization tests with one or more actions, 100% of default human result rows show the correct source and destination ordering and arrow direction.
- **SC-002**: In representative successful mutation tests, default human output contains zero operation-engine fields: internal action names such as `replace_file`, recovery state, verification or durability fields, baseline generations, or generated operation-record identifiers.
- **SC-003**: In a five-mapping fixture, default human `grip list` produces one mapping row per mapping with zero resolved-endpoint rows.
- **SC-004**: In a mixed-status fixture, default human output presents nonempty actionable sections in the approved order and shows each initial collision or ordinary divergent conflict as one path-pair row followed by the two direction-labeled force commands needed to resolve it.
- **SC-005**: Existing `grip diff`, JSON, exit-category, selector, and filesystem-safety tests pass with their expected behavior unchanged.
- **SC-006**: A user can distinguish previewed actions, completed actions, and errors from their first output line in each representative command transcript.
- **SC-007**: Representative human blocked push and pull transcripts contain zero baseline outcome or generation statements while their JSON baseline fields remain unchanged.
- **SC-008**: Representative no-action, blocked, and failed human mutation transcripts contain zero raw blocker identifiers, winner fields, action milestones, recovery fields, baseline statements, or operation-record identifiers; their JSON values and exit categories remain unchanged.

## Assumptions

- `updated-output.md` is the approved normative transcript for changed default human output; its placeholders stand for the actual selected project and endpoint paths.
- `command-output-audit.md` records the current behavior used to identify the presentation changes; it is not a behavior contract after this feature is delivered.
- `N` is the number of file rows in the command result, and the literal `file(s)` wording from the approved transcript applies to the changed action-result headings.
- This feature follows and amends Feature 015’s status presentation only where its revised transcript explicitly changes section ordering or ordinary divergent-conflict detail. Other Feature 015 safety-blocker visibility requirements remain in force.
- `SOURCE` in source-winning guidance is the same CWD-relative source display used by status; `DESTINATION` in destination-winning guidance is the displayed destination endpoint passed with `--destination`.
- Blocked push and pull guidance uses concrete selectors valid from the command invocation directory, rather than placeholder syntax or an absolute source selector that `push` would reject.
- Output scenarios not represented in the approved transcript retain their existing human presentation to keep scope bounded.
- The feature changes presentation and documentation only; it does not change which files Grip selects, plans, copies, accepts into a baseline, or rejects.
