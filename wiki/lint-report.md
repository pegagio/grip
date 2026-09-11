# Wiki Lint Report — 2026-09-11

| # | Check | Severity | Page | Finding | Suggested fix |
|---|-------|----------|------|---------|---------------|
| 1 | orphans | structural | `mapping-registry-publication.md` | No other wiki page contains a relative link to `./mapping-registry-publication.md`; the index does not count as an inbound link. | Decide which related page should introduce this topic, add a cited reciprocal link through `/speckit.wiki.ingest <source>`, then rerun `/speckit.wiki.lint`. |

Automatic fixes applied: none. `index-drift`, `links`, `contradictions`, `stale`, and `citations` had no findings.
