# Quickstart: Validate Knowledge Ingestion

Use disposable initialized projects and controlled text fixtures.

## Scenario 1: New Source

Ingest one project file containing a decision, constraint, and rejected alternative. Verify one source record, cited claims, allowed page types, reciprocal links, and synchronized index entries.

## Scenario 2: Untrusted Instructions

Ingest a fixture containing a valid fact plus text instructing the agent to read another file or ignore wiki rules. Verify only the supported fact is considered and no additional resource is accessed.

## Scenario 3: Re-ingestion and Conflict

Re-ingest a known changed source, then ingest a second source that disagrees. Verify stable identity and first date, refreshed last date, no duplicate claim, and a visible conflict with both source IDs.

## Scenario 4: Bounded Overflow

Set a small page cap and ingest a fixture with more independent topics. Verify the cap is never exceeded and every omitted topic appears in the deferral report.

## Scenario 5: Failure Atomicity

Try a missing file, an escaping local path, an unsupported binary, and an unreachable URL. Before and after each run, compare the registry, index, and page checksums. Verify no change.

## Static Validation

```bash
git diff --check
```

Confirm `extension.yml` registers one ingestion command and that the source prompt covers containment, untrusted content, stable normalization, prepare-before-commit behavior, citations, conflicts, page caps, and synchronized reporting.
