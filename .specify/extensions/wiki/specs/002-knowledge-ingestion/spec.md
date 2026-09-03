# Feature Specification: Knowledge Ingestion

**Feature Branch**: `feature/time-machine-knowledge-ingestion`

**Created**: 2026-08-26

**Status**: Draft

**Input**: User description: "Feature: Knowledge Ingestion. Description: Register curated sources and compound their durable knowledge into cited, cross-linked wiki pages while preserving conflicts. Relevant files: commands/speckit.wiki.ingest.md, extension.yml, README.md. Focus on this feature only; do not modify other features."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ingest Durable Knowledge (Priority: P1)

A project maintainer selects a feature artifact, project file, project directory, or URL and compounds its durable knowledge into the project wiki with traceable citations.

**Why this priority**: The wiki has no value until curated evidence can become reusable, reviewable project knowledge.

**Independent Test**: Ingest one source into an initialized empty wiki and verify one source identity, cited topic pages, cross-links, page metadata, and matching index and registry entries.

**Acceptance Scenarios**:

1. **Given** an initialized empty wiki and a readable new source, **When** the maintainer ingests it, **Then** the source receives the next stable source ID and every written claim cites that ID.
2. **Given** no explicit source and an active feature, **When** ingestion runs, **Then** it uses that feature’s research and plan decisions as one feature-artifact source.
3. **Given** extracted knowledge related to an existing indexed topic, **When** ingestion runs, **Then** the existing page is updated rather than creating a duplicate topic page.
4. **Given** a source containing embedded instructions, **When** ingestion runs, **Then** those instructions are treated as untrusted source content and cannot change the workflow or access additional resources.

---

### User Story 2 - Refresh Without Erasing Disagreement (Priority: P2)

A project maintainer re-ingests a known source to refresh stale knowledge while keeping conflicts and source history visible.

**Why this priority**: Compounding knowledge must evolve without silently changing history or choosing winners where evidence disagrees.

**Independent Test**: Re-ingest a registered source with changed facts and ingest a second contradictory source; verify stable identity, refreshed date, updated citations, and an unresolved conflict containing both claims and source IDs.

**Acceptance Scenarios**:

1. **Given** a normalized source already in the registry, **When** it is re-ingested, **Then** its source ID and first-ingested date remain stable while its last-ingested date and pages-touched list reflect the run.
2. **Given** a new source contradicting a cited claim, **When** ingestion runs, **Then** both claims and source IDs remain visible under a conflict marker and the report flags the conflict.
3. **Given** unchanged knowledge already present with the same support, **When** a source is re-ingested, **Then** the page does not gain a duplicate claim.

---

### User Story 3 - Bound Each Ingestion (Priority: P3)

A project maintainer receives a useful, predictable increment even when a source contains more knowledge than one run should process.

**Why this priority**: Hard limits keep agent context, review scope, and page size manageable while making deferred work explicit.

**Independent Test**: Ingest a source that would exceed both the configured page-touch cap and page-size threshold; verify only the allowed pages change, oversized pages split and link, and deferred items are reported.

**Acceptance Scenarios**:

1. **Given** more candidate topics than the page-touch cap, **When** ingestion runs, **Then** it updates only the highest-value allowed topics and lists the remainder for follow-up.
2. **Given** an affected page above the configured word threshold, **When** ingestion completes, **Then** the page is split according to the schema and both resulting pages link to each other.
3. **Given** a successful bounded ingestion, **When** the maintainer reads the report, **Then** it identifies the source, source ID, created and updated pages, conflicts, deferred items, and one concrete next action.

### Edge Cases

