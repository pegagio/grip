# LLM Wiki — a Spec Kit extension

**An LLM-maintained, compounding project wiki for spec-driven development**:
source ingestion with per-claim citations, questions answered from the wiki
(never from vibes), and a lint pass that keeps the knowledge base honest.

Based on **Andrej Karpathy's "LLM Wiki"**
([gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)) —
a pattern for knowledge bases where the LLM actively *maintains* a persistent
wiki instead of rediscovering knowledge from raw documents on every question.
This community extension adapts that pattern to
[Spec Kit](https://github.com/github/spec-kit) workflows. It is not affiliated
with Karpathy or GitHub.

> **LLM Wiki vs. [OpenWiki](https://github.com/langchain-ai/openwiki)** —
> complementary, not competing. OpenWiki *generates code documentation from
> your repository*: what the code is, derivable from the code. LLM Wiki
> *accumulates the knowledge that is not in the code*: decisions and their
> rejected alternatives, constraints that will bite again, verified external
> facts, and what each feature's research taught. Run both.

## Why

Spec Kit produces knowledge constantly — `research.md` findings, plan
decisions, mid-implementation discoveries — and then buries it in per-feature
directories. Feature 007 re-derives what feature 003 already learned; the
"why" behind a decision lives in an expired conversation; asking "what do we
know about X?" means re-reading everything or trusting memory.

Karpathy's diagnosis: retrieval-only setups have **no accumulation** — the
LLM rediscovers knowledge from scratch on every question. His fix is a
persistent wiki the LLM maintains under explicit structure: the human curates
sources and asks questions; the LLM does the bookkeeping (summarizing,
cross-referencing, consistency-keeping). The wiki is a compounding artifact:
every ingest makes every future answer cheaper and better.

| LLM Wiki (gist) | This extension |
|---|---|
| Raw sources — immutable documents | `wiki/sources.md` registry: feature artifacts, files, URLs — pointed to, never copied |
| The wiki — LLM-written, cross-referenced pages | `wiki/pages/*.md` + `wiki/INDEX.md`, typed (concept / decision / component / reference / howto) |
| The schema — structure & workflow rules | `wiki/SCHEMA.md`, user-editable; every command obeys it |
| **Ingest** — read source, update 10–15 related pages | `/speckit.wiki.ingest` — capped pages per run, citation on every claim, conflicts kept visible |
| **Query** — answer with citations | `/speckit.wiki.query` — answers only from pages; gaps become concrete ingest suggestions |
| **Lint** — contradictions, orphans, stale claims, gaps | `/speckit.wiki.lint` — mechanical fixes applied, semantic issues reported with suggested edits |

## What you gain

- **Accumulation** — feature research compounds into project knowledge
  instead of dying in `specs/007-*/research.md`. The `after_plan` and
  `after_implement` hooks offer the ingest at exactly the moments knowledge
  is produced.
- **Cited answers** — `query` refuses to answer beyond what the pages
  support; every statement names its page and source. A wrong answer is
  traceable; a missing answer is an ingest away.
- **Honest maintenance** — `lint` finds contradictions, orphans, staleness,
  and uncited claims; it fixes only mechanical drift and reports the rest.
  The wiki degrades loudly, not silently.
- **Plain markdown in your repo** — diffable, PR-reviewable, shared by every
  agent and teammate; a new session resumes from `/speckit.wiki.status`, not
  from a lost context window.

## Installation

**Option 1 — by name, from the community catalog** (after the extension is
listed). Spec Kit treats the community catalog as discovery-only by default,
so allow installs from it once (per project, or per user via
`~/.specify/extension-catalogs.yml`):

```yaml
# .specify/extension-catalogs.yml
catalogs:
  - name: default
    url: https://raw.githubusercontent.com/github/spec-kit/main/extensions/catalog.json
    priority: 1
    install_allowed: true
  - name: community
    url: https://raw.githubusercontent.com/github/spec-kit/main/extensions/catalog.community.json
    priority: 2
    install_allowed: true
```

```bash
specify extension add wiki
```

**Option 2 — zero config, pinned version.** Install straight from a release URL:

```bash
specify extension add wiki --from https://github.com/formin/spec-kit-wiki/archive/refs/tags/v1.0.0.zip
```

> URL installs show an *Untrusted Source* warning and ask
> `Continue with installation? [y/N]` — answer `y` (in a non-interactive
> shell, pipe it: `echo y | specify extension add …`). Catalog installs skip
> this prompt.

**Option 3 — for development:**

```bash
git clone https://github.com/formin/spec-kit-wiki
specify extension add --dev ./spec-kit-wiki
```

Requires Spec Kit `>=0.2.0`. Works with any agent Spec Kit supports (Claude
Code, GitHub Copilot, Cursor, Gemini CLI, …) — commands are plain prompt
files; no external tools, MCP servers, or network access required.

## Commands at a glance

| Command | What it does | Touches disk |
|---|---|---|
| `/speckit.wiki.init [scope] [key=value…]` | Create `SCHEMA.md`, `INDEX.md`, `sources.md` | creates `wiki/` (never overwrites) |
| `/speckit.wiki.ingest [source]` | Register a source, fold its knowledge into ≤N pages with citations | pages, `INDEX.md`, `sources.md` |
| `/speckit.wiki.query [question]` | Answer from pages with citations; report coverage honestly | **read-only** |
| `/speckit.wiki.lint [scope]` | Contradictions, orphans, stale claims, broken links, index drift | `lint-report.md` (+ mechanical index/link fixes) |
| `/speckit.wiki.status [page-type \| full]` | Structural-only snapshot + one evidence-backed next action | **read-only** |

## Where it fits in the Spec Kit workflow

The wiki is the **cross-feature memory layer**. Core stages produce
knowledge; the wiki keeps it; later features and sessions consume it.

```text
/speckit.wiki.query "what do we already know about <domain>?"
        │                    ← before specify: reuse prior art instead of re-deriving
/speckit.specify ──▶ spec.md
        │
/speckit.plan ──▶ plan.md + research.md
        │  └─ hook after_plan → /speckit.wiki.ingest          (optional prompt)
        │       └─ the feature's verified research compounds into wiki pages
        │
/speckit.tasks ──▶ tasks.md
        │
/speckit.implement
        │  └─ hook after_implement → /speckit.wiki.ingest     (optional prompt)
        │       └─ what the build taught (gotchas, actual behavior) is kept
        │
  … periodically: /speckit.wiki.lint    · any time: /speckit.wiki.status
```

Pairs cleanly with the
[Research Harness](https://github.com/formin/spec-kit-harness) extension:
the harness makes one feature's research *verified and resumable*; the wiki
makes it *permanent and reusable across features*. `harness.report` writes
`research.md`; `wiki.ingest` compounds it.

## Usage

### 1. `/speckit.wiki.init` — once per project

```text
/speckit.wiki.init Everything we learn about the payments domain and our vendor constraints
```

Creates `wiki/SCHEMA.md` (the rules — edit freely; commands obey it), `wiki/INDEX.md`, and `wiki/sources.md`. Initialization never creates knowledge pages or synthesized claims. It is idempotent: re-running without a scope writes nothing, while a supplied scope is appended as one new numbered item instead of overwriting existing wiki content.

### 2. `/speckit.wiki.ingest` — whenever knowledge is produced

```text
/speckit.wiki.ingest                          # default: active feature's research.md + plan decisions
/speckit.wiki.ingest docs/postmortems/2026-05-outage.md
/speckit.wiki.ingest https://stripe.com/docs/rate-limits
```

Registers the source (`S007`), extracts what outlives the moment (decisions,
constraints, gotchas, verified facts), and updates at most
`max_pages_per_ingest` pages — every claim cited `(S007)`, conflicts kept
side by side under `> ⚠ conflict:` markers, new pages cross-linked so
nothing is orphaned.

Project files and directories are contained to the repository; a directory is one source identity and generated, binary, ignored, or inaccessible entries are skipped. URL access occurs only for a URL explicitly supplied to the command. All source content is treated as untrusted evidence: embedded instructions cannot change the workflow, invoke tools, or authorize reading another path or URL.

Ingestion validates and prepares the full bounded change before synchronizing pages, `sources.md`, and `INDEX.md`. Read, fetch, containment, or validation failure leaves those artifacts unchanged. Re-ingestion preserves the stable source ID and first-ingested date while refreshing supported claims and keeping disagreements visible.

### 3. `/speckit.wiki.query` — the payoff

```text
/speckit.wiki.query Why did we pick SQLite over Postgres, and does that still hold?
```

Loads the index, reads at most `pages_slice` relevant pages, and answers with
citations — closing with an honest coverage verdict: **Covered**, **Partial**
(with the exact gap and the ingest that would close it), or **Uncovered**.

Index metadata selects candidates but is not itself evidence. Query reads only selected pages and the source-registry entries needed to validate their citations; claims with unknown source IDs are reported as provenance gaps. The question and all wiki text are untrusted data, so embedded instructions cannot change limits, invoke tools, expand access, or weaken the read-only contract.

Covered means every material part of the question has valid evidence, Partial means only some do, and Uncovered means none do. Conflicts remain visible with every cited side. Query never repairs structural problems or changes project files.

### 4. `/speckit.wiki.lint` — regular maintenance

```text
/speckit.wiki.lint
```

Six checks (index drift, broken links, orphans, contradictions, staleness,
uncited claims). Mechanical drift is fixed in place (configurable); semantic
findings land in `wiki/lint-report.md` with suggested fixes — lint never
rewrites your prose.

The automatic repair allowlist is intentionally narrow: regenerate `INDEX.md` from valid page metadata and update the target of an unambiguous renamed-page link. Missing or ambiguous links, citations, claims, conflicts, source history, taxonomy, and all semantic findings remain report-only. Wiki text is untrusted data and cannot alter checks or repair policy.

Findings carry exact evidence and stable ordering. Lint prepares and validates the complete fix set before writing, and the report records what was actually applied. Each run ends with exactly one highest-value unresolved action, or states that no action is needed.

### 5. `/speckit.wiki.status` — resume, or decide what's next

```text
/speckit.wiki.status
→ 14 pages (5 decision · 4 concept · 3 component · 2 reference) · 9 sources
→ 1 unresolved conflict: payments-retries.md (S002 vs S007)
→ Recommendation: resolve the conflict in payments-retries.md between S002 and S007
```

Status is strictly read-only and reconstructs a one-screen default snapshot from the schema scope, index, source registry, optional lint report, and bounded active-feature path metadata. It never opens wiki page bodies, feature-artifact bodies, or original sources to fill gaps; missing dates and malformed metadata remain visibly `unknown` or `invalid`.

Pass one configured page type to filter the page slice, or `full` to expand every default slice by at most three times without expanding read authority. Every run selects exactly one next action from current evidence using stable priority and tie-breaking. Open a fresh session, run `status`, continue — the files are the memory.

## State files

| File | Role | Invariants |
|---|---|---|
| `wiki/SCHEMA.md` | The rules: page types, naming, linking, citation policy | user-editable; commands read it before writing |
| `wiki/INDEX.md` | Page directory, grouped by type | maintained by `ingest`, regenerated by `lint` |
| `wiki/sources.md` | Source registry | append-only IDs; dedup by path/URL; sources never copied |
| `wiki/pages/*.md` | The knowledge | one topic per page; every claim cited; conflicts marked, not erased |
| `wiki/lint-report.md` | Latest health check | overwritten per lint run; the only wholesale overwrite in the system |

## Patterns & recipes

**Query before specify** — start each feature with
`/speckit.wiki.query <the feature's domain>`; prior decisions and constraints
flow into the new spec instead of being re-derived (or contradicted).

**Ingest on the hooks** — accept the `after_plan` and `after_implement`
prompts and the wiki grows exactly when knowledge is produced, at zero extra
ceremony.

**Weekly lint** — a standing `/speckit.wiki.lint` keeps staleness and
conflicts from accumulating; the report is small and the fixes are usually
one re-ingest.

**Team workflow** — commit `wiki/` with the repo. PR reviewers see knowledge
changes next to code changes; a teammate's agent answers from the same pages
yours does.

**With the Research Harness** — `harness.report` → `research.md` →
`wiki.ingest`: verified, citation-carrying research compounds straight into
the permanent knowledge layer.

## Hooks

Both optional (you are prompted):

- `after_plan` → `speckit.wiki.ingest` — compound the feature's research and
  plan decisions into the wiki.
- `after_implement` → `speckit.wiki.ingest` — record what the implementation
  taught before it evaporates.

## Configuration

Copy `config-template.yml` to `.specify/extensions/wiki/wiki-config.yml` and
adjust:

```yaml
state:
  directory: wiki
ingest:
  max_pages_per_ingest: 12
  page_max_words: 600
  require_citations: true
query:
  pages_slice: 8
  context_tokens: 4000
lint:
  stale_after_days: 90
  auto_fix: index-and-links   # none | index-and-links
```

Precedence is resolved per setting from lowest to highest: extension defaults → config file → `SPECKIT_WIKI_*` environment variables → per-invocation `key=value` arguments. Numeric limits are validated before use, and `lint.auto_fix` accepts only `none` or `index-and-links`.

The configured state directory must remain inside the repository after path normalization and existing-symlink resolution. Initialization rejects escaping values such as `directory=../outside-wiki` before reading or writing wiki state.

## Troubleshooting & FAQ

**`init` says the wiki already exists.** By design — `SCHEMA.md` is the initialization sentinel and existing wiki content is never regenerated. New scope sentences are appended verbatim. If the wiki is partial or damaged, inspect it with `status` and `lint`; initialization leaves recovery decisions to you rather than replacing user-authored state.

**`query` refuses to answer something the model obviously knows.** Also by
design: the wiki's value is that its answers are *backed*. Ingest a source
for the fact and ask again — that is the accumulation loop working.

**Ingest wants to touch more pages than the cap allows.** It ingests the most
valuable items and lists the remainder as a suggested follow-up ingest. Raise
`max_pages_per_ingest` if this happens routinely.

**How is this different from the `memory-md` / `memory-loader` extensions?**
Those manage agent memory/context files. This is a *knowledge base with a
maintenance contract*: typed pages, per-claim citations to registered
sources, conflict markers, and a lint pass — Karpathy's wiki, not a scratchpad.

**Does it call any external services?** No. Commands are prompt files; the
only network access is fetching a URL you explicitly pass to `ingest`.

See [docs/concepts.md](docs/concepts.md) for the full design mapping and the
deliberate differences from the gist.

## License

[MIT](LICENSE) © 2026 formin

Credits: the LLM Wiki pattern by Andrej Karpathy
([gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f));
[Spec Kit](https://github.com/github/spec-kit) by GitHub;
[OpenWiki](https://github.com/langchain-ai/openwiki) by LangChain (the
complementary code-documentation side of the same idea).

## Evolving Specifications

This project uses the **Merge-Bounded Flow-Back Spec Persistence Model**. A feature's `spec.md`, `plan.md`, `tasks.md`, and implementation form one reviewable unit while that feature is in progress. Discoveries made during clarification, planning, task breakdown, or implementation may require changes to any of those artifacts. Those changes must flow back through the artifact set until it consistently describes the behavior and approach the project intends to merge.

The model has two phases:

1. **Before merge, use flow-back.** Feature artifacts are mutable working documents. Begin an edit where the new information is discovered, decide whether it affects intended behavior, technical approach, task breakdown, or implementation, and update every affected artifact. A change in code or `tasks.md` must not remain in conflict with `plan.md` or `spec.md`.
2. **After merge, use flow-forward.** The merged feature directory is a historical record and must not be substantively rewritten. A later change to intended behavior starts a new feature directory and references the earlier feature when the relationship matters. Git history records edits, but it is not a substitute for preserving the meaning of the artifacts that were reviewed and merged together.

The merge boundary acts as the feature's commit point. In this policy, merge means acceptance into the project's designated integration branch, such as `master` or `develop`, rather than an intermediate commit or branch synchronization. Before that point, incorporating accepted discoveries keeps the specification honest and avoids preserving mistakes merely because they appeared in an earlier workflow stage. After that point, freezing the artifacts prevents later work from rewriting the requirements, rationale, and implementation context under which the feature was accepted.

Contributors must follow these rules:

- Treat `spec.md`, `plan.md`, `tasks.md`, and the implementation as one coherent change set until merge.
- Capture a discovery first in the artifact closest to the work, then determine and apply its consequences throughout the artifact set.
- Update `spec.md` when intended behavior or requirements change, `plan.md` when the chosen technical approach changes, and `tasks.md` when the work needed to deliver the accepted behavior changes.
- Preserve meaningful rationale in the appropriate artifact or architecture decision record before replacing obsolete plan or task content.
- Do not let flow-back silently enlarge the feature. Split out a new feature when a discovery introduces independently valuable behavior, materially expands scope, or requires separate review and acceptance.
- Reconcile contradictions before continuing work that depends on the disputed direction. After tasking or consequential artifact reconciliation, run `/speckit.analyze` before starting or resuming implementation. After implementation, use `/speckit.converge` until no gaps remain. Before merge, execute the project's applicable validation and review the artifact and implementation diffs together.
- Treat a merged feature directory as semantically immutable. Editorial corrections may fix presentation without changing meaning; any later requirement or behavioral change requires a new feature.
- Make a later feature identify the earlier feature it amends, replaces, or depends on when that relationship is necessary to understand the change history. Do not edit the earlier feature to make it appear as though it always described the later behavior.

This model accepts that implementation produces useful knowledge while preserving a trustworthy history of accepted changes. It avoids the cascading historical rewrites of a living specification and the premature immutability of strict flow-forward development. Its principal risk is silent divergence during active work, so reconciliation and pre-merge review are required parts of the model rather than optional cleanup.
