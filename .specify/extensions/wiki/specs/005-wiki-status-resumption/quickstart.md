# Quickstart: Validate Wiki Status and Resumption

Use disposable fixtures and record a full-project checksum before each run.

## Scenario 1: Populated Snapshot

Seed valid scope, index metadata, sources, and lint output. Verify compact counts, bounded slices, structural freshness, prioritized issues, exactly one action, and zero page-body reads.

## Scenario 2: Missing and Malformed Metadata

Remove dates and corrupt one structural row. Verify affected values are unknown or invalid and no hidden page read or repair occurs.

## Scenario 3: Recommendation Priority

Run conflict, stale-lint, empty-wiki, un-ingested-artifact, healthy-wiki, and no-wiki fixtures. Verify one action follows the declared priority and ties are stable.

## Scenario 4: Filters and Bounds

Run default, each configured page type, `full`, and an invalid argument. Verify page filtering, at most threefold slice expansion, accepted-value reporting, and unchanged non-page sections.

## Scenario 5: Trust and Read-Only Boundary

Embed executable instructions in every structural artifact and simulate malformed or missing files. Verify content remains data, no new access is authorized, and project checksums remain unchanged.

## Static Validation

```bash
git diff --check
```

Confirm the prompt defines structural-only reads, unknown freshness, bounded scopes, deterministic one-action selection, untrusted input, and strict zero-write behavior.
