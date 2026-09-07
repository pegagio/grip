# CLI Contract: Pull

## Grammar

```text
grip [--output human|json] [-v...] pull [-n|--dry-run] [--destination] [--] [PATH]
```

- `pull` always transfers eligible destination state to established managed sources.
- Execute mode is the default.
- `-n` and `--dry-run` are equivalent and produce no payload, registry, baseline, recovery, operation-record, or coordination mutation.
- One optional `PATH` uses source path space by default.
- `--destination` changes only selector interpretation.
- `--` ends option parsing.
- More than one positional selector is invalid usage with exit `2`.

## Selection

The established Feature 004 selector contract remains authoritative. An omitted path selects all mappings plus retained baseline-only evidence. A source-space or destination-space selector may resolve one mapping, one established managed entry, or one component-boundary subtree. Complete registry validation still precedes scoped work.

An unmanaged destination-only item can be reported when it falls within the inspected destination side of the selected mapping scope, but it cannot itself establish selection or ownership.

## Outcomes

| Condition | Result | Exit | Mutation |
|---|---|---:|---|
| Complete plan has blockers | `blocked` | stable existing error category | none |
| Complete plan has no actions | `no_op` | 0 | none |
| Dry run has eligible actions | `planned` | 0 | none |
| Execute applies and accepts all actions | `applied` | 0 | source and accepted baseline |
| Evidence changes before operation initialization | stale failure | existing operational category | none |
| First action fails after execution starts | `failed` or `partial` | 20 unless an existing more specific category applies | completed source changes remain; no new baseline |
| Baseline becomes visible but durability is unconfirmed | failed with visible authoritative generation | 20 | source changes and visible baseline remain |
| Result delivery fails after acceptance | operational failure | 20 | accepted source and baseline remain authoritative |

## Shared JSON result

The top-level `ResultEnvelopeV1` remains unchanged. Pull uses the same mutation details shape as push and adds the explicit direction discriminator required for both commands:

```json
{
  "schema_version": 1,
  "status": "ok",
  "code": "ok",
  "message": "Pull preview complete: 3 selected; 1 action(s); 0 blockers",
  "details": {
    "operation": "pull",
    "direction": "pull",
    "mode": "dry_run",
    "completion": "complete",
    "result": "planned",
    "scope": {},
    "plan_id": {"algorithm": "sha256", "digest": "<digest>"},
    "counts": {},
    "entries": [],
    "actions": [],
    "blockers": [],
    "operation_record": null,
    "baseline": {}
  }
}
```

Every push mutation result also carries `direction: push`; its remaining semantics stay unchanged. Mapping-role `source_path` and `destination_path` fields never swap in either direction.

## Human output

Human output uses pull terminology and identifies source publication outcomes. It reports every selected entry disposition, including each destination-only unmanaged item as unmanaged and non-actionable. Action lines distinguish planned, completed, failed, and unattempted source replacements and communicate visibility, verification, durability, recovery availability, and baseline authority. Diagnostics remain on stderr.
