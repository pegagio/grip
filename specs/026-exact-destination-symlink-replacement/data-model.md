# Data Model: Exact Destination Symlink Replacement

Feature 026 adds ephemeral observation and action evidence only. Project descriptor V2 and accepted state V4 remain the durable authorities.

## Destination Leaf Link Evidence

Represents one exact managed destination leaf observed as a symbolic link through a safe opened parent.

| Field | Meaning |
|---|---|
| Managed identity | Active file identity or current/retained tree member paired to the destination leaf |
| Destination path | Exact leaf spelling used for diagnostics and replacement binding |
| Parent ancestry | Existing no-follow evidence for every required destination ancestor |
| Link object identity | No-follow device, inode, mode, timestamp, and other stable metadata needed to detect final-object substitution |
| Observation result | `unresolved_destination_link` |
| Target data | Deliberately absent; Grip does not read or store it |

Validation rules:

- The final node must be a symbolic link; an absent or supported final node is a different outcome.
- Every required ancestor must be an existing safe directory reached through no-follow operations.
- Source links, root links, ancestor links, incompatible leaves, special nodes, and mounts never produce this evidence.
- The evidence is discarded after the operation and is never baseline, descriptor, or State V4 content.

## Unresolved Managed Entry

Represents an active managed identity whose source is a supported regular file or directory but whose exact destination leaf is link evidence rather than supported payload.

| Field | Meaning |
|---|---|
| Identity | Existing mapping and raw relative identity |
| Source state | Complete observed supported source state |
| Destination state | Absent because the link is not payload |
| Link evidence | Exact destination leaf evidence |
| Classification | `unresolved_destination_link` |
| Baseline | Absent until verified source-winning replacement |
| Ordinary disposition | Blocking |
| Force disposition | Actionable only for source winner and one exact source-space identity |

The unresolved state changes neither mapping ownership nor the source-defined membership rule. It does not let a destination-only link become managed.

## Link Replacement Action

Represents one force-authorized replacement of a verified destination link object.

| Field | Meaning |
|---|---|
| Direction | Push only |
| Operation | Resolve with source winner |
| Identity | One exact selected managed identity |
| Origin evidence | Existing complete supported source state |
| Expected target | Destination leaf link evidence, not supported destination payload |
| Action kind | `replace_destination_link_file` or `replace_destination_link_directory` |
| Publication primitive | Verified private sibling plus atomic rename through the revalidated parent descriptor |
| Verification | Resulting destination is a supported source-equivalent state; former target is not inspected |

Validation rules:

- The action requires exactly one source-space selector after current observation.
- Rebuild and per-action revalidation must reproduce the same source state, mapping safety conditions, parent ancestry, and final link identity.
- File replacement stages source bytes into a verified sibling. Directory replacement stages an empty verified sibling directory, publishes it by rename, then relies on existing descendant and directory-finalizer actions.
- Any failed or stale action produces the existing precise partial-failure result and cannot publish accepted state.

## State Transitions

### Mapping addition

`source-kind-resolved` → `candidate registry validated` → `exact managed destinations probed` → `unresolved link retained without baseline` → `fenced descriptor and state publication`

Unsupported source links, link ancestors, absent source kind, topology conflicts, or stale candidate evidence transition to `rejected` before descriptor publication or payload mutation.

### Read-only inspection

`active mapping` → `managed identities derived` → `exact destinations probed` → `unresolved destination link classified` → `path-specific diagnostic`

Ordinary operations retain `blocked`; read-only output has no state transition or endpoint side effect.

### Exact forced replacement

`exact source selector` → `one unresolved identity confirmed` → `plan` → `mutation lock and plan rebuild` → `link and ancestry revalidated` → `private sibling staged` → `atomic rename over link object` → `supported-state verified` → `accepted state published`

Any pre-publication mismatch transitions to `stale` with no payload mutation. Any later failure is reported through the existing partial-failure contract and has no accepted-state publication.
