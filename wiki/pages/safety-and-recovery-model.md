---
title: Safety model and recovery boundary
type: decision
sources: [S001]
updated: 2026-09-10
---

# Safety model and recovery boundary

Grip protects mutations with temporary staging, lock-held revalidation, post-publication verification, and atomic baseline publication. A failed operation reports its current result; Grip does not retain replaced payloads or create recovery archives. (S001)

Use Git for history and recovery. Grip is a deployment and synchronization mechanism, not a backup or version-control system. (S001)
