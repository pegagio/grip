---
title: Roadmap open questions
type: reference
sources: [S003]
updated: 2026-09-14
---

# Roadmap open questions

The roadmap retains numbered questions for the specification that owns each decision; resolved questions remain recorded as provenance and do not change the completed initial-product boundary. (S003)

- **001:** confirm package and executable identity, choose registry and state serialization, and establish stable machine-output and exit-code contracts. (S003)
- **002 (resolved):** the historical nested mapping lifecycle is superseded by the flat `add`, `list [SOURCE]`, and `remove` contract. Tracking records intent only, and complete-registry validation conservatively rejects duplicate, overlapping, equal, nested, and cross-recursive ownership. (S003)
- **003 (resolved for discovery):** `.gripignore` follows the feature's enumerated Gitignore-compatible contract and remains policy-only; empty directories enter discovered membership; symbolic links are unsupported and never followed. Retirement, independent directory metadata, and any later explicit link-object support remain assigned to Features 008 and 009. (S003)
- **004 (resolved for initial classification):** the first metadata set compares regular-file content and permission mode, represents unavailable evidence explicitly, and keeps symbolic links unsupported; final identity, time, directory-metadata, link-object, and cross-filesystem capability behavior remains assigned to Feature 009. (S003)
- **005 (resolved):** an operational failure stops execution at the first failed action, preserves completed effects and recovery evidence, leaves later actions unattempted, and never publishes a partial baseline. (S003)
- **005 and 008 (resolved):** Feature 005 defines private recovery creation, preservation, binding, and reporting for replacement actions. Feature 008 completes the lifecycle with inspection, exact compatible restoration, and explicitly confirmed cleanup while retaining immutable tombstone metadata. (S003)
- **008 (resolved):** newly ignored, explicitly untracked, and converged-deletion records remain pending until an explicit path-selected or `--all` retirement removes their accepted membership evidence without changing payloads. Differing surviving copies require a reviewed `--force` authorization. (S003)
- **009 (resolved):** supported equality includes independent file and directory metadata; symbolic links remain unsupported non-followed objects; current macOS on APFS is the qualification boundary; exact path bytes are preserved; and endpoint-observed case and Unicode behavior blocks incompatible managed identities rather than normalizing or renaming them. (S003)
- **004 (resolved for automation):** `status`, `diff`, and baseline acceptance expose stable structured results and distinguish successful inspection, attention-required state, and operational failure; the former `check` interface is superseded. (S003)
- **Governance:** name the integration branch whose acceptance freezes a feature directory under merge-bounded persistence. (S003)
- **010, 012, and 014 (resolved):** project metadata is `.grip/config.toml` plus canonical `.grip/.gitignore`; sources are project-relative; destinations may be absolute, `~`, `~/`-prefixed, or relative to the selected project root and retain their accepted declaration text; local state lives beneath `.grip/state/` without a generated project ID; initialization is exactly idempotent; and discovery is exhaustive and fail-closed across candidate boundaries. (S003)
- **021 (resolved):** the representative large-file workload uses isolated local macOS ARM64 release builds with differing 19 MiB regular endpoints, exercising `grip add` and JSON `grip status` for 100 warm samples each. Both operations have a one-second p95 regression threshold; the final recorded p95 values were 573.669375 ms and 184.788375 ms respectively. (S003)
- **022 (resolved):** `grip add --force` is explicit confirmation to replace exactly one active equal-destination file mapping. It does not bypass tree, nested, source-overlap, or multi-mapping conflicts; exact removal retires that mapping's ownership and accepted comparison evidence before a later ordinary add. (S003)

Each feature must clarify its assigned questions without silently expanding scope. Performance measurements precede new caches, indexing, parallelism, or coordination complexity, and filesystem behavior is tested in isolated temporary roots. (S003)

## Related pages

- [Initial delivery roadmap](./initial-delivery-roadmap.md)
