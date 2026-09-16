# Quickstart: Configurable External Diff Program

Use only disposable project and home directories for this implementation-validation guide; do not point Grip at real mappings or a real user configuration.

## Build and Prepare Fixtures

```sh
mise exec -- cargo build
```

Create an isolated `HOME`, an initialized project with one selected managed file or directory pair, and a recording executable. The helper should record each received argument separately, create a sentinel when invoked, and support controlled normal exits and `SIGTERM`.

## Verify Precedence and Literal Arguments

1. Write `HOME/.grip/config.toml` with `[diff] tool = "record"` and a `[difftool.record]` executable plus literal arguments.
2. Run human `grip diff SOURCE`; verify configured arguments precede the resolved source and destination paths.
3. Add a project selection and `args = ["--project"]`; verify the global executable is inherited and only `--project` is passed.
4. Set `GRIP_EXTERNAL_DIFF` to a recording wrapper; verify it receives exactly the two endpoint arguments and no TOML arguments.
5. Remove selections and use a test `diff` on `PATH`; verify the fallback is `diff SOURCE DESTINATION`.

Use strings containing whitespace and shell-looking text such as `$(not-run)` and `*`. Assert they are recorded literally and cause no shell side effect.

## Verify Non-Launch and Completion

1. Run `grip diff` without a selector and verify no sentinel.
2. Run `grip --output json diff SOURCE`; verify existing JSON structure and no sentinel.
3. Exercise missing, unsupported, and unsafe-symlink endpoints; verify established diagnostics, no sentinel, and unchanged mapping/state snapshots.
4. Test source and `--destination` selectors for file and mapping-root directory comparisons.
5. Have the helper exit `0`, `1`, and another nonzero code. Verify Grip returns each unchanged and reports completion.
6. Have the helper terminate with `SIGTERM`. Verify Grip identifies the signal and returns `143`.
7. Configure an unavailable executable. Verify a launch diagnostic and no payload, mapping, state, or temporary comparison files.

## Automated Validation

After implementation, run:

```sh
cargo test --test diff_cli_contract
cargo test --test classification_cli_contract
cargo test
mise run validate
```

For planning artifact hygiene, run:

```sh
git diff --check
rg -n 'NEEDS[[:space:]]CLARIFICATION|\\[(FEATURE|DATE|###-feature)\\]' specs/028-configurable-external-diff/plan.md specs/028-configurable-external-diff/research.md specs/028-configurable-external-diff/data-model.md specs/028-configurable-external-diff/quickstart.md specs/028-configurable-external-diff/contracts
```
