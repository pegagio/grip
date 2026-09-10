# Quickstart: Git-Inspired Command Hierarchy

This guide describes the expected behavior after implementation. Use a disposable project while exercising endpoint publication and force behavior.

## Initialize and declare a mapping

```bash
grip init
grip add ./local/settings.toml ./deploy/settings.toml
grip list
```

`add` succeeds if at least one endpoint exists. It does not create `./deploy/settings.toml`; use a later directional command to choose the content source.

## Inspect a project

```bash
grip status
grip status -e
grip status -d ./deploy/settings.toml
grip diff ./local/settings.toml
grip diff -d ./deploy/settings.toml
grip -o json status
```

Grip records the relevant `.gripignore` policy revision with baseline evidence. A later policy change makes the old baseline unusable even if the rule is removed before the next Grip command. `status` may then prune only that private evidence; it does not change either endpoint or mapping declaration.

## Synchronize safely

```bash
grip push ./local/settings.toml
grip pull -n ./local/settings.toml
grip sync
```

Ordinary directional commands and `sync` refuse a one-sided absence. `sync` also refuses different endpoints without a baseline so that a user chooses a direction explicitly.

## Resolve a deliberate authoritative choice

```bash
grip push -f ./local/settings.toml
grip pull -f -d ./deploy/settings.toml
```

Each forced command must resolve to exactly one managed entry. `push -f` makes the source endpoint authoritative; `pull -f` makes the destination endpoint authoritative. If that winning endpoint is absent, the command intentionally propagates the absence to the peer.

## Remove a declaration

```bash
grip remove ./local/settings.toml
```

This removes the mapping and its private comparison baseline only. Endpoint files remain in place. Re-adding the same mapping begins without historical baseline evidence.

## Verify the implementation

```bash
mise run validate
mise run performance
```

The normal validation gate checks formatting, linting, tests, and a release build. The performance command runs the ignored release-performance acceptance test intentionally. On the feature branch, the representative 1,000-entry tree scenario passed with one measurement sample: `status` and `diff` were below 200 ms p95, and each dry-run synchronization command was below 140 ms p95. The default harness takes 25 samples; increase `GRIP_PERFORMANCE_SAMPLE_COUNT` for a longer workstation qualification run.
