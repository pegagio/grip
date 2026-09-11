# Research: Simplify Status Output

## Status-only presentation boundary

**Decision**: Add a dedicated default-human `status` renderer in `src/result.rs` and leave the generic detailed classification renderer in place for `diff` and any other existing detailed inspection operation.

**Rationale**: `CommandOutcome::classification` already creates the status message and stable serialized details, while `render` selects human or JSON presentation. A status-only branch changes the reported human view without changing inspection, classification, JSON construction, or command execution.

**Alternatives considered**: Changing classification records or counts was rejected because those values are used by JSON consumers and mutation planning. Changing the generic classification renderer was rejected because `diff` is the existing detailed read-only inspection surface.

## Directional sections and deterministic counts

**Decision**: Derive five mutually exclusive presentation groups from each existing record: current (`not attention`), conflicts (`blocking`), changes to push (`attention`, not blocking, source-to-destination), changes to pull (`attention`, not blocking, destination-to-source), and needs baseline (the remaining non-blocking attention records). Render only nonempty actionable sections, preserving existing identity-sorted record order within each section.

**Rationale**: The existing `attention_count` includes blockers, which makes the current summary hard to read. A presentation-only partition produces counts that add up to the inspected entry count and maps ordinary action directions directly to Grip's established `push` and `pull` vocabulary. `Needs baseline` makes every nonblocking non-directional reconciliation state visible without misleadingly placing it in either directional section; examples include equal peer pairs, deletion convergence, and metadata-migration readiness.

**Alternatives considered**: Retaining the current overlapping attention and blocker counts was rejected because it repeats the ambiguity in the reported output. A generic `Changes` section was rejected because it hides the established push/pull direction. Reordering or reclassifying records in the domain model was rejected because stable JSON ordering and classification semantics must remain unchanged.

## Plain-language status entries and suggestions

**Decision**: Render source and destination display paths on every actionable row using the section's status symbol: `->` for push, `<-` for pull, `<->` for conflict, and `>-<` for a non-directional reconciliation state. Do not print command suggestions. For safety blockers that are not an endpoint-content conflict, retain a concise plain-language explanation alongside the pair without exposing raw reason identifiers.

**Rationale**: Classification and compatibility records already contain safe source and destination paths, a prospective direction, blocking state, and user-oriented metadata messages. The symbols make direction legible while `<->` preserves conflict neutrality and `>-<` makes clear that a non-directional state needs accepted-baseline or reconciliation attention without asserting that peer payloads match.

**Alternatives considered**: Rendering raw classification names, changed dimensions, or compatibility reason codes was rejected because those are the source of the reported noise. Textual command suggestions were rejected because the directional section headings and symbols convey the ordinary next action without creating the impression that a forceful operation is recommended.

## Contract and validation strategy

**Decision**: Add exact default-human output assertions for clean, mixed, metadata-blocked, informational-only, empty, and each nonblocking non-directional reconciliation classification scope. Retain existing JSON and `status -e` assertions and revise existing human-output tests that currently require raw compatibility reason codes.

**Rationale**: The behavior is a public CLI presentation contract. Isolated fixtures already produce the required classifications and metadata findings, so focused integration tests can prove the new view and the unchanged structured interface without touching real files.

**Alternatives considered**: Snapshotting all output was rejected because small unrelated text changes would obscure the behavioral guarantees. Testing only JSON was rejected because it would leave the user-visible regression unprotected.
