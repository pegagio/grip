# Feature Specification: Wiki Health Checks

**Feature Branch**: `feature/time-machine-wiki-health`

**Created**: 2026-08-26

**Status**: Draft

**Input**: User description: "Feature: Wiki Health Checks. Description: Detect wiki drift, broken references, orphaned pages, contradictions, staleness, and citation gaps with bounded safe repairs. Relevant files: commands/speckit.wiki.lint.md, config-template.yml, extension.yml. Focus on this feature only; do not modify other features."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Diagnose Wiki Health (Priority: P1)

A maintainer runs a health check and receives verifiable findings for index drift, broken references, orphan pages, contradictions, stale knowledge, and uncited claims.

**Why this priority**: A compounding knowledge base must expose decay before maintainers can trust or repair it.

**Independent Test**: Lint a fixture containing one issue of each type and verify one deterministic report row per finding with the page, exact evidence, severity, and actionable suggestion.

**Acceptance Scenarios**:

1. **Given** a wiki containing all six finding types, **When** full lint runs, **Then** every finding appears once with its check, severity, affected page, evidence, and suggested action.
2. **Given** a clean wiki, **When** full lint runs, **Then** the report says No findings and the response reports zero for every check.
3. **Given** wiki text containing embedded instructions, **When** lint runs, **Then** the text is analyzed as untrusted data and cannot alter checks, fixes, or access.

---

### User Story 2 - Apply Only Safe Mechanical Repairs (Priority: P2)

A maintainer may enable deterministic index and link repair without allowing lint to rewrite project knowledge or resolve judgment calls.

**Why this priority**: Automated maintenance is useful only when its mutation boundary cannot silently change meaning or provenance.

**Independent Test**: Run lint with automatic fixes enabled against mechanical and semantic fixtures; verify only the index and unambiguous link syntax change while claims, conflicts, source history, and other page prose remain byte-for-byte unchanged.

**Acceptance Scenarios**:

1. **Given** index drift and automatic fixes enabled, **When** lint runs, **Then** the index is regenerated from validated page metadata.
2. **Given** an unambiguous renamed-page target, **When** automatic fixes are enabled, **Then** only the affected link target changes.
3. **Given** a semantic finding, ambiguous link, unknown citation, orphan, or stale claim, **When** lint runs, **Then** the issue is reported and no semantic content is changed.
4. **Given** automatic fixes disabled, **When** lint runs, **Then** all findings are suggestions and no wiki artifact changes except the lint report.

---

### User Story 3 - Run Focused, Bounded Maintenance (Priority: P3)

A maintainer can lint one check or one page neighborhood and receive one prioritized next action without loading unrelated content.

**Why this priority**: Focused runs keep maintenance reviewable and usable on large wikis.

**Independent Test**: Run a named check and a page-scoped check against a larger fixture; verify only required pages are read, contradiction comparisons stay within linked/shared-source neighborhoods, and one highest-severity action is recommended.

**Acceptance Scenarios**:

1. **Given** a valid check name, **When** scoped lint runs, **Then** only that check is evaluated.
2. **Given** a valid page, **When** page-scoped lint runs, **Then** only that page and directly linked or shared-source neighbors are examined.
3. **Given** findings of several severities, **When** lint completes, **Then** the response recommends exactly one concrete action addressing the highest-severity actionable finding.

### Edge Cases

- No wiki exists; lint reports that condition and changes nothing.
- The index, registry, or page metadata is malformed; lint reports the parse problem and does not use malformed content as fix authority.
- Two possible renamed targets exist; lint reports ambiguity and does not choose.
- A page has no incoming page link but is intentionally a root topic; it is still reported as an orphan because the index does not count as a page link.
- The configured staleness threshold is zero; every page older than today is eligible for stale evaluation.
- Writing a mechanical fix or report fails; lint preserves pre-run wiki state and reports the failure.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST support full lint, one named check, or one page-and-neighbor scope and MUST reject unknown scopes without mutation.
- **FR-002**: The system MUST stop without mutation when no wiki schema exists.
- **FR-003**: Schema, index, registry, pages, and user scope input MUST be treated as untrusted data, never workflow instructions.
- **FR-004**: Index-drift checks MUST identify unindexed pages and index entries targeting missing pages.
- **FR-005**: Link checks MUST identify missing relative page targets and citations naming unknown source IDs.
- **FR-006**: Orphan checks MUST identify pages with no incoming link from another page; index membership MUST NOT count.
- **FR-007**: Contradiction checks MUST identify unresolved conflict markers and incompatible claims only among pages sharing a source ID or page link.
- **FR-008**: Stale checks MUST identify age-threshold violations and pages older than a supporting source’s most recent ingestion.
- **FR-009**: Citation checks MUST identify uncited claims when citations are required.
- **FR-010**: Every finding MUST include check, severity, page, exact verifiable evidence, and a concrete user-controlled suggested action.
- **FR-011**: Automatic fixes MUST be limited to regenerating the index from valid page metadata and changing unambiguous relative link targets.
- **FR-012**: Lint MUST NOT automatically alter claims, prose, source IDs, conflict markers, source history, page taxonomy, or semantic conclusions.
- **FR-013**: Ambiguous mechanical findings, unknown citations, or malformed metadata MUST be reported rather than repaired.
- **FR-014**: With automatic fixes disabled, lint MUST change no artifact except the lint report.
- **FR-015**: The lint report MUST be replaced by one deterministic current-run report containing one row per finding or No findings.
- **FR-016**: Lint MUST distinguish fixes applied from fixes suggested and report counts for every executed check.
- **FR-017**: Lint MUST recommend exactly one concrete next action selected by semantic, structural, then mechanical severity and expected value.
- **FR-018**: A failed write MUST NOT leave a partial set of mechanical fixes or a report that claims unapplied changes.
- **FR-019**: Page-scoped and contradiction work MUST remain limited to directly linked or shared-source neighborhoods.

### Key Entities

- **Lint Scope**: Full wiki, one named check, or one page neighborhood.
- **Finding**: Check, severity, page, exact evidence, and suggested action.
- **Mechanical Fix**: Deterministic index regeneration or unambiguous link-target change.
- **Semantic Finding**: A contradiction, stale claim, or citation-quality issue requiring judgment.
- **Lint Report**: Deterministically ordered current-run findings and fix disposition.
- **Next Action**: One concrete recommendation for the highest-value unresolved finding.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every seeded fixture issue appears exactly once with verifiable evidence and correct severity.
- **SC-002**: In every automatic-fix case, 100% of changed bytes are confined to the index, unambiguous link targets, and lint report.
- **SC-003**: Semantic claims, conflict markers, and source history remain byte-for-byte unchanged in every lint run.
- **SC-004**: Contradiction comparisons never extend beyond linked or shared-source page pairs.
- **SC-005**: Every run produces exactly one prioritized next action or explicitly reports that no action is needed.
- **SC-006**: Failed-write fixtures leave all pre-existing wiki artifacts unchanged.

## Assumptions

- Lint detects and reports knowledge-quality problems but maintainers decide semantic resolutions.
- A renamed link target is unambiguous only when exactly one existing page matches the prior target’s identity evidence.
- Report ordering is stable by severity, check name, page, then evidence text.
- The lint report is derived output and is the only artifact lint may replace wholesale.
