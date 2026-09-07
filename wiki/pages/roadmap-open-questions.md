---
title: Roadmap open questions
type: reference
sources: [S003]
updated: 2026-09-07
---

# Roadmap open questions

The roadmap retains thirteen questions for the specification that owns each decision; they do not change the approved feature order or initial-product boundary. (S003)

- **001:** confirm package and executable identity, choose registry and state serialization, and establish stable machine-output and exit-code contracts. (S003)
- **002 (resolved):** the `mapping add`, `list`, `show`, and `remove` family defines lifecycle syntax; tracking records intent only; and complete-registry validation conservatively rejects duplicate, overlapping, equal, nested, and cross-recursive ownership. (S003)
- **003 (resolved for discovery):** `.gripignore` follows the feature's enumerated Gitignore-compatible contract and remains policy-only; empty directories enter discovered membership; symbolic links are unsupported and never followed. Retirement, independent directory metadata, and any later explicit link-object support remain assigned to Features 008 and 009. (S003)
- **004 (resolved for initial classification):** the first metadata set compares regular-file content and permission mode, represents unavailable evidence explicitly, and keeps symbolic links unsupported; final identity, time, directory-metadata, link-object, and cross-filesystem capability behavior remains assigned to Feature 009. (S003)
- **005 (resolved):** an operational failure stops execution at the first failed action, preserves completed effects and recovery evidence, leaves later actions unattempted, and never publishes a partial baseline. (S003)
- **005 and 008 (resolved):** Feature 005 defines private recovery creation, preservation, binding, and reporting for replacement actions. Feature 008 completes the lifecycle with inspection, exact compatible restoration, and explicitly confirmed cleanup while retaining immutable tombstone metadata. (S003)
- **008 (resolved):** newly ignored, explicitly untracked, and converged-deletion records remain pending until an explicit path-selected or `--all` retirement removes their accepted membership evidence without changing payloads. Differing surviving copies require a reviewed `--force` authorization. (S003)
- **009:** settle case-sensitivity and Unicode-normalization behavior as part of final filesystem compatibility. (S003)
- **004 (resolved for automation):** `status`, `check`, `diff`, and baseline acceptance expose stable structured results and distinguish successful inspection, attention-required state, and operational failure. (S003)
- **Governance:** name the integration branch whose acceptance freezes a feature directory under merge-bounded persistence. (S003)

Each feature must clarify its assigned questions without silently expanding scope. Performance measurements precede new caches, indexing, parallelism, or coordination complexity, and filesystem behavior is tested in isolated temporary roots. (S003)

## Related pages

- [Initial delivery roadmap](./initial-delivery-roadmap.md)
