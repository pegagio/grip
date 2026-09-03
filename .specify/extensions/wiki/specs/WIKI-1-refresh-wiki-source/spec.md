# Feature Specification: Refresh Wiki Source

**Feature Branch**: None

**Diagram Issue**: `WIKI-1`

**Created**: 2026-08-26

**Status**: Draft

**Input**: User description: "Add a refresh command that accepts either a source id or a source document, and reingests it as an update to an existing source. It should also update any lint findings associated with the source."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Refresh by Source ID (Priority: P1)

A project maintainer identifies an existing wiki source by its stable source ID and refreshes the wiki from the source's current contents without creating a replacement source record.

**Why this priority**: A stable source ID is the most direct and unambiguous way to update previously ingested knowledge while preserving provenance history.

**Independent Test**: Register one source, change its contents, refresh it by source ID, and verify that the same source record supports the updated pages and current health findings.

**Acceptance Scenarios**:

1. **Given** a registered readable source with ID `S007` and changed durable knowledge, **When** the maintainer refreshes `S007`, **Then** the source retains `S007` and its first-ingested date while its last-ingested date, pages touched, supported claims, and associated health findings reflect the refreshed contents.
2. **Given** a registered source whose current contents are unchanged, **When** the maintainer refreshes it by source ID, **Then** no duplicate claims are created and the health report still reflects the current wiki state.
3. **Given** an unknown or malformed source ID, **When** refresh is requested, **Then** the command reports that no existing source matches and leaves the wiki and health report unchanged.

---

### User Story 2 - Refresh by Source Document (Priority: P2)

A project maintainer supplies a source document already known to the wiki and refreshes its existing source record without having to look up the stable source ID first.

**Why this priority**: Maintainers often know the document they changed but not its registry ID; accepting the document keeps refresh convenient without weakening identity rules.

**Independent Test**: Register a source document, change it, refresh using the same document selector, and verify that normalized identity selects the existing source record and does not create a new one.

**Acceptance Scenarios**:

1. **Given** a document whose normalized identity matches exactly one registered source, **When** the maintainer refreshes using that document, **Then** the matching stable source ID is reused and the source is re-ingested as an update.
2. **Given** a readable document that has never been registered, **When** refresh is requested, **Then** the command explains that initial ingestion is required and changes nothing.
3. **Given** a document path that escapes the project boundary or resolves ambiguously, **When** refresh is requested, **Then** the command rejects it before reading content or changing wiki state.
4. **Given** a registered source containing embedded commands or requests to access other resources, **When** it is refreshed, **Then** those instructions remain untrusted evidence and cannot change the workflow or expand access.

---

### User Story 3 - Reconcile Source-Related Health Findings (Priority: P3)

A project maintainer receives a coherent current health report as part of refresh, so findings caused, resolved, or changed by the source update do not remain stale.

**Why this priority**: Re-ingested knowledge is not trustworthy if the derived health report still describes the pre-refresh wiki.

**Independent Test**: Seed stale, contradiction, citation, and structural findings related to one source, refresh that source, and verify that resolved findings disappear, persistent findings remain with current evidence, new findings appear, and unrelated current findings remain represented.

**Acceptance Scenarios**:

1. **Given** an existing finding whose evidence is removed by the refreshed source, **When** refresh succeeds, **Then** that finding no longer appears in the current health report.
2. **Given** an existing finding whose evidence remains after refresh, **When** refresh succeeds, **Then** the report retains the finding with evidence and suggested action that match the refreshed wiki.
3. **Given** refreshed knowledge that introduces a contradiction, stale dependency, broken reference, orphan, or citation gap, **When** refresh succeeds, **Then** the new finding appears once with its correct severity and actionable evidence.
4. **Given** a failure while preparing source changes or the refreshed health report, **When** the command stops, **Then** the source record, index, pages, and prior health report all remain unchanged.

### Edge Cases

