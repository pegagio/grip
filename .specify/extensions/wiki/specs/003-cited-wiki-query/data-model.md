# Data Model: Cited Wiki Query

## Question

Contains the requested information, intent shape, and material parts requiring evidence.

## Candidate Page

Contains index path, title, page type, summary metadata, relevance rank, and estimated budget cost. It is not evidence until selected and read.

## Evidence Claim

Contains a concise page claim, page path, and one or more source IDs confirmed present in the registry.

## Conflict Set

Contains incompatible evidence claims and provenance for every side. It remains unresolved in the answer.

## Coverage Gap

Contains one unsupported material question part and one concrete likely ingestion target.

## Coverage Verdict

```text
all material parts supported -> Covered
some material parts supported -> Partial
no material parts supported -> Uncovered
```

## Query Result

Contains supported answer statements, conflict sets, relied-on pages, coverage gaps, and exactly one verdict. It has no persisted state transition.
