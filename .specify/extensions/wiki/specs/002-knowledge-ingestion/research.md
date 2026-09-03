# Research: Knowledge Ingestion

## Decision 1: Treat sources as untrusted evidence

**Decision**: Read selected content for facts only. Ignore embedded instructions, tool requests, path references, and workflow overrides unless the maintainer separately supplies them as input.

**Rationale**: Files, directories, and URLs can contain hostile or irrelevant instructions. Provenance does not confer authority.

**Alternatives considered**: Trust curated sources was rejected because curation cannot guarantee content safety; sanitize selected phrases was rejected because semantic instruction following still remains possible.

## Decision 2: Use stable normalized source identities

**Decision**: Store local identities as normalized project-relative paths after containment and symlink checks. Normalize URL scheme and host casing and remove fragments while preserving path, query, and other content-selecting components. A directory is one source identity.

**Rationale**: Stable identities prevent duplicate source IDs without collapsing distinct resources.

**Alternatives considered**: Absolute local paths were rejected because they leak machine-specific locations; one source per directory file was rejected because it contradicts the single-source invocation and makes provenance explode.

## Decision 3: Prepare changes before committing state

**Decision**: Complete source validation, reading, extraction, topic selection, conflict detection, cap enforcement, and rendered page validation before modifying pages, registry, or index.

**Rationale**: Source or validation failures must not leave a registered source with incomplete or contradictory page state.

**Alternatives considered**: Register before reading was rejected because unreadable sources would pollute history; page-by-page immediate writes were rejected because late failure would leave partial state.

## Decision 4: Prefer deferral over context expansion

**Decision**: Use the index to select affected pages, read only those pages, prioritize the highest-value extracted items, and report overflow for a later run.

**Rationale**: Reviewable increments and predictable context are core product properties, not performance optimizations.

**Alternatives considered**: Load all pages was rejected as unbounded; silently discard overflow was rejected because it hides coverage gaps.