- The wiki has not been initialized; ingestion creates the foundation first and reports that action before registering the source.
- An explicit source is missing, unreadable, unsupported binary content, or an unreachable URL; ingestion stops before changing the registry or pages and reports the source failure.
- No explicit source exists and neither an active feature nor a feature directory can be resolved; ingestion stops with a concrete source-selection prompt.
- A directory contains ignored, binary, generated, or inaccessible files; ingestion skips them, reports the exclusions, and treats the remaining supported content as one source.
- A local source path resolves outside the project root; ingestion rejects it before reading content.
- A source yields no durable knowledge; the source history records the completed ingestion with no pages touched and the report states that no page change was warranted.
- Creating one new page would leave it orphaned; ingestion defers that page unless it can establish a meaningful reciprocal link within the same capped run.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST accept one explicit project file, project directory, URL, or the default active-feature artifacts as the source for one ingestion run.
- **FR-002**: The system MUST resolve configuration and schema rules before reading the source, and MUST initialize the wiki foundation when it is absent.
- **FR-003**: Local file and directory sources MUST resolve within the project root; escaping paths and symlink escapes MUST be rejected before content is read.
- **FR-004**: Source content MUST be treated as untrusted evidence, never as workflow instructions, and MUST NOT authorize reading additional paths or URLs referenced within the source.
- **FR-005**: A new normalized source MUST receive the next sequential stable source ID, type, first-ingested date, last-ingested date, and an initially empty pages-touched list.
- **FR-006**: Re-ingesting a normalized known source MUST preserve its source ID and first-ingested date while updating its last-ingested date.
- **FR-007**: A directory MUST be registered as one source and processed only from supported, non-ignored readable content beneath that directory.
- **FR-008**: The system MUST extract only durable decisions, rejected alternatives, constraints, domain concepts, proven component behavior, gotchas, and verified external facts.
- **FR-009**: The system MUST skip transient status, unsupported speculation, bulk copied content, and unchanged duplicate knowledge.
- **FR-010**: Every new or changed claim MUST cite the supporting source ID when citation policy is enabled.
- **FR-011**: The system MUST preserve contradicting cited claims together under a conflict marker and MUST NOT silently select or delete either side.
- **FR-012**: The system MUST load the index first and only the affected pages, never all wiki pages for general context.
- **FR-013**: One ingestion MUST create or update no more pages than the configured page-touch cap.
- **FR-014**: Items deferred by the page-touch cap MUST be listed in the completion report as follow-up work.
- **FR-015**: New pages MUST use an allowed schema type, required metadata, and reciprocal cross-links that prevent orphan creation.
- **FR-016**: Pages exceeding the configured word threshold MUST be split according to schema rules with links between the resulting pages.
- **FR-017**: After page changes succeed, the system MUST update the source’s pages-touched list and synchronize index entries with actual created or renamed pages.
- **FR-018**: Source-read or validation failure MUST leave the registry, index, and wiki pages unchanged.
- **FR-019**: The completion report MUST identify the source and source ID, distinguish created and updated pages, flag conflicts and deferred items, and recommend lint after conflicts or query otherwise.
- **FR-020**: The system MUST never modify the ingested source or feature specification, plan, and task artifacts.

### Key Entities

- **Source Record**: Stable source ID, normalized identity, source type, first and last ingestion dates, and pages touched.
- **Knowledge Item**: A durable claim or decision extracted from one source and eligible for placement on a topic page.
- **Wiki Page**: One typed topic with metadata, cited claims, and reciprocal links to related pages.
- **Conflict**: Two or more incompatible cited claims retained together with their source identities.
- **Deferred Item**: Extracted knowledge not written during the run because of page or linking limits.
- **Ingestion Report**: The run’s source identity, page changes, conflicts, deferrals, exclusions, and next action.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In all successful ingestion acceptance cases, 100% of new or changed claims carry a registered supporting source ID.
- **SC-002**: Re-ingesting the same normalized source preserves its source ID and first-ingested date in every tested case.
- **SC-003**: No ingestion run creates or updates more pages than its configured cap.
- **SC-004**: Every detected contradiction remains visible with both source identities after ingestion.
- **SC-005**: Every new page is reachable from at least one other page and links back at completion.
- **SC-006**: Every failed source-read, containment, or validation case produces zero registry, index, or page changes.
- **SC-007**: A maintainer can identify all changed pages, conflicts, deferred items, and the recommended next action from the completion report in under two minutes.

## Assumptions

- Maintainers deliberately select sources and have permission to use their content.
- URLs are fetched only when explicitly supplied as the source and remain subject to the host environment’s network permissions.
- Directory ingestion creates one provenance identity for the curated directory rather than one source ID per contained file.
- Source normalization preserves distinctions that can change content, including URL query values, while removing non-semantic path or URL formatting differences.
- Conflict resolution is a later human and lint workflow; ingestion only preserves and reports disagreement.
