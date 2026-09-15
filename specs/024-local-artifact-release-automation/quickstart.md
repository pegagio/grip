# Quickstart: Prepare and Publish a Local Artifact Release

This guide validates the Feature 024 workflow without making external publication implicit.

## Prerequisites

- macOS on Apple Silicon.
- A clean `master` checkout whose package version has not already been released locally.
- Installed project tools:

```sh
mise trust
mise install
```

## Prepare the Release

Confirm the candidate is the intended merged release commit, then run:

```sh
git switch master
git pull --ff-only origin master
git status --short
mise run release
```

Expected results:

- The release gates complete successfully.
- `dist/grip-v<version>-darwin-arm64.tar.gz` and its `.sha256` file exist.
- `v<version>` is an annotated local tag pointing to the checked `master` commit.
- No remote branch, remote tag, GitHub Release, upload, or registry publication occurs.

## Inspect the Artifact

```sh
(cd dist && shasum -a 256 -c grip-v<version>-darwin-arm64.tar.gz.sha256)
tar -tzf dist/grip-v<version>-darwin-arm64.tar.gz
git show v<version>
```

The checksum succeeds, the archive lists `grip`, `README.md`, and `LICENSE` beneath its versioned top-level directory, and the tag resolves to the intended release commit.

## Publish Deliberately

Only after inspection, push the tag explicitly:

```sh
git push origin v<version>
```

Then create the GitHub Release manually from that tag and upload the archive plus checksum. Do not use the release task to publish or push.

## Failure Checks

The release task must fail without creating a new tag when any of these conditions are true:

- The checkout is dirty or is not on `master`.
- The host is not macOS on Apple Silicon.
- The package version is invalid.
- The tag, final archive, or final checksum already exists.
- Validation, performance qualification, packaging, or checksum verification fails.

## Validation Evidence

On 2026-09-15, the isolated `mise run test-release` contract suite, `mise run validate`, and `mise run performance` all passed on Darwin/arm64. The contract suite used disposable Git repositories and did not prepare an artifact or tag in this checkout.
