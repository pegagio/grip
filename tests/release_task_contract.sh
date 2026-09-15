#!/usr/bin/env bash

# Contract coverage for the maintainer-only local release task.
set -euo pipefail

source_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
real_git="$(command -v git)"
real_ln="$(command -v ln)"
real_tar="$(command -v tar)"
real_shasum="$(command -v shasum)"
test_root="$(mktemp -d "${TMPDIR:-/private/tmp}/grip-release-contract.XXXXXX")"
trap 'rm -rf "$test_root"' EXIT
fixture_number=0

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
assert() { "$@" || fail "assertion failed: $*"; }
expect_failure() { if "$@"; then fail "expected failure: $*"; fi; }

new_fixture() {
  fixture_number=$((fixture_number + 1))
  fixture="$test_root/fixture-$fixture_number"
  repo="$fixture/repo"
  bin="$fixture/bin"
  mkdir -p "$repo/scripts" "$bin" "$fixture/remote.git"
  cp "$source_root/scripts/prepare-release.sh" "$repo/scripts/"
  printf '[package]\nname = "grip"\nversion = "1.2.%s"\nedition = "2024"\n' "$fixture_number" > "$repo/Cargo.toml"
  printf '# Grip\n' > "$repo/README.md"
  printf 'license\n' > "$repo/LICENSE"
  printf 'target/\ndist/\n' > "$repo/.gitignore"
  "$real_git" -C "$repo" init -q -b master
  "$real_git" -C "$repo" config user.email test@example.invalid
  "$real_git" -C "$repo" config user.name release-test
  "$real_git" -C "$repo" add .
  "$real_git" -C "$repo" commit -qm fixture
  "$real_git" init --bare -q "$fixture/remote.git"
  "$real_git" -C "$repo" remote add origin "$fixture/remote.git"
  version="1.2.$fixture_number"
  tag="v$version"
  artifact="$repo/dist/grip-$tag-darwin-arm64.tar.gz"
  checksum="$artifact.sha256"
  publish_log="$fixture/publish.log"

  cat > "$bin/git" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == push ]]; then printf 'git push\n' >> "$GRIP_RELEASE_PUBLISH_LOG"; exit 97; fi
if [[ "${1:-}" == tag && "${GRIP_RELEASE_TEST_FAIL_STAGE:-}" == tag ]]; then exit 48; fi
exec "$REAL_GIT" "$@"
EOF
  cat > "$bin/mise" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" != run ]]; then exit 98; fi
if [[ "${GRIP_RELEASE_TEST_FAIL_STAGE:-}" == validate && "${2:-}" == validate ]]; then exit 44; fi
if [[ "${GRIP_RELEASE_TEST_FAIL_STAGE:-}" == performance && "${2:-}" == performance ]]; then exit 45; fi
if [[ "${2:-}" == validate ]]; then mkdir -p target/release; printf '#!/usr/bin/env bash\necho grip\n' > target/release/grip; chmod +x target/release/grip; exit 0; fi
[[ "${2:-}" == performance ]] && exit 0
exit 98
EOF
  cat > "$bin/cargo" <<'EOF'
#!/usr/bin/env bash
if [[ "${GRIP_RELEASE_TEST_VERSION:-}" == invalid ]]; then printf '{"packages":[{"manifest_path":"%s/Cargo.toml","version":"invalid"}]}' "$PWD"; exit 0; fi
if [[ "${GRIP_RELEASE_TEST_VERSION:-}" == malformed ]]; then printf '{"packages":[{"manifest_path":"%s/Cargo.toml","version":"1.2.3-"}]}' "$PWD"; exit 0; fi
printf '{"packages":[{"manifest_path":"%s/Cargo.toml","version":"%s"}]}' "$PWD" "${GRIP_RELEASE_TEST_VERSION}"
EOF
  cat > "$bin/uname" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  -s) printf '%s\n' "${GRIP_RELEASE_TEST_OS:-Darwin}" ;;
  -m) printf '%s\n' "${GRIP_RELEASE_TEST_ARCH:-arm64}" ;;
  *) exit 98 ;;
esac
EOF
  cat > "$bin/tar" <<'EOF'
#!/usr/bin/env bash
[[ "${GRIP_RELEASE_TEST_FAIL_STAGE:-}" == package ]] && exit 46
exec "$REAL_TAR" "$@"
EOF
  cat > "$bin/shasum" <<'EOF'
#!/usr/bin/env bash
[[ "${GRIP_RELEASE_TEST_FAIL_STAGE:-}" == checksum ]] && exit 47
exec "$REAL_SHASUM" "$@"
EOF
  cat > "$bin/ln" <<'EOF'
#!/usr/bin/env bash
if [[ "${GRIP_RELEASE_TEST_FAIL_STAGE:-}" == artifact_collision && "${2:-}" == *.tar.gz ]]; then printf 'concurrent artifact\n' > "$2"; fi
if [[ "${GRIP_RELEASE_TEST_FAIL_STAGE:-}" == checksum_collision && "${2:-}" == *.sha256 ]]; then printf 'concurrent checksum\n' > "$2"; fi
exec "$REAL_LN" "$@"
EOF
  for command in gh curl npm; do
    cat > "$bin/$command" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$(basename "$0")" >> "$GRIP_RELEASE_PUBLISH_LOG"
