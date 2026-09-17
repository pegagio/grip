# Quickstart: Adopt One Destination File

Assume a tree mapping already relates `home` to `~/` and a destination-only regular file exists at `~/.config/example/settings.json`.

Preview the exact adoption first:

```bash
grip pull --adopt --dry-run ~/.config/example/settings.json
```

Apply it after reviewing the plan:

```bash
grip pull -a ~/.config/example/settings.json
```

Grip creates `home/.config/example/settings.json` and only any missing source ancestor directories required to contain it. It leaves the destination unchanged, verifies supported managed state, and publishes accepted evidence for the target and newly created ancestor chain.

An ignored paired source path is rejected normally. To make a deliberate one-time exception:

```bash
grip pull -a --force ~/.config/example/settings.json
```

Grip does not edit `.gripignore`. It prints the policy-file location and precise negation rule set needed for ordinary discovery to retain the member later; the set may include ignored ancestor directories as well as the file. Add those rules yourself if persistent membership is intended.

These uses are invalid and make no changes:

```bash
grip pull --adopt --source home/.config/example/settings.json
grip pull --adopt ~/.config/example
```

The first has incompatible selector spaces. The second selects a directory, while initial adoption supports one regular file only.
