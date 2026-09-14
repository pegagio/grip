# CLI Contract: Force Add Replacement

This contract defines the new `grip add` modifier. It complements the existing mapping-add contract; all existing endpoint validation, ownership errors, and output conventions remain in force unless explicitly changed below.

## Command

```text
grip add [--force|-f] SOURCE DESTINATION
```

`--force` is explicit confirmation to replace one qualifying active mapping. Omitting it retains ordinary append-only add behavior.

## Preconditions

- `SOURCE` and `DESTINATION` pass the existing parse, kind, endpoint, and safety checks.
- The requested declaration is valid under the complete candidate registry.
- Force replacement is authorized only when exactly one active, distinct `File` mapping has the same canonical destination as the requested mapping.
- No source-overlap, tree, nested, ambiguous, incompatible, unsupported, or other ownership/topology conflict remains after the exact candidate replacement is constructed.

The command does not copy, delete, or modify source or destination payloads.

## Successful Results

### Ordinary Add

Ordinary successful adds retain the existing result shape:

```json
{
  "ok": true,
  "message": "Mapping recorded",
  "details": {
    "operation": "add",
    "mapping": {
      "kind": "file",
      "source": "target/release/grip",
      "destination": "~/.local/bin/grip"
    }
  }
}
```

### Forced Replacement

A successful force replacement uses `operation: "add"`, reports the requested mapping as `mapping`, and reports the retired mapping as `replaced_mapping`:

```json
{
  "ok": true,
  "message": "Mapping replaced",
  "details": {
    "operation": "add",
    "mapping": {
      "kind": "file",
      "source": "target/release/grip",
      "destination": "~/.local/bin/grip"
    },
    "replaced_mapping": {
      "kind": "file",
      "source": "target/debug/grip",
      "destination": "~/.local/bin/grip"
    }
  }
}
```

Human output is concise and identifies both mappings:

```text
Mapping replaced:
  old: target/debug/grip -> ~/.local/bin/grip
  new: target/release/grip -> ~/.local/bin/grip
```

The output uses the accepted portable declaration spelling; replacement eligibility uses canonical resolved endpoints.

## Failure Results

| Condition | Result |
|---|---|
| Ordinary add conflicts with an active owner | Existing `destination_overlap` mapping error; no change. |
| Force finds zero or more than one qualifying mapping | Mapping error explaining that no single equal-destination file mapping is authorized for replacement; no change. |
| Candidate has any remaining ownership or topology conflict | Existing conflict category and detail; no change. |
| Endpoint validation or publication revalidation fails | Existing precise validation/drift category; prior accepted descriptor/state pair remains authoritative or is restored. |
| Another transition fence is active | Existing actionable incomplete-publication error; no unbounded wait or unrelated mutation. |

Failures must not change endpoint payloads. They also must not expose a partially replaced declaration or State V4 evidence.

## Removal Compatibility

`grip remove SOURCE` continues to select a mapping by its resolved source. A successful exact removal removes its declaration and all accepted baseline identities for that mapping through the same recoverable descriptor/state transition discipline. A later ordinary add to its old destination must therefore succeed if no other active mapping owns that destination.
