---
title: Destination adoption
type: component
sources: [S029]
updated: 2026-09-17
---

# Destination adoption

Destination adoption is an exact, explicit enrollment path for a destination-only regular file below an existing tree mapping. `grip pull -a DESTINATION` and `grip pull --adopt DESTINATION` resolve one destination-space path, copy it into the paired source location, verify equivalent supported managed state, and publish accepted evidence for the file and any necessary newly created source ancestor directories. (S029)

Ordinary `pull`, `status`, and `diff` do not discover or adopt destination-only tree members. Adoption requires the requested path to be strictly below a mapped tree destination, requires an existing source-side directory ancestor, may create only intervening source directories, and does not create a separate mapping or import siblings. (S029)

The operation is available with dry-run. It rejects an existing source counterpart, directories and non-regular destination nodes, paths outside a tree mapping, ownership or topology conflicts, unsupported metadata, stale evidence, and symlink ancestors. `--force` is not a conflict winner in this mode: it only overrides an applicable ignore-policy rejection for the one selected adoption. (S029)

Forced adoption never edits `.gripignore`. Its human and JSON results identify the policy-file placement and exact negation rules needed to keep the path in ordinary future discovery; until the operator makes that policy change, ordinary discovery may ignore the adopted member. [Mappings and managed membership](./mappings-and-managed-membership.md) records the source-defined ownership model, and [Command-line and path selection](./command-line-and-path-selection.md) records selector semantics. (S029)
