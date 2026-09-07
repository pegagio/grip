# Domain Contract: Reverse Synchronization

## Direction table

Pull evaluates the complete inherited classification set. A blocking record is always blocked. Otherwise:

| Classification | Disposition | Reason |
|---|---|---|
| `destination_only_change` with accepted managed identity | `action` | `replace_source` |
| `destination_only_unmanaged` | `no_action` | `unmanaged_destination` |
| `synchronized` | `no_action` | `already_synchronized` |
| source-only state | `no_action` | `wrong_direction` |
| initial state without accepted identity | `no_action` | `not_established` |
| converged two-sided change | `no_action` | `explicit_acceptance_required` |
| deletion or pending retirement | `no_action` or inherited `blocked` | `separate_authorization_required` |
| conflict, unsupported, or unsafe evidence | inherited `blocked` | stable inherited reason |

Unmanaged destination items appear individually in the plan. Their presence alone never blocks an otherwise eligible action.

## Plan invariants

- Direction is `pull` and participates in `plan_id`.
- Every selected observation appears exactly once in `entries`.
- Every pull action has an accepted managed identity and `destination_only_change` classification.
- Pull actions are replacements only; there are no add, directory-create, or synthetic-parent actions.
- Mapping-role source and destination paths remain stable in records.
- Internal origin is destination; internal target is source.
- Actions sort by canonical mapping source bytes and source-relative raw bytes.
- Blockers are complete before execution begins.

## Execution sequence

1. Validate the complete registry, accepted state, selection, observation, and classification without scanning unrelated operation history.
2. Build a pure complete pull plan.
3. Return a blocked, no-op, or dry-run result without acquiring writer coordination or creating operation artifacts.
4. Acquire the per-user mutation lock as operation `pull`.
5. Reload registry and state, repeat observation and planning, and require semantic plan equality.
6. Initialize a Partitioned Operation Record V1 bound to `pull` immediately before the first mutation.
7. For each replacement in canonical order, checkpoint in-progress, revalidate both mapping sides and source ancestry, preserve the source, stage the destination beside the source, publish, sync, verify, and checkpoint completion.
8. Stop after the first failure, preserve visible changes and recovery, leave later actions unattempted, and do not invoke baseline publication.
9. After every action verifies, repeat complete selected observation and require actioned pairs to equal the planned destination state.
10. Acquire registry then state publication guards and publish one State V2 generation updating only actioned identities.
11. Checkpoint terminal baseline and result-delivery evidence, release coordination, and render the typed result.

## Failure invariants

- Complete preflight accumulates every known blocker and starts zero actions.
- Lock-held drift creates no operation record or payload mutation.
- A checkpoint failure before a side effect prevents that side effect.
- Completed source changes are not automatically rolled back.
- Any failed or partial execution leaves the prior baseline authoritative unless a replacement baseline is explicitly reported visible.
- Output failure cannot revoke a successfully published baseline.
- A prior nonterminal operation record is immutable evidence, not an active lock or resumable instruction.

## Exclusions

Pull never imports unmanaged destination entries, recreates missing sources or source parents, executes deletion, resolves conflicts, accepts converged two-sided edits, pushes source changes, performs bidirectional sync, cleans recovery, repairs state, elevates privileges, invokes Git, follows links, or claims snapshot isolation.
