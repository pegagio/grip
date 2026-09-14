# Large-File Performance Contract

This contract adds no public CLI option, output field, storage schema, or compatibility mode. It defines the acceptance harness and the existing command behavior that Feature 021 must preserve.

## Shared Workload Contract

- The harness invokes the local release `grip` executable in a fresh isolated temporary project with a controlled home and working directory.
- The release binary and isolated temporary fixture are mandatory preconditions; if either cannot be prepared, the harness fails with an explicit diagnostic and does not substitute any operator-managed path.
- Source and destination are existing regular files with different approximately 19 MB contents.
- Fixture creation, project initialization, and any mapping setup happen outside a timed sample.
- Each workflow runs once untimed before its 100 measured process samples.
- Every sample must match the expected exit status, standard output, and standard error for that workflow.
- The harness records p50, p95, and maximum durations. `p95 <= 1 second` is required for each workflow.

## `grip add` Contract

```text
grip add SOURCE DESTINATION
```

| Condition | Required result |
|---|---|
| Existing unequal regular source and destination | Command succeeds and records the mapping without copying either payload |
| Published mapping | Retains the submitted mapping intent and uses the existing initial comparison-state semantics |
| After the timed command | Source and destination bytes are unchanged; a subsequent untimed status check reports the established expected source-to-destination attention state |
| Endpoint or publication drift | Existing failure and fence behavior remains authoritative; the benchmark must not suppress or reinterpret it |

Each measured `add` sample uses a fresh mapping-less fixture because the command publishes mapping and state data.

## `grip status` Contract

```text
grip --output=json status
```

| Condition | Required result |
|---|---|
| Established mapping with unequal regular endpoints | Command succeeds and returns the existing classification and direction for the mapping |
| Stable endpoint evidence | Content fingerprints, supported metadata, selection, and output retain their existing semantics |
| Changed-during-observation endpoint | Existing drift error remains authoritative and does not become a successful measurement |

The status fixture establishes the mapping outside the timed interval. Timed samples do not mutate mapping intent, baseline state, source, or destination.

## Observation Safety Contract

- Every retained observation pass continues to validate ordinary file eligibility, avoid following the final path component, stream full content for SHA-256 identity, capture supported metadata, and reject changed-during-read evidence.
- The first and second stable-observation passes remain independent. Their mismatch remains a stale-observation error.
- Only duplicate observation of the same normal file-mapping endpoints inside one pass may be eliminated. Missing/discovery-absent mappings retain a direct-observation fallback.
- No persistent cache, index, background work, parallel execution, broad lock, state schema change, or public contract change is introduced.
