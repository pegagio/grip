# Data Model: Knowledge Ingestion

## Source Record

| Field | Rule |
|---|---|
| ID | Sequential immutable `S` identifier |
| Identity | Normalized project-relative path or normalized URL; unique |
| Type | `feature-artifact`, `file`, `directory`, or `url` |
| First ingested | Set once |
| Last ingested | Updated after each successful run |
| Pages touched | Replaced with the successful run’s affected page filenames |

## Knowledge Item

| Field | Rule |
|---|---|
| Topic | Maps to one indexed or new page |
| Claim | Concise durable statement, never bulk source text |
| Kind | Decision, alternative, constraint, concept, component behavior, gotcha, or verified fact |
| Source ID | Exactly one registered supporting identity for the new or changed claim |
| Priority | Used to select within the page cap |

## Wiki Page

Required metadata: title, allowed type, unique source IDs, and updated date. Claims carry citations. New pages must participate in reciprocal topic links and remain within the word threshold after optional splitting.

## Conflict

Contains the incompatible claims, each source ID, and an unresolved marker. Ingestion creates or extends conflicts but never resolves them.

## Deferred Item

Contains the topic, concise reason for deferral, and source ID. It is reported but not persisted as a wiki claim.

## Ingestion Report

Contains source identity and ID, initialization notice if applicable, created pages, updated pages, exclusions, conflicts, deferred items, and one next action.

## Transaction Lifecycle

```text
input -> validate -> normalize -> read once -> extract -> select -> render -> validate changes -> commit pages/registry/index -> report
             \----------------------------- failure ------------------------------> report unchanged state
```
