# Data Model: Source Discovery and Gripignore

This model is ephemeral. It defines the facts needed to produce one deterministic discovery result and does not add persisted registry or state schema.

## Table of Contents

- [Discovery request](#discovery-request)
- [Discovery pass](#discovery-pass)
- [Safe path](#safe-path)
- [Discovery record](#discovery-record)
- [Gripignore policy](#gripignore-policy)
- [Node evidence](#node-evidence)
- [Discovery inventory](#discovery-inventory)
- [Lifecycle](#lifecycle)
- [Validation order](#validation-order)

## Discovery request

| Field | Type | Rules |
|---|---|---|
| `scope` | `AllMappings` or `OneMapping` | Required |
| `source_selector` | optional path | Present exactly once for `OneMapping`; resolves through the accepted canonical source-identity policy |

Destination-side and nested-member selection are not supported. Before resolving the selector, the complete Registry V1 must pass the accepted Feature 002 validation.

## Discovery pass

One complete read-only inspection of the selected accepted mappings.

| Field | Type | Rules |
|---|---|---|
| `registry_evidence` | accepted bytes and node identity | Must match the loaded valid registry |
| `records` | ordered collection of Discovery Record | Ordered by mapping source, relative raw bytes, and category |
| `node_evidence` | map keyed by side, mapping, and relative raw bytes | Ephemeral and complete for every inspected node |
| `policy_evidence` | map keyed by policy Safe Path | Exact bytes and node evidence for every loaded policy |

The first and second passes are equivalent only when all four fields compare equal.

## Safe path

A non-ambiguous machine and human representation of a path observed during discovery.

| Field | Type | Rules |
|---|---|---|
| `display` | escaped UTF-8 string | Printable form that never emits raw control characters |
| `raw_hex` | optional lowercase hexadecimal | Present when raw Unix bytes cannot be represented exactly by `display` alone |

Canonical mapping roots remain valid UTF-8 strings under Feature 002. A relative name with invalid UTF-8 is not eligible; its Safe Path retains exact raw-byte identity for reporting.

## Discovery record

| Field | Type | Rules |
|---|---|---|
| `category` | stable category | `eligible`, `ignored`, `destination_only`, `unsupported_source`, or `unsafe_destination_collision` |
| `mapping_kind` | `file` or `tree` | Owning accepted mapping |
| `mapping_source` | canonical UTF-8 path | Stable mapping identity |
| `relative_path` | optional Safe Path | Absent for an exact file mapping; non-empty for a tree member |
| `source_path` | optional Safe Path | Present when a source-side path was inspected |
| `destination_path` | Safe Path | Exact paired or destination-only path |
| `node_kind` | stable node kind | Ordinary file/directory or detected unsupported kind |
| `reason` | optional stable reason | Required for unsupported/collision records and for a pruned destination-only boundary |
| `blocking` | boolean | True only for `unsupported_source` and `unsafe_destination_collision` |

Stable unsupported and collision reasons are `non_utf8_path`, `symlink`, `hard_link`, `sparse_file`, `socket`, `fifo`, `character_device`, `block_device`, `whiteout`, `unknown_special`, `nested_mount`, and `wrong_node_kind`.

An ignored directory produces one record for that directory and no fabricated descendant records. A destination-only unsupported boundary remains nonblocking but records its detected reason and is not traversed.

## Gripignore policy

| Field | Type | Rules |
|---|---|---|
| `path` | Safe Path | Exact source-side `.gripignore`; never a payload record |
| `directory_relative_path` | raw relative bytes | Defines scope for contained rules |
| `bytes` | UTF-8 byte sequence | May begin with a stripped UTF-8 BOM; CRLF and final unterminated line are accepted |
| `matcher` | ordered Gitignore rules | Built successfully from every line or the request fails |
| `evidence` | Node Evidence plus byte digest | Must match in the second pass |

Matcher stacks are ordered from tree root to current directory. The deepest non-neutral matcher decides; within one policy file the last matching rule decides. `.gripignore` itself is policy-only regardless of a negated rule.

## Node evidence

Ephemeral metadata sufficient to detect whether an inspected namespace changed between passes.

| Field | Type | Rules |
|---|---|---|
| `side` | `source` or `destination` | Required |
| `mapping_source` | canonical path | Required |
| `relative_bytes` | byte sequence | Exact identity within the mapping |
| `device`, `inode` | unsigned integers | Evidence only; never persisted ownership identity |
| `mode`, `link_count` | unsigned integers | Type and hard-link evidence |
| `size`, `allocated_blocks` | unsigned integers | Sparse-file evidence without reading payload |
| `modified`, `changed` | seconds and nanoseconds | Drift evidence |
| `child_names` | optional ordered byte sequences | Present for traversed directories |

Directory evidence is captured relative to an open parent descriptor. The source root's device establishes the allowed traversal filesystem.

## Discovery inventory

| Field | Type | Rules |
|---|---|---|
| `scope` | request scope | Required |
| `records` | ordered Discovery Records | Equal to both completed passes |
| `counts` | category counts | Derived exactly from records |
| `blocking_count` | non-negative integer | Count of records with `blocking=true` |

The inventory is returned to the renderer and then discarded. It is never written beneath Grip Home or either payload root.

## Lifecycle

```text
Request
  -> complete registry validation
  -> first discovery pass
  -> second discovery pass
  -> equivalent evidence and records -> return inventory
  -> mismatch or incomplete inspection -> return failure without inventory
```

Every transition is read-only. Unsupported records may produce a successful inventory with blockers. A policy or operational error prevents a complete inventory.

## Validation order

1. Resolve Grip Home and load the accepted registry snapshot without a publication lock.
2. Decode, version-check, canonicalize, and validate the complete registry and ownership graph.
3. Resolve an optional source selector to exactly one accepted mapping.
4. For each selected mapping in canonical source order, perform non-following source discovery and policy evaluation.
5. Inspect each eligible paired destination and walk destination-only paths without applying ignore policy.
6. Sort and finalize the first pass records and evidence.
7. Repeat steps 1 through 6 from fresh descriptors and policy reads.
8. Compare registry, node, directory, policy, and record evidence.
9. Return one inventory only on exact equivalence; otherwise return the applicable invalid, operational, or stale-evidence failure.
