# Design Model: Large-File Performance Evidence

Feature 021 adds no persisted application entity or schema. Its design uses operation-local evidence and isolated test-fixture concepts only.

## RepresentativeLargeFileWorkload

| Field | Meaning | Validation |
|---|---|---|
| command | `grip add` or `grip status` | One of the two feature workflows |
| binary | Local release `grip` executable | Must exist before measurement |
| project root | Fresh isolated temporary project | Must not be a developer-owned path |
| source and destination | Existing regular files with different approximately 19 MB content | Must remain ordinary supported files |
| mapping state | Mapping-less for `add`; established unequal mapping for `status` | Setup occurs outside a timed interval |
| sample count | 100 warm process runs | Positive and fixed for this feature case |
| expected result | Existing exit status, stdout, stderr, and semantic result | Must remain deterministic over samples |
| execution preconditions | Release binary and isolated temporary fixture readiness | A missing prerequisite fails with an explicit diagnostic; no operator path is substituted |

## PerformanceDistribution

| Field | Meaning | Validation |
|---|---|---|
| p50 | Median elapsed process duration | Computed from all warm samples |
| p95 | 95th-percentile elapsed process duration | Must be at most one second after improvement |
| maximum | Largest elapsed process duration | Reported for diagnosis; not the acceptance threshold |
| environment | Release-build and workload conditions | Recorded with each distribution |

## ObservationPassEvidence

| Field | Meaning | Invariant |
|---|---|---|
| complete endpoint observation | Content fingerprint, supported metadata, and endpoint identity produced during one pass | Ordinary file semantics and no-follow behavior remain unchanged |
| discovery representation | Inventory record for an eligible normal file mapping | Supplies the same mapping identity used by the pass |
| fallback observation | Direct observation for a file mapping not represented by discovery | Prevents missing or retained mappings from being silently skipped |
| stable observation result | Equality of first and second pass evidence | A mismatch remains a drift error, not a reused result |

## PerformanceDecision

| Field | Meaning | Validation |
|---|---|---|
| baseline distribution | Before-change p50/p95/maximum for each command | Uses the representative workload |
| causal finding | Measured repeated work or another observed cost | Required before a code change is accepted |
| selected change | Operation-local observation deduplication or an explicitly justified alternative | Must retain all safety invariants |
| after distribution | p50/p95/maximum after the change | Both commands meet p95 <= one second |
| semantic evidence | Mapping, baseline, payload, and classification checks | Must match existing contracts |

## State Transitions

```text
mapping-less unequal endpoints
  -- release `grip add` -->
published mapping + destination-derived initial comparison state
  -- release `grip status` -->
unchanged mapping/state + expected pending source-to-destination classification
```

The benchmark creates this transition in isolated fixtures. It never writes an operator's project, source, destination, or Grip-owned state directory.
