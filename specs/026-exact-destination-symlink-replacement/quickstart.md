# Quickstart: Exact Destination Symlink Replacement

This walkthrough is an acceptance guide for the planned implementation. It uses a disposable temporary home and must not be run against real dotfiles.

## Build and isolate the environment

From the repository root, build the debug binary and prepare a contained-source fixture.

```bash
mise exec -- cargo build

grip_fixture_root="$(mktemp -d)"
grip_fixture_home="$grip_fixture_root/home"
grip_fixture_project="$grip_fixture_home/dotfiles"
grip_fixture_binary="$PWD/target/debug/grip"
grip_fixture_target="$grip_fixture_root/original-bash-profile"

mkdir -p "$grip_fixture_project/home"
printf '%s\n' 'export EDITOR=vi' > "$grip_fixture_project/home/.bash_profile"
printf '%s\n' 'target stays unchanged' > "$grip_fixture_target"
ln -s "$grip_fixture_target" "$grip_fixture_home/.bash_profile"
```

All following commands bind `HOME` to the fixture. The project source tree is inside that home, matching the supported contained-source layout.

## Admit the mapping without changing a link

Initialize the project and add the source tree.

```bash
HOME="$grip_fixture_home" "$grip_fixture_binary" init "$grip_fixture_project"
cd "$grip_fixture_project"
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json add home/ '~/'
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json status
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json push --dry-run
```

Expected result after implementation:

- `add` records the mapping without changing the source, destination link object, or target file.
- Status and dry-run identify `home/.bash_profile` as an unresolved destination link and offer only exact source-winning force guidance.
- Ordinary dry-run and ordinary push do not replace the link.
- The target still contains `target stays unchanged`.

Confirm the link and target remain intact before any force request.

```bash
test -L "$grip_fixture_home/.bash_profile"
test "$(cat "$grip_fixture_target")" = 'target stays unchanged'
```

## Preview and execute exact source-winning replacement

Preview the exact action, then execute it.

```bash
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json push --force --dry-run home/.bash_profile
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json push --force home/.bash_profile
```

Expected result:

- The dry run changes neither link object, target, source, descriptor, nor State V4.
- The executed force replaces only `$grip_fixture_home/.bash_profile` with the source file state.
- The path is no longer a symbolic link and contains `export EDITOR=vi`.
- `$grip_fixture_target` is unchanged.
- Accepted state is published only after the replacement verifies.

```bash
test ! -L "$grip_fixture_home/.bash_profile"
test "$(cat "$grip_fixture_home/.bash_profile")" = 'export EDITOR=vi'
test "$(cat "$grip_fixture_target")" = 'target stays unchanged'
```

## Demonstrate hard boundaries

Restore the fixture, create a source link, and then create a destination ancestor link. Each case must fail before mutation.

```bash
rm "$grip_fixture_home/.bash_profile"
ln -s "$grip_fixture_target" "$grip_fixture_home/.bash_profile"
ln -s "$grip_fixture_target" "$grip_fixture_project/home/source-link"

HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json status
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json push --force home/source-link

rm "$grip_fixture_project/home/source-link"
mkdir -p "$grip_fixture_project/home/.config/app"
printf '%s\n' 'value' > "$grip_fixture_project/home/.config/app/settings"
ln -s "$grip_fixture_root" "$grip_fixture_home/.config"
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json status
```

Expected result:

- A source link remains unsupported and is never force-replaced.
- A destination ancestor link is a path-specific blocker; Grip neither follows nor replaces it.
- The original target remains unchanged throughout.

## Demonstrate bounded destination inspection

Create unrelated destination content, including a link, after mapping admission.

```bash
mkdir -p "$grip_fixture_home/unrelated"
for grip_fixture_index in $(seq 1 10000); do
  : > "$grip_fixture_home/unrelated/item-$grip_fixture_index"
done
ln -s "$grip_fixture_home/unrelated" "$grip_fixture_home/unrelated-link"

HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json status
```

Expected result: unrelated files and links are not read, classified, or emitted. Only current source-defined and retained accepted identities participate in destination inspection.

## Automated validation

Run focused regression suites while developing, then the repository validation gate.

```bash
cargo test --test contained_source_tree_integration
cargo test --test filesystem_boundary_integration
cargo test --test force_resolution_guidance_contract
cargo test --test push_filesystem_integration
cargo test --test sync_filesystem_integration
mise run validate
```

Run the release performance acceptance separately. Its one-second p95 threshold is claimed only on the supported macOS ARM64 release platform.

```bash
mise run performance
```
