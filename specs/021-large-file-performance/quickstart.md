# Feature 021 Validation Guide

## Prerequisites

Use the repository's pinned toolchain and run from the repository root.

```bash
mise trust
mise install
```

## Release Performance Acceptance

Run the ignored release harness, which builds the release executable and measures the isolated large-file `add` and `status` workloads.

```bash
mise run performance
```

Expected result:

- The harness reports 100 warm-run p50, p95, and maximum distributions for both feature workflows and fails if either p95 exceeds one second.
- Both `grip add` and `grip status` report p95 at or below one second.
- Each sample preserves its established exit status and output, while complementary checks preserve mapping, payload, baseline, and classification semantics.

If the release binary is unavailable or the isolated temporary fixture cannot be prepared, the harness must fail with an explicit diagnostic; it must never fall back to a real Grip project or operator-managed files. If the workstation cannot provide stable representative measurements, record the environment and do not claim acceptance from that run.

## Correctness and Repository Validation

Run the normal repository gate after the focused performance acceptance.

```bash
mise run validate
git diff --check
```

Expected result:

- Formatting, lint, default tests, and a release build succeed.
- The working-tree diff has no whitespace errors.
- The performance harness is separate from the default test suite because it is an ignored representative-workstation acceptance test.

## Reference Artifacts

- [Design model](data-model.md) defines the temporary benchmark evidence.
- [Large-file performance contract](contracts/large-file-performance.md) defines expected command and safety behavior.
- [Research decisions](research.md) define the bounded observation-deduplication approach.
