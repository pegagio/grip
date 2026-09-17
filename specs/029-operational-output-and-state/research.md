# Research: Operational Output and State Rebinding

## Decisions

- **External tool owns standard output**: A selected external handoff writes no Grip result header or footer to standard output. Verbose Grip explanation belongs on standard error, preserving the tool's stream and exit semantics.
- **Differences are property-oriented**: Grouping by changed managed property with source, baseline, and destination values is more actionable than repeated directional field lists.
- **Rebinding follows mapping intent**: A raw descriptor digest remains publication evidence, but only project location and resolved mapping identity decide whether existing baselines require reinspection after configuration changes.

## Rejected Alternatives

- Keep normal detailed inspection on standard output before every external diff: rejected because it pollutes the configured tool's output.
- Treat every descriptor-byte change as payload authorization drift: rejected because output-only profiles cannot affect mapping ownership or topology.
