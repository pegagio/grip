# Discovery Contract

This contract defines the complete read-only boundary for file mappings, tree traversal, destination overlay inspection, and stale-evidence rejection.

## Source discovery

File mappings contribute exactly one eligible record when their accepted source remains the mapped ordinary-file kind. Grip does not enumerate either parent.

For a tree mapping, Grip opens the accepted source root without following its final node and walks directory entries relative to open parent descriptors. Each directory's raw names are sorted before processing. The source root establishes the permitted device; a child directory on another device is a `nested_mount` unsupported record and is not opened.

Policy is evaluated before payload classification. An ignored path produces an `ignored` record. An ignored directory is not traversed and no record is invented for a descendant that was not inspected.

## Node allowlist

After policy permits a path, classify it in this order:

1. symbolic link;
2. socket, FIFO, character device, block device, whiteout, or unknown special type;
3. directory on a device different from the source root;
4. regular file with link count greater than one;
5. regular file whose reported allocated 512-byte blocks cannot cover its logical length;
6. ordinary regular file or directory.

Steps 1 through 5 produce `unsupported_source` with the corresponding stable reason and stop all opening, hashing, reading, or traversal of that path. A non-UTF-8 relative name is also unsupported and is represented through Safe Path raw bytes.

Ordinary directories, including empty directories beneath the mapping root, are eligible records. The mapping root itself is ownership intent rather than a discovered member and is not emitted as a tree-member record.

## Destination inspection

For every eligible source-relative path, inspect the paired destination without following its final node:

- absent or ordinary compatible destination: retain the source record without synchronization classification;
- unsupported node: add `unsafe_destination_collision` and mark it blocking;
- ordinary incompatible file/directory kind: add `unsafe_destination_collision` with `wrong_node_kind`.

Walk each existing ordinary destination tree without source ignore policy. Any path absent from the eligible-source relative set is `destination_only` and nonblocking, regardless of detected kind. A destination-only symbolic link, special node, or nested mount is recorded once with its reason and not followed. Discovery does not infer prior management because Feature 003 has no baseline.

## Deterministic order

Mappings use accepted canonical source order. Within a mapping, compare exact raw relative-path bytes; the file-mapping record sorts before tree-member paths only according to mapping order. For multiple records at one relative path, category order is:

1. `eligible`
2. `ignored`
3. `destination_only`
4. `unsupported_source`
5. `unsafe_destination_collision`

Counts are derived after sorting. The renderer must not depend on traversal arrival order.

## Evidence and completeness

One pass captures:

- accepted registry bytes and node identity;
- non-following metadata for every inspected source and destination node;
- sorted child-name bytes for every traversed directory;
- exact bytes, digest, and metadata for every loaded policy file;
- the final ordered records.

Grip repeats the complete pass from fresh root descriptors. It returns an inventory only when both passes are equivalent. A mismatch, disappearance, replacement, directory enumeration error, or changed policy returns `stale_discovery_evidence` or the more specific operational failure. The result makes no filesystem snapshot-isolation claim.

Evidence is command-local and discarded after rendering. It must never become a persisted inode identity, watcher index, or cache.

## Read boundary

Discovery may read registry and `.gripignore` bytes. It may enumerate directories and read non-following metadata. It must not read or hash ordinary payload file contents, open unsupported nodes, follow symbolic links, traverse nested mounts, acquire Grip publication locks, or write any path.
