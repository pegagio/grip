---
title: Mappings and managed membership
type: concept
sources: [S001, S015]
updated: 2026-09-11
---

# Mappings and managed membership

`grip add SOURCE DESTINATION` declares a file or tree mapping without copying endpoints. At least one endpoint must exist and existing endpoints must have compatible kinds. Equivalent endpoints establish a baseline; differing or one-sided endpoints remain unbaselined. (S001)

A destination declaration is either an absolute path, `~`, or a `~/`-prefixed path. Grip preserves the accepted declaration spelling and rejects other relative or expansion-like forms before publishing a mapping. (S001)

Ownership and topology validation apply after destination resolution, so distinct declarations that resolve to overlapping namespaces remain invalid. [Destination path forms](./destination-path-forms.md) records this declaration-versus-endpoint distinction. (S015)

`grip list [SOURCE]` reads declarations. `grip remove SOURCE` removes the declaration and its baseline evidence without changing either endpoint. Re-adding a mapping begins new membership and never restores a previous baseline. (S001)

Tree membership comes from source-side discovery subject to `.gripignore`. Operators edit `.gripignore` directly; Grip provides no `ignore` command. (S001)
