---
title: Cited query evidence pipeline
type: component
sources:
  - S003
updated: 2026-08-26
---

# Cited query evidence pipeline

A wiki query is a bounded evidence pipeline that produces one response from selected, provenance-valid page claims without changing persisted state. (S003)

## Selection

The question is decomposed into material parts. Index entries become candidate pages ranked by topical relevance and intent-matching page type, but index metadata is not evidence and page bodies are not read until selected. (S003)

Selection never exceeds `query.pages_slice` and is narrowed until rendered content also fits `query.context_tokens`. Only selected page bodies are read. (S003)

## Provenance

A selected claim becomes evidence only when each relied-on source ID exists in the registry. Query reads only the registry entries needed by selected claims; it neither trusts unknown IDs nor reopens underlying sources to re-prove their content. (S003)

Claims with invalid provenance are excluded and reported as gaps. Equivalent supported claims are stated once while retaining distinct supporting pages and source IDs. (S003)

## Result

Every material answer statement names a relied-on page and registered source ID. The result lists relied-on pages, exact gaps, concrete ingestion suggestions, unresolved conflicts, and exactly one [coverage verdict](./coverage-verdicts.md). (S003)

With no question, query returns a one-screen overview from structural metadata without loading every page body. (S003)

The pipeline is invoked through the [wiki command lifecycle](./wiki-command-lifecycle.md), constrained by [bounded and auditable maintenance](./bounded-and-auditable-maintenance.md), and protected by the [query trust boundary](./query-trust-boundary.md). (S003)
