# Research: Wiki Status and Resumption

## Decision 1: Structural metadata is the only status authority

**Decision**: Status reads schema scope, index, source registry, the optional lint report, and bounded active-feature metadata: the feature pointer plus artifact paths, existence, and timestamps. It never opens page bodies, feature-artifact bodies, or original sources to fill gaps.

**Rationale**: A resumption snapshot must stay cheap and predictable. Missing dates are more honestly reported as unknown than reconstructed through hidden reads.

## Decision 2: Use explicit bounded slices

**Decision**: Default output uses compact fixed slices, a page-type argument filters only the page slice, and `full` expands every slice by at most three times.

**Rationale**: Users can focus or expand the snapshot without converting status into query or lint.

## Decision 3: Select one action deterministically

**Decision**: Generate candidates only from current structural evidence, then choose by declared priority, newest relevant evidence, and lexical page or source identity.

**Rationale**: One stable recommendation makes the next session actionable and keeps repeated runs reviewable.

## Decision 4: Make read-only and trust boundaries explicit

**Decision**: Treat all arguments and structural text as untrusted data. Status cannot repair, normalize, create, or update any artifact.

**Rationale**: A diagnostic entry point must not acquire mutation authority from content it reads.
