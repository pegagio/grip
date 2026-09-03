# Data Model: Wiki Foundation

The feature stores ordinary Markdown artifacts rather than application records. These entities describe the durable contract and validation rules for those artifacts.

## Wiki Configuration

Represents the effective initialization and maintenance policy.

| Field | Meaning | Validation |
|---|---|---|
| `state.directory` | Wiki location relative to the project root | Must resolve to the project root or a descendant; must not escape it |
| `ingest.max_pages_per_ingest` | Maximum pages a later ingestion may touch | Positive whole number |
| `ingest.page_max_words` | Page-size split threshold | Positive whole number |
| `ingest.require_citations` | Whether synthesized claims require source IDs | Boolean |
| `query.pages_slice` | Maximum pages a later query may load | Positive whole number |
| `query.context_tokens` | Soft rendered-context limit | Positive whole number |
| `lint.stale_after_days` | Age threshold used by later health checks | Non-negative whole number |
| `lint.auto_fix` | Allowed mechanical repair policy | `none` or `index-and-links` |
| `schema.page_types` | Allowed wiki page categories | Non-empty unique names |

### Resolution

Each field resolves independently using this order: invocation override, environment override, saved project value, extension default. Missing higher-priority values fall through.

## Wiki Schema

The authoritative, user-editable initialization sentinel and wiki policy document.

| Field or section | Cardinality | Rule |
|---|---:|---|
| Scope | One or more numbered items | Initial item is user input or an explicit unset prompt; later supplied scopes append |
| Page types | One table | Defines concept, decision, component, reference, and how-to categories by default |
| Rules | One list | Covers location, naming, citations, links, splitting, conflicts, and page metadata |
| Maintenance workflows | One list | Identifies grow, use, check, and resume operations |

### Lifecycle

```text
absent --initialize--> created
created --initialize without scope--> unchanged
created --initialize with scope--> scope appended
```

The schema is never replaced by initialization.

## Page Index

The page directory consumed by later wiki operations.

- Starts with zero page entries.
- Contains guidance to ingest the first source.
- Is created only when the schema is absent.

## Source Registry

The catalog of immutable source references and their ingestion history.

| Field | Initial state |
|---|---|
| ID | No rows |
| Source | No rows |
| Type | No rows |
| First ingested | No rows |
| Last ingested | No rows |
| Pages touched | No rows |

## Scope Statement

A user-authored sentence describing what the wiki exists to accumulate.

- Preserved verbatim.
- Stored as a numbered item under the schema’s Scope section.
- May be appended during repeated initialization.
- Is not deduplicated or rewritten by initialization.

## Relationships

- Wiki Configuration selects the directory containing one Wiki Schema, one Page Index, and one Source Registry.
- Wiki Schema governs the later pages referenced by Page Index and sources referenced by Source Registry.
- Scope Statements belong to Wiki Schema and accumulate without replacing earlier statements.
