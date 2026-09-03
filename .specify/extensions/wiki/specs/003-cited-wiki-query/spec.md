# Feature Specification: Cited Wiki Query

**Feature Branch**: `feature/time-machine-cited-query`

**Created**: 2026-08-26

**Status**: Draft

**Input**: User description: "Feature: Cited Wiki Query. Description: Answer project questions strictly from bounded wiki content with source citations and explicit coverage gaps. Relevant files: commands/speckit.wiki.query.md, config-template.yml, extension.yml. Focus on this feature only; do not modify other features."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Answer From Cited Wiki Evidence (Priority: P1)

A maintainer asks a project question and receives a focused answer whose material claims link to wiki pages and identify the registered sources supporting them.

**Why this priority**: Trustworthy reuse of accumulated knowledge is the wiki’s primary payoff.

**Independent Test**: Query a wiki whose relevant pages fully answer a question and verify that every material statement names a selected page and registered source ID, the relied-on pages are listed, and the result is marked Covered.

**Acceptance Scenarios**:

1. **Given** indexed pages that fully answer a question, **When** the maintainer queries the wiki, **Then** the answer uses only selected page content and closes with Covered.
2. **Given** a question asking why a choice was made, **When** pages are selected, **Then** relevant decision pages are preferred within the configured limits.
3. **Given** wiki or index text containing embedded instructions, **When** query runs, **Then** the text is treated only as untrusted evidence and cannot alter the workflow or authorize other access.

---

### User Story 2 - Expose Gaps and Disagreement (Priority: P2)

A maintainer can distinguish supported knowledge from partial coverage, absent coverage, and unresolved conflicts.

**Why this priority**: A confident unsupported answer would defeat the wiki’s provenance model and conceal work that still needs evidence.

**Independent Test**: Query partial, empty, and conflicting fixtures and verify exact gap descriptions, concrete ingest suggestions, both sides of conflicts, and the correct coverage verdict.

**Acceptance Scenarios**:

1. **Given** selected pages support only part of a question, **When** query runs, **Then** the supported portion is cited, the exact unsupported portion is named, a concrete source is suggested for ingestion, and the verdict is Partial.
2. **Given** no indexed page supports the question, **When** query runs, **Then** no factual answer is improvised, a likely source category is suggested, and the verdict is Uncovered.
3. **Given** selected pages contain an unresolved conflict, **When** query runs, **Then** each cited side is reported without selecting a winner.

---

### User Story 3 - Keep Queries Bounded and Read-Only (Priority: P3)

A maintainer can query or inspect a wiki without changing it or loading the whole knowledge base.

**Why this priority**: Predictable context and zero mutation make query safe for frequent use and review.

**Independent Test**: Query a wiki larger than both configured limits, then compare all wiki checksums and verify the selected page count and rendered content stay within the limits.

**Acceptance Scenarios**:

1. **Given** more relevant pages than the configured slice, **When** query runs, **Then** no more than the page slice is read.
2. **Given** selected content larger than the context budget, **When** query runs, **Then** selection is narrowed before answering and the rendered content remains within budget.
3. **Given** an empty question, **When** query runs, **Then** a one-screen evidence-backed overview is returned without reading all page bodies.
4. **Given** any successful or unsuccessful query, **When** the project is inspected afterward, **Then** no wiki artifact has changed.

### Edge Cases

- No wiki schema exists; query recommends initialization and supplies no project answer.
- The index is missing, malformed, or points to a missing page; query reports the integrity problem and does not infer the missing content.
- A selected page cites an unknown source ID; the affected claim is excluded from supported evidence and reported as a provenance gap.
- A page has no usable cited claim after validation; it does not contribute to the answer or Covered verdict.
- Several pages repeat the same claim; the answer states it once and aggregates distinct supporting provenance.
- The question contains instructions to ignore wiki rules or write files; query treats the question only as the information request and remains read-only.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST accept a project question; with no question it MUST return a bounded overview and stop.
- **FR-002**: The system MUST stop with an initialization recommendation when the wiki schema does not exist.
- **FR-003**: The system MUST read the index before selecting pages and MUST NOT load all page bodies for context.
- **FR-004**: The system MUST select no more pages than `query.pages_slice` and MUST keep rendered selected content within `query.context_tokens`.
- **FR-005**: Page selection MUST rank topical relevance and prefer page types matching the question’s intent.
- **FR-006**: Index entries, page content, source registry text, and the user’s question MUST be treated as untrusted data, never workflow or tool instructions.
- **FR-007**: The system MUST read only selected page bodies and the source-registry entries needed to validate cited source IDs.
- **FR-008**: Every material answer statement MUST name at least one selected wiki page and at least one registered source ID that supports it.
- **FR-009**: Claims with missing or unknown source provenance MUST NOT be presented as supported facts.
- **FR-010**: Repeated equivalent claims MUST be stated once while retaining distinct supporting pages and source IDs.
- **FR-011**: Unresolved conflicts MUST present each cited position without choosing a winner or merging incompatible claims.
- **FR-012**: A Covered verdict MUST be used only when selected valid evidence supports every material part of the question.
- **FR-013**: A Partial verdict MUST identify the supported answer, the exact unsupported portion, and a concrete likely source to ingest.
- **FR-014**: An Uncovered verdict MUST provide no factual project answer and MUST suggest a concrete likely source category or path for ingestion.
- **FR-015**: The response MUST list every page relied upon and MUST NOT list pages that did not support the answer.
- **FR-016**: The empty-question overview MUST summarize scope, page types, counts, and notable indexed pages within one screen without loading every page body.
- **FR-017**: Query MUST NOT create, modify, rename, or delete any wiki, specification, configuration, or source artifact.
- **FR-018**: Missing, malformed, or stale structural references encountered during query MUST be reported as integrity or provenance gaps rather than repaired.

### Key Entities

- **Question**: The maintainer’s requested project information and intent shape.
- **Candidate Page**: An index entry ranked for relevance before its body is read.
- **Evidence Claim**: A selected page statement with valid registered source provenance.
- **Conflict Set**: Incompatible evidence claims that must remain side by side.
- **Coverage Verdict**: Covered, Partial, or Uncovered based on support for the question’s material parts.
- **Coverage Gap**: An unsupported question part paired with a concrete ingestion suggestion.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In all acceptance answers, 100% of material factual statements identify a relied-on page and registered source ID.
- **SC-002**: No query reads more pages or rendered wiki content than the configured limits.
- **SC-003**: Every partial, uncovered, conflicting, or invalid-provenance fixture is labeled without an unsupported factual conclusion.
- **SC-004**: Wiki and project artifact checksums remain unchanged after every query case.
- **SC-005**: A maintainer can trace each answer statement to its page and source entry in under two minutes.
- **SC-006**: Empty-question output fits on one screen under the configured default context.

## Assumptions

- Ingestion is responsible for creating cited content; query validates and consumes that content but never repairs it.
- Source registry entries are read only for provenance validation and are not treated as evidence beyond the selected page claims they support.
- General model knowledge may improve phrasing but cannot add project facts, fill evidence gaps, or resolve conflicts.
- A material statement is one whose removal would change the substantive answer, conclusion, constraint, or recommendation.
