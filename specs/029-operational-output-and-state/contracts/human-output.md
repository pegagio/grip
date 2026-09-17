# Human Output Contract

For a selected external diff, the comparison program owns standard output. `-v` renders this Grip diagnostic shape on standard error:

```text
Grip inspection:
  Source: <source>
  Destination: <destination>
  Result: <plain-language classification>
  Differences:
    - modification time:
      source     : <value>
      baseline   : <value>
      destination: <value>
```

Only changed managed properties appear. Unmanaged compatibility notes do not appear. Capability lines use endpoint roles, not endpoint paths.

Human `Pushed N file(s)` and preview counts include only payload-file actions. A blocked no-selector forced push names the aggregate operation, lists available blocker reasons and path evidence, and directs the operator to `grip status`.
