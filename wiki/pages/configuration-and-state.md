---
title: Configuration and state
type: component
sources: [S001, S004, S015, S017, S028]
updated: 2026-09-16
---

# Configuration and state

Portable mapping declarations live in the Grip project. Mutable current evidence lives in the ignored `.grip/state/` directory. State contains only baselines for active managed membership and is pruned when mappings are removed or ignore policy removes entries. (S001)

The descriptor retains the exact accepted destination declaration, including absolute, lexically non-normalized home-relative, and project-relative spelling. Runtime validation and state binding use the separately resolved endpoint rather than rewriting that declaration. (S001) (S017)

Registry publication and State V4 identity carry the original declaration, which lets project-relative destinations, absolute destinations, and non-normalized `~/` spellings survive reload, removal, and later operations. See [Destination path forms](./destination-path-forms.md). (S015) (S017)

The state is deployment evidence, not an archive. It contains no retained payload history and must not revive a baseline when a mapping or ignored entry is later reintroduced. (S001)

External-diff preference is separate from mappings and mutable state. A selected project may choose a named diff tool in `.grip/config.toml`; its matching fields override the optional machine-wide `~/.grip/config.toml` profile, including replacement of the entire `args` array. A non-empty `GRIP_EXTERNAL_DIFF` takes precedence as an executable-only override and receives only the two selected endpoints. (S001) (S004)

[External diff program](./external-diff-program.md) records the selected-comparison handoff, endpoint revalidation, literal invocation, and child completion boundary. (S028)
