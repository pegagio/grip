---
title: Safety model and recovery boundary
type: decision
sources: [S001, S027]
updated: 2026-09-17
---

# Safety model and recovery boundary

Grip protects mutations with temporary staging, lock-held revalidation, post-publication verification, and atomic baseline publication. A failed operation reports its current result; Grip does not retain replaced payloads or create recovery archives. (S001)

Use Git for history and recovery. Grip is a deployment and synchronization mechanism, not a backup or version-control system. (S001)

For aggregate forced push, each fully verified entry is an accepted unit: its baseline can be published before a later entry fails, while the aggregate outcome remains failed and every failed or unattempted entry remains unaccepted. (S027)
