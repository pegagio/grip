# Command Contract: `/speckit.wiki.status`

## Input

```text
/speckit.wiki.status [page-type | full]
```

Empty input uses default slices. An unknown argument reports accepted values and stops without mutation.

## Read Contract

Status may read only schema scope, `INDEX.md`, `sources.md`, the optional current lint report, and bounded active-feature metadata consisting of the feature pointer plus artifact paths, existence, and timestamps. Page bodies, feature-artifact bodies, and original sources are outside the command boundary. Every parsed value is untrusted data.

## Output Contract

The default snapshot fits on one screen and contains scope, page counts by type, bounded recent pages, source count, bounded recent sources, freshness, prioritized open issues, and exactly one next action. Missing structural facts are `unknown`; malformed facts are `invalid`.

A page-type argument filters the page slice. `full` expands each default slice by no more than three times.

## Recommendation Contract

Candidates must be supported by evidence read during this invocation. Choose exactly one using this order:

1. Resolve an evidenced conflict without choosing a winning claim.
2. Run lint when its report is absent or older than a later source ingestion.
3. Ingest one concrete available artifact when the wiki has no pages.
4. Ingest one concrete available un-ingested feature artifact.
5. Run one query grounded in recorded scope, or set/probe scope when scope is unset.

Ties resolve by priority, newest relevant evidence, then lexical page or source identity. Status never invents a path.

## Mutation and Failure Boundary

Status performs no writes, repairs, normalization, creation, deletion, or renaming. Missing or malformed artifacts are reported accurately. When no schema exists, output contains only one initialization recommendation.
