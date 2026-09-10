# Research: Git-Inspired Command Hierarchy

## Decision: Keep the public command tree flat

Use direct subcommands rather than a `mapping` namespace or recovery-oriented verb groups.

```text
grip init [PATH]
grip version
grip add <SOURCE> <DESTINATION>
grip list [SOURCE]
grip remove <SOURCE>
grip status [-e] [-d DESTINATION] [PATH]
grip diff [-d DESTINATION] [PATH]
grip push [-n] [-f] [-d DESTINATION] [PATH]
grip pull [-n] [-f] [-d DESTINATION] [PATH]
grip sync [-n] [-d DESTINATION] [PATH]
```

**Rationale**: The commands each express an operation that users can name directly. Flat verbs match Git’s familiar command vocabulary and eliminate ambiguous pairs such as `validate`/`check`, `mapping.inspect`/`mapping.show`, and `delete`/`retire`.

**Alternatives considered**:

- Keep `mapping add/list/remove`: rejected because mappings are Grip’s primary objects, not a subsystem that needs a namespace.
- Keep aliases for compatibility: rejected because aliases preserve the ambiguity that the redesign removes.

## Decision: `add` records intent but does not synchronize

`add` requires that source or destination exists. It discovers the mapping kind from an existing endpoint, rejects incompatible endpoints, records the declaration, and performs no endpoint copy or creation. It records a baseline only when both endpoints already match.

**Rationale**: Registration should not silently select a winner. A missing peer remains a valid declaration and can be populated by later directional `push` or `pull`.

**Alternatives considered**:

- Require both endpoints: rejected because a new mapping should support a not-yet-created peer.
- Infer a winner and copy during `add`: rejected because differing content has no user-selected direction.

## Decision: Directional force selects the winner

Ordinary `push` and `pull` block when one endpoint is absent. `push -f` declares the source authoritative; `pull -f` declares the destination authoritative. Force resolves exactly one managed entry and may publish its absence to delete the peer.

**Rationale**: Direction is both understandable and sufficient to resolve change/deletion conflicts without a separate `resolve`, `delete`, or `accept` command.

**Alternatives considered**:

- Overload `add` as conflict resolution: rejected because registration and mutation have different safety expectations.
- Add `delete` or `rm`: rejected because deletion is already an authoritative directional outcome.

## Decision: `sync` is conservative composition

`sync` applies the appropriate ordinary direction only for entries with a clear non-absent winner. It blocks one-sided absence and differing unbaselined entries; users choose `push` or `pull` in those cases.

**Rationale**: An aggregate command must not hide conflict choice or make an initial baseline decision.

## Decision: Baselines follow active membership, not history

Removing a mapping removes its internal baseline. A baseline also records the applicable `.gripignore` policy-revision token: its content digest plus filesystem change identity/time. When policy changes, the old baseline is no longer eligible; when an ignore rule excludes an entry, membership reconciliation prunes it. If it is later re-added or unignored, it is a newly discovered, unbaselined entry.

**Rationale**: Old baseline evidence cannot safely describe a new membership relationship. The policy token prevents a rule that was added and later removed between Grip commands from reviving old state, while allowing `diff` to remain non-mutating. This gives direct `.gripignore` editing Git-like semantics without an extra command.

**Alternatives considered**:

- Keep a dormant baseline and revive it when unignored: rejected because it could silently apply stale conflict evidence.
- Introduce a public `ignore` command: rejected because direct `.gripignore` editing is simpler and already familiar.

## Decision: Internal completion markers are not recovery history

Registry/state membership changes use prevalidated candidates and bounded locks. If a process is interrupted between durable writes, a narrowly scoped, ephemeral completion marker may finish the state-pruning transition on the next Grip operation. It stores only enough identity and intended final state to complete that transition, is deleted on completion, and never restores endpoint payloads.

**Rationale**: This preserves atomic membership semantics without reintroducing the removed recovery product surface.

**Alternatives considered**:

- Retain operation logs and payload backups: rejected because Git or another user-selected system owns history and recovery.
- Leave stale baselines after interruption: rejected because a later re-add/unignore could revive invalid evidence.

## Decision: Human output is the default

Commands print human-readable results unless `-o json` or `--output json` is supplied. The grammar deliberately leaves room for future output values without accepting `human` today.

**Rationale**: The common interactive path needs no option, while scripts have one explicit stable format.

## Decision: Status owns ordinary attention exit behavior

`status` always reports the same content. `status -e` makes ordinary attention return nonzero; invalid metadata and operational failures are errors whether or not `-e` is supplied.

**Rationale**: This mirrors Git’s status-checking automation pattern while keeping corruption visibly exceptional.
