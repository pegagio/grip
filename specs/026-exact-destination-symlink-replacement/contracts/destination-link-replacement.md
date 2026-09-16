# Contract: Exact Destination Symlink Replacement

This contract defines Feature 026's externally observable admission, diagnostic, force, and safety behavior. It adds no command, option, descriptor version, accepted-state version, or link-target interface.

## Admission Matrix

| Source | Destination condition | `grip add` result |
|---|---|---|
| Supported regular file or directory | Exact paired leaf is a symbolic link | Admit mapping intent; retain the leaf as unresolved; do not mutate payload or target |
| Supported tree source | Exact paired current or retained member leaf is a symbolic link | Admit only that member as unresolved; retain bounded managed-identity inspection |
| Any supported source | Destination root or required ancestor is a symbolic link | Reject with a path-specific blocking diagnostic |
| Symbolic-link source | Any destination | Reject as unsupported source payload |
| Missing source and destination link | Any | Reject because mapping kind cannot be inferred without following the link |
| Unsupported, incompatible, ownership-conflicting, or topology-conflicting endpoint | Any | Preserve existing rejection |

Admission preserves all current candidate fencing, registry validation, and descriptor/state publication rules. No accepted baseline is created for an unresolved destination-link identity.

## Inspection and Diagnostics

Status, dry-run, and blocked mutation output for an unresolved leaf must include:

- The managed source identity and exact destination-link path.
- A stable unresolved destination-link reason in machine-readable output.
- A statement that ordinary synchronization does not replace the link.
- A source-winning force command only when the rendered source selector maps to exactly one active identity.

The following illustrative JSON fields describe semantic content, not a new envelope version:

```json
{
  "classification": "unresolved_destination_link",
  "blocking": true,
  "source_path": "home/.bash_profile",
  "destination_path": "<fixture home>/.bash_profile",
  "reason": "destination_leaf_symlink",
  "force": {
    "direction": "source_to_destination",
    "command": "grip push --force home/.bash_profile"
  }
}
```

No human or machine output may include target contents, target resolution, or a target-derived canonical path.

## Exact Force Matrix

| Request | Result |
|---|---|
| `grip push --force SOURCE` selecting one unresolved link identity | Eligible for planning and replacement after all normal safety checks and revalidation |
| `grip push --force --dry-run SOURCE` selecting one unresolved link identity | Shows the same planned replacement with no endpoint or state mutation |
| `grip pull --force`, destination-winning force, ordinary `push`, `pull`, `sync`, or `delete` | Blocked; no link replacement |
| Mapping, subtree, aggregate, ambiguous, multiple, or destination-space selection | Rejected before mutation |
| Exact entry with source link, link ancestor, ownership/topology issue, or stale evidence | Blocked by the applicable existing or path-specific reason |

Exact force does not generalize support for `UnsafeCollision`, unsupported nodes, or source links.

## Replacement Contract

Before publication, Grip must verify the current registry and State V4, selected source state, complete selected-mapping safety conditions, safe destination ancestry, and exact destination link-object identity. It must then stage a private sibling and atomically rename that sibling over the verified link object.

- A regular file source produces a staged regular-file sibling.
- A directory source produces a staged empty directory sibling, after which ordinary planned descendant actions and the existing directory metadata finalizer run.
- The link target is never opened, read, modified, renamed, removed, or verified.
- A changed, missing, or different final object or unsafe ancestor is stale evidence and stops before replacement.
- Only verified source-equivalent resulting state may enter accepted state. Any failure follows the existing partial-failure result and publishes no new accepted baseline.

## Compatibility

- Project descriptor V2, State V4, command syntax, selector interpretation, output envelope versions, ownership validation, and topology rules remain unchanged.
- Feature 025's exact managed-identity probe remains the only destination inspection boundary for tree mappings.
- Feature 009's no-follow unsupported-node boundary remains authoritative except for this exact runtime-only destination-leaf admission and source-winning replacement.
- Features 011, 019, and 023 retain exact force authority, executable guidance rules, and missing-peer behavior.
