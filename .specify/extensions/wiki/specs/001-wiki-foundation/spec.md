# Feature Specification: Wiki Foundation

**Feature Branch**: `feature/time-machine-wiki-foundation`

**Created**: 2026-08-26

**Status**: Draft

**Input**: User description: "Feature: Wiki Foundation. Description: Initialize a project wiki with configurable schema, index, and source-registry state without overwriting existing knowledge. Relevant files: commands/speckit.wiki.init.md, config-template.yml, extension.yml. Focus on this feature only; do not modify other features."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Initialize a Project Wiki (Priority: P1)

A project maintainer initializes a wiki so the project has a durable schema, page index, and source registry ready for later knowledge ingestion.

**Why this priority**: Every other wiki capability depends on this foundation existing with a predictable structure.

**Independent Test**: Initialize a project with no existing wiki and verify that the three required artifacts are created with the resolved scope and policy values while no knowledge pages are pre-populated.

**Acceptance Scenarios**:

1. **Given** a project with no initialized wiki, **When** the maintainer initializes the wiki with a scope statement, **Then** the schema, empty page index, and empty source registry are created and the schema records that scope.
2. **Given** a project with no initialized wiki and no scope statement, **When** the maintainer initializes the wiki, **Then** the required artifacts are created and the schema clearly indicates that the scope has not yet been set.
3. **Given** a newly initialized wiki, **When** the maintainer inspects its contents, **Then** no knowledge pages or unsupported claims have been added.

---

### User Story 2 - Preserve an Existing Wiki (Priority: P2)

A project maintainer can safely run initialization again without losing or replacing existing wiki knowledge.

**Why this priority**: Initialization must be safe to invoke in existing projects because accidental replacement would destroy durable project knowledge.

**Independent Test**: Initialize an existing wiki containing edited schema text, indexed pages, and registered sources, then verify that existing content remains byte-for-byte unchanged except for an explicitly supplied additional scope statement.

**Acceptance Scenarios**:

1. **Given** an already initialized wiki, **When** the maintainer initializes it again without a new scope, **Then** no wiki artifact is overwritten and the maintainer receives the current wiki status.
2. **Given** an already initialized wiki, **When** the maintainer initializes it with a new scope statement, **Then** that statement is appended as an additional scope item and all other wiki content remains unchanged.

---

### User Story 3 - Configure the Foundation (Priority: P3)

A project maintainer can choose where the wiki lives and tune its operating limits while retaining safe defaults when no customization is supplied.

**Why this priority**: Projects need predictable defaults first, but repository layout and maintenance policies vary enough to require controlled overrides.

**Independent Test**: Initialize otherwise identical projects using defaults, saved project configuration, environment overrides, and invocation overrides, then verify that each project receives the expected effective settings according to precedence.

**Acceptance Scenarios**:

1. **Given** no custom configuration, **When** the wiki is initialized, **Then** the documented default directory, ingestion limits, query limits, staleness threshold, and repair policy are recorded.
2. **Given** conflicting saved, environment, and invocation settings, **When** the wiki is initialized, **Then** invocation settings take precedence over environment settings, which take precedence over saved settings, which take precedence over defaults.
3. **Given** a custom wiki directory relative to the project root, **When** initialization succeeds, **Then** all foundation artifacts are created together in that directory.

### Edge Cases

- The configured wiki directory exists but contains unrelated files and no wiki schema; initialization creates only the missing foundation artifacts and preserves unrelated content.
- The schema exists while the index or source registry is missing; the wiki is treated as already initialized and no artifact is silently reconstructed or overwritten.
- A repeated scope statement is supplied; it is appended as a new numbered scope item because scope history is user-controlled and initialization does not deduplicate or rewrite it.
- A configuration source omits some settings; omitted values fall through to the next lower-precedence source.
- The configured wiki directory is outside the project root; initialization rejects the location rather than writing outside the project boundary.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST resolve wiki settings in this precedence order, from highest to lowest: invocation overrides, environment overrides, saved project configuration, extension defaults.
- **FR-002**: The system MUST resolve the wiki directory relative to the project root and MUST reject any resolved location outside the project boundary.
- **FR-003**: When no schema exists in the resolved wiki directory, the system MUST create exactly one schema, one page index, and one source registry.
- **FR-004**: The created schema MUST record the wiki scope, supported page types, naming and linking rules, citation requirements, conflict-preservation rule, page metadata requirements, and available maintenance workflows.
- **FR-005**: When no scope is supplied, the created schema MUST include an explicit prompt indicating that the scope is unset.
- **FR-006**: The created page index MUST contain no page entries and MUST direct the maintainer to ingest the first source.
- **FR-007**: The created source registry MUST contain the registry columns and no registered source rows.
- **FR-008**: Initializing a new wiki MUST NOT create knowledge pages or synthesize project knowledge.
- **FR-009**: If the schema already exists, the system MUST NOT overwrite, truncate, regenerate, or delete any existing wiki artifact.
- **FR-010**: If a new scope statement is supplied for an existing wiki, the system MUST append it as a numbered scope item without changing other schema content.
- **FR-011**: After detecting an existing wiki, the system MUST report the current wiki status instead of performing new-wiki initialization.
- **FR-012**: After creating a new wiki, the system MUST report the resolved directory, created artifacts, effective scope, active limits, and the next available wiki actions.
- **FR-013**: The extension MUST expose wiki initialization as an optional-config, read-write project capability compatible with its declared Spec Kit version requirement.

### Key Entities

- **Wiki Configuration**: The effective directory and operating policies resolved from defaults and optional override sources.
- **Wiki Schema**: The user-editable contract defining scope, page taxonomy, citation and linking rules, conflict handling, metadata, and maintenance workflows.
- **Page Index**: The directory of wiki pages, initially empty and maintained by later wiki operations.
- **Source Registry**: The append-only catalog of source identities and ingestion history, initially containing no source records.
- **Scope Statement**: A numbered declaration of the knowledge the project wiki exists to accumulate.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In every new-wiki acceptance case, maintainers receive all three required foundation artifacts after a single initialization attempt.
- **SC-002**: In every repeated-initialization acceptance case, 100% of pre-existing wiki content remains unchanged except for an explicitly requested appended scope statement.
- **SC-003**: All documented configuration-precedence cases resolve to the expected effective value without requiring maintainers to edit generated artifacts.
- **SC-004**: A maintainer can identify the wiki’s scope, supported page types, active limits, and next action within two minutes of initialization.
- **SC-005**: New-wiki initialization produces zero knowledge pages and zero synthesized claims before a source is explicitly ingested.
- **SC-006**: All tested paths that escape the project boundary are rejected without creating or changing files outside the project.

## Assumptions

- The project already has a working Spec Kit installation and a writable project root.
- The initialization capability establishes wiki state only; source ingestion, querying, linting, and detailed status behavior are separate features.
- Scope statements are trusted user-authored project metadata and are preserved verbatim.
- Existing wiki artifacts may have been intentionally edited by maintainers and therefore remain authoritative during repeated initialization.
- Missing artifacts in a partially initialized existing wiki are surfaced through status or maintenance workflows rather than silently recreated.
