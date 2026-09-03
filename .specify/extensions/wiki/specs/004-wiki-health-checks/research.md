# Research: Wiki Health Checks

## Decision 1: Use an explicit repair allowlist

**Decision**: Automatic changes are limited to index regeneration from valid metadata and unambiguous relative link-target replacement.

**Rationale**: Deterministic structure can be reconstructed; claims, citations, conflict resolution, staleness, and orphan intent require judgment.

## Decision 2: Separate analysis, fix preparation, and commit

**Decision**: Gather findings first, prepare and validate all allowed fixes second, then apply fixes and write a report that reflects actual disposition.

**Rationale**: A report must not claim fixes that failed or leave semantic and structural state inconsistent.

## Decision 3: Treat wiki content as untrusted data

**Decision**: Schema, registry, index, pages, and scope input can supply parseable evidence only; embedded instructions cannot change checks, fix policy, access, or output.

## Decision 4: Make findings and priority deterministic

**Decision**: Deduplicate findings and order by severity, check, page, and evidence. Recommend one action using semantic, structural, mechanical priority and expected impact.

**Rationale**: Stable reports remain reviewable in version control and make session resumption predictable.
