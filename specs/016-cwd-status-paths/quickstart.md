# Quickstart: Current-Directory Status Paths

Run status from a nested source directory to see source paths in the form you would use from that directory.

```text
$ cd project/app
$ grip status
Changes to push:
  main.py -> ~/workspace/app/main.py
```

Push the displayed ordinary source path without translating it to the project root.

```text
$ grip push main.py
```

Paths outside the invocation directory remain understandable and usable when they still belong to the selected project.

```text
$ grip status
Needs baseline:
  ../shared/config.yml >-< ~/workspace/shared/config.yml

$ grip push ../shared/config.yml
```

Attempts to push a selector outside the selected project fail before Grip inspects or changes an unmanaged path. Structured output remains available through the existing JSON mode and continues to use canonical path values.
