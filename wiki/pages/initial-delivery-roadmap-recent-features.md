---
title: Initial delivery roadmap recent features
type: reference
sources: [S003]
updated: 2026-09-17
---

# Initial delivery roadmap recent features

This continuation records Features 021 through 026 from the current roadmap ledger. [Initial delivery roadmap future features](./initial-delivery-roadmap-future-features.md) continues with Features 027 and 028. (S003)

21. **021 — Large-File Operation Performance (`verified`):** defines isolated local macOS ARM64 release workloads with differing 19 MiB regular endpoints for `grip add` and JSON `grip status`, then removes demonstrated same-pass duplicate observation while retaining full content and metadata evidence, independent stable-observation passes, and fenced add reinspection. A 100-sample p95 gate requires both operations to complete within one second; the recorded final p95 values were 573.669375 ms for `grip add` and 184.788375 ms for `grip status`. (S003)
22. **022 — Force Mapping Replacement (`verified`):** permits `grip add --force` to replace exactly one active file mapping at the same resolved destination, preserves other ownership and topology rejections, and keeps add payload-nonmutating. Exact removal retires the corresponding ownership and accepted comparison evidence so a later ordinary add may reuse the destination. Its no-finding debrief verified the implementation against the roadmap. (S003)
23. **023 — Forced Missing-Peer Restoration (`verified`):** lets exact forced push or pull restore a missing peer from the selected present winner, while preserving force's exact-entry boundary and existing absent-winner deletion behavior. Its pre-implementation brief and post-implementation debrief returned `PROCEED` with no findings. (S003)
24. **024 — Local Artifact Release Automation (`verified`):** provides one local release-preparation task that validates a clean `master` candidate, prepares the macOS Apple Silicon archive and checksum, and creates an annotated local tag without pushing or publishing externally. Its debrief returned `PROCEED` with no findings. (S003)
25. **025 — Contained-Source Tree Mappings (`verified`):** permits a tree source to reside strictly beneath its destination, including a repository `home/` tree mapped to `~/`, while limiting destination inspection to current source members and retained accepted identities. Equal roots, destination-beneath-source roots, contained files, cross-mapping overlap, and recursive managed members remain blocked. Its complete-worktree debrief returned `PROCEED` with no findings. (S003)
26. **026 — Exact Destination Symlink Replacement (`verified`):** lets nonmutating `grip add` record mappings whose exact managed destination leaves are symlinks, reports each unresolved link precisely, and lets exact-entry source-winning `push --force` replace only the verified link object without following its target. Its no-finding debrief verified the implementation. Source symlinks, symlink ancestors, aggregate force, pull-side replacement, and mutation during `add` remain outside scope. (S003)

Features 021 through 026 establish measured large-file responsiveness, exact mapping replacement, missing-peer restoration, local release preparation, safe contained-source tree mappings, and explicit exact destination-leaf symlink replacement while retaining no-follow filesystem safety. (S003)

## Related pages

- [Initial delivery roadmap](./initial-delivery-roadmap.md)
- [Initial delivery roadmap later features](./initial-delivery-roadmap-later-features.md)
- [Initial delivery roadmap future features](./initial-delivery-roadmap-future-features.md)
- [Contained-source tree mappings](./contained-source-tree-mappings.md)
