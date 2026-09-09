# Quickstart: Validate Project-Scoped Grip

This is an implementation acceptance guide. Use isolated temporary project and home roots; never point commands at a developer's real home, project, or legacy state.

## Prerequisites and Setup

Use the repository's pinned Rust toolchain and build the CLI before running manual scenarios:

```bash
mise install
cargo build
```

Create disposable roots with `mktemp -d`, set `HOME` only for the child command under test, and use `--project` or the child process working directory to control project selection. Preserve the paths long enough to compare descriptor, state, and payload bytes, then remove them through the test harness's normal temporary-directory cleanup.

## 1. Initialize

Create an existing empty directory and run `grip init` from it. Verify Descriptor V2, exact `/state/` in `.grip/.gitignore`, and absent `.grip/state/`. Run init again and verify an exact `already_initialized` no-op. Repeat with an explicit existing path.

Exercise missing targets, files, symlinks, unsafe modes, partial metadata, altered `.gitignore`, unsupported config, concurrent init, and a nested target. Every failure must preserve prior bytes and create no state or payload.

## 2. Select a Project

Create two isolated projects. From each root and a deep descendant, run `validate`, mapping inspection, and dry-run synchronization without `--project`; verify only the enclosing project is reported. From elsewhere, select each exact root explicitly.

Verify failures for zero candidates, explicit descendants, invalid roots, multiple enclosing projects, malformed inner metadata, and descriptor replacement. Set `GRIP_HOME` to a populated trap and prove no command accesses it. Run help and version outside a project and prove they inspect neither project nor home.

## 3. Use Portable Mappings

Create source content and add project-relative sources with `~` destinations. Confirm the descriptor contains only declarations while output distinguishes declared and resolved values.

Reject absolute paths, `..`, repeated and trailing separators, `${HOME}`, other-user tilde, source `.grip`, and symlink escapes. Verify that tree source `.` includes ordinary content but never `.grip/`.

Copy committed metadata to another root and use a different isolated home. Descriptor bytes must stay identical while endpoints resolve differently.

## 4. Validate State and Rebinding

Accept baselines independently in two clones and confirm each writes only its own state. Hold one project's mutation lock and prove only that project contends.

Move or copy a project with State V4. Status and dry-run under a changed root and home must fully evaluate rebinding without changing bytes. With equivalent evidence, an authorized state writer records the current binding in final publication.

Repeat with descriptor drift, missing mappings, endpoint drift, corrupt state, ambiguous identity, unsafe ancestry, and mismatched recovery. Retain evidence but block acceptance and mutation.

## 5. Run Regression and Performance Validation

Run every existing mapping, discovery, classification, baseline, synchronization, deletion, retirement, recovery, contention, failure, metadata, filesystem, and product-acceptance suite using project fixtures. Dry runs preserve config, state, operation, recovery, and payload bytes.

```bash
cargo test
mise run validate
```

Scan current runtime, active tests, `README.md`, `docs/product-definition.md`, and current wiki pages for supported global registry, absolute stored mapping, and `GRIP_HOME` workflows. Exclude historical Features 001–009 and their reviews.

Run 100 release-mode samples from a directory 100 levels below a project root and record p50, p95, maximum, revision, and host evidence. Acceptance requires p95 at or below one second. Preserve existing 10,000-entry performance targets after fixture conversion.

## Acceptance Evidence

Evidence was collected on 2026-09-09 from an arm64 workstation running macOS 26.6.2 (build 25G83), APFS bundle 2811.160.7, Rust 1.98, release profile, at revision `3caf2a13e9ec449be9878ee90720f018e4739a4b`. The local hostname is intentionally omitted from committed evidence.

The automated acceptance suites exercise every scenario in Sections 1 through 4 using disposable project and destination-home roots. `cargo test` passed the initialization, project-selection, portable-path, clone-portability, project-state, rebinding, recovery, regression, interface-removal, and product-acceptance matrices. `mise run validate` additionally passed formatting, Clippy with warnings denied, the complete default suite, and the release build. Read-only and dry-run preservation assertions compare the complete disposable roots before and after execution.

`mise run performance` completed 100 release-mode samples and passed every asserted threshold. The measured distributions were:

| Workload | p50 | p95 | Maximum |
| --- | ---: | ---: | ---: |
| Help | 4.269 ms | 5.563 ms | 8.350 ms |
| Version | 3.922 ms | 4.539 ms | 4.769 ms |
| Validate, 1,000 mappings | 473.180 ms | 479.985 ms | 499.029 ms |
| Mapping list, 1,000 mappings | 476.601 ms | 485.453 ms | 507.714 ms |
| Implicit discovery, 100 levels | 4.666 ms | 4.960 ms | 5.262 ms |
| Discovery, 10,000 entries | 4.046 ms | 4.618 ms | 5.228 ms |
| Status, 10,000 accepted paired entries | 1.659 s | 1.706 s | 1.776 s |
| Push dry-run, 10,000 entries | 1.410 s | 1.447 s | 1.498 s |
| Push execute-plan, 10,000 entries | 1.190 s | 1.213 s | 1.309 s |
| Pull dry-run, 10,000 entries | 1.626 s | 1.676 s | 1.706 s |
| Pull execute-plan, 10,000 entries | 1.366 s | 1.410 s | 1.486 s |
| Sync dry-run, mixed 10,000 entries | 1.679 s | 1.733 s | 1.779 s |
| Sync execute-plan, mixed 10,000 entries | 1.391 s | 1.422 s | 1.515 s |
| Delete preview, 10,000 entries | 142.269 ms | 144.290 ms | 169.414 ms |
| Recovery inventory, 10,000 entries | 1.402 s | 1.455 s | 1.492 s |

The representative synchronization of 10,000 entries with 10 changes completed in 29.819 seconds. The Feature 010 one-second requirement applies to 100-level implicit project discovery, whose p95 was 4.960 ms; the converted Feature 009 workloads retained their existing individual thresholds.

The explicit ignored APFS suites ran against two distinct disposable 256 MB disk images formatted as case-sensitive APFS and case-insensitive APFS. Filesystem behavior was independently confirmed by creating two case-distinct names: the insensitive volume exposed one file, the sensitive volume exposed two, and their device identities differed. The case-sensitive and case-insensitive product matrices, cross-volume logical-state transfer, cross-volume capability comparison, and case-collision detection all passed. The images were detached and removed after qualification.
