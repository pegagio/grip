---
title: Roadmap open questions
type: reference
sources: [S003]
updated: 2026-09-04
---

# Roadmap open questions

The roadmap retains thirteen questions for the specification that owns each decision; they do not change the approved feature order or initial-product boundary. (S003)

- **001:** confirm package and executable identity, choose registry and state serialization, and establish stable machine-output and exit-code contracts. (S003)
- **002 (resolved):** the `mapping add`, `list`, `show`, and `remove` family defines lifecycle syntax; tracking records intent only; and complete-registry validation conservatively rejects duplicate, overlapping, equal, nested, and cross-recursive ownership. (S003)
- **003:** select normative Gitignore behavior, decide whether `.gripignore` may ever be payload, and establish discovery treatment for empty directories and symbolic links. (S003)
- **004 and 009:** define initial and final metadata equality, modification-time semantics, user and group identity representation, directory metadata, symbolic-link disposition, and behavior across differing filesystem capabilities. (S003)
- **005:** confirm stop-after-first-failure behavior and specify reporting and recovery for remaining planned actions. (S003)
- **005 and 008:** define backup inspection and removal plus bounded recovery when registry or baseline state is missing, corrupt, unreadable, or inconsistent. (S003)
- **008:** define the exact retirement command and transition for newly ignored or explicitly untracked managed entries. (S003)
- **009:** settle case-sensitivity and Unicode-normalization behavior as part of final filesystem compatibility. (S003)
- **Governance:** name the integration branch whose acceptance freezes a feature directory under merge-bounded persistence. (S003)

Each feature must clarify its assigned questions without silently expanding scope. Performance measurements precede new caches, indexing, parallelism, or coordination complexity, and filesystem behavior is tested in isolated temporary roots. (S003)

## Related pages

- [Initial delivery roadmap](./initial-delivery-roadmap.md)
