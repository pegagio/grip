# Status and Push Path Contract

This contract applies only to the default human rendering of `grip status` and source-side selectors accepted by `grip push`.

## Human status output

Source paths are displayed relative to the directory from which the command was invoked. Destination paths retain their existing display form.

```text
$ cd project/app
$ grip status
Changes to push:
  main.py -> ~/workspace/app/main.py

Needs baseline:
  ../shared/config.yml >-< ~/workspace/shared/config.yml
```

An ordinary displayed source path is valid input to `grip push` from the same working directory.

```text
$ grip push main.py
$ grip push ../shared/config.yml
```

Names requiring human quoting use Git-style C notation. That notation improves readability but is not a promise that every displayed value is a shell-token serialization for every shell.

```text
Changes to push:
  "my file.py" -> ~/workspace/app/my file.py
```

The headings, relation symbols, classification grouping, destination display, clean message, and all JSON output remain unchanged.

## Source-side push selection

`grip push [PATH]`, including force and dry-run forms, resolves a relative source selector from the invocation directory. Absolute source selectors retain their existing rejection. The selected source path must remain in the selected project after traversal and symlink checks.

| Input | Expected behavior |
|---|---|
| `main.py` from `project/app` | Selects `project/app/main.py` |
| `../shared/config.yml` from `project/app` | Selects a source inside `project/shared` |
| `/project/app/main.py` | Retains the existing rejection of an absolute source selector |
| `../../outside.txt` | Fails before unmanaged-path inspection or mutation |
| In-project symlink resolving outside the project | Fails before unmanaged-path inspection or mutation |

`--destination` continues to use destination-space parsing. Status, diff, pull, sync, mapping, and other non-push commands continue to use their existing portable project-relative selector behavior.
