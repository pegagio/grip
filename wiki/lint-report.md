# Wiki Lint Report — 2026-09-15

The full wiki pass found no broken links, orphan pages, unknown source IDs, missing required frontmatter, unresolved conflict markers, or contradictory claims. The configured `index-and-links` auto-fix regenerated `INDEX.md` from page titles and types, restoring canonical labels and component ordering. Semantic findings were reported without changing the affected claims or freshness dates.

| # | Check | Severity | Page | Finding | Suggested fix |
|---|---|---|---|---|---|
| 1 | stale | semantic | baseline-classification-and-status.md | Page updated 2026-09-13, but cited source S001 was re-ingested 2026-09-15. | Review the product-definition-backed claims and refresh the page only where the source changes them. |
| 2 | stale | semantic | baseline-classification-and-status.md | Page updated 2026-09-13, but cited source S004 was re-ingested 2026-09-15. | Review the README-backed claims and refresh the page only where the source changes them. |
| 3 | stale | semantic | command-line-and-path-selection.md | Page updated 2026-09-13, but cited source S004 was re-ingested 2026-09-15. | Review the README-backed claims and refresh the page only where the source changes them. |
| 4 | stale | semantic | configuration-and-state.md | Page updated 2026-09-11, but cited source S001 was re-ingested 2026-09-15. | Review the product-definition-backed claims and refresh the page only where the source changes them. |
| 5 | stale | semantic | deletion-retirement-and-recovery.md | Page updated 2026-09-11, but cited source S001 was re-ingested 2026-09-15. | Review the product-definition-backed claims and refresh the page only where the source changes them. |
| 6 | stale | semantic | filesystem-support-boundaries.md | Page updated 2026-09-14, but cited source S001 was re-ingested 2026-09-15. | Review the product-definition-backed claims and refresh the page only where the source changes them. |
| 7 | stale | semantic | filesystem-support-boundaries.md | Page updated 2026-09-14, but cited source S004 was re-ingested 2026-09-15. | Review the README-backed claims and refresh the page only where the source changes them. |
| 8 | stale | semantic | grip-product-model.md | Page updated 2026-09-11, but cited source S001 was re-ingested 2026-09-15. | Review the product-definition-backed claims and refresh the page only where the source changes them. |
| 9 | stale | semantic | implementation-and-delivery-direction.md | Page updated 2026-09-13, but cited source S001 was re-ingested 2026-09-15. | Review the product-definition-backed claims and refresh the page only where the source changes them. |
| 10 | stale | semantic | implementation-and-delivery-direction.md | Page updated 2026-09-13, but cited source S003 was re-ingested 2026-09-15. | Review the roadmap-backed claims and refresh the page only where the source changes them. |
| 11 | stale | semantic | implementation-and-delivery-direction.md | Page updated 2026-09-13, but cited source S004 was re-ingested 2026-09-15. | Review the README-backed claims and refresh the page only where the source changes them. |
| 12 | stale | semantic | mapping-addition-and-initial-baselines.md | Page updated 2026-09-14, but cited source S004 was re-ingested 2026-09-15. | Review the README-backed claims and refresh the page only where the source changes them. |
| 13 | stale | semantic | roadmap-open-questions.md | Page updated 2026-09-14, but cited source S003 was re-ingested 2026-09-15. | Review the roadmap-backed claims and refresh the page only where the source changes them. |
| 14 | stale | semantic | safety-and-recovery-model.md | Page updated 2026-09-11, but cited source S001 was re-ingested 2026-09-15. | Review the product-definition-backed claims and refresh the page only where the source changes them. |
| 15 | stale | semantic | synchronization-and-conflicts.md | Page updated 2026-09-14, but cited source S001 was re-ingested 2026-09-15. | Review the product-definition-backed claims and refresh the page only where the source changes them. |
| 16 | stale | semantic | synchronization-and-conflicts.md | Page updated 2026-09-14, but cited source S004 was re-ingested 2026-09-15. | Review the README-backed claims and refresh the page only where the source changes them. |
| 17 | page-size | semantic | local-development-workflows.md | Page exceeds the configured 600-word split threshold. | Split the page by durable workflow topic and link both resulting pages to each other. |

## Summary

- **Mechanical fixes applied**: 1 regenerated index covering canonical titles and deterministic type-group ordering.
- **Semantic findings**: 17 total: 16 source-freshness reviews and 1 page-size split.
- **Blocking integrity findings**: 0.
