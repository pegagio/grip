# Data Model: Simplify Default Command Output

This feature introduces no persistent entity, migration, schema, or lifecycle change. It defines human-only views derived from the existing command-result data.

## Human output views

| View | Existing source data | Human projection | JSON effect |
| --- | --- | --- | --- |
| Declared mapping | Mapping kind, declared source, declared destination, resolved endpoints | One declared row, with mapping kind only where the approved transcript requires it | None; declared and resolved values remain available |
| Mutation action | Operation, mode, result, action status, action source, action destination, action direction | A concise heading plus a direction-correct file row | None; actions, milestones, baseline, and operation record remain available |
| Status group | Classification, attention, blocking, prospective direction, compatibility findings | Ordered actionable sections and force-resolution guidance for initial collisions and ordinary divergent conflicts | None; records and diagnostic details remain available |
| Terminal mutation | Operation, direction or winner, result, counts, plan blockers, invocation-relative source display, destination endpoint, failure evidence | A concise no-action, conflict-resolution, technical-blocker, or status-before-retry result | None; plan, blocker, winner, failure, baseline, and operation evidence remain available |
| Human error | Parser or domain error message | First line begins `Error:` | None; JSON error envelope remains unchanged |

## Projection rules

### Declared mapping row

- The displayed source and destination come from the declared mapping, never from the resolved endpoint.
- All-mapping lists omit the mapping-kind prefix.
- Selected-list and removal rows retain the mapping-kind prefix required by the approved transcript.

### Concise mutation row

- A push action renders `SOURCE -> DESTINATION`.
- A pull action renders `SOURCE <- DESTINATION`.
- A synchronization result uses the individual action direction for every row.
- The heading count equals the number of rows rendered.
- Action status, recovery, verification, durability, baseline, and operation-record data are not part of the concise view but remain in the existing structured result.

### Status group

- Pushable records precede pullable records, which precede conflicts, which precede needs-baseline records.
- Initial collisions and ordinary divergent conflicts include source-winning and destination-winning force choices after their path-pair row.
- Compatibility and unsupported-state blockers continue to use the existing safety-significant detail path.

### Force-resolution guidance

- Source-winning guidance uses the CWD-relative source display with `grip push --force`.
- Destination-winning guidance uses the displayed destination endpoint with `grip pull --force --destination`.
- Guidance is derived only for initial collisions and ordinary divergent conflicts; other blocker classes retain existing diagnostics.
- Baseline outcome and generation data are excluded from the default human blocked result but remain in the structured result.

### Terminal mutation result

- A no-action plan states `Nothing to push.`, `Nothing to pull.`, or `Nothing to synchronize.`.
- A planned or applied baseline-only mutation states that it would establish or established a baseline for its accepted-entry count.
- A conflict-only blocked plan displays each source/destination pair and the two force-resolution choices. The human label for an internal source-winning or destination-winning `resolve` is `Push` or `Pull`, respectively.
- A technical or mixed blocked plan directs the operator to `Run: grip status`; the status command remains the current detailed safety view.
- A failed mutation retains its existing action-completion count and adds `Run: grip status before retrying.`.
- Default human terminal views never render planner reason IDs, conflict winner fields, action milestones, recovery fields, baseline authority, or operation-record identifiers.

## State transitions

Rendering does not change a command result’s state. A preview remains non-mutating, an applied mutation retains its existing state publication, and an error retains its existing exit category and no-mutation guarantee.

## Validation rules

- Human-only fields must not be serialized into the JSON envelope.
- Terminal mutation projections are used only for recognized push, pull, sync, and forced directional-resolution outcome results; unrelated errors continue through their established renderers.
- A default human output change must not alter selection, plan construction, mutation execution, result-delivery finalization, or baseline publication.
