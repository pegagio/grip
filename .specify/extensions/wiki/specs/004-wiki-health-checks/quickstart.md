# Quickstart: Validate Wiki Health Checks

Use disposable fixtures and record checksums for claims, source history, and conflict markers.

## Scenario 1: Six Findings

Seed one issue per check and verify one deterministic, evidenced report row each.

## Scenario 2: Mechanical Auto-Fix

Seed index drift and one unambiguous renamed target. Enable `index-and-links`; verify only index, link target, and report change.

## Scenario 3: Semantic Preservation

Seed conflicts, stale claims, unknown citations, and orphans. Verify only findings and suggestions; semantic checksums remain identical.

## Scenario 4: Scope and Bounds

Run one named check and one page scope. Verify unrelated pages are not read and contradiction pairs share a source or link.

## Scenario 5: Ambiguity, Injection, and Failure

Seed ambiguous rename candidates, embedded instructions, malformed metadata, and a failed write. Verify no unsafe repair, no instruction execution, and no partial state.

## Static Validation

```bash
git diff --check
```

Confirm the lint prompt defines all six checks, the fix allowlist, deterministic reporting, untrusted data, transaction behavior, and one prioritized next action.
