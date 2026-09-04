# Storage Contract: Mapping Registry

This contract extends the Feature 001 Grip-home layout with mapping tuples and bounded registry-publication artifacts.

## Layout

```text
<GRIP_HOME>/
├── config.toml
├── .registry.lock
├── .config.tmp-<unique-token>
└── state/
    ├── ... Feature 001 machine-owned state, if present
    └── recovery/
        └── registry/
            └── sha256-<digest>/
                └── config.toml
```

`config.toml` is accepted user intent. `.registry.lock` is a stable Grip-owned coordination file with no mapping data. Staging files are Grip-owned incomplete candidates and are never accepted configuration. Registry recovery generations are Grip-owned exact prior documents; mapping commands do not create or modify synchronization state.

## Registry V1

An empty registry remains:

```toml
schema_version = 1
mappings = []
```

A registry with file and tree mappings is:

```toml
schema_version = 1

[[mappings]]
kind = "file"
source = "/Users/pegagio/GripSource/git/.gitconfig"
destination = "/Users/pegagio/.gitconfig"

[[mappings]]
kind = "tree"
source = "/Users/pegagio/GripSource/editor"
destination = "/Users/pegagio/.config/editor"
```

All fields are required. Unknown top-level or mapping fields, unsupported kinds, unsupported versions, relative or non-UTF-8 paths, and invalid ownership graphs are rejected. Accepted mappings are serialized in canonical source-path order. Updates preserve supported field values but may normalize whitespace, comments, and presentation-only ordering.

## Registry publication protocol

An add or remove writer performs this bounded operation:

1. Validate the Grip home, `config.toml`, every accepted mapping, and the complete accepted ownership graph.
2. Retain the exact accepted registry bytes and relevant canonical path evidence.
3. Construct and validate the complete candidate registry.
4. Create or validate `.registry.lock` as a current-user-owned non-symlink regular file with mode `0600`, then acquire a nonblocking exclusive advisory lock.
5. Reread and revalidate the accepted registry and relevant path evidence. Reject the operation if the bytes, canonical identities, node kinds, or safe ancestry changed.
6. Compute the lowercase SHA-256 digest of the exact accepted registry bytes and select `state/recovery/registry/sha256-<digest>/config.toml`.
7. Create Grip-owned recovery directories with mode `0700`; publish the recovery file with mode `0600`, sync it, reread it, verify its digest and registry meaning, and reuse it only when an existing generation is byte-identical and safe. Any collision or verification failure blocks replacement.
8. Serialize the complete candidate deterministically.
9. Exclusively create a unique same-directory staging regular file with mode `0600`, write and sync all candidate bytes, reread and decode it, and require semantic equality with the candidate.
10. Apply the exact accepted `config.toml` permission mode to the staged file.
11. Rename the staging file over `config.toml`, sync the Grip-home directory where supported, report success, and release the descriptor-backed lock.

The recovery publication must complete before the accepted-registry rename. The rename is the replacement acceptance point. Any earlier failure leaves the prior `config.toml` authoritative and may retain a verified recovery generation. A post-rename directory-sync failure is reported as an operational failure with the new complete document visible and the prior document recoverable; the result must state that publication visibility changed but durability confirmation failed.

## Coordination and cleanup

The stable lock file remains after release. Its existence never indicates contention and it is never deleted based on age or a recorded process identifier. An unlocked safe file is reused. A held lock causes immediate operational failure.

Failed writers remove only the exact staging path they created when it is still identifiable as their own. Readers ignore staging files. Unexpected pre-existing staging names are not accepted as registry data and are reported diagnostically. Verified registry recovery generations remain immutable after failure or success; restoration, listing, retention policy, and cleanup are outside Feature 002.

## Permissions and node safety

New lock, staging, and recovery files request mode `0600`; new recovery directories request mode `0700`. All Grip-owned nodes are verified as current-user-owned and non-symlink with the required type and mode. The accepted registry must be a current-user-owned non-symlink regular file with group/other write bits unset. Add and remove additionally require its owner-write bit. Replacement preserves the exact accepted registry mode; Grip does not silently widen permissions, change ownership, or change the pre-existing Grip-home mode.

Path validation and registry publication do not open, read, hash, create, copy, move, or delete mapped payload entries. Non-following metadata and directory access checks are the only permitted payload-path observations.
