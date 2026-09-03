# Research: CLI, Configuration, and State Foundation

This research resolves every technical unknown from the implementation plan while preserving Feature 001's non-mutating payload boundary.

## Table of Contents

- [Toolchain and package policy](#toolchain-and-package-policy)
- [CLI and command boundary](#cli-and-command-boundary)
- [Serialization and schema dispatch](#serialization-and-schema-dispatch)
- [Grip home resolution](#grip-home-resolution)
- [State integrity and publication](#state-integrity-and-publication)
- [Publication coordination](#publication-coordination)
- [Errors, results, and diagnostics](#errors-results-and-diagnostics)
- [Testing and performance](#testing-and-performance)
- [Dependency boundary](#dependency-boundary)

## Toolchain and package policy

**Decision**: Pin Rust 1.98.0 in `mise.toml`, use Rust 2024, declare `rust-version = "1.98"`, commit `Cargo.lock`, and use conventional `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, and `cargo build --release` validation.

**Rationale**: Grip is a greenfield executable, and the current stable compiler avoids inventing an older-support promise and CI matrix without user value. Rust 2024 is already part of the approved roadmap direction. Exact pinning keeps development and CI aligned. See the [Rust release list](https://blog.rust-lang.org/releases/), [Rust 1.85 and the 2024 edition](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/), [Cargo `rust-version`](https://doc.rust-lang.org/stable/cargo/reference/rust-version.html), and [Cargo.lock guidance](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html).

**Alternatives considered**: Rust 1.85 would be the edition floor but would create an untested minimum-support commitment. A floating `stable` toolchain would make local and CI builds drift.

## CLI and command boundary

**Decision**: Use `clap` 4.6's derive API for global `--output <human|json>` and repeatable `-v`/`--verbose`, plus `validate` and `version` application commands. Keep built-in `-h`/`--help` and `-V`/`--version` as conventional text displays. Use `try_parse` so Grip classifies parse outcomes rather than allowing a dependency to terminate the process.

**Rationale**: Typed parsing and generated help cover the planned command surface with less handwritten validation. An ordinary `version` command gives automation a JSON result without replacing mature help/version behavior. Parser types are converted immediately into Grip-owned command types. See [`clap::Parser`](https://docs.rs/clap/latest/clap/trait.Parser.html), [`ErrorKind`](https://docs.rs/clap/latest/clap/error/enum.ErrorKind.html), [`ValueEnum`](https://docs.rs/clap/latest/clap/trait.ValueEnum.html), and [global arguments](https://docs.rs/clap/latest/clap/struct.Arg.html#method.global).

**Alternatives considered**: Hand parsing creates more usage and help behavior to maintain. Custom JSON help/version duplicates parser functionality. A `--json` boolean is initially smaller but makes future output formats an accumulating flag set.

## Serialization and schema dispatch

**Decision**: Use typed Serde DTOs, strict unknown-field rejection on every named wire object, TOML for `RegistryV1`, and JSON for `StateEnvelopeV1` and `ResultEnvelopeV1`. Decode in two stages: read and validate `schema_version`, then dispatch to the supported concrete schema and convert into version-neutral domain types.

**Rationale**: Two-stage decoding distinguishes unsupported versions from malformed content. Version-specific wire types keep serialization annotations and migration concerns out of domain behavior. Direct writer serialization avoids generic intermediate JSON values for results. See [Serde container attributes](https://serde.rs/container-attrs.html), [TOML typed deserialization](https://docs.rs/toml/latest/toml/fn.from_str.html), and [`serde_json::to_writer`](https://docs.rs/serde_json/latest/serde_json/fn.to_writer.html).

**Alternatives considered**: A permissive struct with defaults would hide operator mistakes. Untagged version enums weaken failure classification. `toml_edit` is unnecessary because this feature validates but does not preserve-format edit configuration.

## Grip home resolution

**Decision**: Read `GRIP_HOME` as an operating-system string. If present, require it to be non-empty and absolute and retain the exact accepted spelling. If absent, obtain the user's home through the `home` crate and append `.grip`. Validate an existing final root with non-following metadata, reject a symlink or non-directory, compare ownership with the effective user, and prove accessibility by opening it.

**Rationale**: This exactly implements the specified override and default without substituting XDG or macOS application-support conventions. `rustix` supplies safe effective-user identity and the Unix operations needed to validate Grip-owned descendants. See [`home::home_dir`](https://docs.rs/home/latest/home/fn.home_dir.html), [`symlink_metadata`](https://doc.rust-lang.org/std/fs/fn.symlink_metadata.html), Unix [`MetadataExt`](https://doc.rust-lang.org/stable/std/os/unix/fs/trait.MetadataExt.html), and [`rustix::process::geteuid`](https://docs.rs/rustix/latest/rustix/process/fn.geteuid.html).

**Alternatives considered**: Platform project-directory APIs violate the exact `~/.grip/` contract. Reading only `HOME` omits the platform fallback behavior supplied by the narrow `home` crate. Canonicalizing the accepted root would change the externally reported path.

## State integrity and publication

**Decision**: Store one accepted state envelope at `state/state.json`. Its SHA-256 integrity value covers the version-specific canonical JSON bytes of `schema_version` and `payload`, excluding the integrity member. Before replacement, copy the valid accepted state into `state/recovery/generation-<N>/state.json`, verify and sync that recovery copy, and reuse an existing byte-identical generation copy on retry. Then publish by exclusively creating a `0600` staging file in the same state directory, writing and syncing the complete bytes, rereading and verifying them, renaming over the accepted path, and syncing the directory where supported. Only the final rename establishes acceptance.

**Rationale**: Same-directory staging prevents cross-filesystem fallback, while synchronization plus rename gives old-or-new atomic visibility. Retaining the prior generation satisfies the recovery obligation without defining restoration or cleanup prematurely. A checksum in each envelope makes both accepted and recovery state independently verifiable. The product promises atomic visibility and explicit error reporting, not universal survival across every device cache and sudden power loss. See POSIX [`rename`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/rename.html), [`open`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/open.html), the [POSIX durability rationale](https://pubs.opengroup.org/onlinepubs/9799919799/xrat/V4_xbd_chap01.html), Apple [`fsync`](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fsync.2.html), and Rust [`File::sync_all`](https://doc.rust-lang.org/std/fs/struct.File.html).

**Alternatives considered**: Direct overwrite exposes truncated documents. A system temporary directory may cross filesystems. A checksum sidecar requires two-file atomicity. Omitting recovery would conflict with the constitution. SQLite or a journal is disproportionate for one small document. Always requesting macOS `F_FULLFSYNC` is slower, platform-specific, and still cannot promise universal hardware behavior.

## Publication coordination

**Decision**: Keep one `0600` regular file at `state/state.lock` and acquire a nonblocking exclusive OS advisory lock for the revalidation, recovery-copy, and publication critical section. A held lock is an internal `state_contention` error in Feature 001; exit `13` and its symbolic code remain reserved until a public state-publishing command exists. Other lock failures are operational. Never infer ownership from file existence, delete the stable lock after normal use, or break a lock using PID age.

**Rationale**: Kernel release on descriptor or process termination makes an unlocked leftover file normal and avoids a stale-sentinel recovery protocol. Retaining one inode prevents split-lock races after unlink/recreate. Rust 1.98 includes `File::try_lock`; Apple documents `flock` as advisory and `LOCK_NB` as nonblocking. See Rust [`File::try_lock`](https://doc.rust-lang.org/std/fs/struct.File.html) and Apple [`flock`](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/flock.2.html).

**Alternatives considered**: Locking `state.json` is unsafe across replacement because processes may lock different file objects. Exclusive-create sentinel locks require crash cleanup and unsafe stale ownership guesses. A blocking retry would hide contention and violate the actionable failure contract.

## Errors, results, and diagnostics

**Decision**: Use `thiserror` for narrow internal error enums and one exhaustive Grip result category that owns both symbolic and numeric codes. Application results go to stdout in either human or JSON form; optional diagnostics go to stderr. Inject both writers for tests. Result-stream failure maps to operational failure with only a best-effort concise stderr notice because the requested result channel is itself unavailable.

**Rationale**: Stable categories cannot depend on source-error prose. Injected writers make separation, broken pipes, and deterministic rendering directly testable. Feature 001 has no async or cross-component observability need that justifies `tracing`. See [`thiserror`](https://docs.rs/thiserror/latest/thiserror/), Rust [`ExitCode`](https://doc.rust-lang.org/std/process/struct.ExitCode.html), and [`stderr` locking](https://doc.rust-lang.org/std/io/fn.stderr.html).

**Alternatives considered**: `anyhow` favors opaque errors over exhaustive contractual classification. Handwritten error boilerplate removes a small dependency but adds repetition. Human errors on stderr would mix application results with diagnostics.

## Testing and performance

**Decision**: Use built-in unit/integration testing, `tempfile` as the sole initial dev dependency, and `std::process::Command` with `CARGO_BIN_EXE_grip` for black-box tests. Each child receives its own `HOME`, `GRIP_HOME`, and working directory; library tests receive explicit paths and writers. Add deterministic fault injection around write, sync, verify, rename, and directory-sync boundaries. Keep the 100-run release performance acceptance harness ignored by default and record its environment.

**Rationale**: This covers exact stdout, stderr, exit status, filesystem effects, permissions, symlinks, lock contention, and interruption without extra assertion, snapshot, property-test, or benchmark frameworks. Child-local environments avoid unsafe process-global environment mutation in Rust 2024. The acceptance harness directly measures the specified order statistic rather than substituting a microbenchmark. See [Rust test organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html), [Cargo's binary path environment](https://doc.rust-lang.org/cargo/reference/environment-variables.html), [`TempDir`](https://docs.rs/tempfile/latest/tempfile/struct.TempDir.html), and [Rust 2024 environment safety](https://doc.rust-lang.org/edition-guide/rust-2024/newly-unsafe-functions.html).

**Alternatives considered**: `assert_cmd`, `assert_fs`, `proptest`, Criterion, and snapshots are useful later if repeated pain justifies them, but each adds dependency and maintenance cost before demonstrated need. Shared-CI timing is too noisy for the workstation acceptance threshold.

## Dependency boundary

The initial dependency set is deliberately small and capability-driven:

| Dependency | Scope | Capability |
|------------|-------|------------|
| `clap` 4.6 | Runtime | Typed CLI parsing and conventional help/version |
| `serde` 1 | Runtime | Strict typed wire schemas |
| `toml` 1.1 | Runtime | User registry parsing |
| `serde_json` 1 | Runtime | State and result encoding |
| `thiserror` 2 | Runtime | Typed error derivation |
| `home` 0.5 | Runtime | Default user-home resolution |
| `rustix` 1.1 | Runtime | Safe Unix identity and descriptor-relative filesystem operations |
| `sha2` 0.10 | Runtime | State-envelope SHA-256 integrity |
| `tempfile` 3 | Test only | Isolated roots and same-directory staging tests |
