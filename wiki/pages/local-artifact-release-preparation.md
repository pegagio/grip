---
title: Local artifact release preparation
type: component
sources: [S024]
updated: 2026-09-15
---

# Local artifact release preparation

`mise run release` is a maintainer-only local workflow for an attached, clean `master` checkout on Darwin/arm64. It derives the package version, requires the release validation and performance gates, and rejects an existing version tag, archive, or checksum before preparation begins. (S024)

The workflow creates the executable, README, and license archive in an attempt-owned staging directory, generates and verifies its SHA-256 checksum, and uses no-clobber final publication. It creates an annotated local `v<version>` tag only after the artifacts are ready and verifies that the tag identifies the captured release commit. (S024)

Failure cleanup removes only outputs owned by the failed attempt. The contract suite uses disposable repositories to prove successful archive extraction, invalid-candidate rejection, collision preservation, failure cleanup, and the absence of Git push, GitHub API, upload, or package-registry publication. (S024)

## Related pages

- [Local development workflows](./local-development-workflows.md)