exit 96
EOF
  done
  chmod +x "$bin"/* "$repo/scripts/prepare-release.sh"
}

run_release() {
  (cd "$repo" && PATH="$bin:$PATH" REAL_GIT="$real_git" REAL_LN="$real_ln" REAL_TAR="$real_tar" REAL_SHASUM="$real_shasum" GRIP_RELEASE_PUBLISH_LOG="$publish_log" GRIP_RELEASE_TEST_VERSION="${GRIP_RELEASE_TEST_VERSION:-$version}" bash scripts/prepare-release.sh)
}

assert_no_final_identity() {
  assert test ! -e "$artifact"
  assert test ! -e "$checksum"
  if "$real_git" -C "$repo" rev-parse --verify --quiet "refs/tags/$tag" >/dev/null; then fail "unexpected tag $tag"; fi
}

new_fixture
before_refs="$($real_git --git-dir "$fixture/remote.git" for-each-ref)"
release_output="$(run_release)"
assert test -f "$artifact"
assert test -f "$checksum"
(cd "$(dirname -- "$checksum")" && "$real_shasum" -a 256 -c "$(basename -- "$checksum")")
archive_listing="$($real_tar -tzf "$artifact")"
printf '%s\n' "$archive_listing" | grep -qx "grip-$tag-darwin-arm64/grip" || fail 'archive lacks grip executable'
printf '%s\n' "$archive_listing" | grep -qx "grip-$tag-darwin-arm64/README.md" || fail 'archive lacks README'
printf '%s\n' "$archive_listing" | grep -qx "grip-$tag-darwin-arm64/LICENSE" || fail 'archive lacks LICENSE'
extract_directory="$fixture/extracted"
mkdir -p "$extract_directory"
"$real_tar" -xzf "$artifact" -C "$extract_directory"
assert test -x "$extract_directory/grip-$tag-darwin-arm64/grip"
assert test -f "$extract_directory/grip-$tag-darwin-arm64/README.md"
assert test -f "$extract_directory/grip-$tag-darwin-arm64/LICENSE"
assert test "$("$real_git" -C "$repo" cat-file -t "$tag")" = tag
assert test "$("$real_git" -C "$repo" rev-parse "$tag^{}")" = "$("$real_git" -C "$repo" rev-parse HEAD)"
assert test "$before_refs" = "$("$real_git" --git-dir "$fixture/remote.git" for-each-ref)"
assert test ! -e "$publish_log"
printf '%s\n' "$release_output" | grep -Fx 'no branch or tag was pushed; no GitHub API call, upload, or package-registry publication was performed.' >/dev/null || fail 'release output does not state the complete non-publication boundary'

new_fixture; printf dirty > "$repo/dirty"; expect_failure run_release; assert_no_final_identity
new_fixture; "$real_git" -C "$repo" checkout -qb feature; expect_failure run_release; assert_no_final_identity
new_fixture; GRIP_RELEASE_TEST_OS=Linux expect_failure run_release; assert_no_final_identity
new_fixture; GRIP_RELEASE_TEST_ARCH=x86_64 expect_failure run_release; assert_no_final_identity
new_fixture; GRIP_RELEASE_TEST_VERSION=invalid expect_failure run_release; assert_no_final_identity
new_fixture; GRIP_RELEASE_TEST_VERSION=malformed expect_failure run_release; assert_no_final_identity
new_fixture; "$real_git" -C "$repo" tag -a "$tag" -m existing; expect_failure run_release; assert test ! -e "$artifact"
new_fixture; mkdir -p "$repo/dist"; printf existing > "$artifact"; expect_failure run_release; assert test "$(cat "$artifact")" = existing
new_fixture; mkdir -p "$repo/dist"; printf existing > "$checksum"; expect_failure run_release; assert test "$(cat "$checksum")" = existing; if "$real_git" -C "$repo" rev-parse --verify --quiet "refs/tags/$tag" >/dev/null; then fail "unexpected tag $tag"; fi
new_fixture; GRIP_RELEASE_TEST_FAIL_STAGE=validate expect_failure run_release; assert_no_final_identity
new_fixture; GRIP_RELEASE_TEST_FAIL_STAGE=performance expect_failure run_release; assert_no_final_identity
new_fixture; GRIP_RELEASE_TEST_FAIL_STAGE=package expect_failure run_release; assert_no_final_identity
new_fixture; GRIP_RELEASE_TEST_FAIL_STAGE=checksum expect_failure run_release; assert_no_final_identity
new_fixture; GRIP_RELEASE_TEST_FAIL_STAGE=tag expect_failure run_release; assert_no_final_identity
new_fixture; GRIP_RELEASE_TEST_FAIL_STAGE=artifact_collision expect_failure run_release; assert test "$(cat "$artifact")" = 'concurrent artifact'; assert test ! -e "$checksum"
new_fixture; GRIP_RELEASE_TEST_FAIL_STAGE=checksum_collision expect_failure run_release; assert test ! -e "$artifact"; assert test "$(cat "$checksum")" = 'concurrent checksum'

printf 'release task contract tests passed\n'
