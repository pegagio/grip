---
title: Baselines, classification, and status
type: component
sources: [S001]
updated: 2026-09-10
---

# Baselines, classification, and status

A baseline is current accepted evidence for an actively managed entry, not stored file content or recovery history. `grip status` classifies source, destination, and baseline without mutation. `status -e` returns attention when work is needed. `grip diff` is likewise read-only. (S001)

Unambiguous one-sided changes can synchronize normally. Divergence, initial collision, unbaselined differences, and one-sided absence block ordinary operations. Removing a mapping or ignoring an entry prunes its baseline; later reintroduction is new membership. (S001)
