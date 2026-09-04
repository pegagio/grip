# Ownership Topology Contract

This contract defines complete-registry rejection before Feature 003 discovers tree members.

## Namespace extents

- A file mapping source and destination each reserve one exact path.
- A tree mapping source and destination each reserve the root and every prospective descendant path.
- Path relationships are component-aware after canonicalization. `/a/b` does not contain `/a/beta`.

## Required rejection matrix

| First namespace | Second namespace | Relation | Result |
|---|---|---|---|
| Exact | Exact | Equal | Reject duplicate or ambiguous ownership |
| Exact | Tree | Exact equals or descends from tree | Reject overlap |
| Tree | Exact | Tree equals or contains exact | Reject overlap |
| Tree | Tree | Equal, ancestor, or descendant | Reject overlap |
| Any | Any | Disjoint | No conflict from this pair |

The matrix applies independently to all source-source pairs and destination-destination pairs.

## Recursive and cross-side rejection

For each mapping, reject equal source and destination endpoints. Reject any tree source containing its destination and any tree destination containing its source.

Across mappings, reject when any source namespace overlaps any destination namespace in either direction. This conservative rule prevents future source discovery from reading a path whose ownership is also defined as a destination and prevents cycles longer than two mappings without needing a separate graph traversal.

## Complete-registry behavior

Validation evaluates every accepted mapping even when a command targets one source identity. Adding a candidate validates all accepted mappings plus the candidate. Removing a mapping first validates the accepted registry, then validates the remaining candidate registry.

All conflicts are deduplicated and sorted by canonical mapping source, conflicting path, and stable reason. A non-empty conflict collection rejects the operation before publication. No payload entry is traversed or opened to determine topology.

## Examples

These relationships are rejected:

```text
tree source /source/editor -> destination /home/editor
file source /source/editor/config -> destination /backup/config
```

The second source overlaps the first source tree.

```text
tree source /source/editor -> destination /home/editor
tree source /home -> destination /backup/home
```

The second source contains the first destination, creating cross-mapping recursion.

These relationships are disjoint and permitted:

```text
file source /source/gitconfig -> destination /home/.gitconfig
tree source /source/editor -> destination /home/.config/editor
```

Final endpoint node-kind and safe-ancestry validation still apply independently of topology.
