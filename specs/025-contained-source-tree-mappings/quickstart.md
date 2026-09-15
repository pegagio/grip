# Quickstart: Contained-Source Tree Mappings

This walkthrough exercises the planned `home/` to `~/` behavior entirely inside a disposable temporary home. It is an acceptance scenario for the implementation, not a statement that the feature already exists.

## Build and isolate the environment

From the repository root, build the debug binary and create a temporary home with the Grip project beneath it.

```bash
mise exec -- cargo build

grip_fixture_root="$(mktemp -d)"
grip_fixture_home="$grip_fixture_root/home"
grip_fixture_project="$grip_fixture_home/dotfiles"
grip_fixture_binary="$PWD/target/debug/grip"

mkdir -p "$grip_fixture_project/home/.local/bin"
printf '%s\n' 'export EDITOR=vi' > "$grip_fixture_project/home/.bashrc"
printf '%s\n' '[user]' '    name = Example User' > "$grip_fixture_project/home/.gitconfig"
printf '%s\n' '#!/bin/sh' 'printf "%s\n" tool' > "$grip_fixture_project/home/.local/bin/tool"
chmod +x "$grip_fixture_project/home/.local/bin/tool"
```

All following commands explicitly bind `HOME` to the fixture. Do not run this scenario against a real home directory.

## Initialize and add the contained mapping

Initialize the project, enter it, and add `home/` with the fixture home as destination.

```bash
HOME="$grip_fixture_home" "$grip_fixture_binary" init "$grip_fixture_project"
cd "$grip_fixture_project"
HOME="$grip_fixture_home" "$grip_fixture_binary" add home '~/'
```

Expected result after implementation:

- The tree mapping is accepted even though `$grip_fixture_project/home` is beneath `$grip_fixture_home`.
- Add changes neither source nor destination payload.
- `.bashrc`, `.gitconfig`, and `.local/bin/tool` are current managed identities.
- The Grip project itself and other home content are not destination inventory.

## Inspect and synchronize safe members

Use machine-readable inspection and a dry-run before an ordinary push.

```bash
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json status
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json diff
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json push --dry-run
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json push
```

Expected result:

- Only current or retained managed identities appear.
- The dry run changes no payload or accepted state.
- The push creates or updates only the paired home paths.
- Accepted state is published only after successful verification.

Edit an established destination member and exercise the existing pull rules:

```bash
printf '%s\n' 'export EDITOR=nvim' > "$grip_fixture_home/.bashrc"
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json pull --dry-run
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json pull
```

## Demonstrate bounded destination inspection

Create 10,000 unrelated entries plus an unrelated symlink. None corresponds to current or accepted source membership.

```bash
mkdir -p "$grip_fixture_home/unrelated"
for grip_fixture_index in $(seq 1 10000); do
  : > "$grip_fixture_home/unrelated/item-$grip_fixture_index"
done
ln -s "$grip_fixture_home/unrelated" "$grip_fixture_home/unrelated-link"

HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json status
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json diff
```

Expected result:

- No unrelated entry or link appears in output.
- No unrelated entry is classified, baselined, selected, or changed.
- Changes beneath `unrelated/` do not stale a managed operation.

The automated performance acceptance test supplies repeatable warm-run timing and access assertions; this shell fixture is only a behavioral smoke test.

## Demonstrate the recursive-member blocker

The source is at destination-relative path `dotfiles/home`. Creating that relative subtree beneath the source introduces an ancestor member at `dotfiles` and an equal member at `dotfiles/home`; both relationships are unsafe.

```bash
mkdir -p "$grip_fixture_project/home/dotfiles/home"
printf '%s\n' 'unsafe' > "$grip_fixture_project/home/dotfiles/home/example"

HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json status
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json push --dry-run
```

Expected result:

- Both commands report `recursive_member_topology` for the unsafe subtree. Deterministic detail includes `dotfiles` with relation `ancestor`; implementations that report all findings also include `dotfiles/home` with relation `equal`.
- No payload mutation or accepted-state publication begins.
- Force and exact-entry selection do not bypass the blocker.

If the subtree is intentionally not managed, exclude it through ordinary source policy and inspect again:

```bash
printf '%s\n' 'dotfiles/' > "$grip_fixture_project/home/.gripignore"
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json status
```

Expected result: the excluded current member owns no destination and no recursive-member blocker is emitted. Any previously accepted identity beneath the ignored prefix follows normal ignored-retirement behavior.

## Preserve a retained source deletion

Remove an accepted source member after a successful synchronization and inspect its exact destination identity.

```bash
rm "$grip_fixture_project/home/.gitconfig"
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json status
HOME="$grip_fixture_home" "$grip_fixture_binary" --output=json sync --dry-run
```

Expected result: `.gitconfig` retains its existing source-deletion or conflict classification without Grip walking unrelated destination siblings. Dry-run remains non-mutating.

## Automated validation

Run focused tests while developing, then the complete repository gate.

```bash
cargo test --test contained_source_tree_integration
cargo test --test mapping_topology_integration
cargo test --test gripignore_conformance
cargo test --test project_state_rebinding_integration
cargo test --test apfs_name_compatibility_integration
mise run validate
```

Run the release performance acceptance separately because it is intentionally ignored by the normal suite.

The contained-home one-second p95 assertion is enabled only for the supported macOS ARM64 release platform; other targets do not claim this workstation-specific threshold.

```bash
mise run performance
```
