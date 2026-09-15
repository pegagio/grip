#!/usr/bin/env bash

# Prepare a local, reviewable Grip release artifact without publishing it.
set -euo pipefail

fail() {
  printf 'release preparation failed: %s\n' "$*" >&2
  exit 1
}

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(git -C "$script_dir/.." rev-parse --show-toplevel 2>/dev/null)" || fail 'run from a Git checkout'
cd "$repo_root"

branch="$(git symbolic-ref --quiet --short HEAD 2>/dev/null)" || fail 'HEAD must be attached to master'
[[ "$branch" == 'master' ]] || fail "current branch is '$branch'; expected master"
[[ -z "$(git status --porcelain --untracked-files=all)" ]] || fail 'working tree must be clean'
[[ "$(uname -s)" == 'Darwin' && "$(uname -m)" == 'arm64' ]] || fail 'supported release host is Darwin arm64'

candidate_commit="$(git rev-parse HEAD)"
metadata="$(cargo metadata --no-deps --format-version 1)" || fail 'could not read Cargo package metadata'
version="$(printf '%s' "$metadata" | python3 -c '
import json
import pathlib
import sys
metadata = json.load(sys.stdin)
manifest = pathlib.Path("Cargo.toml").resolve()
packages = [package for package in metadata["packages"] if pathlib.Path(package["manifest_path"]).resolve() == manifest]
if len(packages) != 1:
    raise SystemExit("could not identify the root Cargo package")
print(packages[0]["version"])
')" || fail 'could not identify the root package version'

if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?(\+[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$ ]]; then
  fail "package version '$version' is not a valid release version"
fi

tag="v$version"
artifact_directory="$repo_root/dist"
artifact_name="grip-$tag-darwin-arm64.tar.gz"
artifact_path="$artifact_directory/$artifact_name"
checksum_path="$artifact_path.sha256"

if git rev-parse --verify --quiet "refs/tags/$tag" >/dev/null; then fail "local tag '$tag' already exists"; fi
[[ ! -e "$artifact_path" ]] || fail "release artifact already exists: $artifact_path"
[[ ! -e "$checksum_path" ]] || fail "release checksum already exists: $checksum_path"

printf 'validating release candidate %s for %s\n' "$candidate_commit" "$tag"
mise run validate
mise run performance

[[ "$(git symbolic-ref --quiet --short HEAD 2>/dev/null)" == 'master' ]] || fail 'branch changed during validation'
[[ "$(git rev-parse HEAD)" == "$candidate_commit" ]] || fail 'HEAD changed during validation'
[[ -z "$(git status --porcelain --untracked-files=all)" ]] || fail 'working tree changed during validation'

binary="$repo_root/target/release/grip"
[[ -f "$binary" && -x "$binary" ]] || fail "release binary is missing or not executable: $binary"
[[ -f README.md ]] || fail 'README.md is missing'
[[ -f LICENSE ]] || fail 'LICENSE is missing'

mkdir -p "$artifact_directory"
stage_directory="$(mktemp -d "$artifact_directory/.staging-$tag.XXXXXX")" || fail 'could not create release staging directory'
artifact_committed=false
checksum_committed=false
tag_committed=false
release_complete=false
cleanup() {
  local status=$?
  rm -rf "$stage_directory"
  if [[ "$release_complete" != true ]]; then
    [[ "$tag_committed" == true ]] && git tag -d "$tag" >/dev/null 2>&1 || true
    [[ "$artifact_committed" == true ]] && rm -f "$artifact_path"
    [[ "$checksum_committed" == true ]] && rm -f "$checksum_path"
  fi
  exit "$status"
}
trap cleanup EXIT

package_directory="$stage_directory/grip-$tag-darwin-arm64"
mkdir -p "$package_directory"
cp "$binary" "$package_directory/grip"
cp README.md LICENSE "$package_directory/"

staged_artifact="$stage_directory/$artifact_name"
staged_checksum="$stage_directory/$artifact_name.sha256"
tar -C "$stage_directory" -czf "$staged_artifact" "grip-$tag-darwin-arm64" || fail 'could not create release archive'
(cd "$stage_directory" && shasum -a 256 "$(basename -- "$staged_artifact")" > "$(basename -- "$staged_checksum")") || fail 'could not generate release checksum'
(cd "$stage_directory" && shasum -a 256 -c "$(basename -- "$staged_checksum")") || fail 'release checksum verification failed'

if git rev-parse --verify --quiet "refs/tags/$tag" >/dev/null; then fail "local tag '$tag' appeared during release preparation"; fi
[[ ! -e "$artifact_path" ]] || fail "release artifact appeared during release preparation: $artifact_path"
[[ ! -e "$checksum_path" ]] || fail "release checksum appeared during release preparation: $checksum_path"
[[ "$(git rev-parse HEAD)" == "$candidate_commit" ]] || fail 'HEAD changed before release tag creation'
[[ -z "$(git status --porcelain --untracked-files=all)" ]] || fail 'working tree changed before release tag creation'

ln "$staged_artifact" "$artifact_path" || fail "release artifact appeared during publication: $artifact_path"
artifact_committed=true
ln "$staged_checksum" "$checksum_path" || fail "release checksum appeared during publication: $checksum_path"
checksum_committed=true
git tag -a "$tag" "$candidate_commit" -m "Grip $tag"
tag_committed=true

[[ "$(git rev-parse "$tag^{}")" == "$candidate_commit" ]] || fail "local tag '$tag' does not identify the release candidate"
[[ "$(git cat-file -t "$tag")" == 'tag' ]] || fail "local tag '$tag' is not annotated"
release_complete=true

printf 'prepared local release %s\n' "$tag"
printf 'archive: %s\n' "$artifact_path"
printf 'checksum: %s\n' "$checksum_path"
printf 'next steps: inspect the archive and tag, then explicitly run git push origin %s and publish the GitHub Release.\n' "$tag"
printf 'no branch or tag was pushed; no GitHub API call, upload, or package-registry publication was performed.\n'