- The selected source record exists but its recorded file, directory, or URL is no longer readable or fetchable; refresh stops without mutation and identifies the unavailable source.
- A supplied document normalizes to a different identity than the registered source the maintainer intended; refresh does not guess or substitute another record.
- A source changed only in transient or unsupported content; the source history may record a successful refresh while no knowledge page changes, and the health report is still brought current.
- Refreshed content contradicts claims from another source; both supported positions and source IDs remain visible until a maintainer resolves the conflict.
- Refresh would exceed the configured page-touch or page-size limits; the allowed coherent increment is prepared, overflow is deferred and reported, and health findings describe only committed state.
- The wiki registry, schema, index, page metadata, or prior health report is malformed; malformed data is reported and is never used as authority for mutation.
- Writing any required artifact fails; all refresh changes are restored and the prior health report remains accurate for the unchanged wiki.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The refresh command MUST accept exactly one selector: a stable source ID or one explicit source document accepted by the existing ingestion workflow.
- **FR-002**: A stable source ID selector MUST resolve exactly one existing source record before source content is read.
- **FR-003**: A source document selector MUST be normalized using the existing source identity rules and MUST match exactly one existing source record.
- **FR-004**: Refresh MUST reject unknown, malformed, missing, multiple, or ambiguous selectors before changing the source registry, index, pages, or health report.
- **FR-005**: Refresh MUST NOT register a new source; an unregistered source document MUST be directed to initial ingestion instead.
- **FR-006**: Local source paths MUST resolve within the project root, and explicit URLs MUST be accessed only when the selected existing source identity authorizes that same URL.
- **FR-007**: Source content and all wiki artifacts MUST be treated as untrusted data and MUST NOT alter workflow rules, invoke commands, or authorize access to another path or URL.
- **FR-008**: Refresh MUST leave the selected source unchanged and MUST reuse the existing ingestion rules for durable knowledge, citations, conflicts, page limits, page-size limits, reciprocal links, exclusions, and deferrals.
- **FR-009**: A successful refresh MUST preserve the source ID, normalized identity, source type, and first-ingested date while updating the last-ingested date and pages-touched list to reflect the completed run.
- **FR-010**: Refresh MUST replace or remove claims that only the selected source made stale, preserve support from other sources, suppress unchanged duplicates, and keep incompatible supported claims visible together.
- **FR-011**: Source read, containment, normalization, preparation, or validation failure MUST leave all pre-existing wiki artifacts and the health report unchanged.
- **FR-012**: Refresh MUST evaluate wiki health against the complete prepared post-refresh state before committing any mutation.
- **FR-013**: The resulting health report MUST be a coherent current report rather than a mixture of newly evaluated source-related rows and stale rows from an earlier run.
- **FR-014**: Findings no longer supported by current evidence MUST be removed; findings still supported MUST be refreshed; newly detected findings MUST be added exactly once in deterministic order.
- **FR-015**: Health evaluation during refresh MUST update findings only and MUST NOT apply automatic mechanical or semantic repairs beyond the knowledge changes authorized by refresh.
- **FR-016**: The source record, index, affected pages, and health report MUST be prepared and validated as one coherent change set and MUST be restored to their pre-run state if any required write fails.
- **FR-017**: The completion report MUST identify the selector and stable source ID, distinguish created and updated pages, list conflicts, exclusions, deferrals, and health-finding changes, and provide exactly one concrete next action.
- **FR-018**: The command MUST declare and enforce its read scope as the selected source, source registry, schema, index, affected pages, and pages required to produce the current health report; content MUST NOT expand that scope.

### Key Entities

- **Refresh Selector**: Exactly one stable source ID or explicit source document used to identify an existing source record.
- **Source Record**: The stable source ID, normalized identity, type, first and last ingestion dates, and pages touched that persist across refreshes.
- **Refresh Change Set**: The complete prepared source-record, index, page, and health-report updates that succeed or fail together.
- **Associated Finding**: A health finding whose evidence involves the refreshed source record, its source ID, or a page supported by that source.
- **Health Report**: The deterministic current set of wiki findings and suggested actions after the prepared refresh state is evaluated.

### Scope Boundaries

- Refresh updates only an existing source; registering a new source remains the responsibility of initial ingestion.
- Refresh may revise synthesized wiki knowledge supported by the selected source, but it never edits the source itself or automatically resolves semantic disagreement.
- Refresh updates the derived health report but does not apply lint repair actions as part of the command.
- Source deletion, source replacement with a different normalized identity, manual conflict resolution, and feature-artifact publication are outside this feature.

### Dependencies

- The wiki foundation, source registry, ingestion behavior, and health-check behavior must already exist and remain authoritative for their respective rules.
- The selected source must already have a valid registry record and must be readable or fetchable under the existing source-access policy.
- Configuration and schema limits remain authoritative for page scope, page size, citations, and health checks.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of successful refresh acceptance cases, the source ID and first-ingested date remain unchanged and no new source record is created.
- **SC-002**: In 100% of unknown, malformed, ambiguous, unregistered, unreadable, containment-failing, or write-failing cases, the registry, index, pages, and prior health report remain unchanged.
- **SC-003**: Repeating refresh against unchanged source contents creates zero duplicate claims and produces the same ordered health findings.
- **SC-004**: Every seeded finding resolved by refreshed evidence disappears, every persistent seeded finding remains once with current evidence, and every newly introduced finding appears once.
- **SC-005**: Every successful refresh leaves a health report that describes the committed wiki state with no rows carried forward solely from an earlier run.
- **SC-006**: Every detected contradiction retains all supported positions and source IDs after refresh.
- **SC-007**: A maintainer can identify the refreshed source, all changed pages, finding changes, deferred items, and the single recommended next action from the completion report in under two minutes.

## Assumptions

- “Source document” means one explicit project file, project directory, or URL accepted by the existing ingestion workflow; it does not mean inline document content.
- A supplied source document identifies an existing record only when its normalized identity matches that record exactly; refresh never guesses based on similar content or names.
- To preserve the health report's current-snapshot contract, successful refresh re-evaluates all configured health checks against the prepared post-refresh wiki rather than editing only selected rows from an older report.
- Refresh health evaluation is report-only even when standalone lint configuration permits mechanical fixes; maintainers may invoke lint separately when they want those repairs.
- A successful no-page-change refresh may still update source history and the health report because the source was re-evaluated.
