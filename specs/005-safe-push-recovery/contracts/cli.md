# CLI Contract: Safe Push and Recovery

This contract extends Result Envelope V1 and the Feature 004 selector, classification, and exit-code contracts.

## Grammar

```text
grip [--output human|json] [-v...] push [-n|--dry-run] [--destination] [--] [PATH]
```

`push` mutates by default. `-n` and `--dry-run` are aliases. Source path space is the selector default; `--destination` changes selector interpretation only and never reverses push direction. At most one path is accepted, and `--` terminates option parsing.

Feature 005 adds no `--apply`, deletion flag, conflict winner, recovery command, multiple selector, or operation-inspection command.

## Successful machine-readable preview

```json
{
  "schema_version": 1,
  "status": "ok",
  "code": "ok",
  "message": "Push preview complete: 4 selected; 2 actions; 0 blockers",
  "details": {
    "operation": "push",
    "mode": "dry_run",
    "completion": "complete",
    "result": "planned",
    "scope": {
      "kind": "mapping",
      "path_space": "source",
      "selector": {"display": "/example/source"},
      "mapping_source": "/example/source"
    },
    "plan_id": {
      "algorithm": "sha256",
      "digest": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    },
    "counts": {
      "selected": 4,
      "actions": 2,
      "no_action": 2,
      "blockers": 0,
      "completed": 0,
      "failed": 0,
      "unattempted": 0
    },
    "entries": [],
    "actions": [],
    "blockers": [],
    "operation_record": null,
    "baseline": {
      "outcome": "not_attempted",
      "prior_generation": 3,
      "published_generation": null,
      "authoritative_generation": 3,
      "publication_visible": false,
      "durability_confirmed": true
    }
  }
}
```

The abbreviated arrays above are fully populated in actual results according to [the push contract](push.md). Safe paths carry `raw_hex` whenever display text is not lossless.

## Successful execution

An accepted execution uses `mode: "execute"`, `completion: "complete"`, `result: "applied"`, completed action counts, an opaque operation record, and `baseline.outcome: "published"`:

```json
{
  "operation_record": {"id": "operation-opaque", "available": true},
  "baseline": {
    "outcome": "published",
    "prior_generation": 3,
    "published_generation": 4,
    "authoritative_generation": 4,
    "publication_visible": true,
    "durability_confirmed": true
  }
}
```

The actual object appears inside `details`; this fragment shows only fields that differ materially from preview.

## No-op

An unblocked plan with zero actions exits `0` with `result: "no_op"`, no operation record, and no baseline publication. Entries still explain the complete selected scope.

## Blocked plan

A completely inspected plan with blockers exits `10` and uses Result Envelope V1 with `status: "error"`, `code: "invalid_configuration"`, `completion: "blocked"`, `result: "blocked"`, every selected entry disposition, and every blocker. It reports zero started actions, no operation record, and the prior authoritative baseline.

## Partial or failed execution

An operational failure exits `20` and reports:

- every action as `completed`, `failed`, or `unattempted`;
- the first failure's stable reason and safe paths;
- recovery state and opaque availability;
- publication visibility, verification, and durability independently;
- `baseline.outcome: "not_published"` when payload execution did not fully verify;
- the authoritative generation known from the publication result.

If accepted-state rename succeeds but its directory sync fails, the result uses `baseline.outcome: "publication_failed"`, `publication_visible: true`, the candidate as authoritative visible state, and `durability_confirmed: false`. It never claims the prior baseline remains visible.

## Output failure

If final output delivery fails, the process emits the established concise diagnostic to stderr and exits `20`. No complete result envelope is promised through the failed channel. The durable operation record is finalized best-effort with `result_delivery: "failed"`; any successfully published baseline remains authoritative.

## Human-readable contract

Human output derives from the same typed Push Result. Representative forms are:

```text
Push preview complete: 4 selected; 2 actions; 0 blockers
planned add_file /example/source/a -> /example/destination/a recovery=not_required
planned replace_file /example/source/b -> /example/destination/b recovery=planned
Baseline not attempted; generation 3 remains authoritative
```

```text
Push failed after 1 of 3 actions
completed add_file /example/source/a -> /example/destination/a
failed replace_file /example/source/b -> /example/destination/b reason=publication_failure visible=no verified=no recovery=preserved
unattempted add_file /example/source/c -> /example/destination/c
Baseline not published; generation 3 remains authoritative
Operation record operation-opaque preserved
```

Diagnostics remain on stderr and never change result semantics.

## Exit behavior

| Exit | Code | Push use |
|---:|---|---|
| 0 | `ok` | Completed unblocked preview, no-op, or fully applied and accepted execution |
| 2 | `invalid_usage` | Invalid grammar, option, or extra selector |
| 10 | `invalid_configuration` | Invalid selector/configuration or completely discovered blocked plan |
| 11 | `unsupported_schema` | Unsupported registry, state, operation, or lock-owner schema |
| 12 | `corrupt_state` | Unsafe, malformed, integrity-invalid, escaping, duplicate, or inconsistent private state/recovery/operation evidence |
| 13 | `state_contention` | Another Grip writer holds the per-user mutation lock |
| 20 | `operational_failure` | Stale evidence, I/O, staging, recovery, publication, verification, journal, baseline, or output failure |

Exit `1` remains exclusive to a completed `check` result that requires attention.

## Non-mutation boundaries

- Dry run, blocked plans, invalid requests, and no-ops create no mutation lock, operation record, recovery entry, payload change, or baseline generation.
- Read-only preflight may consume but never rewrite valid prior operation records.
- Push never changes source payloads, mapping intent, ignore policy, destination-only unmanaged content, or version-control state.
