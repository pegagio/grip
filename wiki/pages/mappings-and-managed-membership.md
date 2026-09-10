---
title: Mappings and managed membership
type: concept
sources: [S001]
updated: 2026-09-10
---

# Mappings and managed membership

`grip add SOURCE DESTINATION` declares a file or tree mapping without copying endpoints. At least one endpoint must exist and existing endpoints must have compatible kinds. Equivalent endpoints establish a baseline; differing or one-sided endpoints remain unbaselined. (S001)

`grip list [SOURCE]` reads declarations. `grip remove SOURCE` removes the declaration and its baseline evidence without changing either endpoint. Re-adding a mapping begins new membership and never restores a previous baseline. (S001)

Tree membership comes from source-side discovery subject to `.gripignore`. Operators edit `.gripignore` directly; Grip provides no `ignore` command. (S001)
