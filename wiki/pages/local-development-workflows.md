---
title: Local development workflows
type: howto
sources: [S004]
updated: 2026-09-03
---

# Local development workflows

Install the pinned toolchain with `mise trust` followed by `mise install`. A direct release build is available through `mise exec -- cargo build --release`. (S004)

The preferred project entry points are `mise run build` for a development build, `mise run test` for the default test suite, and `mise run clean` to remove Cargo build artifacts before a clean rebuild. (S004)

Use `mise run validate` for the complete verification sequence: formatting, Clippy with warnings denied, default tests, and a release build. Use `mise run performance` to build in release mode and execute the otherwise ignored 100-run performance acceptance harness. (S004)

## Related pages

- [Implementation and delivery direction](./implementation-and-delivery-direction.md)
- [Command-line and path selection](./command-line-and-path-selection.md)
