---
description: "Create the project wiki skeleton (schema, index, source registry) — the three-layer LLM Wiki structure"
---

# Initialize LLM Wiki

Set up the persistent, LLM-maintained project wiki. After this command, the
project has a compounding knowledge layer — ordinary markdown, committed with
the repo — that `ingest` grows, `query` answers from, and `lint` keeps honest.

This implements the three-layer structure of Karpathy's LLM Wiki pattern:
**raw sources** (immutable — your repo, spec artifacts, external documents)
→ **the wiki** (LLM-written, cross-referenced pages) → **the schema** (a
configuration document defining the wiki's structure and maintenance rules).

## User Input

```text
$ARGUMENTS
```

If provided, treat `$ARGUMENTS` as the **wiki scope** — one or two sentences
describing what this wiki exists to accumulate (e.g. "Everything we learn
about the payments domain and our integration constraints"). `key=value`
tokens (e.g. `directory=docs/wiki`) are configuration overrides.

## Steps

### 1. Resolve configuration

1. Start with the extension defaults (`wiki/` directory, 12 pages max per ingest, 600 words per page, citations required, 8-page query slice, 4000 context tokens, 90-day staleness, `index-and-links` auto-fix, and the default page types).
2. Overlay `.specify/extensions/wiki/wiki-config.yml` if it exists, then `SPECKIT_WIKI_*` environment variable overrides, then `key=value` overrides from `$ARGUMENTS`. Resolve each setting independently; a missing higher-precedence value falls through to the next source.
3. Validate the effective settings before reading or writing wiki state. The directory must be a non-empty repository-relative path; page, word, query, and context limits must be positive whole numbers; the staleness threshold must be a non-negative whole number; citation policy must be boolean; `lint.auto_fix` must be `none` or `index-and-links`; page types must be a non-empty list of unique names. Reject unknown override keys and invalid values with the setting name and accepted form.
4. Resolve the repository root and configured directory to normalized absolute paths, resolving any existing symbolic-link components. `WIKI_DIR` must be the repository root itself or a descendant of it. If the candidate escapes that boundary, stop before any wiki read or write, identify the rejected directory, and report that nothing changed.
5. Use the validated canonical candidate as `WIKI_DIR` for every later check and write.

### 2. Idempotency check

If `WIKI_DIR/SCHEMA.md` already exists, **do not overwrite, regenerate, or repair any wiki artifact**. If the user supplied a new scope sentence in `$ARGUMENTS`, append that text verbatim to the `## Scope` section as exactly one additional numbered item; otherwise write nothing. Report that the wiki is already initialized, behave exactly like `/speckit.wiki.status`, and stop. Treat the schema as the sole initialization sentinel: missing companion artifacts are surfaced by status or lint and are never silently reconstructed here.

### 3. Create the wiki skeleton

Before writing, verify that `WIKI_DIR` can be created or written and that none of the three required artifact paths already exists without the schema sentinel. If partial foundation artifacts would collide, stop and preserve them for explicit user recovery. Render all three artifact contents first, then create `WIKI_DIR/` and write each file below exactly once. If creation fails partway through, remove only the artifacts created by this invocation and report the failure; never alter content that existed before the invocation.

`SCHEMA.md` — the schema layer (user-editable; every command obeys it):

```markdown
# Wiki Schema

This file defines how this wiki is structured and maintained. Edit it to
change the rules; `/speckit.wiki.*` commands read it before writing anything.

## Scope
1. <scope from $ARGUMENTS, or "(not set — pass a sentence to /speckit.wiki.init)">

## Page types
| Type | Holds | Example title |
|------|-------|---------------|
| concept | domain ideas and definitions that outlive any feature | "Idempotency keys" |
| decision | choices made, alternatives rejected, and why | "Why SQLite over Postgres" |
| component | how a part of the system actually works | "Auth middleware" |
| reference | distilled external facts (APIs, papers, vendor limits) | "Stripe rate limits" |
| howto | procedures that took effort to figure out | "Local TLS setup" |

## Rules
- Pages live in `pages/`, one topic per page, kebab-case filenames.
- Every synthesized claim cites a source ID from `sources.md` (e.g. `(S003)`).
- Pages cross-reference with relative links: `[title](./other-page.md)`.
- A page that outgrows <ingest.page_max_words> words is split, and both halves
  link to each other.
- Conflicting claims are kept side by side under a `> ⚠ conflict:` marker
  until resolved — never silently overwritten.
- Frontmatter per page: `title`, `type`, `sources`, `updated` (ISO date).

## Maintenance workflows
- Grow: `/speckit.wiki.ingest <source>` — the only way knowledge enters.
- Use: `/speckit.wiki.query <question>` — answers come from pages, cited.
- Check: `/speckit.wiki.lint` — drift, orphans, contradictions, staleness.
- Resume: `/speckit.wiki.status` — the session-resume entry point.
```

`INDEX.md` — the page directory (maintained by `ingest` and `lint`):

```markdown
# Wiki Index

One line per page, grouped by type. Maintained by `/speckit.wiki.ingest` and
regenerated by `/speckit.wiki.lint` — hand edits will be preserved only in
page files, not here.

_No pages yet. Run `/speckit.wiki.ingest` to add the first source._
```

`sources.md` — the source registry (raw sources are never copied, only pointed to):

```markdown
# Source Registry

Append-only IDs (S001, S002…). Dedup key: normalized path or URL.
Sources are immutable inputs — the wiki never edits them.

| ID | Source | Type | First ingested | Last ingested | Pages touched |
|----|--------|------|----------------|---------------|---------------|
```

Replace every `<...>` placeholder with the resolved configuration value. Do not create `pages/` yet — the first `ingest` creates it. Do not synthesize project claims or pre-populate the index or source registry.

### 4. Report

Output a short confirmation:

- The resolved `WIKI_DIR` and the three files created.
- The scope (or a note that none is set).
- The active caps (pages per ingest, words per page, query slice).
- Next step: `/speckit.wiki.ingest` to compound the first source — a feature's
  `research.md`, a file, or a URL; `/speckit.wiki.status` at any time —
  including from a brand-new session — to resume from the files.

## Guardrails

- Never delete or truncate an existing wiki; this command only creates.
- Reject targets outside the project root before checking or creating wiki artifacts.
- Never overwrite partial foundation artifacts when the schema sentinel is absent; report the collision for user-directed recovery.
- Do not pre-populate pages from general knowledge — pages exist only when a
  registered source supports them.
- Keep this command's own context usage minimal: do not scan the repo here;
  `ingest` reads what it needs when it needs it.
