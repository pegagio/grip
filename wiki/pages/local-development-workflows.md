---
title: Local development workflows
type: howto
sources: [S004, S024]
updated: 2026-09-17
---

# Local development workflows

Install the pinned toolchain with `mise trust` followed by `mise install`. A direct release build is available through `mise exec -- cargo build --release`. (S004)

The preferred project entry points are `mise run build` for a debug build, `mise run build --release` for a release build, `mise run test` for the default test suite, and `mise run clean` to remove Cargo build artifacts before a clean rebuild. (S004)

Use `mise run validate` for the complete verification sequence: formatting, Clippy with warnings denied, default tests, and a release build. Use `mise run performance` to build in release mode and execute the otherwise ignored 100-run performance acceptance harness. (S004)

On a clean `master` checkout on macOS Apple Silicon, `mise run release` prepares the versioned archive and checksum under `dist/` and creates an annotated local version tag. Inspect the checksum, archive, and tag before deliberately pushing the tag and creating the GitHub Release; the task itself does not publish externally. (S004)

The release contract suite runs the workflow in disposable Git repositories, including validation, packaging, checksum, tag, platform, collision, and external-publication boundaries. [Local artifact release preparation](./local-artifact-release-preparation.md) records the implementation-specific safety boundary. (S024)

[Performance qualification](./performance-qualification.md) records the representative workload results and their safety boundaries.

## Related pages

- [Implementation and delivery direction](./implementation-and-delivery-direction.md)
- [Command-line and path selection](./command-line-and-path-selection.md)
- [Metadata and filesystem contract](./metadata-and-filesystem-contract.md)
- [Local artifact release preparation](./local-artifact-release-preparation.md)
- [Performance qualification](./performance-qualification.md)
