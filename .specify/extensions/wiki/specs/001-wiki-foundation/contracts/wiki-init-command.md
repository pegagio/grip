# Command Contract: `/speckit.wiki.init`

## Purpose

Create the project wiki foundation once, or safely report an existing wiki while optionally appending a new scope statement.

## Input

```text
/speckit.wiki.init [scope sentence] [key=value ...]
```

- **Scope sentence**: Optional one- or two-sentence description of the knowledge the wiki should accumulate.
- **Overrides**: Optional configuration values with highest precedence for this invocation.
- **Environment overrides**: Optional values using the documented `SPECKIT_WIKI_*` naming convention.
- **Saved configuration**: Optional project configuration in the installed extension’s wiki configuration file.

## New-Wiki Outcome

When the resolved schema does not exist, one invocation creates:

```text
<wiki-directory>/
├── SCHEMA.md
├── INDEX.md
└── sources.md
```

The operation does not create `pages/` and does not synthesize knowledge.

The completion response includes:

- resolved wiki directory;
- three created artifacts;
- effective scope;
- active page, word, and query limits;
- the ingest and status next actions.

## Existing-Wiki Outcome

When `SCHEMA.md` already exists:

- no existing artifact is overwritten, truncated, regenerated, or deleted;
- a supplied scope statement is appended as a numbered Scope item;
- without a supplied scope, no file changes;
- the response follows the wiki-status behavior.

## Failure Contract

Initialization stops before writing when:

- the configured directory resolves outside the project root;
- the resolved target cannot be written;
- required configuration values are invalid or internally inconsistent.

The response identifies the rejected setting or target and confirms that no foundation artifact was changed.

## Configuration Precedence

For each value, the first defined source wins:

1. invocation `key=value` override;
2. `SPECKIT_WIKI_*` environment override;
3. saved project configuration;
4. extension default.

## Examples

```text
/speckit.wiki.init Everything we learn about payment processing and vendor constraints
/speckit.wiki.init directory=docs/wiki max_pages_per_ingest=6
```
