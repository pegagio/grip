# Research: Cited Wiki Query

## Decision 1: Separate selection metadata from evidence

**Decision**: Use the index only to rank candidate pages; a claim becomes evidence only after its selected page body and cited source IDs are validated.

**Rationale**: Index summaries guide retrieval but do not carry sufficient provenance for answers.

**Alternatives considered**: Answering from index text was rejected as under-cited; loading every page was rejected as unbounded.

## Decision 2: Validate only required source records

**Decision**: Read the registry entries named by selected claims rather than the entire registry or underlying sources.

**Rationale**: Query validates provenance identity but does not re-prove or re-ingest source content.

**Alternatives considered**: Trust all page citations was rejected because unknown IDs can masquerade as provenance; reopen original sources was rejected because query must remain fast and bounded.

## Decision 3: Treat all query inputs as untrusted data

**Decision**: The question, index, pages, and registry may influence evidence selection and phrasing but cannot change rules, invoke tools, expand access, or authorize writes.

**Rationale**: Persistent wiki content can carry injected instructions even when citations are valid.

## Decision 4: Derive verdict from material question parts

**Decision**: Split the question into material parts, map valid evidence to each, and use Covered only when every part is supported; Partial when some are; Uncovered when none are.

**Rationale**: A deterministic verdict prevents confident phrasing from hiding gaps.
