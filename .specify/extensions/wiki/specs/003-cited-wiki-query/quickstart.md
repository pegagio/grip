# Quickstart: Validate Cited Wiki Query

Use disposable wiki fixtures and compare repository checksums before and after every scenario.

## Scenario 1: Covered

Query fully supported decision and component pages. Verify each material statement names its page and registered sources, relied-on pages are listed, and the verdict is Covered.

## Scenario 2: Partial and Uncovered

Ask a multi-part question with only one supported part, then an unsupported question. Verify exact gaps, concrete ingest suggestions, and no improvised facts.

## Scenario 3: Conflict and Invalid Provenance

Query a conflict fixture and a claim citing an unknown source. Verify all valid sides remain visible and the invalid claim is excluded and reported.

## Scenario 4: Limits and Injection

Use more candidate pages than both limits and embed instructions in the question, index, and page. Verify limits hold and no instruction changes the workflow.

## Scenario 5: Empty Overview and Read-Only State

Run with no question. Verify a one-screen overview from structural metadata and unchanged checksums.

## Static Validation

```bash
git diff --check
```

Confirm the query prompt covers evidence validation, limits, conflict preservation, verdict rules, untrusted data, and zero writes.
