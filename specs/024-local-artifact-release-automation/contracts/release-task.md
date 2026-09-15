# Local Release Task Contract

## Public Command

```sh
mise run release
```

## Preconditions

- Run from any directory within the Grip repository; the task resolves the repository root.
- Git `HEAD` is attached to exactly `master`.
- The repository working tree is clean before preparation.
- The host is macOS on Apple Silicon.
- Cargo package metadata supplies one valid version.
- No local `v<version>` tag or final `dist/` artifact/checksum for that version exists.

## Successful Results

The task runs the established validation and performance gates, then creates:

```text
dist/grip-v<version>-darwin-arm64.tar.gz
dist/grip-v<version>-darwin-arm64.tar.gz.sha256
local annotated tag v<version> -> captured master HEAD
```

The archive contains the optimized `grip` executable, `README.md`, and `LICENSE` under one versioned top-level directory. The checksum verifies the exact archive. The task reports the artifact paths, tag, and the required manual publication steps.

## Failure Contract

- A failed precondition reports the failed condition before validation, final artifact creation, or tag creation.
- A validation, packaging, or checksum failure reports its stage, leaves any pre-existing identity untouched, and creates no new tag.
- A collision reports the tag or artifact path and never overwrites it.
- Attempt-owned temporary material is not a releasable result and is cleaned on normal failure paths.

## Explicit Non-Effects

The task never runs `git push`, creates a GitHub Release, uploads artifacts, calls a GitHub API, contacts a package registry, or changes Grip mappings, payloads, or state.

## Manual Publication Contract

After inspecting the local output and tag, the maintainer separately pushes the tag and uploads the archive and checksum through GitHub's release workflow. These steps are documented but are not executed by `mise run release`.
