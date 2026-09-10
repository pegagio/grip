# Wiki Lint Report — 2026-09-10

| # | Check | Severity | Page | Finding | Suggested fix |
|---|-------|----------|------|---------|---------------|
| 1 | contradictions | semantic | mapping-registry-publication.md | The page contains the unresolved marker: `S005 records the historical <GRIP_HOME> Registry V1 layout; S013 replaces it as the current publication contract.` | Preserve the V1 account as labeled history or move it to a historical page, then remove the conflict marker only after the current Descriptor V2 account is unambiguous. |
| 2 | contradictions | semantic | roadmap-open-questions.md | The page says Feature 002's `mapping add`, `list`, `show`, and `remove` family defines lifecycle syntax, while the linked current roadmap says Feature 011 supersedes that syntax with flat `add`, `list`, and `remove` and excludes `show`. | Update the question history to identify the Feature 002 command family as superseded by Feature 011, preserving the S003 provenance. |
| 3 | stale | semantic | filesystem-support-boundaries.md | `updated: 2026-09-08` predates re-ingested source S001 on 2026-09-10. | Re-ingest or review `docs/product-definition.md` for this page and refresh only claims that remain current. |
| 4 | stale | semantic | local-development-workflows.md | `updated: 2026-09-09` predates re-ingested source S004 on 2026-09-10. | Re-ingest `README.md` and confirm the documented validation and performance workflow remains current. |
| 5 | stale | semantic | roadmap-open-questions.md | `updated: 2026-09-09` predates re-ingested source S003 on 2026-09-10. | Re-ingest `.specify/memory/roadmap.md` and reconcile superseded Feature 011 questions. |
| 6 | orphans | structural | mapping-registry-publication.md | No wiki page links to `mapping-registry-publication.md`; its only index entry does not count as an inbound relationship. | Add a meaningful reciprocal link from the current configuration or mapping page after resolving its historical/current boundary. |
